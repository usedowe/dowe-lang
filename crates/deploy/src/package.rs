use crate::access::DeployAccess;
use crate::error::DeployResult;
use crate::files::{collect_files, copy_file, copy_tree, write_file};
use crate::model::{DeployEnvironment, DeployTarget};
use serde_json::json;
use std::fs;
use std::path::Path;

pub fn generate_static(root: &Path, output: &Path) -> DeployResult<()> {
    copy_static_assets(root, output)?;
    write_manifest(output, DeployTarget::Static, DeployEnvironment::Live, false)
}

pub fn generate_cloudflare_pages(
    root: &Path,
    output: &Path,
    project_name: &str,
    environment: DeployEnvironment,
    access: Option<&DeployAccess>,
) -> DeployResult<()> {
    let assets = output.join("assets");
    copy_static_assets(root, &assets)?;
    normalize_cloudflare_pages_assets(&assets)?;
    write_cloudflare_pages_redirects(&assets)?;
    if let Some(access) = access {
        write_file(&assets.join("_worker.js"), access.pages_worker())?;
    }
    let mut content = serde_json::to_string_pretty(&json!({
        "version": 1,
        "surface": "web",
        "provider": "cloudflare-pages",
        "projectName": project_name,
        "environment": environment,
        "accessProtected": access.is_some()
    }))?;
    content.push('\n');
    write_file(&output.join("deploy.json"), content)
}

fn normalize_cloudflare_pages_assets(root: &Path) -> DeployResult<()> {
    for relative_path in collect_files(root)? {
        if relative_path.extension().and_then(|value| value.to_str()) != Some("html") {
            continue;
        }
        let path = root.join(relative_path);
        let document = fs::read_to_string(&path)?;
        let document = normalize_cloudflare_pages_html(&document);
        write_file(&path, document)?;
    }
    Ok(())
}

fn write_cloudflare_pages_redirects(root: &Path) -> DeployResult<()> {
    let manifest = fs::read_to_string(root.join("manifest.json"))?;
    let redirects = cloudflare_pages_redirects(&manifest)?;
    if !redirects.is_empty() {
        write_file(&root.join("_redirects"), redirects)?;
    }
    Ok(())
}

pub(crate) fn cloudflare_pages_redirects(manifest: &str) -> DeployResult<String> {
    let manifest: serde_json::Value = serde_json::from_str(manifest)?;
    let mut redirects = String::new();
    let Some(routes) = manifest.get("routes").and_then(serde_json::Value::as_array) else {
        return Ok(redirects);
    };

    for route in routes {
        let Some(path) = route.get("path").and_then(serde_json::Value::as_str) else {
            continue;
        };
        if path == "/" {
            continue;
        }
        let Some(static_file) = route.get("staticFile").and_then(serde_json::Value::as_str) else {
            continue;
        };
        let static_file = static_file.strip_prefix("web/").unwrap_or(static_file);
        let destination = format!("/{static_file}");
        redirects.push_str(&format!("{path} {destination} 200\n"));
        if !path.ends_with('/') {
            redirects.push_str(&format!("{path}/ {destination} 200\n"));
        }
    }

    Ok(redirects)
}

fn normalize_cloudflare_pages_html(document: &str) -> String {
    ["../../", "../", ""]
        .into_iter()
        .fold(document.to_string(), |document, prefix| {
            document
                .replace(
                    &format!(r#"href="{prefix}design.css""#),
                    r#"href="/design.css""#,
                )
                .replace(&format!(r#"href="{prefix}chunks/"#), r#"href="/chunks/"#)
                .replace(&format!(r#"href="{prefix}icons/"#), r#"href="/icons/"#)
                .replace(
                    &format!(r#"src="{prefix}router.js""#),
                    r#"src="/router.js""#,
                )
                .replace(&format!(r#"src="{prefix}chunks/"#), r#"src="/chunks/"#)
        })
}

pub fn copy_static_assets(root: &Path, output: &Path) -> DeployResult<()> {
    copy_tree(&root.join(".dowe/web"), output)?;
    copy_tree(&root.join("assets"), &output.join("assets"))?;
    copy_tree(&root.join(".dowe/fonts"), &output.join("fonts"))
}

pub fn write_manifest(
    output: &Path,
    target: DeployTarget,
    environment: DeployEnvironment,
    access_protected: bool,
) -> DeployResult<()> {
    let mut content = serde_json::to_string_pretty(&json!({
        "version": 1,
        "target": target,
        "environment": environment,
        "accessProtected": access_protected,
    }))?;
    content.push('\n');
    write_file(&output.join("deploy.json"), content)
}

pub(crate) fn copy_app(root: &Path, output: &Path) -> DeployResult<()> {
    for file_name in ["main.dowe", "theme.dowe", ".env.example"] {
        let source = root.join(file_name);
        if source.is_file() {
            copy_file(&source, &output.join(file_name))?;
        }
    }
    copy_project_modules(root, output)?;
    copy_tree(&root.join("assets"), &output.join("assets"))?;
    copy_tree(&root.join("icons"), &output.join("icons"))?;
    copy_tree(&root.join(".dowe/fonts"), &output.join("assets/fonts"))
}

fn copy_project_modules(root: &Path, output: &Path) -> DeployResult<()> {
    let mut entries = fs::read_dir(root)?.collect::<Result<Vec<_>, _>>()?;
    entries.sort_by_key(|entry| entry.file_name());
    for entry in entries {
        let file_type = entry.file_type()?;
        if !file_type.is_dir() || file_type.is_symlink() {
            continue;
        }
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if name.starts_with('.')
            || matches!(
                name.as_ref(),
                "assets" | "icons" | "target" | "node_modules" | "dist" | "build"
            )
        {
            continue;
        }
        copy_dowe_modules(&entry.path(), &output.join(name.as_ref()))?;
    }
    Ok(())
}

fn copy_dowe_modules(source: &Path, destination: &Path) -> DeployResult<()> {
    let mut entries = fs::read_dir(source)?.collect::<Result<Vec<_>, _>>()?;
    entries.sort_by_key(|entry| entry.file_name());
    for entry in entries {
        let file_type = entry.file_type()?;
        if file_type.is_symlink() {
            continue;
        }
        let source = entry.path();
        let destination = destination.join(entry.file_name());
        if file_type.is_dir() {
            copy_dowe_modules(&source, &destination)?;
        } else if file_type.is_file()
            && source.extension().and_then(|value| value.to_str()) == Some("dowe")
        {
            copy_file(&source, &destination)?;
        }
    }
    Ok(())
}
