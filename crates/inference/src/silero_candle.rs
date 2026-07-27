use crate::{ModelError, VadEngine, expected_silero_frame_size, validate_silero_frame};
use candle_core::{Device, Tensor};
use candle_onnx::{onnx, simple_eval};
use std::collections::HashMap;
use std::path::Path;

pub struct SileroCandleVad {
    model: onnx::ModelProto,
    state: Tensor,
    context: Vec<f32>,
    last_sample_rate: u32,
}

impl SileroCandleVad {
    pub fn load(path: impl AsRef<Path>) -> Result<Self, ModelError> {
        let mut model =
            candle_onnx::read_file(path).map_err(|error| ModelError::Model(error.to_string()))?;
        normalize_ifless_sr_input_shape(&mut model);
        let state = Tensor::zeros((2, 1, 128), candle_core::DType::F32, &Device::Cpu)
            .map_err(|error| ModelError::Model(error.to_string()))?;

        Ok(Self {
            model,
            state,
            context: Vec::new(),
            last_sample_rate: 0,
        })
    }

    fn reset_for_rate(&mut self, sample_rate: u32) -> Result<(), ModelError> {
        self.state = Tensor::zeros((2, 1, 128), candle_core::DType::F32, &Device::Cpu)
            .map_err(|error| ModelError::Model(error.to_string()))?;
        self.context = vec![0.0; context_size(sample_rate)?];
        self.last_sample_rate = sample_rate;
        Ok(())
    }
}

impl VadEngine for SileroCandleVad {
    fn speech_probability(&mut self, samples: &[f32], sample_rate: u32) -> Result<f32, ModelError> {
        validate_silero_frame(samples, sample_rate)?;
        if self.last_sample_rate != sample_rate || self.context.is_empty() {
            self.reset_for_rate(sample_rate)?;
        }

        let mut input_samples = Vec::with_capacity(self.context.len() + samples.len());
        input_samples.extend_from_slice(&self.context);
        input_samples.extend_from_slice(samples);

        let input = Tensor::from_vec(
            input_samples.clone(),
            (1, input_samples.len()),
            &Device::Cpu,
        )
        .map_err(|error| ModelError::Model(error.to_string()))?;
        let sr = if uses_ifless_sr_reshape(&self.model) {
            Tensor::from_vec(vec![sample_rate as i64], (1,), &Device::Cpu)
        } else {
            Tensor::new(sample_rate as i64, &Device::Cpu)
        }
        .map_err(|error| ModelError::Model(error.to_string()))?;

        let mut inputs = HashMap::new();
        inputs.insert("input".to_string(), input);
        inputs.insert("state".to_string(), self.state.clone());
        inputs.insert("sr".to_string(), sr);

        let mut outputs = simple_eval(&self.model, inputs)
            .map_err(|error| ModelError::Model(format!("candle ONNX inference failed: {error}")))?;

        let graph = self
            .model
            .graph
            .as_ref()
            .ok_or_else(|| ModelError::Model("ONNX graph missing".to_string()))?;
        let output_name = graph
            .output
            .first()
            .map(|output| output.name.as_str())
            .ok_or_else(|| ModelError::Model("Silero ONNX output missing".to_string()))?;
        let state_name = graph
            .output
            .get(1)
            .map(|output| output.name.as_str())
            .ok_or_else(|| {
                ModelError::Model("Silero ONNX recurrent state output missing".to_string())
            })?;

        let output = outputs
            .remove(output_name)
            .ok_or_else(|| ModelError::Model(format!("Silero output {output_name} missing")))?;
        self.state = outputs.remove(state_name).ok_or_else(|| {
            ModelError::Model(format!("Silero state output {state_name} missing"))
        })?;

        let context_len = context_size(sample_rate)?;
        self.context = input_samples[input_samples.len() - context_len..].to_vec();

        output
            .flatten_all()
            .and_then(|tensor| tensor.to_vec1::<f32>())
            .map_err(|error| ModelError::Model(error.to_string()))?
            .first()
            .copied()
            .ok_or_else(|| ModelError::Model("Silero output was empty".to_string()))
    }

    fn reset(&mut self) {
        let _ = self.reset_for_rate(self.last_sample_rate.max(16_000));
    }
}

fn context_size(sample_rate: u32) -> Result<usize, ModelError> {
    let frame = expected_silero_frame_size(sample_rate)?;
    Ok(frame / 8)
}

fn normalize_ifless_sr_input_shape(model: &mut onnx::ModelProto) {
    if !uses_ifless_sr_reshape(model) {
        return;
    }
    let Some(graph) = model.graph.as_mut() else {
        return;
    };
    let Some(input) = graph.input.iter_mut().find(|input| input.name == "sr") else {
        return;
    };
    let Some(type_proto) = input.r#type.as_mut() else {
        return;
    };
    let Some(onnx::type_proto::Value::TensorType(tensor_type)) = type_proto.value.as_mut() else {
        return;
    };

    tensor_type.shape = Some(onnx::TensorShapeProto {
        dim: vec![onnx::tensor_shape_proto::Dimension {
            denotation: String::new(),
            value: Some(onnx::tensor_shape_proto::dimension::Value::DimValue(1)),
        }],
    });
}

fn uses_ifless_sr_reshape(model: &onnx::ModelProto) -> bool {
    model
        .graph
        .as_ref()
        .map(|graph| {
            graph.node.iter().any(|node| {
                node.op_type == "Reshape" && node.input.first().is_some_and(|name| name == "sr")
            })
        })
        .unwrap_or(false)
}
