use std::net::{TcpStream, TcpListener};
use hexapod::hexapod::Hexapod;
use std::time::{Instant, Duration};
use hexapod::telemetry::TelemetryMessage;
use std::io::Write;

pub const MESSAGE_FREQUENCY: u32 = 30; //Hz

pub struct TelemetryReporter{
    socket: TcpStream,
    lastReport: Instant,
}

impl TelemetryReporter {
    pub fn new() -> Self {
        let listener = TcpListener::bind("0.0.0.0:8766").unwrap();
        let (mut socket, _addr) = listener.accept().unwrap();
        println!("connected");

        Self {
            socket,
            lastReport: Instant::now(),
        }
    }

    pub fn report(&mut self, msg: &TelemetryMessage){
        let interval = Duration::from_secs(1) / MESSAGE_FREQUENCY;
        if self.lastReport.elapsed() > interval {
            let message = serde_json::to_string(msg).unwrap() + "\n";
            self.socket.write_all(message.as_bytes()).unwrap();
            self.lastReport = Instant::now();
        }
    }
}