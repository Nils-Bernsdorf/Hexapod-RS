use crate::{Vector2, Point2, Point3, Vector3, Rotation3, Angle};
use crate::config::{WALKING_TRANSLATING_RESOLUTION, WALKING_ROTATING_RESOLUTION};
use crate::utils::clamp_abs;
use std::cell::RefCell;
use std::borrow::BorrowMut;

pub struct Isometry3 {
    rotation: Rotation3,
    inv_rotation: Rotation3,
    translation: Vector3,
}

impl Isometry3 {
    pub fn identity() -> Self {
        Self {
            rotation: Rotation3::identity(),
            inv_rotation: Rotation3::identity(),
            translation: Vector3::zero(),
        }
    }

    pub fn new(translation: Vector3, rotation: Rotation3) -> Self {
        Self {
            rotation, translation,
            inv_rotation: rotation.inverse(),
        }
    }

    pub fn rotate(&mut self, roll: Angle, pitch: Angle, yaw: Angle) {
        let delta = Rotation3::euler(roll, pitch, yaw);
        self.rotation = self.rotation.then(&delta);
        self.inv_rotation = self.rotation.inverse();
    }

    pub fn translate(&mut self, offset: Vector3){
        self.translation += offset;
    }

    pub fn translation(&self) -> Vector3 {
        self.translation
    }
    pub fn rotation(&self) -> Rotation3 {
        self.rotation
    }

    pub fn transform_point(&self, point: Point3) -> Point3{
        self.rotation.transform_point3d(point) + self.translation
    }

    pub fn inv_transform_point(&self, point: Point3) -> Point3{
        self.inv_rotation.transform_point3d(point - self.translation)
    }

    pub fn transform_vector(&self, vec: Vector3) -> Vector3{
        self.rotation.transform_vector3d(vec)
    }

    pub fn inv_transform_vector(&self, vec: Vector3) -> Vector3{
        self.inv_rotation.transform_vector3d(vec)
    }
}