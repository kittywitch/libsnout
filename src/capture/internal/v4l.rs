use image::GrayImage;
use v4l::video::capture::Parameters;
use v4l::{
    buffer::Type,
    io::traits::CaptureStream,
    prelude::{MmapStream, UserptrStream},
    video::Capture,
};

use crate::capture::internal::Camera;
use crate::capture::{CameraError, discovery::V4lSource};

#[derive(Copy, Clone, Debug)]
enum PixelFormat {
    Grey,
    Yuyv,
    Uyvy,
    Mjpeg,
}

enum V4lStream {
    Userptr(UserptrStream),
    Mmap(MmapStream<'static>),
}

impl V4lStream {
    fn next(&mut self) -> std::io::Result<(&[u8], &v4l::buffer::Metadata)> {
        match self {
            V4lStream::Userptr(s) => s.next(),
            V4lStream::Mmap(s) => s.next(),
        }
    }
}

pub struct V4lCamera {
    _device: v4l::Device,
    stream: V4lStream,
    pixel_format: PixelFormat,
    pub width: usize,
    pub height: usize,
}

impl Camera for V4lCamera {
    fn read_frame(&mut self) -> Result<GrayImage, CameraError> {
        let mut destination = GrayImage::new(self.width as _, self.height as _);
        self.read_frame(&mut destination)?;
        Ok(destination)
    }
}

impl V4lCamera {
    pub fn open(source: V4lSource) -> Result<Self, CameraError> {
        tracing::debug!(
            index = source.index,
            width = source.format.width,
            height = source.format.height,
            fps = source.format.fps,
            fourcc = %String::from_utf8_lossy(&source.fourcc),
            "Opening V4L2 device"
        );

        let device = v4l::Device::new(source.index as usize)?;

        let format = v4l::Format::new(
            source.format.width,
            source.format.height,
            v4l::FourCC::new(&source.fourcc),
        );
        let format = device.set_format(&format)?;

        tracing::debug!(
            width = format.width,
            height = format.height,
            "Format negotiated"
        );

        let params = Parameters::with_fps(source.format.fps);
        device.set_params(&params)?;

        let pixel_format = match &source.fourcc {
            b"GREY" => PixelFormat::Grey,
            b"YUYV" => PixelFormat::Yuyv,
            b"UYVY" => PixelFormat::Uyvy,
            b"MJPG" => PixelFormat::Mjpeg,
            _ => {
                return Err(CameraError::InvalidFormat(format!(
                    "Unknown pixel format: {:?}",
                    &source.fourcc
                )));
            }
        };

        let width = format.width as usize;
        let height = format.height as usize;

        let stream = match UserptrStream::new(&device, Type::VideoCapture) {
            Ok(s) => {
                tracing::debug!("Using userptr streaming mode");
                V4lStream::Userptr(s)
            }
            Err(e) => {
                tracing::warn!(error = %e, "Userptr streaming not supported, falling back to mmap");
                let s = MmapStream::new(&device, Type::VideoCapture)?;
                tracing::debug!("Using mmap streaming mode");
                V4lStream::Mmap(s)
            }
        };

        Ok(Self {
            _device: device,
            stream,
            pixel_format,
            width,
            height,
        })
    }

    pub fn read_frame(&mut self, destination: &mut GrayImage) -> Result<(), CameraError> {
        let (buf, _meta) = self.stream.next().map_err(|e| {
            tracing::error!(
                error = %e,
                pixel_format = ?self.pixel_format,
                width = self.width,
                height = self.height,
                "Failed to read frame from V4L2 stream"
            );
            e
        })?;
        match self.pixel_format {
            PixelFormat::Grey => destination.copy_from_slice(buf),
            PixelFormat::Yuyv => {
                // extract Y channel: every other byte
                for (dst, &y) in destination.iter_mut().zip(buf.iter().step_by(2)) {
                    *dst = y;
                }
            }
            PixelFormat::Uyvy => {
                for (dst, &y) in destination.iter_mut().zip(buf[1..].iter().step_by(2)) {
                    *dst = y;
                }
            }
            PixelFormat::Mjpeg => {
                let img = image::load_from_memory(&buf[..])
                    .map_err(|e| CameraError::InvalidFrame(e.to_string()))?
                    .into_luma8();
                destination.copy_from_slice(img.as_raw());
            }
        }
        Ok(())
    }
}
