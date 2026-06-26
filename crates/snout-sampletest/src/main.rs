use std::process::{Child, Command};
use std::time::Duration;

use snout::sample::net::{
    OverlayConnection, OverlayMessage, Packet,
    RunVariableLengthRoutinePacket,
};

const OVERLAY_PATH: &str = "/home/proto/Downloads/Baballonia.x64.v1.1.1.0rc6/Calibration/Linux/Overlay/BabbleCalibration.x86_64";

fn start_overlay() -> Child {
    Command::new(OVERLAY_PATH)
        .arg("--use-debug")
        .spawn()
        .expect("failed to start overlay process")
}

fn wait_for_finish(conn: &mut OverlayConnection) {
    loop {
        match conn.try_recv() {
            Ok(Some(msg)) => match &msg {
                OverlayMessage::PositionalData(data) => {
                    // println!(
                    //     "  pos: pitch={:.2} yaw={:.2} | L({:.2},{:.2}) R({:.2},{:.2})",
                    //     data.routine_pitch, data.routine_yaw,
                    //     data.left_eye_pitch, data.left_eye_yaw,
                    //     data.right_eye_pitch, data.right_eye_yaw,
                    // );
                }
                OverlayMessage::RoutineFinished(name) => {
                    println!("  routine finished: {name}");
                    return;
                }
                OverlayMessage::Unknown(name) => {
                    println!("  unknown: {name}");
                }
            },
            Ok(None) => {
                std::thread::sleep(Duration::from_millis(1));
            }
            Err(e) => {
                eprintln!("Connection error: {e}");
                return;
            }
        }
    }
}

fn main() {
    println!("Launching overlay...");
    let mut child = start_overlay();

    // std::thread::sleep(Duration::from_millis(250));

    println!("Starting TCP server on 127.0.0.1:2425...");
    let mut conn = OverlayConnection::listen()
        .expect("failed to accept overlay connection");
    println!("Overlay connected.");

    std::thread::sleep(Duration::from_millis(250));

    // Send tutorial first (gives overlay time to fully init scene)
    // let tutorial = Packet::new(
    //     "RunVariableLenghtRoutinePacket",
    //     &RunVariableLengthRoutinePacket::new("gazetutorial", 5.0),
    // );
    // println!("Sending: gazetutorial (5s)");
    // conn.send(&tutorial).expect("failed to send");
    // wait_for_finish(&mut conn);

    // Now send gaze
    let gaze = Packet::new(
        "RunVariableLenghtRoutinePacket",
        &RunVariableLengthRoutinePacket::new("gaze", 10.0),
    );
    println!("Sending: gaze (10s)");
    conn.send(&gaze).expect("failed to send");
    wait_for_finish(&mut conn);

    println!("Shutting down overlay...");
    let _ = child.kill();
    let _ = child.wait();
}
