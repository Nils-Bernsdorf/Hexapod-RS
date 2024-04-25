use euclid::UnknownUnit;

pub type Vector2 = euclid::Vector2D<f64, UnknownUnit>;
pub type Point2 = euclid::Point2D<f64, UnknownUnit>;
pub type Vector3 = euclid::Vector3D<f64, UnknownUnit>;
pub type Point3 = euclid::Point3D<f64, UnknownUnit>;
pub type Translation2 = euclid::Translation2D<f64, UnknownUnit, UnknownUnit>;
pub type Rotation2 = euclid::Rotation2D<f64, UnknownUnit, UnknownUnit>;
pub type Transform2 = euclid::Transform2D<f64, UnknownUnit, UnknownUnit>;
pub type Rotation3 = euclid::Rotation3D<f64, UnknownUnit, UnknownUnit>;
pub type Transform3 = euclid::Transform3D<f64, UnknownUnit, UnknownUnit>;
pub type Angle = euclid::Angle<f64>;
pub use isometry2::Isometry2;
//pub use isometry3::Isometry3;
pub type Isometry3 = euclid::RigidTransform3D<f64, UnknownUnit, UnknownUnit>;


pub mod hexapod;
pub mod telemetry;
pub mod input_handler;
pub mod config;
mod utils;
mod leg;
mod isometry2;
mod modes;
//mod isometry3;

