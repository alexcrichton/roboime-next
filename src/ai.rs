use std::process::{Stdio, Command, Child};
use std::sync::mpsc::Sender;
use std::thread::{Builder, JoinHandle};
use ::prelude::*;
use ::{game, Result, Error, ErrorKind};

/// Builder for a subprocess (child) AI
pub struct Interface {
    command: Command,
    is_yellow: bool,
}

impl Interface {
    pub fn new(command: Command) -> Interface {
        Interface {
            command: command,
            is_yellow: true,
        }
    }

    /// Whether the AI will play with the yellow team, blue otherwise.
    pub fn is_yellow(&mut self, is_yellow: bool) -> &mut Interface {
        self.is_yellow = is_yellow;
        self
    }

    /// Spawn a thread which spawns a child subprocess and feeds it the game_state at 60Hz and
    /// sends commands through tx.
    pub fn spawn(&mut self, game_state: game::SharedState, tx: Sender<game::Command>) -> Result<AiHandle> {
        use std::io::prelude::*;
        use std::io::{BufReader, BufWriter};

        debug!("AI is playing as {} with: {:?}", if self.is_yellow { "yellow" } else { "blue" }, self.command);

        let is_yellow = self.is_yellow;
        let mut child = try!(self.command.stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped()).spawn());

        info!("AI subprocess spawned");

        let child_in = try!(child.stdin.take().ok_or_else(|| Error::new(ErrorKind::Io, "missing stdin from child")));
        let child_out = try!(child.stdout.take().ok_or_else(|| Error::new(ErrorKind::Io, "missing stdout from child")));
        let child_err = try!(child.stderr.take().ok_or_else(|| Error::new(ErrorKind::Io, "missing stderr from child")));

        let thread_builder = Builder::new().name("child AI Debug".to_string());
        let debug_thread = try!(thread_builder.spawn(move || -> Result<()> {
            for line in BufReader::new(child_err).lines() {
                // TODO: infrastructure to redirect this, this is different from logging
                println!("AI debug: {}", try!(line));
            }
            Ok(())
        }));

