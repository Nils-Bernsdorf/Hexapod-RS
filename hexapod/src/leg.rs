use crate::{Point3, Rotation2, Vector2, Vector3, Isometry2, Transform3, Angle};
use crate::config::{HIP_LENGTH, UPPER_LEG_LENGTH, LOWER_LEG_LENGTH};
use crate::telemetry::LegTelemetry;
use std::f64::consts::PI;

#[derive(Debug, Copy, Clone)]
pub struct Leg{
    //joint positions: (all relative to the leg origin)
    hip: Point3,
    knee: Point3,
    foot: Point3,
    invalid_state: bool, //the desired foot pos can not be reached
}

impl Leg{
    pub fn new() -> Leg {
        Leg {
            hip: Point3::zero(),
            knee: Point3::zero(),
            foot: Point3::zero(),
            invalid_state: false,
        }
    }

    pub fn set_rel_foot_pos(&mut self, foot_pos: Point3){
        self.foot = foot_pos;

        //hip always points towards the foot in the xy-plane
        let mut origin_to_hip = self.foot.to_vector();
        origin_to_hip.z = 0.0;
        self.hip = (origin_to_hip.normalize() * HIP_LENGTH).to_point();

        //the hip, knee and foot form a triangle with sides UPPER_LEG_LENGTH,
        //LOWER_LEG_LENGTH and hip_foot_dist. The angle for the vertical hip servo (=beta)
        //can be solved using the law of cosines
        let hip_to_foot = self.foot - self.hip;
        let hip_foot_dist = hip_to_foot.length();
        let beta = ((UPPER_LEG_LENGTH.powi(2) + hip_foot_dist.powi(2) - LOWER_LEG_LENGTH.powi(2)) /
            (2.0 * UPPER_LEG_LENGTH * hip_foot_dist)).acos();
        if beta.is_nan() {
            self.invalid_state = true;
            return;
        }

        //now switch to a 2d side view
        let hip_to_foot_top_view = hip_to_foot.xy();
        let hip_to_foot_side_view = Vector2::new(hip_to_foot_top_view.length(), hip_to_foot.z);
        let rot = Rotation2::new(Angle::radians(beta));
        let mut hip_to_knee_side_view = rot.transform_vector(hip_to_foot_side_view);
        hip_to_knee_side_view = hip_to_knee_side_view.normalize() * UPPER_LEG_LENGTH;

        //the 3d hip_to_knee vector can now be calculated using hip_to_knee_side_view
        let knee_height = hip_to_knee_side_view.y + self.hip.z;
        let knee_dist_xy = hip_to_knee_side_view.x;
        let mut hip_to_knee_top_view = hip_to_foot_top_view;
        hip_to_knee_top_view = hip_to_knee_top_view.normalize() * knee_dist_xy;
        let hip_to_knee = hip_to_knee_top_view.extend(knee_height);
        self.knee = self.hip + hip_to_knee;

        self.invalid_state = false;
    }

    //returns the angles of the three servos in the order [hip_xy, hip_z, knee]
    pub fn get_angles(&self, orientation: Vector2) -> Option<[Angle; 3]>{
        if self.invalid_state { return None; }
        let alpha = orientation.angle_to(self.hip.xy().to_vector());
        let hip_to_knee = self.knee - self.hip;
        let beta = Vector2::new(hip_to_knee.xy().length(), hip_to_knee.z).angle_from_x_axis();
        let gamma = (self.hip - self.knee).angle_to(self.foot - self.knee) + Angle::radians(-PI/2.0);
        Some([alpha, beta, gamma])
    }

    pub fn get_telemetry(&self, joint: Vector2, body_to_word: &Transform3) -> LegTelemetry {
        let joint_3d = joint.to_3d();
        LegTelemetry {
            joint: body_to_word.transform_point3d(joint_3d.to_point()).unwrap().to_array(),
            hip: body_to_word.transform_point3d(self.hip + joint_3d).unwrap().to_array(),
            knee: body_to_word.transform_point3d(self.knee + joint_3d).unwrap().to_array(),
            foot: body_to_word.transform_point3d(self.foot + joint_3d).unwrap().to_array(),
        }
    }
}