use crate::config::Config;
use crate::hexapod::Hexapod;
use crate::modes::gait::GaitEngine;
use crate::modes::idle::IdleMode;
use crate::modes::Mode;
use crate::modes::move_body::MoveBodyMode;
//use crate::modes::walking::WalkingEngine;

#[derive(Default, Debug, Copy, Clone)]
pub struct ControllerEvent{
    pub timestamp: usize,
    pub lx: f64,
    pub ly: f64,
    pub rx: f64,
    pub ry: f64,
    pub pressed: [bool; 12],
    pub triggered: [bool; 12],
}

pub enum Button{
    A, B, X, Y,
    L, R, ZL, ZR,
    UP, DOWN, LEFT, RIGHT
}

pub struct InputHandler{
    //current_mode: Box<dyn Mode>,
    state: State,
    last_timestamp: usize,
    modes: [Box<dyn Mode>;2],
}

#[derive(Debug, PartialEq, Copy, Clone)]
enum State {
    IN_MODE(usize),
    CHANGING(usize, usize),
}

impl InputHandler{
    pub fn new() -> Self {
        Self {
            state: State::IN_MODE(0),
            last_timestamp: 0,
            //modes: [Box::new(WalkingEngine::new()), Box::new(GaitEngine::new()), Box::new(MoveBodyMode::new())]
            modes: [Box::new(GaitEngine::new()), Box::new(MoveBodyMode::new())]
        }
    }

    pub fn handle_input(&mut self, mut input: ControllerEvent, hexapod: &mut Hexapod, conf: &mut Config){
        if input.timestamp == self.last_timestamp {
            //we have already received and processed this controller event
            //-> clear the triggered bit for each button, otherwise the application
            //will think the button has been pressed twice
            input.clear_triggered();
        }
        self.last_timestamp = input.timestamp;

        match self.state {
            State::IN_MODE(current_mode) => {
                if input.triggered(Button::ZR) {
                    self.state = State::CHANGING(current_mode, (current_mode+1) % self.modes.len());
                } else if input.triggered(Button::ZL) {
                    self.state = State::CHANGING(current_mode, (current_mode as isize-1).rem_euclid(self.modes.len() as isize) as usize);
                } else if input.triggered(Button::UP) {
                    conf.anim_timestep *= 1.2;
                    println!("{}", conf.anim_timestep);
                } else if input.triggered(Button::DOWN) {
                    conf.anim_timestep /= 1.2;
                    println!("{}", conf.anim_timestep);
                }

                self.modes[current_mode].handle_input(&input, hexapod, conf);
            },
            State::CHANGING(from_mode, to_mode) => {
                if self.modes[from_mode].return_to_idle(hexapod, conf) {
                    self.state = State::IN_MODE(to_mode);
                }
            }
        }

    }
}

impl ControllerEvent{
    pub fn clear_triggered(&mut self) {
        self.triggered.iter_mut().for_each(|t| *t = false);
    }

    pub fn pressed(&self, button: Button) -> bool {
        self.pressed[button as usize]
    }

    pub fn triggered(&self, button: Button) -> bool {
        self.triggered[button as usize]
    }
}