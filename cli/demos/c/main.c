#include <stdio.h>
#include <string.h>
#include <math.h>

int main() {
    fprintf(stderr, "started\n");

    // Version check I/O

    const int compat_version = 1;
    char protocol_version_string[32];
    int version;
    scanf("%s %i", protocol_version_string, &version);
    if (strcmp(protocol_version_string, "ROBOIME_INTEL_PROTOCOL_VERSION") == 0 && version == compat_version) {
        printf("COMPATIBLE %i\n", compat_version);
    } else {
        printf("NOT_COMPATIBLE %i\n", compat_version);
        return 0;
    }
    fflush(stdout);

    fprintf(stderr, "compatible\n");

    // Geometry input

    float field_length;
    float field_width;
    float goal_width;
    float center_circle_radius;
    float defense_radius;
    float defense_stretch;
    float free_kick_from_defense_distance;
    float penalty_spot_from_field_line_dist;
    float penalty_line_from_spot_dist;

    scanf("%f %f %f %f %f %f %f %f %f",
        &field_length,
        &field_width,
        &goal_width,
        &center_circle_radius,
        &defense_radius,
        &defense_stretch,
        &free_kick_from_defense_distance,
        &penalty_spot_from_field_line_dist,
        &penalty_line_from_spot_dist
    );

    fprintf(stderr, "initialized\n");

    // Game state I/O

    while (1) {

        // State

        int ids[6];
        int ids_count = 0;
        float x = 0.0f, y = 0.0f, w = 0.0f;
        float tx = 0.0f, ty = 0.0f, tw = 0.0f;

        // Input

        int counter;
        float timestamp;
        char referee_state;
        float referee_time_left;
        int score_player;
        int score_opponent;
        int goalkeeper_id_player;
        int goalkeeper_id_opponent;
        int robot_num_player;
        int robot_num_opponent;
        float ball_x, ball_y, ball_vx, ball_vy;

        scanf("%i %f %c %f %i %i %i %i %i %i",
            &counter,
            &timestamp,
            &referee_state,
            &referee_time_left,
            &score_player,
            &score_opponent,
            &goalkeeper_id_player,
            &goalkeeper_id_opponent,
            &robot_num_player,
            &robot_num_opponent
        );

        scanf("%f %f %f %f",
            &ball_x,
            &ball_y,
            &ball_vx,
            &ball_vy
        );

        for (int i = 0; i < robot_num_player; ++i) {
            int robot_id;
            float robot_x, robot_y, robot_w, robot_vx, robot_vy, robot_vw;

            scanf("%i %f %f %f %f %f %f",
                &robot_id,
                &robot_x,
                &robot_y,
                &robot_w,
                &robot_vx,
                &robot_vy,
                &robot_vw
            );

            ids[ids_count++] = robot_id;
            if (robot_id == 0) {
                x = robot_x;
                y = robot_y;
                w = robot_w;
            }
        }

        for (int i = 0; i < robot_num_opponent; ++i) {
            int robot_id;
            float robot_x, robot_y, robot_w, robot_vx, robot_vy, robot_vw;

            scanf("%i %f %f %f %f %f %f",
                &robot_id,
                &robot_x,
                &robot_y,
                &robot_w,
                &robot_vx,
                &robot_vy,
                &robot_vw
            );
        }

        tx = ball_x;
        ty = ball_y;
        tw = 0;

        printf("%i\n", counter);

        for (int i = 0; i < ids_count; ++i) {
            float v_tan = 0.0f;
            float v_norm = 0.0f;
            float v_ang = 0.0f;
            float kick_x = 0.0f;
            float kick_z = 0.0f;
            int spin = 0;

            if (ids[i] == 0) {
                const float PL = 0.40f;
                const float PW = 0.80f;
                v_tan  = PL * ((tx - x) * cos(w) + (ty - y) * sin(w));
                v_norm = PL * ((ty - y) * cos(w) + (tx - x) * sin(w));
                v_ang  = PW * (tw - w);
                kick_x = 4.0f;
                kick_z = 0.0f;
                spin = 1;
            }

            printf("%f %f %f %f %f %i\n", v_tan, v_norm, v_ang, kick_x, kick_z, spin);
        }

        fflush(stdout);
    }
}
