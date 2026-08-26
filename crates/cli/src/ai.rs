use dowe_ai::{AiDevice, AiManifest, AiRegistry};
use std::env;
use std::fs;
use std::path::PathBuf;

pub(crate) async fn run_ai_command(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    match args.first().map(String::as_str) {
        Some("setup") => setup(&args[1..]),
        Some("models") => models(&args[1..]),
        Some("chat") => {
            Err("dowe ai chat is not available until a local Gemma model is installed".into())
        }
        Some(prompt) if !prompt.starts_with('-') => Err("use `dowe ai chat <prompt>`".into()),
        _ => Err("Usage: dowe ai setup [--registry <url>] | dowe ai models".into()),
    }
}

fn setup(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let registry_url = option_value(args, "--registry")?;
    let root = ai_root()?;
    let registry = AiRegistry::new(root.join("models"))?;
    let manifest = registry.fetch_manifest(registry_url.as_deref())?;
    let device = AiDevice::detect();
    let model = manifest.model_for(device, None)?;
    println!(
        "Installing {} {} for {}",
        model.id,
        model.version,
        device.as_str()
    );
    let path = registry.install(&manifest, model)?;
    fs::create_dir_all(&root)?;
    fs::write(
        root.join("manifest.json"),
        serde_json::to_vec_pretty(&manifest)?,
    )?;
    println!("Installed at {}", path.display());
    Ok(())
}

fn models(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let manifest_path = ai_root()?.join("manifest.json");
    let manifest = if manifest_path.is_file() {
        serde_json::from_slice::<AiManifest>(&fs::read(manifest_path)?)?
    } else {
        let registry_url = option_value(args, "--registry")?;
        AiRegistry::new(ai_root()?.join("models"))?.fetch_manifest(registry_url.as_deref())?
    };
    manifest.validate()?;
    let device = AiDevice::detect();
    println!("AVAILABLE MODELS");
    for model in &manifest.models {
        let selected = manifest_model_selected(&model.id, &manifest, device);
        let marker = if selected { "*" } else { " " };
        println!(
            "{} {:18} {:8} {}",
            marker, model.id, model.parameters, model.version
        );
    }
    Ok(())
}

fn manifest_model_selected(id: &str, manifest: &AiManifest, device: AiDevice) -> bool {
    manifest
        .model_for(device, None)
        .map(|model| model.id == id)
        .unwrap_or(false)
}

fn ai_root() -> Result<PathBuf, Box<dyn std::error::Error>> {
    Ok(env::current_dir()?.join(".dowe").join("ai"))
}

fn option_value(args: &[String], name: &str) -> Result<Option<String>, Box<dyn std::error::Error>> {
    let Some(index) = args.iter().position(|value| value == name) else {
        return Ok(None);
    };
    let value = args
        .get(index + 1)
        .ok_or_else(|| format!("{name} requires a value"))?;
    if value.starts_with('-') {
        return Err(format!("{name} requires a value").into());
    }
    Ok(Some(value.clone()))
}
