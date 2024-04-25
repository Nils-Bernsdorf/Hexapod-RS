use std::net::{TcpStream, ToSocketAddrs};
use std::io::prelude::*;
use std::convert::TryInto;
use std::sync::mpsc::{self, Sender, Receiver};
use std::thread;
use hexapod::input_handler::ControllerEvent;

pub fn capture_thread_inner<A: ToSocketAddrs>(addr: A, sender: Sender<ControllerEvent>) -> std::io::Result<()> {
    let mut stream = TcpStream::connect(addr)?;

    let mut pressed = [false; 12];
    let mut triggered = [false; 12];
    for timestamp in 0.. {
        let mut msg = [0u8; 32];
        stream.read_exact(&mut msg)?;

        pressed.iter_mut()
            .zip(msg[8..].iter())
            .for_each(|(p, e)| *p = *e != 0);

        triggered.iter_mut()
            .zip(msg[20..].iter())
            .for_each(|(p, e)| *p = *e != 0);

        let event = ControllerEvent{
            timestamp,
            lx: stick_input_from_bytes(&msg[0..2]),
            ly: stick_input_from_bytes(&msg[2..4]),
            rx: stick_input_from_bytes(&msg[4..6]),
            ry: stick_input_from_bytes(&msg[6..8]),
            pressed,
            triggered
        };
        sender.send(event).unwrap();
    }

    Ok(())
}

pub fn start_controller_thread(addr: &'static str) -> Receiver<ControllerEvent> {
    let (tx, rx): (Sender<ControllerEvent>, Receiver<ControllerEvent>) = mpsc::channel();
    thread::spawn(move || {
        capture_thread_inner(addr, tx).unwrap();
    });

    rx
}

fn stick_input_from_bytes(bytes: &[u8]) -> f64 {
    let integer = i16::from_be_bytes(bytes.try_into().unwrap());
    integer as f64 / i16::MAX as f64
}