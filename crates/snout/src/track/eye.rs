use crate::{
    calibration::{EyeFusion, EyeShape},
    capture::{
        CameraError, Frame, StereoCamera,
        discovery::{CameraInfo, CameraSource, resolve_source},
        processing::FramePreprocessor,
    },
    config::Config,
    filter::EyeFilter,
    pipeline::EyePipeline,
    track::TrackerError,
    weights::Weights,
};

pub struct EyeReport<'a> {
    pub left_raw_frame: &'a Frame,
    pub left_processed_frame: &'a Frame,
    pub right_raw_frame: &'a Frame,
    pub right_processed_frame: &'a Frame,
    pub weights: &'a Weights<EyeShape>,
}

pub struct EyeTracker {
    pub left_preprocessor: FramePreprocessor,
    pub right_preprocessor: FramePreprocessor,
    pub pipeline: EyePipeline,
    pub calibrator: EyeFusion,
    pub filter: EyeFilter,

    camera: Option<StereoCamera>,
    left_source: Option<CameraSource>,
    right_source: Option<CameraSource>,
}

impl EyeTracker {
    pub fn new() -> Self {
        Self {
            left_preprocessor: FramePreprocessor::new(),
            right_preprocessor: FramePreprocessor::new(),
            pipeline: EyePipeline::new(),
            calibrator: EyeFusion::new(),
            filter: EyeFilter::new(),

            camera: None,
            left_source: None,
            right_source: None,
        }
    }

    pub fn with_config(cameras: &[CameraInfo], config: &Config) -> Result<Self, TrackerError> {
        let mut tracker = Self::new();

        tracker.pipeline.set_model(config.eye.model.as_ref())?;

        let left_camera = resolve_source(cameras, &config.eye.left.camera);
        let right_camera = resolve_source(cameras, &config.eye.right.camera);

        tracker.set_source(left_camera, right_camera);

        if let Some(filter) = config.eye.filter {
            tracker.filter.set_parameters(filter);
        }

        if let Some(center) = config.eye.left.center {
            tracker.calibrator.set_left_center(center);
        }

        if let Some(center) = config.eye.right.center {
            tracker.calibrator.set_right_center(center);
        }

        // Left preprocessor
        tracker.left_preprocessor.set_crop(config.eye.left.crop);

        if let Some(transform) = &config.eye.left.transform {
            tracker.left_preprocessor.set_config(*transform);
        }

        // Right preprocessor
        tracker.right_preprocessor.set_crop(config.eye.right.crop);

        if let Some(transform) = &config.eye.right.transform {
            tracker.right_preprocessor.set_config(*transform);
        }

        Ok(tracker)
    }

    /// Sets the camera source for the eye tracker.
    ///
    /// If the source has changed, the camera will be re-opened.
    /// If left equals right, the camera will be opened in side-by-side mode.
    /// If one source is `None`, then the source will be duplicated.
    /// If both sources are `None`, the camera will not be opened.
    pub fn set_source(&mut self, left: Option<CameraSource>, right: Option<CameraSource>) {
        if self.left_source != left || self.right_source != right {
            self.camera = None;
        }

        self.left_source = left;
        self.right_source = right;
    }

    pub fn track(&mut self) -> Result<Option<EyeReport<'_>>, TrackerError> {
        if !self.ensure_camera()? {
            return Ok(None);
        }

        let camera = self.camera.as_mut().unwrap();

        let (left_raw_frame, right_raw_frame) = match camera.get_frames() {
            Ok(frames) => frames,
            Err(CameraError::InvalidFrame(_)) => {
                // TODO: Keep track of the amount of invalid frames
                return Ok(None);
            }
            Err(e) => return Err(e.into()),
        };

        let left_processed_frame = self.left_preprocessor.process(left_raw_frame)?;
        let right_processed_frame = self.right_preprocessor.process(right_raw_frame)?;

        let Ok(Some(raw_weights)) = self
            .pipeline
            .run(left_processed_frame, right_processed_frame)
        else {
            return Ok(None);
        };

        let fused = self.calibrator.calibrate(raw_weights);
        let weights = self.filter.filter(fused);

        Ok(Some(EyeReport {
            left_raw_frame,
            right_raw_frame,
            left_processed_frame,
            right_processed_frame,
            weights,
        }))
    }

    fn ensure_camera(&mut self) -> Result<bool, TrackerError> {
        if self.camera.is_none() {
            // Nothing configured yet: skip tracking rather than erroring.
            if self.left_source.is_none() && self.right_source.is_none() {
                return Ok(false);
            }

            tracing::debug!(
                left = ?self.left_source,
                right = ?self.right_source,
                "Opening eye tracker camera"
            );

            let camera =
                StereoCamera::from_sources(self.left_source.as_ref(), self.right_source.as_ref())
                    .map_err(|e| {
                    tracing::error!(error = %e, "Failed to open eye tracker camera");
                    TrackerError::Open(e.to_string())
                })?;

            self.camera = Some(camera);
        }

        Ok(true)
    }
}
