use crate::{Isometry2, Vector3, Vector2, Point2, Translation2, Angle, Rotation2};
use std::time::Instant;
use splines::{Interpolation, Key, Spline};
use crate::config::{INPUT_MIN_MAG, INPUT_FINALIZED_DELAY, Config};
use crate::hexapod::{Hexapod, Foot};
use crate::input_handler::{Button, ControllerEvent};
use crate::modes::Mode;
use crate::utils::clamp_abs;

pub struct GaitEngine{
    state: State,
    gait: GaitType,
    last_input: WalkingInput,
    last_input_change: Instant,
    input_finalized: bool,
}

#[derive(Copy, Clone)]
enum State {
    IDLE,
    STEP(WalkingState),
}

#[derive(Copy, Clone)]
struct WalkingState{
    gait: GaitInfo,
    feet: [FootState; 6],
    centers: [Isometry2; 6],
    fixed_center: Isometry2,
    step_progress: f64
}

impl WalkingState {
    pub fn new(gait: &GaitType, origin: Isometry2) -> Self {
        Self {
            gait: gait.get_config(),
            feet: [FootState::STANDING(origin, false); 6],
            centers: [origin; 6],
            fixed_center: origin,
            step_progress: 0.
        }
    }

    pub fn do_step(mut self, input: &WalkingInput, mut hexapod: &mut Hexapod, conf: &Config) -> State {
        for foot in Foot::all() {
            let (progress, height) = self.gait.get_foot_state(foot, self.step_progress, conf);

            let should_step = (progress > 0. && progress < 1.);

            self.feet[foot.id()] = match self.feet[foot.id()] {
                FootState::STEPPING(from, to, is_idle) if !should_step => {
                    FootState::STANDING(to, is_idle)
                },
                FootState::STANDING(pos, _) if should_step => {
                    if input.is_significant() {
                        let translation = self.fixed_center.rotation().transform_vector(input.translation());
                        let desired_pos = translation * 2. * conf.walking_max_step_dist * self.gait.step_weight[foot.id()] + self.fixed_center.translation();
                        let desired_rot = (conf.walking_max_rotation_dist * input.rot()) * self.gait.step_weight[foot.id()] + self.fixed_center.rotation().get_angle();
                        let desired = Isometry2::new(desired_pos, Rotation2::new(desired_rot));

                        self.fixed_center = desired;
                        FootState::STEPPING(pos, desired, false)
                    } else if pos.approx_eq(&self.fixed_center) {
                        FootState::STANDING(pos, true)
                    } else {
                        FootState::STEPPING(pos, self.fixed_center, true)
                    }
                },
                other => other
            };

            self.centers[foot.id()] = self.feet[foot.id()].current_pos(progress);

            if let FootState::STEPPING(_,_,_) = self.feet[foot.id()] {
                let mut new_pos = self.centers[foot.id()].transform_point3(foot.initial_foot_pos());
                new_pos.z = height * conf.walking_step_height;
                hexapod.set_abs_foot_pos(foot, new_pos);
            }
        }

        hexapod.origin.center_between_many(&self.centers);

        self.step_progress += conf.anim_timestep;
        if self.step_progress >= self.gait.duration * 2. {
            self.step_progress -= self.gait.duration;
        }

        if self.feet.iter().all(|f| f.can_return_to_idle()) {
            State::IDLE
        } else {
            State::STEP(self)
        }
    }
}

impl Mode for GaitEngine{
    fn new() -> Self {
        Self {
            state: State::IDLE,
            gait: GaitType::default(),
            last_input: WalkingInput::new(0.0, 0.0, 0.0),
            last_input_change: Instant::now(),
            input_finalized: false,
        }
    }

    fn handle_input(&mut self, input: &ControllerEvent, hexapod: &mut Hexapod, conf: &Config) {
        self.next_step(input, hexapod, conf);
    }

    fn return_to_idle(&mut self, hexapod: &mut Hexapod, conf: &Config) -> bool {
        self.next_step(&ControllerEvent::default(), hexapod, conf);
        matches!(self.state, State::IDLE)
    }
}

impl GaitEngine {
    pub fn next_step(&mut self, event: &ControllerEvent, mut hexapod: &mut Hexapod, conf: &Config){
        let input = WalkingInput::from(event);
        self.handle_input(&input);

        self.state = match self.state {
            State::IDLE => {
                if event.triggered(Button::LEFT) {
                    self.gait = self.gait.prev();
                } else if event.triggered(Button::RIGHT) {
                    self.gait = self.gait.next();
                }

                if input.is_significant() {
                    State::STEP(WalkingState::new(&self.gait, hexapod.origin))
                } else {
                    State::IDLE
                }
            },
            State::STEP(state) => {
                state.do_step(&input, hexapod, conf)
            }
        }
    }

    fn reset_input_finalized(&mut self) {
        self.input_finalized = false;
        self.last_input_change = Instant::now();
    }

    //TODO: move to own struct
    fn handle_input(&mut self, input: &WalkingInput){
        if self.last_input.is_similar_to(input) == false {
            self.last_input_change = Instant::now();
        }
        self.last_input = input.clone();
        self.input_finalized = self.last_input_change.elapsed().as_millis() > INPUT_FINALIZED_DELAY;
    }
}

//TODO: think of a better name
#[derive(Copy, Clone)]
struct DirectionEngine {
    pos: Isometry2,
    idle_steps: usize,
}

