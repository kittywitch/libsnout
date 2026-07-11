use std::io;
use std::net::{ToSocketAddrs, UdpSocket};
use std::path::PathBuf;
use std::str::FromStr;

use rosc::{OscMessage, OscPacket, OscType};

use crate::calibration::FaceShape;

pub mod event;

pub use event::{ControlEvent, EyeEvent, FaceEvent, Side};

/// The largest OSC datagram we'll accept.
const MAX_PACKET_SIZE: usize = 1024;

/// Receives control commands over OSC/UDP.
pub struct OscControl {
    socket: UdpSocket,
    buf: [u8; MAX_PACKET_SIZE],
}

impl OscControl {
    /// Binds a UDP socket to `addr` and listens for control messages.
    pub fn bind(addr: impl ToSocketAddrs) -> io::Result<Self> {
        Ok(Self {
            socket: UdpSocket::bind(addr)?,
            buf: [0; MAX_PACKET_SIZE],
        })
    }

    /// Blocks until a datagram arrives.
    ///
    /// Invokes `f` once for every recognized event it carries.
    pub fn receive(&mut self, mut f: impl FnMut(ControlEvent)) -> io::Result<()> {
        let (len, _) = self.socket.recv_from(&mut self.buf)?;

        if let Ok((_, packet)) = rosc::decoder::decode_udp(&self.buf[..len]) {
            dispatch_packet(packet, &mut f);
        }

        Ok(())
    }
}

fn dispatch_packet(packet: OscPacket, f: &mut impl FnMut(ControlEvent)) {
    match packet {
        OscPacket::Message(message) => {
            if let Some(event) = decode_message(message) {
                f(event);
            }
        }
        OscPacket::Bundle(bundle) => {
            for packet in bundle.content {
                dispatch_packet(packet, f);
            }
        }
    }
}

fn decode_message(message: OscMessage) -> Option<ControlEvent> {
    match message.addr.as_str() {
        "/snout/face/bounds" => {
            let [
                OscType::String(shape),
                OscType::Float(lower),
                OscType::Float(upper),
            ] = message.args.as_slice()
            else {
                return None;
            };

            let shape = FaceShape::from_str(shape).ok()?;

            Some(ControlEvent::Face {
                event: FaceEvent::SetBounds {
                    shape,
                    lower: *lower,
                    upper: *upper,
                },
            })
        }
        "/snout/face/calibrate/lower" => {
            let [OscType::Int(frames)] = message.args.as_slice() else {
                return None;
            };

            let frames = (*frames).max(1) as u32;

            Some(ControlEvent::Face {
                event: FaceEvent::CalibrateLower { frames },
            })
        }
        "/snout/face/calibrate/upper" => {
            let [OscType::String(shape), OscType::Int(frames)] = message.args.as_slice() else {
                return None;
            };

            let shape = FaceShape::from_str(shape).ok()?;
            Some(ControlEvent::Face {
                event: FaceEvent::CalibrateUpper {
                    shape,
                    frames: (*frames).max(1) as u32,
                },
            })
        }
        "/snout/face/capture" => {
            let [OscType::String(path)] = message.args.as_slice() else {
                return None;
            };
            let path = PathBuf::from(path);

            if !path.is_absolute() {
                return None;
            }

            Some(ControlEvent::Face {
                event: FaceEvent::Capture { path },
            })
        }
        "/snout/eye/capture" => {
            let [OscType::String(side), OscType::String(path)] = message.args.as_slice() else {
                return None;
            };

            let side = match side.as_str() {
                "left" => Side::Left,
                "right" => Side::Right,
                _ => return None,
            };

            let path = PathBuf::from(path);

            if !path.is_absolute() {
                return None;
            }

            Some(ControlEvent::Eye {
                event: EyeEvent::Capture { side, path },
            })
        }
        _ => return None,
    }
}
