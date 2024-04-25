use crate::config::{BODY_DIST_TO_GROUND, Config};
use crate::hexapod::Hexapod;
use crate::input_handler::ControllerEvent;
use crate::modes::Mode;
use crate::utils::clamp_abs;

#[derive(Debug, Copy, Clone)]
pub struct IdleMode{}

impl Mode for IdleMode {
    fn new() -> Self { Self{} }

    fn handle_input(&mut self, _: &ControllerEvent, hexapod: &mut Hexapod, conf: &Config){
        move_body_to_height(hexapod, 0.0, conf);
    }

    fn return_to_idle(&mut self, hexapod: &mut Hexapod, conf: &Config) -> bool {
        move_body_to_height(hexapod, BODY_DIST_TO_GROUND, conf)
    }
}

fn move_body_to_height(hexapod: &mut Hexapod, height: f64, conf: &Config) -> bool {
    let delta = height - hexapod.bodyTransform.translation.z;
    hexapod.bodyTransform.translation.z += clamp_abs(delta, conf.walking_translating_resolution);
    delta < conf.walking_translating_resolution*0.1
}