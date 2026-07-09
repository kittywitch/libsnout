use std::path::Path;

use crate::{
    calibration::FaceShape,
    capture::Frame,
    pipeline::{
        PipelineError,
        internal::{FrameToBurnTensor, inference::FaceInference},
    },
    weights::Weights,
};

pub struct FacePipeline {
    transfer: FrameToBurnTensor,
    inference: Option<FaceInference>,
    weights: Weights<FaceShape>,
    output_map: Vec<Option<FaceShape>>,
}

impl FacePipeline {
    pub fn new() -> Self {
        let output_map: Vec<Option<FaceShape>> = (0..FaceShape::count())
            .map(|i| Some(FaceShape::from(i)))
            .collect();

        Self {
            transfer: FrameToBurnTensor::new(1, 224, 224),
            inference: None,
            weights: Weights::new(),
            output_map,
        }
    }

    pub fn set_model(&mut self, path: Option<impl AsRef<Path>>) -> Result<(), PipelineError> {
        if let Some(path) = path {
            let inference = FaceInference::new(path)?;
            self.inference = Some(inference);
        } else {
            self.inference = None;
        }

        Ok(())
    }

    pub fn run(&mut self, frame: &Frame) -> Result<Option<&Weights<FaceShape>>, PipelineError> {
        let Some(inference) = self.inference.as_mut() else {
            return Ok(None);
        };

        self.transfer
            .transfer_frame(frame, &mut inference.input_tensor);

        let weights = inference.run()?;

        self.weights.fill_with(weights, &self.output_map);

        Ok(Some(&self.weights))
    }
}
