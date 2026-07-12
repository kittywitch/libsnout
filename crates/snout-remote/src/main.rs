use std::{io, net::UdpSocket, path::PathBuf};

use clap::{Parser, Subcommand, ValueEnum};
use rosc::{OscMessage, OscPacket, OscType};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    /// Address of the snout control listener.
    #[arg(short, long, default_value = "127.0.0.1:9500")]
    target: String,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Face tracker controls.
    Face {
        #[command(subcommand)]
        command: FaceCommand,
    },
    /// Eye tracker controls.
    Eye {
        #[command(subcommand)]
        command: EyeCommand,
    },
}

#[derive(Subcommand)]
enum FaceCommand {
    /// Set the input calibration range for a face shape.
    Bounds {
        /// Face shape name, e.g. `jawOpen`, `mouthSmileLeft`, `tongueOut`.
        shape: String,
        /// Lower bound of the input range.
        lower: f32,
        /// Upper bound of the input range.
        upper: f32,
    },
    /// Begin a neutral-hold face calibration pass.
    CalibrateLower {
        /// Number of frames to capture the peak over.
        #[arg(short, long, default_value_t = 100)]
        frames: u32,
    },
    /// Capture the peak of a face shape over a window and set its upper bound.
    ///
    /// Hold (or sweep to) the maximum of the expression for the duration of the
    /// capture; the highest value seen becomes the shape's upper bound.
    CalibrateUpper {
        /// Face shape name, e.g. `jawLeft`, `tongueOut`.
        shape: String,
        /// Number of frames to capture the peak over.
        #[arg(short, long, default_value_t = 100)]
        frames: u32,
    },
    /// Capture the next processed face frame to an image file.
    Capture {
        /// Where to write the captured frame. Relative paths are resolved
        /// against the current directory before being sent.
        path: PathBuf,
    },
}

#[derive(Subcommand)]
enum EyeCommand {
    /// Capture the next processed frame from one eye to an image file.
    Capture {
        /// Which eye to capture.
        side: Side,
        /// Where to write the captured frame. Relative paths are resolved
        /// against the current directory before being sent.
        path: PathBuf,
    },
}

#[derive(Clone, Copy, ValueEnum)]
enum Side {
    Left,
    Right,
}

impl Side {
    fn as_str(self) -> &'static str {
        match self {
            Side::Left => "left",
            Side::Right => "right",
        }
    }
}

impl Command {
    fn into_message(self) -> io::Result<OscMessage> {
        match self {
            Command::Face { command } => command.into_message(),
            Command::Eye { command } => command.into_message(),
        }
    }
}

impl FaceCommand {
    fn into_message(self) -> io::Result<OscMessage> {
        let message = match self {
            FaceCommand::Bounds {
                shape,
                lower,
                upper,
            } => OscMessage {
                addr: "/snout/face/bounds".to_string(),
                args: vec![
                    OscType::String(shape),
                    OscType::Float(lower),
                    OscType::Float(upper),
                ],
            },
            FaceCommand::CalibrateLower { frames } => OscMessage {
                addr: "/snout/face/calibrate/lower".to_string(),
                args: vec![OscType::Int(frames as i32)],
            },
            FaceCommand::CalibrateUpper { shape, frames } => OscMessage {
                addr: "/snout/face/calibrate/upper".to_string(),
                args: vec![OscType::String(shape), OscType::Int(frames as i32)],
            },
            FaceCommand::Capture { path } => OscMessage {
                addr: "/snout/face/capture".to_string(),
                args: vec![OscType::String(absolute_path(path)?)],
            },
        };

        Ok(message)
    }
}

impl EyeCommand {
    fn into_message(self) -> io::Result<OscMessage> {
        let message = match self {
            EyeCommand::Capture { side, path } => OscMessage {
                addr: "/snout/eye/capture".to_string(),
                args: vec![
                    OscType::String(side.as_str().to_string()),
                    OscType::String(absolute_path(path)?),
                ],
            },
        };

        Ok(message)
    }
}

fn absolute_path(path: PathBuf) -> io::Result<String> {
    let path = std::path::absolute(path)?;
    Ok(path.to_string_lossy().into_owned())
}

fn main() {
    let args = Args::parse();

    if let Err(error) = run(args) {
        eprintln!("error: {error}");
        std::process::exit(1);
    }
}

fn run(args: Args) -> Result<(), Box<dyn std::error::Error>> {
    let message = args.command.into_message()?;
    send(&args.target, message)
}

fn send(target: &str, message: OscMessage) -> Result<(), Box<dyn std::error::Error>> {
    let packet = OscPacket::Message(message);
    let buf = rosc::encoder::encode(&packet)
        .map_err(|e| format!("failed to encode OSC packet: {e:?}"))?;

    let socket = UdpSocket::bind("0.0.0.0:0")?;
    socket.send_to(&buf, target)?;

    Ok(())
}