impl DirectionEngine {
    pub fn get_next_pos(&mut self, input: &WalkingInput, foot: Foot, gait: &GaitInfo, conf: &Config) -> Isometry2{
        if !input.is_significant() {
            self.idle_steps += 1;
            return self.pos;
        } else {
            self.idle_steps = 0;
        }

        let translation = self.pos.rotation().transform_vector(input.translation());
        let desired_pos = translation * 2. * conf.walking_max_step_dist * gait.step_weight[foot.id()] + self.pos.translation();
        let desired_rot = (conf.walking_max_rotation_dist * input.rot()) * gait.step_weight[foot.id()] + self.pos.rotation().get_angle();
        let next_pos = Isometry2::new(desired_pos, Rotation2::new(desired_rot));

        self.pos = next_pos;
        next_pos
    }
}

#[derive(Copy, Clone)]
enum FootState {
    STANDING(Isometry2, bool),
    STEPPING(Isometry2, Isometry2, bool),
}

impl FootState {
    pub fn can_return_to_idle(&self) -> bool {
        matches!(self, Self::STANDING(_, true))
    }

    pub fn current_pos(&self, progress: f64) -> Isometry2 {
        match self {
            Self::STEPPING(from, to, _) => Isometry2::lerp(&from, &to, progress),
            Self::STANDING(pos, _) => *pos
        }
    }
}

#[derive(Debug, Copy, Clone)]
enum GaitType {
    TRIPOD,
    DELAYED_TRIPOD,
    RIPPLE,
    WAVE
}

impl GaitType {
    fn get_config(&self) -> GaitInfo {
        match *self {
            Self::TRIPOD => GaitInfo::from_parts(
                [0.0, 1.0, 0.0, 1.0, 0.0, 1.0],
                [0.5, 0.5, 0., 0., 0., 0.], 2.0
            ),
            Self::DELAYED_TRIPOD => GaitInfo::from_parts(
                [0.0, 1.2, 0.2, 1.4, 0.4, 1.6],
                [0.4, 0.4, 0.05, 0.05, 0.05, 0.05], 2.4
            ),
            Self::RIPPLE => GaitInfo::from_parts(
                [1.0, 0.0, 2.0, 0.5, 1.5, 2.5],
                [1. / 6.; 6], 3.0
            ),
            Self::WAVE => GaitInfo::from_step_start(
                [0.0, 1.0, 2.0, 3.0, 4.0, 5.0]
            )
        }
    }

    fn next(&self) -> Self {
        match *self {
            Self::TRIPOD => Self::DELAYED_TRIPOD,
            Self::DELAYED_TRIPOD => Self::RIPPLE,
            Self::RIPPLE => Self::WAVE,
            Self::WAVE => Self::TRIPOD
        }
    }

    fn prev(&self) -> Self {
        match *self {
            Self::TRIPOD => Self::WAVE,
            Self::DELAYED_TRIPOD => Self::TRIPOD,
            Self::RIPPLE => Self::DELAYED_TRIPOD,
            Self::WAVE => Self::RIPPLE
        }
    }
}

impl Default for GaitType{
    fn default() -> Self {
        Self::TRIPOD
    }
}

#[derive(Debug, Copy, Clone)]
struct GaitInfo {
    step_start: [f64; 6], //stores the time each foot begins its step
    step_weight: [f64; 6], //how much each step should influence the direction of travel, should add up to 1
    duration: f64, //the duration of one cycle
}

impl GaitInfo {
    fn from_step_start(step_start: [f64; 6]) -> Self {
        Self::from_parts(step_start, [1./6.; 6], step_start.iter().copied().fold(f64::NAN, f64::max) + 1.0)
    }

    fn from_parts(step_start: [f64; 6], step_weight: [f64; 6], duration: f64) -> Self {
        Self {
            step_start, duration, step_weight,
            //foot_height: Self::default_foot_height(),
        }
    }

    pub fn get_foot_state(&self, foot: Foot, time: f64, config: &Config) -> (f64, f64) {
        assert!(time >= 0.);
        let delta = time - self.step_start[foot.id()];
        let progress = (delta % self.duration).clamp(0.0, 1.0);
        let height = config.foot_height.clamped_sample(progress).unwrap();
        (progress, height)
    }
}

//TODO: combine with identical struct in walking.rs
#[derive(Clone, Debug)]
pub struct WalkingInput {
    inner: Vector3
}

impl WalkingInput {
    pub fn new(mut x: f64, mut y: f64, mut rot: f64) -> Self{
        if x.abs() < INPUT_MIN_MAG { x = 0.0 }
        if y.abs() < INPUT_MIN_MAG { y = 0.0 }
        if rot.abs() < INPUT_MIN_MAG { rot = 0.0 }
        Self{ inner: Vector3::new(x, y, -rot) }
    }

    pub fn is_similar_to(&self, other: &WalkingInput) -> bool{
        (self.inner - other.inner).length() < INPUT_MIN_MAG
    }

    pub fn is_significant(&self) -> bool{
        self.inner.length() > INPUT_MIN_MAG
    }

    pub fn translation(&self) -> Vector2 { self.inner.xy() }
    pub fn rot(&self) -> f64 { self.inner.z }
}

impl From<&ControllerEvent> for WalkingInput {
    fn from(event: &ControllerEvent) -> Self {
        Self::new(event.lx, event.ly, event.rx)
    }
}