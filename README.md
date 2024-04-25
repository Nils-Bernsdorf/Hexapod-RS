# Hexapod-RS

This is the code I wrote for my 3d printed hexapod robot (see https://www.youtube.com/shorts/cVpfmvFyv4o). The robot itself is heavily inspired by the one @SmallpTsai built (check out https://github.com/SmallpTsai/hexapod-v2-7697 for more details). I adjusted some of his designs for the parts and used a Raspberry Pi Zero W instead of a Linkit 7697 to control everything. This repo contains the Rust code that is running on the Raspberry Pi.

## Features
- multiple walking gaits (including tripod, ripple and wave)
- dynamic blending between translational walking motion (body still points in the same direction, eg. walking sideways) and rotational motion (body turns with the robot, like you would eg. steer a car)
- rotation and translation of the robot body while it is standing
- wireless controller support
- real time telemetry of all the joint positions and angles via periodic Json messages

## Code overview
- `/hexapod`: contains the main control logic as a library crate. Implements the different gaits (`modes/gait.rs`), body movements (`modes/move_body.rs`), inverse kinematics (`hexapod.rs` and `legs.rs`) and configuration (`config.rs`)
- `/io_utils`: receives and decodes controller inputs over WiFi (`controller.rs`) and stends the telemetry (`telemetry.rs`)
- `/raspberry`: binary crate that has to be executed on the Raspberry Pi. It implements the main event loop (`main.rs`) and the communication with the two PCA9685 PWM controllers that control the individual servo motors (`servo_controller.rs`)
