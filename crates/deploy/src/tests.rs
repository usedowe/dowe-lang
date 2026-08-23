use super::{
    BuildOptions, BuildTarget, DeployEnvironment, DeployOptions, DeploySurface, DeployTarget,
    available_build_targets, available_deploy_surfaces, build, deploy, deploy_with_linux_runtime,
};
use crate::docker::{docker_build_command, resolve_docker_image};
use crate::package::cloudflare_pages_redirects;
use crate::publish::{
    cloudflare_command, cloudflare_pages_command, configure_npm_cache, vercel_command,
};
use dowe_compiler::{compile_dev, generate_database_migrations};
use std::ffi::OsStr;
use std::fs;
use std::path::Path;
use std::process::Command;
use tempfile::TempDir;

fn hashed_router_path(root: &Path) -> std::path::PathBuf {
    fs::read_dir(root)
        .expect("web assets")
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .find(|path| {
            path.file_name()
                .and_then(OsStr::to_str)
                .is_some_and(|name| name.starts_with("router-") && name.ends_with(".js"))
        })
        .expect("hashed router")
}

fn hashed_design_path(root: &Path) -> std::path::PathBuf {
    fs::read_dir(root)
        .expect("web assets")
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .find(|path| {
            path.file_name()
                .and_then(OsStr::to_str)
                .is_some_and(|name| name.starts_with("design-") && name.ends_with(".css"))
        })
        .expect("hashed design css")
}

mod tests_cloudflare;
mod tests_docker;
mod tests_environment_contracts;
mod tests_native;
mod tests_queue;
mod tests_static_environment;
mod tests_vercel;

fn write_fixture(root: &Path, init: &str) {
    fs::create_dir_all(root.join("layouts")).expect("layouts");
    fs::create_dir_all(root.join("pages")).expect("pages");
    fs::create_dir_all(root.join("routes")).expect("routes");
    fs::write(
        root.join("main.dowe"),
        format!(
            "import viewRoutes from \"@/routes/view\"\n\nmain\n  views:viewRoutes\n  server port:8080\n    route \"/api/status\"\n      response text:\"OK\"\n{init}"
        ),
    )
    .expect("main");
    fs::write(root.join("theme.dowe"), "theme\n").expect("theme");
    fs::write(
        root.join(".env.example"),
        "BACKEND_URL=\nDOWE_DEPLOY_ACCESS_PASSWORD=\n",
    )
    .expect("env example");
    fs::write(root.join(".env"), "BACKEND_URL=\n").expect("env");
    fs::write(
        root.join("routes/view.dowe"),
        "import RootLayout from \"../layouts/root\"\nimport homePage from \"../pages/home\"\n\nviews viewRoutes\n  group path:\"/\" layout:RootLayout\n    route path:\"\" page:homePage\n",
    )
    .expect("views");
    fs::write(
        root.join("layouts/root.dowe"),
        "layout RootLayout\n  Box\n    Text\n      \"Layout\"\n    children\n",
    )
    .expect("layout");
    fs::write(
        root.join("pages/home.dowe"),
        "page homePage\n  Text\n    \"Home\"\n",
    )
    .expect("page");
}

fn write_environment(root: &Path, environment: DeployEnvironment, password: &str) {
    fs::write(
        root.join(format!(".env.{}", environment.as_str())),
        format!("BACKEND_URL=\nDOWE_DEPLOY_ACCESS_PASSWORD={password}\n"),
    )
    .expect("deploy environment");
}

fn linux_application_runtime() -> Vec<u8> {
    let mut runtime = vec![0u8; 96];
    runtime[..4].copy_from_slice(b"\x7fELF");
    runtime[4] = 2;
    runtime[5] = 1;
    runtime[18..20].copy_from_slice(&62u16.to_le_bytes());
    runtime[64..72].copy_from_slice(b"DOWESRV1");
    runtime
}