        let thread_builder = Builder::new().name("child AI Game I/O".to_string());
        let child_thread = try!(thread_builder.spawn(move || -> Result<()> {

            macro_rules! throw_err {
                ( $( $tt:tt )* ) => {{
                    return Err(Error::new(ErrorKind::AiProtocol, format!($($tt)*)));
                }}
            }

            // TODO: optimize by using `with_capacity` after calculating a low but enough estimate
            let mut output = BufReader::new(child_out).lines();
            let mut input = BufWriter::new(child_in);

            let version = 1;
            try!(writeln!(input, "ROBOIME_AI_PROTOCOL {}", version));

            // flush to child stdin
            try!(input.flush());

            {
                let line = try!(match output.next() {
                    Some(thing) => thing,
                    None => throw_err!("expected a line"),
                });
                match line.as_ref() {
                    "COMPATIBLE 1" => {}
                    s if s.starts_with("COMPATIBLE") => throw_err!("AI not protocol compatible (implicit)"),
                    s if s.starts_with("NOT_COMPATIBLE") => throw_err!("AI not protocol compatible (explicit)"),
                    s => throw_err!("AI protocol compatibility error, output was '{}'", s),
                }
            }

            {
                let state = try!(game_state.wait_and_read());
                let geom = state.get_field_geom();

                // FIELD_LENGTH
                // FIELD_WIDTH
                // GOAL_WIDTH
                // CENTER_CIRCLE_RADIUS
                // DEFENSE_RADIUS
                // DEFENSE_STRETCH
                // FREE_KICK_FROM_DEFENSE_DIST
                // PENALTY_SPOT_FROM_FIELD_LINE_DIST
                // PENALTY_LINE_FROM_SPOT_DIST
                try!(writeln!(input, "{:.03} {:.03} {:.03} {:.03} {:.03} {:.03} {:.03} {:.03} {:.03}",
                    geom.field_length,
                    geom.field_width,
                    geom.goal_width,
                    geom.center_circle_radius,
                    geom.defense_radius,
                    geom.defense_stretch,
                    geom.free_kick_from_defense_dist,
                    geom.penalty_spot_from_field_line_dist,
                    geom.penalty_line_from_spot_dist,
                ));
            }

            // flush to child stdin
            try!(input.flush());

            loop {
                let state = try!(game_state.wait_and_read());
                let timestamp = state.get_timestamp();
                let counter = state.get_counter();

                let (robots_player, robots_opponent) = {
                    let robots_yellow = state.get_robots_yellow();
                    let robots_blue = state.get_robots_blue();
                    if is_yellow {
                        (robots_yellow, robots_blue)
                    } else {
                        (robots_blue, robots_yellow)
                    }
                };

                // COUNTER
                // TIMESTAMP
                // REFEREE_STATE
                // REFEREE_TIME_LEFT
                // SCORE_PLAYER
                // SCORE_OPPONENT
                // GOALIE_ID_PLAYER
                // GOALIE_ID_OPPONENT
                // ROBOT_COUNT_PLAYER
                // ROBOT_COUNT_OPPONENT
                try!(writeln!(input, "{} {} {} {} {} {} {} {} {} {}",
                    counter,
                    timestamp,
                    'N', // TODO: REFEREE_STATE
                    -1, //  TODO: REFEREE_TIME_LEFT
                    0, //   TODO: SCORE_PLAYER
                    0, //   TODO: SCORE_OPPONENT
                    0, //   TODO: GOALIE_ID_PLAYER
                    0, //   TODO: GOALIE_ID_OPPONENT
                    robots_player.len(),
                    robots_opponent.len(),
                ));

                let ball = state.get_ball();

                // BALL_X
                // BALL_Y
                // BALL_VX
                // BALL_VY
                try!(writeln!(input, "{:.04} {:.04} {:.04} {:.04}",
                    ball.get_x(),
                    ball.get_y(),
                    ball.get_vx(),
                    ball.get_vy(),
                ));

                // ROBOT_COUNT_PLAYER x
                for (robot_id, robot) in robots_player {
                    // ROBOT_ID
                    // ROBOT_X
                    // ROBOT_Y
                    // ROBOT_W
                    // ROBOT_VX
                    // ROBOT_VY
                    // ROBOT_VW
                    try!(writeln!(input, "{} {:.04} {:.04} {:.04} {:.04} {:.04} {:.04}",
                        robot_id,
                        robot.get_x(),
                        robot.get_y(),
                        robot.get_w(),
                        robot.get_vx(),
                        robot.get_vy(),
                        robot.get_vw(),
                    ));
                }

                // ROBOT_COUNT_OPPONENT x
                for (robot_id, robot) in robots_opponent {
                    // ROBOT_ID
                    // ROBOT_X
                    // ROBOT_Y
                    // ROBOT_W
                    // ROBOT_VX
                    // ROBOT_VY
                    // ROBOT_VW
                    try!(writeln!(input, "{} {:.04} {:.04} {:.04} {:.04} {:.04} {:.04}",
                        robot_id,
                        robot.get_x(),
                        robot.get_y(),
                        robot.get_w(),
                        robot.get_vx(),
                        robot.get_vy(),
                        robot.get_vw(),
                     ));
                }

                // flush to child stdin
                try!(input.flush());

                {
                    let line = try!(match output.next() {
                        Some(thing) => thing,
                        None => throw_err!("expected a line"),
                    });
                    // COUNTER
                    let ai_counter: u64 = try!(line.parse());
                    if ai_counter != counter {
                        throw_err!("wrong command counter, expected {} got {}", counter, ai_counter);
                    }
                }

                let mut command = game::Command::new(is_yellow);
                {
                    let mut robot_commands = &mut command.robots;

                    // ROBOT_COUNT_PLAYER x
                    for (robot_id, _) in robots_player {

                        let line = try!(match output.next() {
                            Some(thing) => thing,
                            None => throw_err!("expected a line"),
                        });

                        let vars: Vec<_> = line.split(' ').collect();
                        let vars_len = vars.len();
                        if vars_len != 6 {
                            throw_err!("expected 6 values for robot command, got {}", vars_len);
                        }

                        // V_TANGENT
                        // V_NORMAL
                        // V_ANGULAR
                        // KICK_FORCE
                        // CHIP_FORCE
                        // DRIBBLE
                        let v_tangent:  f32 = try!(vars[0].parse());
                        let v_normal:   f32 = try!(vars[1].parse());
                        let v_angular:  f32 = try!(vars[2].parse());
                        let kick_force: f32 = try!(vars[3].parse());
                        let chip_force: f32 = try!(vars[4].parse());
                        let dribble:   bool = try!(vars[5].parse::<i32>()) == 1;

                        robot_commands.insert(*robot_id, game::RobotCommand {
                            v_tangent: v_tangent,
                            v_normal: v_normal,
                            v_angular: v_angular,
                            action: if kick_force > 0.0 {
                                game::RobotAction::Kick(kick_force)
                            } else if chip_force > 0.0 {
                                game::RobotAction::ChipKick(chip_force)
                            } else if dribble {
                                game::RobotAction::Dribble
                            } else {
                                game::RobotAction::Normal
                            }
                        });
                    }
                }

                match tx.send(command) {
                    Ok(_) => (),
                    Err(_) => break
                }
            }

            Ok(())
        }));

        Ok(AiHandle {
            subproc_handle: child,
            debug_handle: debug_thread,
            child_handle: child_thread,
        })
    }
}

pub struct AiHandle {
    subproc_handle: Child,
    debug_handle: JoinHandle<Result<()>>,
    child_handle: JoinHandle<Result<()>>,
}

impl InterfaceHandle for AiHandle {
    fn join(self) -> Result<()> {
        let AiHandle {
            subproc_handle: mut child,
            debug_handle: debug_thread,
            child_handle: child_thread,
        } = self;
        let child_result = try!(child_thread.join());
        let debug_result = try!(debug_thread.join());
        let exit_status = try!(child.wait());
        try!(child_result);
        try!(debug_result);
        if exit_status.success() {
            Ok(())
        } else {
            Err(Error::new(ErrorKind::Io, "child AI exited with failure"))
        }
    }
}
