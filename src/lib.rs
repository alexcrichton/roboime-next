//!
//! Currently this is a simlpe proof of concpet.
//!

extern crate roboime_next_protocol as protocol;
extern crate net2;
extern crate time;

pub use self::error::{Result, Error, ErrorKind};
pub use subproc::{CommandExt, ChildExt, ChildAiBuilder};
pub use state::{GameState, RobotState, BallState, Position, Pose, SharedGameState, new_shared_game_state};
pub use interface::{GrSimUpdaterBuilder, GrSimCommanderBuilder};

mod error;
mod subproc;
mod state;
mod interface;
