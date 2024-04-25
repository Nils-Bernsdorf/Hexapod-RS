use crate::{Point2, Isometry2, Vector2, Point3, Isometry3, Vector3, Transform3, Rotation3, Angle};
use crate::config::{CENTER_TO_FOOT_X, CENTER_TO_FOOT_Y, MIDDLE_FOOT_OFFSET, BODY_WIDTH, BODY_HEIGHT, BODY_WIDTH_MIDDLE, BODY_DIST_TO_GROUND, CORNER_JOINT_ROTATION};
use std::ops::{Deref, DerefMut};
use crate::telemetry::TelemetryMessage;
use crate::leg::Leg;
use std::f64::consts::PI;

pub struct Hexapod{
    pub origin: Isometry2, //2d position and rotation of the center
    pub bodyTransform: Isometry3, //3d body offset and rotation relative to origin
    legs: [Leg; 6],
    legJoints: [Vector2; 6],
    feet: [Point3; 6],
}

impl Hexapod{
    pub fn new() -> Self{
        let mut this = Self{
            origin: Isometry2::identity(),
            legs: [Leg::new(); 6],
            bodyTransform: Isometry3::identity(),
            legJoints:  [Vector2::zero(); 6],
            feet: [Point3::zero(); 6],
        };
        this.bodyTransform.translation.z = BODY_DIST_TO_GROUND;
        this.bodyTransform.rotation = Rotation3::identity();
        for foot in Foot::all() {
            this.set_abs_foot_pos(foot, foot.initial_foot_pos());
            this.legJoints[foot as usize] = foot.leg_joint_pos();
        }
        this
    }

    //TODO: remove getter and setter because legs not only need to be updated when the foot pos changes
    pub fn get_abs_foot_pos(&self, foot: Foot) -> Point3{
        self.feet[foot as usize].clone()
    }

    pub fn set_abs_foot_pos(&mut self, foot: Foot, pos: Point3){
        self.feet[foot as usize] = pos;
    }

    pub fn get_center(&self) -> [f64; 2] {
        self.origin.translation().to_array()
    }

    pub fn get_foot_pos(&self, foot: Foot) -> [f64; 3] {
        self.feet[foot as usize].into()
    }

    pub fn update_all_legs(&mut self) -> Transform3 {
        let transform = self.bodyTransform.to_transform().then(&self.origin.get_transform().to_3d());
        let inv_transform = transform.inverse().unwrap();
        for foot in Foot::all(){
            let mut rel_pos = inv_transform.transform_point3d(self.feet[foot as usize]).unwrap();
            rel_pos -= self.legJoints[foot as usize].to_3d();
            self.legs[foot as usize].set_rel_foot_pos(rel_pos);
        }
        transform
    }

    pub fn get_telemetry(&mut self) -> TelemetryMessage {
        let transform = self.update_all_legs();
        TelemetryMessage {
            center: transform.transform_point2d(Point2::origin()).unwrap().to_array(),
            rotation: transform.transform_vector2d(Vector2::new(1., 0.)).angle_from_x_axis().radians,
            legs: Foot::all().map(|f| {self.legs[f as usize].get_telemetry(self.legJoints[f as usize], &transform) }),
            angles: self.get_angles(),
        }
    }

    pub fn get_angles(&self) -> [Option<f64>; 6*3]{
        let mut result = [None; 6*3];
        for foot in Foot::all(){
            let angles = self.legs[foot as usize].get_angles(foot.leg_joint_orientation());
            if let Some(angles) = angles {
                for (i, angle) in angles.iter().enumerate() {
                    result[foot as usize * 3 + i] = Some(angle.signed().radians);
                }
            }
        }
        result
    }

    /*
    pub fn assert_initial_pos(&self) {
        for foot in Foot::all() {
            assert_point_equal(*self.feet[foot as usize].to_relative(&self.origin), *foot.initial_pos());
        }
    }*/
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Foot{
    RIGHT_FRONT,
    RIGHT_MIDDLE,
    RIGHT_BACK,
    LEFT_BACK,
    LEFT_MIDDLE,
    LEFT_FRONT
}

impl Foot{
    pub fn all() -> [Foot; 6] {
        [Self::RIGHT_FRONT, Self::RIGHT_MIDDLE, Self::RIGHT_BACK, Self::LEFT_BACK, Self::LEFT_MIDDLE, Self::LEFT_FRONT]
    }

    pub fn odd() -> [Foot; 3] {
        [Self::RIGHT_MIDDLE, Self::LEFT_BACK, Self::LEFT_FRONT]
    }

    pub fn even() -> [Foot; 3] {
        [Self::RIGHT_FRONT, Self::RIGHT_BACK, Self::LEFT_MIDDLE]
    }

    pub fn is_even(&self) -> bool{
        match self {
            Self::RIGHT_FRONT | Self::RIGHT_BACK | Self::LEFT_MIDDLE => true,
            _ => false,
        }
    }

    pub fn is_right(&self) -> bool {
        match self {
            Self::RIGHT_BACK | Self::RIGHT_MIDDLE | Self::RIGHT_FRONT => true,
            Self::LEFT_BACK | Self::LEFT_MIDDLE | Self::LEFT_FRONT => false,
        }
    }

    pub fn get_mult_x(&self) -> f64 {
        if self.is_right() { 1.0 } else { -1.0 }
    }

    pub fn get_mult_y(&self) -> f64 {
        match self {
            Self::RIGHT_FRONT | Self::LEFT_FRONT => 1.0,
            Self::RIGHT_MIDDLE | Self::LEFT_MIDDLE => 0.0,
            Self::RIGHT_BACK | Self::LEFT_BACK => -1.0,
        }
    }

    pub fn is_middle(&self) -> bool {
        match self {
             Self::RIGHT_MIDDLE | Self::LEFT_MIDDLE => true,
            _ => false
        }
    }

    pub fn leg_joint_pos(&self) -> Vector2{
        Vector2::new(
            self.get_mult_x() * if self.is_middle() { BODY_WIDTH_MIDDLE } else { BODY_WIDTH }/2.0,
            self.get_mult_y() * BODY_HEIGHT/2.0
        )
    }

    pub fn initial_foot_pos(&self) -> Point3 {
        let mut pos = Point3::new(self.get_mult_x() * CENTER_TO_FOOT_X, self.get_mult_y() * CENTER_TO_FOOT_Y, 0.0);
        if self.is_middle() {
            pos.x += MIDDLE_FOOT_OFFSET * self.get_mult_x();
        }
        pos
    }

    pub fn leg_joint_orientation(&self) -> Vector2 {
        let mut angle = self.get_mult_y()*CORNER_JOINT_ROTATION;
        if !self.is_right() {
            angle = PI - angle;
        }
        Vector2::from_angle_and_length(Angle::radians(angle), 1.0)
    }

    pub fn id(&self) -> usize {
        *self as usize
    }
}