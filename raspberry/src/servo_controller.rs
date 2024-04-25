use pwm_pca9685::{Pca9685, Address, Channel};
use rppal::i2c::I2c;
use std::convert::TryFrom;
use std::f64::consts::PI;

const MIN_PULSE: u16 = 170;
const MAX_PULSE: u16 = 480;
const RADIANS_TO_PULSE: f64 = 380 as f64 / PI;

// the servos are always in the order [hip_xy, hip_z, knee]
const SERVO_LIMITS: [(f64, f64); 3] = [(-PI*0.25, PI*0.25), (-PI*0.33, PI*0.4), (-PI*0.28, PI*0.45)];
const SERVO_INVERT: [bool; 3] = [false, true, false];

// the legs are in the order [right_front, right_middle, right_back], [left_back, left_middle, left_front]
const SERVO_INDEX_TO_PIN_RIGHT: [usize; 9] = [10,9,8,5,6,7,2,3,4];
const SERVO_INDEX_TO_PIN_LEFT: [usize; 9] = [7,6,5,4,3,2,8,9,10];

const SERVO_CALIBRATION_RIGHT: [u16; 9] = [306, 330, 317, 306, 302, 306, 331, 350, 350];
const SERVO_CALIBRATION_LEFT: [u16; 9] = [345, 326, 330, 304, 331, 326, 337, 313, 315];

const ADDR_RIGHT: u8 = 0b1000000;
const ADDR_LEFT: u8 =  0b1000001;


pub struct ServoController{
    inner: Pca9685<I2c>,
    pin_mapping: [usize; 9],
    calibration: [u16; 9]
}

impl ServoController {
    pub fn new(left: bool) -> Self {
        let i2c = I2c::new().unwrap();
        let addr = if left { ADDR_LEFT } else { ADDR_RIGHT };
        let mut pca = Pca9685::new(i2c, Address::from(addr)).unwrap();
        pca.set_prescale(120).unwrap(); //50Hz
        pca.enable().unwrap();

        Self{
            inner: pca,
            pin_mapping: if left { SERVO_INDEX_TO_PIN_LEFT } else { SERVO_INDEX_TO_PIN_RIGHT },
            calibration: if left { SERVO_CALIBRATION_LEFT } else { SERVO_CALIBRATION_RIGHT },
        }
    }

    pub fn set_angles(&mut self, angles: &[Option<f64>]){
        assert_eq!(angles.len(), 9);
        for (servo, angle) in angles.iter().enumerate(){
            if let Some(angle) = angle {
                self.set_angle(servo, *angle);
            }
        }
    }

    pub fn set_angle(&mut self, servo: usize, mut angle: f64){
        if angle.is_nan() || angle < SERVO_LIMITS[servo % 3].0 || angle > SERVO_LIMITS[servo % 3].1 {
            eprintln!("angle out of range! (servo {} (type: {}), angle {})", servo, servo%3, angle);
            return;
        }

        if SERVO_INVERT[servo % 3] { angle *= -1.0; }
        let pulse = (self.calibration[servo] as i16 + ((RADIANS_TO_PULSE * angle) as i16)) as u16;

        if pulse < MIN_PULSE || pulse > MAX_PULSE {
            eprintln!("pulse out of range! (servo {} (type: {}), pulse {})", servo, servo%3, pulse);
            return;
        }

        //let scalar = (angle / PI) + 0.5;
        //assert!(scalar <= 1.0 && scalar >= 0.0);
        //let pulse = MIN_PULSE + (scalar * (MAX_PULSE - MIN_PULSE) as f64) as u16;
        let channel = Channel::try_from(self.pin_mapping[servo]).unwrap();

        //println!("{} pulse {}", servo, pulse);
        self.inner.set_channel_on_off(channel, 0, pulse).unwrap();
    }

    pub fn release(&mut self) {
        for pin in self.pin_mapping {
            let channel = Channel::try_from(pin).unwrap();
            self.inner.set_channel_full_off(channel).unwrap();
        }
    }
}