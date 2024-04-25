use euclid::Rotation3D;
use crate::config::{BODY_DIST_TO_GROUND, Config};
use crate::hexapod::Hexapod;
use crate::input_handler::ControllerEvent;
use crate::modes::Mode;
use crate::{Angle, Vector2, Vector3};

#[derive(Debug, Copy, Clone)]
pub struct MoveBodyMode{
    current_rot: Vector2
}

impl Mode for MoveBodyMode {
    fn new() -> Self {
        Self{
            current_rot: Vector2::zero()
        }
    }

    fn handle_input(&mut self, event: &ControllerEvent, hexapod: &mut Hexapod, conf: &Config){
        let desired = Vector3::new(event.lx * 15.0, event.ly * 15.0, BODY_DIST_TO_GROUND);
        let difference = desired - hexapod.bodyTransform.translation;
        hexapod.bodyTransform.translation += difference.with_max_length(conf.walking_translating_resolution*0.5);

        let desired = Vector2::new(event.rx, event.ry) * 0.25;
        let difference = desired - self.current_rot;
        self.current_rot += difference.with_max_length(conf.walking_rotating_resolution.radians);
        hexapod.bodyTransform.rotation = Rotation3D::euler(Angle::radians(-self.current_rot.y), Angle::radians(self.current_rot.x), Angle::radians(0.));
    }

    fn return_to_idle(&mut self, hexapod: &mut Hexapod, conf: &Config) -> bool {
        self.handle_input(&ControllerEvent::default(), hexapod, conf);
        let offset = hexapod.bodyTransform.translation - Vector3::new(0.0, 0.0, BODY_DIST_TO_GROUND);
        offset.length() < 0.1
    }
}