use crate::capture::{CameraError, Frame, MonoCamera, discovery::CameraSource};

enum StereoDevice {
    /// Two independent cameras, one per eye.
    Dual(MonoCamera, MonoCamera),
    /// A single camera whose frame packs both eyes side-by-side.
    Sbs(MonoCamera, Frame, Frame),
    /// A single camera tracking one eye.
    Duplicate(MonoCamera),
}

pub struct StereoCamera {
    device: StereoDevice,
}

impl StereoCamera {
    /// Opens two independent cameras, one per eye.
    pub fn open(left: &CameraSource, right: &CameraSource) -> Result<Self, CameraError> {
        tracing::debug!(?left, ?right, "Opening stereo camera in dual mode");

        let left = MonoCamera::open(left)?;
        let right = MonoCamera::open(right)?;

        Ok(Self {
            device: StereoDevice::Dual(left, right),
        })
    }

    /// Opens a single camera whose frame packs both eyes side-by-side.
    pub fn open_sbs(source: &CameraSource) -> Result<Self, CameraError> {
        tracing::debug!(?source, "Opening stereo camera in side-by-side mode");

        let camera = MonoCamera::open(source)?;

        Ok(Self {
            device: StereoDevice::Sbs(camera, Frame::empty(0, 0), Frame::empty(0, 0)),
        })
    }

    /// Opens the stereo mode implied by the configured eye sources:
    ///
    /// - both sources set to the *same* camera -> side-by-side
    /// - both sources set to *different* cameras -> dual
    /// - exactly one source set -> duplicate (single-eye tracking)
    /// - neither source set -> error
    pub fn from_sources(
        left: Option<&CameraSource>,
        right: Option<&CameraSource>,
    ) -> Result<Self, CameraError> {
        match (left, right) {
            (Some(left), Some(right)) if left == right => Self::open_sbs(left),
            (Some(left), Some(right)) => Self::open(left, right),
            (Some(source), None) | (None, Some(source)) => Self::open_duplicate(source),
            (None, None) => Err(CameraError::InvalidSources),
        }
    }

    /// Opens a single camera whose frame is duplicated to both eyes.
    pub fn open_duplicate(source: &CameraSource) -> Result<Self, CameraError> {
        tracing::debug!(?source, "Opening stereo camera in duplicate mode");

        let camera = MonoCamera::open(source)?;

        Ok(Self {
            device: StereoDevice::Duplicate(camera),
        })
    }

    pub fn get_frames(&mut self) -> Result<(&Frame, &Frame), CameraError> {
        match &mut self.device {
            StereoDevice::Dual(left, right) => {
                let left = left.get_frame()?;
                let right = right.get_frame()?;

                Ok((left, right))
            }
            StereoDevice::Duplicate(cam) => {
                let frame = cam.get_frame()?;

                Ok((frame, frame))
            }
            StereoDevice::Sbs(camera, left, right) => {
                let full = camera.get_frame()?;
                split_sbs(full, left, right);

                Ok((&*left, &*right))
            }
        }
    }
}

/// Splits a side-by-side frame into its left and right halves.
fn split_sbs(full: &Frame, left: &mut Frame, right: &mut Frame) {
    let full_width = full.width();
    let height = full.height();
    let half_width = full_width / 2;

    // Resize the destination frames if they don't match
    ensure_size(left, half_width, height);
    ensure_size(right, half_width, height);

    let source = full.as_slice();
    let left_destination: &mut [u8] = left.image.as_mut();
    let right_destination: &mut [u8] = right.image.as_mut();

    for y in 0..height {
        let row = &source[y * full_width..(y + 1) * full_width];

        left_destination[y * half_width..(y + 1) * half_width].copy_from_slice(&row[..half_width]);
        right_destination[y * half_width..(y + 1) * half_width].copy_from_slice(&row[half_width..]);
    }
}

fn ensure_size(frame: &mut Frame, width: usize, height: usize) {
    if frame.width() != width || frame.height() != height {
        *frame = Frame::empty(width as u32, height as u32);
    }
}
