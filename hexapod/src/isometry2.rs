use crate::{Rotation2, Vector2, Transform2, Point2, Point3, Vector3};
use euclid::Transform2D;
use crate::utils::clamp_abs;
use std::cell::RefCell;
use std::borrow::BorrowMut;
use euclid::approxeq::ApproxEq;
use crate::config::Config;

#[derive(Copy, Clone)]
pub struct Isometry2 {
    rotation: Rotation2,
    translation: Vector2,
}

impl Isometry2 {
    pub fn identity() -> Self {
        Self {
            rotation: Rotation2::identity(),
            translation: Vector2::zero(),
        }
    }

    pub fn new(translation: Vector2, rotation: Rotation2) -> Self {
        Self {
            rotation, translation,
        }
    }

    pub fn step_towards(&mut self, other: &Isometry2, conf: &Config) -> bool {
        let difference = other.translation - self.translation;
        self.translation += difference.with_max_length(conf.walking_translating_resolution);

        let estimated_steps = difference.length() / conf.walking_translating_resolution;

        let mut angle_delta = self.rotation.get_angle().angle_to(other.rotation.get_angle());
        if estimated_steps > 1.0 {
            angle_delta /= estimated_steps;
        }
        self.rotation.angle += clamp_abs(angle_delta.radians, conf.walking_rotating_resolution.radians);

        estimated_steps < 0.1 && angle_delta.radians.abs() < conf.walking_rotating_resolution.radians * 0.1
    }

    pub fn center_between(&mut self, a: &Isometry2, b: &Isometry2) {
        self.translation = (a.translation + b.translation) / 2.0;
        self.rotation.angle = (a.rotation.angle + b.rotation.angle) / 2.0;
    }

    pub fn center_between_many(&mut self, all: &[Isometry2]){
        self.translation = all.iter().map(|e| e.translation).sum::<Vector2>() / all.len() as f64;
        self.rotation.angle = all.iter().map(|e| e.rotation.angle).sum::<f64>() / all.len() as f64;
    }

    pub fn lerp(a: &Isometry2, b: &Isometry2, t: f64) -> Self {
        let translation = a.translation.lerp(b.translation, t);
        let rotation = Rotation2::new(a.rotation.get_angle().lerp(b.rotation.get_angle(), t));
        Self::new(translation, rotation)
    }

    pub fn approx_eq(&self, other: &Isometry2) -> bool {
        self.translation.approx_eq(&other.translation) && self.rotation.get_angle().approx_eq(&other.rotation.get_angle())
    }

    pub fn translation(&self) -> Vector2 { self.translation }
    pub fn rotation(&self) -> Rotation2 { self.rotation }

    #[inline]
    pub fn get_transform(&self) -> Transform2 {
        self.rotation.to_transform().then(&self.translation.to_transform())
    }

    #[inline]
    fn get_inverse_transform(&self) -> Transform2 {
        self.rotation.inverse().to_transform().then(&self.translation.to_transform().inverse().unwrap())
    }

    pub fn transform_point(&self, point: Point2) -> Point2{
        self.rotation.transform_point(point) + self.translation
    }

    pub fn transform_point3(&self, point: Point3) -> Point3{
        self.transform_point(point.to_2d()).extend(point.z)
    }

    pub fn inv_transform_point(&self, point: Point2) -> Point2{
        self.rotation().inverse().transform_point(point - self.translation)
    }

    pub fn inv_transform_point3(&self, point: Point3) -> Point3{
        self.inv_transform_point(point.to_2d()).extend(point.z)
    }

    pub fn transform_vector(&self, vec: Vector2) -> Vector2{
        self.rotation.transform_vector(vec)
    }

    pub fn transform_vector3(&self, vec: Vector3) -> Vector3{
        self.transform_vector(vec.to_2d()).extend(vec.z)
    }

    pub fn inv_transform_vector(&self, vec: Vector2) -> Vector2{
        self.rotation.inverse().transform_vector(vec)
    }

    pub fn inv_transform_vector3(&self, vec: Vector3) -> Vector3{
        self.inv_transform_vector(vec.to_2d()).extend(vec.z)
    }
}

impl Default for Isometry2 {
    fn default() -> Self {
        Self::identity()
    }
}