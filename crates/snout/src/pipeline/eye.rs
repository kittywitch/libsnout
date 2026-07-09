use std::path::Path;

use crate::{
    calibration::EyeShape,
    capture::Frame,
    pipeline::{
        PipelineError,
        internal::{FrameToBurnTensor, eye_compositor::EyeCompositor, inference::EyeInference},
    },
    weights::Weights,
};

const LEGACY_OUTPUT_MAP: [Option<EyeShape>; 6] = [
    Some(EyeShape::LeftEyePitch),
    Some(EyeShape::LeftEyeYaw),
    Some(EyeShape::LeftEyeLid),
    Some(EyeShape::RightEyePitch),
    Some(EyeShape::RightEyeYaw),
    Some(EyeShape::RightEyeLid),
];

pub struct EyePipeline {
    transfer: FrameToBurnTensor,
    inference: Option<EyeInference>,
    collector: EyeCompositor,
    weights: Weights<EyeShape>,
    output_map: Vec<Option<EyeShape>>,
}

impl EyePipeline {
    pub fn new() -> Self {
        Self {
            transfer: FrameToBurnTensor::new(8, 128, 128),
            inference: None,
            collector: EyeCompositor::new(),
            weights: Weights::new(),
            output_map: LEGACY_OUTPUT_MAP.to_vec(),
        }
    }

    pub fn set_model(&mut self, path: Option<impl AsRef<Path>>) -> Result<(), PipelineError> {
        if let Some(path) = path {
            let inference = EyeInference::new(path)?;

            let output_map = match &inference.output_names {
                Some(names) => names.iter().map(|n| EyeShape::from_model_name(n)).collect(),
                None => {
                    if inference.output_count() != LEGACY_OUTPUT_MAP.len() {
                        tracing::warn!(
                            output_count = inference.output_count(),
                            "using legacy output map on incompatible model",
                        );
                    }
                    tracing::info!("using legacy output map");
                    LEGACY_OUTPUT_MAP.to_vec()
                }
            };

            self.output_map = output_map;
            self.inference = Some(inference);
        } else {
            self.inference = None;
            self.output_map = LEGACY_OUTPUT_MAP.to_vec();
        }

        Ok(())
    }

    pub fn run(
        &mut self,
        left: &Frame,
        right: &Frame,
    ) -> Result<Option<&Weights<EyeShape>>, PipelineError> {
        let Some(inference) = self.inference.as_mut() else {
            return Ok(None);
        };

        let Some(mat) = self.collector.compose(left, right) else {
            return Ok(None);
        };

        self.transfer
            .transfer_composite(mat, &mut inference.input_tensor);

        let weights = inference.run()?;

        self.weights.fill_with(weights, &self.output_map);

        Ok(Some(&self.weights))
    }
}
