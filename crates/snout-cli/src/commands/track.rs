use std::{
    sync::mpsc::{self, Receiver, Sender},
    thread,
    thread::sleep,
    time::Duration,
};

use indicatif::MultiProgress;
use snout::{
    calibration::EyeShape,
    capture::discovery::{CameraInfo, query_cameras},
    config::Config,
    control::{Control, ControlEvent, EyeEvent, FaceEvent},
    track::{eye::EyeTracker, face::FaceTracker, initialize_runtime, output::Output},
};

use crate::status::{Float, Heartbeat, Pair, Rate, StatusBar, Vector};

const IDLE_RETRY: Duration = Duration::from_millis(10);

pub struct TrackCommand {
    config: Config,
    eye_debug: bool,
}

impl TrackCommand {
    pub fn new(config: Config, eye_debug: bool) -> Self {
        Self { config, eye_debug }
    }

    pub fn run(&self, multi: &MultiProgress) {
        initialize_runtime(self.config.libonnxruntime.as_ref());

        let cameras = query_cameras();
        let cameras = &cameras;

        let (face_tx, face_rx) = mpsc::channel();
        let (eye_tx, eye_rx) = mpsc::channel();

        thread::scope(|scope| {
            scope.spawn(move || self.run_control(face_tx, eye_tx));
            scope.spawn(move || self.run_face(cameras, multi, face_rx));
            scope.spawn(move || self.run_eye(cameras, multi, eye_rx));
        });
    }

    /// Face tracking worker: owns its tracker, output, and status line.
    fn run_face(
        &self,
        cameras: &[CameraInfo],
        multi: &MultiProgress,
        control: Receiver<FaceEvent>,
    ) {
        let mut tracker = FaceTracker::with_config(cameras, &self.config).unwrap();
        let mut output = Output::with_config(&self.config).unwrap();

        let mut status = StatusBar::new(multi);
        let heartbeat = status.add(Heartbeat::new("FACE", Duration::from_secs(1)));
        let tick_rate = status.add(Rate::new("TICK", 0));

        loop {
            while let Ok(event) = control.try_recv() {
                tracker.handle_event(event);
            }

            match tracker.track().unwrap() {
                Some(report) => {
                    heartbeat.beat();
                    output.send_face(report.weights);
                    output.flush().unwrap();
                    tick_rate.inc();
                    self.throttle();
                }
                None => sleep(IDLE_RETRY),
            }

            status.display();
        }
    }

    /// Eye tracking worker: owns its tracker, output, and status line.
    fn run_eye(&self, cameras: &[CameraInfo], multi: &MultiProgress, control: Receiver<EyeEvent>) {
        let mut tracker = EyeTracker::with_config(cameras, &self.config).unwrap();
        let mut output = Output::with_config(&self.config).unwrap();

        let mut status = StatusBar::new(multi);
        let heartbeat = status.add(Heartbeat::new("EYE", Duration::from_secs(1)));
        let tick_rate = status.add(Rate::new("TICK", 0));

        let eye_debug = self.eye_debug.then(|| {
            let version = status.add(Vector::new("VERSION"));
            let vergence = status.add(Float::new("VERGENCE"));
            let lids = status.add(Pair::new("LIDS"));
            let brow = status.add(Pair::new("BROW"));
            let widen = status.add(Pair::new("WIDEN"));
            let squint = status.add(Pair::new("SQUINT"));
            (version, vergence, lids, brow, widen, squint)
        });

        loop {
            while let Ok(event) = control.try_recv() {
                tracker.handle_event(event);
            }

            match tracker.track().unwrap() {
                Some(report) => {
                    heartbeat.beat();

                    if let Some((version, vergence, lids, brow, widen, squint)) = &eye_debug {
                        lids.set(
                            report.weights.get(EyeShape::LeftEyeLid).unwrap_or(0.),
                            report.weights.get(EyeShape::RightEyeLid).unwrap_or(0.),
                        );

                        brow.set(
                            report.weights.get(EyeShape::LeftEyeBrow).unwrap_or(0.),
                            report.weights.get(EyeShape::RightEyeBrow).unwrap_or(0.),
                        );
                        widen.set(
                            report.weights.get(EyeShape::LeftEyeWiden).unwrap_or(0.),
                            report.weights.get(EyeShape::RightEyeWiden).unwrap_or(0.),
                        );
                        squint.set(
                            report.weights.get(EyeShape::LeftEyeSquint).unwrap_or(0.),
                            report.weights.get(EyeShape::RightEyeSquint).unwrap_or(0.),
                        );

                        version.set(
                            report.weights.get(EyeShape::EyePitchVersion).unwrap_or(0.),
                            report.weights.get(EyeShape::EyeYawVersion).unwrap_or(0.),
                        );

                        vergence.set(report.weights.get(EyeShape::EyeYawVergence).unwrap_or(0.));
                    }

                    output.send_eyes(report.weights);
                    output.flush().unwrap();
                    tick_rate.inc();
                    self.throttle();
                }
                None => sleep(IDLE_RETRY),
            }

            status.display();
        }
    }

    fn run_control(&self, face: Sender<FaceEvent>, eye: Sender<EyeEvent>) {
        let Some(subconfig) = &self.config.control else {
            tracing::info!("Control disabled");
            return;
        };

        let mut control = match Control::bind(&subconfig.listen) {
            Ok(control) => {
                tracing::info!(listen = %subconfig.listen, "control listener started");
                control
            }
            Err(error) => {
                tracing::error!(%error, listen = %subconfig.listen, "failed to bind control listener");
                return;
            }
        };

        let mut running = true;

        while running {
            let result = control.receive(|event| match event {
                ControlEvent::Face { event } => {
                    if face.send(event).is_err() {
                        running = false;
                    }
                }
                ControlEvent::Eye { event } => {
                    if eye.send(event).is_err() {
                        running = false;
                    }
                }
            });

            if let Err(error) = result {
                tracing::error!(%error, "control listener stopped");
                break;
            }
        }
    }

    /// Per-tick throttle, matching the single-loop behavior: sleep for the
    /// configured `interval` (skipped when it's 0), or 10ms by default.
    fn throttle(&self) {
        if let Some(interval) = self.config.interval {
            if interval > 0 {
                sleep(Duration::from_millis(interval));
            }
        } else {
            sleep(Duration::from_millis(10));
        }
    }
}
