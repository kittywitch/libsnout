use std::time::Duration;

use snout::sample::net::{Event, Mode, Overlay, Routine};

const OVERLAY_PATH: &str = "/home/proto/Downloads/Baballonia.x64.v1.1.1.0rc6/Calibration/Linux/Overlay/BabbleCalibration.x86_64";

fn wait_for_finish(overlay: &mut Overlay) {
    loop {
        match overlay.try_recv() {
            Ok(Some(Event::Position(pos))) => {
                println!(
                    "  pos: pitch={:.2} yaw={:.2} | L({:.2},{:.2}) R({:.2},{:.2})",
                    pos.routine_pitch, pos.routine_yaw,
                    pos.left_eye_pitch, pos.left_eye_yaw,
                    pos.right_eye_pitch, pos.right_eye_yaw,
                );
            }
            Ok(Some(Event::Finished)) => {
                println!("  finished");
                return;
            }
            Ok(None) => {
                std::thread::sleep(Duration::from_millis(1));
            }
            Err(e) => {
                eprintln!("Error: {e}");
                return;
            }
        }
    }
}

fn main() {
    println!("Starting overlay...");
    let mut overlay = Overlay::start(OVERLAY_PATH, Mode::Debug)
        .expect("failed to start overlay");

    println!("Running gaze tutorial (30s)...");
    overlay.begin(Routine::ShortGazeTutorial).expect("failed to begin");
    wait_for_finish(&mut overlay);

    println!("Running gaze (10s)...");
    overlay.begin(Routine::Gaze(Duration::from_secs(10))).expect("failed to begin");
    wait_for_finish(&mut overlay);

    println!("Closing...");
    overlay.close().expect("failed to close");
}
