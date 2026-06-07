use crate::error::DeployResult;
use crate::files::{copy_tree, write_file};
use crate::model::DeployTarget;
use serde_json::json;
use std::path::Path;

pub fn generate_static(root: &Path, output: &Path) -> DeployResult<()> {
    copy_static_assets(root, output)?;
    write_manifest(output, DeployTarget::Static)
}

pub fn generate_docker(root: &Path, output: &Path) -> DeployResult<()> {
    copy_app(root, &output.join("app"))?;
    write_file(
        &output.join("Dockerfile"),
        "FROM dowe/server:0.1.0\nCOPY app /app\nEXPOSE 8080\nENTRYPOINT [\"dowe-server\",\"--root\",\"/app\",\"--bind\",\"0.0.0.0:8080\"]\n",
    )?;
    write_manifest(output, DeployTarget::Docker)
}

pub fn generate_ssh(root: &Path, output: &Path) -> DeployResult<()> {
    copy_app(root, &output.join("app"))?;
    write_file(
        &output.join("run.sh"),
        "#!/bin/sh\nset -eu\nexec ./dowe-server --root ./app --bind 0.0.0.0:8080\n",
    )?;
    write_manifest(output, DeployTarget::Ssh)
}

pub fn copy_static_assets(root: &Path, output: &Path) -> DeployResult<()> {
    copy_tree(&root.join(".dowe/web"), output)?;
    copy_tree(&root.join(".dowe/fonts"), &output.join("fonts"))
}

pub fn write_manifest(output: &Path, target: DeployTarget) -> DeployResult<()> {
    let mut content = serde_json::to_string_pretty(&json!({
        "version": 1,
        "target": target,
    }))?;
    content.push('\n');
    write_file(&output.join("deploy.json"), content)
}

fn copy_app(root: &Path, output: &Path) -> DeployResult<()> {
    copy_tree(&root.join("src"), &output.join("src"))?;
    copy_tree(&root.join("assets"), &output.join("assets"))?;
    copy_tree(&root.join(".dowe/fonts"), &output.join("assets/fonts"))
}
