use crate::config::Config;
use crate::input_handler::ControllerEvent;
use crate::hexapod::Hexapod;

pub mod idle;
//pub mod walking;
pub mod gait;
pub mod move_body;

pub trait Mode {
    fn new() -> Self where Self: Sized;
    fn handle_input(&mut self, input: &ControllerEvent, hexapod: &mut Hexapod, config: &Config);
    fn return_to_idle(&mut self, hexapod: &mut Hexapod, config: &Config) -> bool;
}
