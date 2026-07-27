use crate::{RuntimeError, RuntimeResult};
use dowe_compiler::{
    CompiledProject, ServerConfig, ServerModel, ServerModelEngine, ServerModelKind,
};
use dowe_inference::{EnergyVad, VadEngine};
use std::collections::HashMap;
use std::path::Path;

pub struct LoadedModelRuntime {
    vad: HashMap<String, LoadedVadModel>,
}

pub struct LoadedVadModel {
    engine: Box<dyn VadEngine>,
}

impl LoadedModelRuntime {
    pub fn load_project(project: &CompiledProject) -> RuntimeResult<Self> {
        Self::load_server(&project.root, &project.backend)
    }

    pub fn load_server(root: &Path, server: &ServerConfig) -> RuntimeResult<Self> {
        let mut vad = HashMap::new();
        for model in &server.models {
            match model.kind {
                ServerModelKind::VadSilero => {
                    let loaded = load_vad(root, model)?;
                    if vad.insert(model.name.clone(), loaded).is_some() {
                        return Err(RuntimeError::new(format!(
                            "duplicate VAD model `{}`",
                            model.name
                        )));
                    }
                }
            }
        }
        Ok(Self { vad })
    }

    pub fn vad_mut(&mut self, name: &str) -> Option<&mut LoadedVadModel> {
        self.vad.get_mut(name)
    }

    pub fn vad_names(&self) -> Vec<&str> {
        self.vad.keys().map(String::as_str).collect()
    }
}

impl LoadedVadModel {
    pub fn speech_probability(&mut self, samples: &[f32], sample_rate: u32) -> RuntimeResult<f32> {
        self.engine
            .speech_probability(samples, sample_rate)
            .map_err(|error| RuntimeError::new(error.to_string()))
    }

    pub fn reset(&mut self) {
        self.engine.reset();
    }
}

fn load_vad(root: &Path, model: &ServerModel) -> RuntimeResult<LoadedVadModel> {
    match model.engine {
        ServerModelEngine::Energy => Ok(LoadedVadModel {
            engine: Box::new(EnergyVad::default()),
        }),
        ServerModelEngine::Candle => load_candle_vad(root, model),
    }
}

#[cfg(feature = "candle")]
fn load_candle_vad(root: &Path, model: &ServerModel) -> RuntimeResult<LoadedVadModel> {
    let source = model
        .source
        .as_ref()
        .ok_or_else(|| RuntimeError::new(format!("model `{}` is missing source", model.name)))?;
    let path = root.join(source);
    let engine = dowe_inference::silero_candle::SileroCandleVad::load(&path)
        .map_err(|error| RuntimeError::new(error.to_string()))?;
    Ok(LoadedVadModel {
        engine: Box::new(engine),
    })
}

#[cfg(not(feature = "candle"))]
fn load_candle_vad(_: &Path, model: &ServerModel) -> RuntimeResult<LoadedVadModel> {
    Err(RuntimeError::new(format!(
        "model `{}` requires Dowe runtime feature `candle`",
        model.name
    )))
}

#[cfg(test)]
mod tests {
    use super::LoadedModelRuntime;
    use dowe_compiler::compile_dev;
    use dowe_inference::SILERO_16KHZ_FRAME_SAMPLES;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn loads_energy_vad_from_server_model() {
        let temp = TempDir::new().expect("tempdir");
        fs::create_dir_all(temp.path().join("layouts")).expect("layouts");
        fs::create_dir_all(temp.path().join("pages")).expect("pages");
        fs::create_dir_all(temp.path().join("routes")).expect("routes");
        fs::write(
            temp.path().join("main.dowe"),
            r#"import viewRoutes from "@/routes/view"

main
  views:viewRoutes
  server port:0
    model name:"voice-vad" kind:"vad.silero" engine:"energy" format:"builtin"
    route "/api/status"
      response text:"OK""#,
        )
        .expect("main");
        fs::write(
            temp.path().join("routes/view.dowe"),
            "import appLayout from \"../layouts/app\"\nimport homePage from \"../pages/home\"\n\nviews viewRoutes\n  group path:\"/\" layout:appLayout\n    route path:\"\" page:homePage\n",
        )
        .expect("views");
        fs::write(
            temp.path().join("layouts/app.dowe"),
            "layout appLayout\n  Box\n    children\n",
        )
        .expect("layout");
        fs::write(
            temp.path().join("pages/home.dowe"),
            "page homePage\n  Text\n    \"Home\"\n",
        )
        .expect("page");
        let project = compile_dev(temp.path()).expect("project");
        let mut runtime = LoadedModelRuntime::load_project(&project).expect("models");

        assert_eq!(runtime.vad_names(), vec!["voice-vad"]);
        let probability = runtime
            .vad_mut("voice-vad")
            .expect("vad")
            .speech_probability(&vec![0.0; SILERO_16KHZ_FRAME_SAMPLES], 16_000)
            .expect("probability");
        assert_eq!(probability, 0.0);
    }
}
