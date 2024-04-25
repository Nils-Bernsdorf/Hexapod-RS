use crate::Angle;
use std::f64::consts::PI;
use splines::{Interpolation, Key, Spline};

pub struct Config {
    pub walking_translating_resolution: f64,
    pub walking_rotating_resolution: Angle,
    pub walking_max_step_dist: f64,
    pub walking_max_rotation_dist: Angle,
    pub anim_timestep: f64,
    pub walking_step_height: f64,
    pub foot_height: Spline<f64, f64>, //maps the current foot progress to the desired foot height
}

impl Default for Config {
    fn default() -> Self {
        let foot_progress_keyframes = vec![
            Key::new(0.0, 0.0, Interpolation::Linear),
            Key::new(0.2, 1.0, Interpolation::Linear),
            Key::new(0.8, 1.0, Interpolation::Linear),
            Key::new(1.0, 0.0, Interpolation::Linear),
        ];

        Self {
            walking_translating_resolution: 14.0,
            walking_rotating_resolution: Angle::degrees(4.0),
            walking_max_step_dist: 50.0,
            walking_max_rotation_dist: Angle::degrees(35.0),
            anim_timestep: 0.03,
            walking_step_height: 15.0,
            foot_height: Spline::from_vec(foot_progress_keyframes),
        }
    }
}

//HEXAPOD DIMENSIONS
pub const BODY_WIDTH: f64 = 78.0;
pub const BODY_HEIGHT: f64 = 140.0;
pub const BODY_WIDTH_MIDDLE: f64 = 105.0;

pub const HIP_LENGTH: f64 = 28.0;
pub const UPPER_LEG_LENGTH: f64 = 43.0;
pub const LOWER_LEG_LENGTH: f64 = 92.0;

pub const CORNER_JOINT_ROTATION: f64 = PI/4.0;

//INITIAL POSITION
pub const CENTER_TO_FOOT_X: f64 = 85.0;
pub const CENTER_TO_FOOT_Y: f64 = 130.0;
pub const MIDDLE_FOOT_OFFSET: f64 = 50.0;
pub const BODY_DIST_TO_GROUND: f64 = 70.0;

//INPUT HANDLING
pub const INPUT_MIN_MAG: f64 = 0.1;
pub const INPUT_FINALIZED_DELAY: u128 = 250; //ms


