use serde::{Serialize};

#[derive(Serialize, Debug)]
pub struct TelemetryMessage{
    pub center: [f64; 2],
    pub rotation: f64,
    pub legs: [LegTelemetry; 6],
    pub angles: [Option<f64>; 6*3]
}

#[derive(Serialize, Debug)]
pub struct LegTelemetry {
    pub joint: [f64; 3],
    pub hip: [f64; 3],
    pub knee: [f64; 3],
    pub foot: [f64; 3],
}