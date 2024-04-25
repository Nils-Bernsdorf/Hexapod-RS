use io_utils::controller;
use io_utils::telemetry::TelemetryReporter;

use hexapod::hexapod::{Hexapod, Foot};

use std::thread::sleep;
use std::time::{Duration, Instant};

use pwm_pca9685::{Pca9685, Address, Channel};
use rppal::i2c::I2c;
use hexapod::config::Config;
use hexapod::input_handler::{Button, ControllerEvent, InputHandler};
use crate::servo_controller::ServoController;

mod servo_controller;

fn main() {
    let receiver = controller::start_controller_thread("pop-os.fritz.box:8765");

    let mut telemetry = TelemetryReporter::new();

    let mut last_event = ControllerEvent::default();
    let mut hexapod = Hexapod::new();
    let mut input_handler = InputHandler::new();

    let mut servos_left = ServoController::new(true);
    let mut servos_right = ServoController::new(false);

    let mut conf = Config::default();

    loop {
        if let Ok(event) = receiver.try_recv() {
            if event.triggered(Button::B) {
                servos_left.release();
                servos_right.release();
                break;
            }
            last_event = event;
        }

        input_handler.handle_input(last_event, &mut hexapod, &mut conf);

        telemetry.report(&hexapod.get_telemetry());

        let angles = hexapod.get_angles();
        servos_right.set_angles(&angles[..9]);
        servos_left.set_angles(&angles[9..]);

        sleep(Duration::from_millis(10));
    }
}