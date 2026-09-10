use super::project_files::*;
use super::studio_agent::*;
use super::studio_apps::*;
use super::studio_changes::*;
use super::studio_context::*;
use super::*;
use dowe_database::{init_database, open_database};
use serde_json::json;
use std::fs;
use std::process::Command;
use tempfile::TempDir;

#[tokio::test]
async fn creates_and_lists_folders() {
    let temp = TempDir::new().expect("tempdir");
    let created = create_folder(&json!({ "parent": temp.path(), "name": "workspace" }))
        .await
        .expect("create");
    assert_eq!(
        created.as_str().map(|value| value.ends_with("workspace")),
        Some(true)
    );
    let folders = list_folders(&json!({ "parent": temp.path() }))
        .await
        .expect("list");
    assert_eq!(folders, json!(["workspace"]));
}

#[test]
fn accepts_only_public_github_repository_urls() {
    assert_eq!(
        normalized_github_repository_url("https://github.com/dowe-lang/example")
            .expect("github url"),
        "https://github.com/dowe-lang/example.git"
    );
    assert_eq!(
        normalized_github_repository_url("https://www.github.com/dowe-lang/example.git")
            .expect("www github url"),
        "https://github.com/dowe-lang/example.git"
    );
    assert!(normalized_github_repository_url("git@github.com:dowe-lang/example.git").is_err());
    assert!(normalized_github_repository_url("https://example.com/dowe-lang/example").is_err());
    assert!(
        normalized_github_repository_url("https://github.com/dowe-lang/example?token=secret")
            .is_err()
    );
}

#[tokio::test]
async fn classifies_only_structurally_valid_dowe_projects() {
    let temp = TempDir::new().expect("tempdir");
    assert_eq!(
        inspect_dowe_project(&json!({ "path": temp.path() }))
            .await
            .expect("empty"),
        "empty"
    );
    fs::write(temp.path().join("main.dowe"), "not a main block\n").expect("invalid main");
    assert_eq!(
        inspect_dowe_project(&json!({ "path": temp.path() }))
            .await
            .expect("invalid"),
        "invalid"
    );
}

#[tokio::test]
async fn initializes_a_blank_project_without_agents() {
    let temp = TempDir::new().expect("project");
    let result = initialize_dowe_project(&json!({ "path": temp.path(), "template": "blank" }))
        .await
        .expect("initialize");
    assert_eq!(
        result,
        json!(
            temp.path()
                .canonicalize()
                .expect("canonical project")
                .to_string_lossy()
                .to_string()
        )
    );
    assert!(temp.path().join("main.dowe").is_file());
    assert!(temp.path().join("views/pages/home.dowe").is_file());
    assert!(!temp.path().join("AGENTS.md").exists());
    assert!(!temp.path().join(".agents").exists());
}

#[tokio::test]
async fn registers_an_app_with_its_source() {
    let studio = TempDir::new().expect("studio root");
    let app = TempDir::new().expect("app root");
    fs::write(app.path().join("main.dowe"), "main\n").expect("main");
    init_database(studio.path(), "dowe-studio-apps").expect("initialize database");

    let inserted = register_studio_app(
        studio.path(),
        &json!({ "path": app.path(), "name": "Imported app", "source": "github" }),
    )
    .await
    .expect("register");

    assert_eq!(inserted["source"], "github");
    assert_eq!(
        open_database(studio.path(), "dowe-studio-apps")
            .expect("database")
            .records("workspace_apps")
            .expect("records")
            .len(),
        1
    );
}

#[tokio::test]
async fn deletes_only_the_local_app_record() {
    let studio = TempDir::new().expect("studio root");
    let app = TempDir::new().expect("app root");
    fs::write(app.path().join("main.dowe"), "main\n").expect("main");
    init_database(studio.path(), "dowe-studio-apps").expect("initialize database");

    let inserted = save_studio_app(
        studio.path(),
        &json!({ "path": app.path(), "name": "Test app" }),
    )
    .await
    .expect("save");
    let id = inserted
        .get("id")
        .and_then(|value| value.as_str())
        .expect("id");

    let deleted = delete_studio_app(studio.path(), &json!({ "id": id }))
        .await
        .expect("delete");

    assert_eq!(deleted, json!({ "changed": 1 }));
    assert!(app.path().join("main.dowe").is_file());
    let database = open_database(studio.path(), "dowe-studio-apps").expect("database");
    assert!(
        database
            .records("workspace_apps")
            .expect("records")
            .is_empty()
    );
}

#[tokio::test]
async fn lists_and_reads_project_files() {
    let temp = TempDir::new().expect("tempdir");
    fs::create_dir(temp.path().join("views")).expect("views");
    fs::create_dir(temp.path().join("target")).expect("target");
    fs::write(temp.path().join("main.dowe"), "main\n").expect("main");
    fs::write(temp.path().join("views/editor.dowe"), "page editor\n").expect("editor");
    fs::write(temp.path().join("target/generated"), "generated\n").expect("generated");
    fs::write(temp.path().join(".hidden"), "hidden\n").expect("hidden");
    fs::write(temp.path().join("AGENTS.md"), "private\n").expect("agents");

    let files = list_project_files(&json!({ "root": temp.path() }))
        .await
        .expect("list");
    assert_eq!(
        files,
        json!({
            "files": [{ "id":"main.dowe", "name":"main.dowe", "path":"main.dowe", "icon":"code-file" }],
            "folders": [{
                "id":"views",
                "name":"views",
                "path":"views",
                "files": [{ "id":"views/editor.dowe", "name":"editor.dowe", "path":"views/editor.dowe", "icon":"code-file" }],
                "folders": []
            }]
        })
    );

    let content = read_project_file(&json!({
        "root": temp.path(),
        "path": "views/editor.dowe"
    }))
    .await
    .expect("read");
    assert_eq!(content, json!("page editor\n"));
    assert!(
        read_project_file(&json!({
            "root": temp.path(),
            "path": "../outside"
        }))
        .await
        .is_err()
    );
}

#[tokio::test]
async fn prepares_bounded_context_without_private_files() {
    let temp = TempDir::new().expect("tempdir");
    fs::create_dir_all(temp.path().join("views/components")).expect("components");
    fs::create_dir_all(temp.path().join("server")).expect("server");
    fs::create_dir(temp.path().join("target")).expect("target");
    fs::write(
        temp.path().join("main.dowe"),
        "import viewRoutes from \"@/views/routes\"\n\nmain\n  views:viewRoutes\n",
    )
    .expect("main");
    fs::write(
            temp.path().join("views/routes.dowe"),
            "import Card from \"@/views/components/card\"\n\nviews viewRoutes\n  group path:\"/\" layout:Card\n",
        )
        .expect("routes");
    fs::write(
        temp.path().join("views/components/card.dowe"),
        "component Card\n  Text\n    \"Card\"\n",
    )
    .expect("component");
    fs::write(temp.path().join("server/private.dowe"), "fn private\n").expect("server");
    fs::write(temp.path().join(".env"), "OPENROUTER_API_KEY=secret\n").expect("env");
    fs::write(temp.path().join("AGENTS.md"), "private\n").expect("agents");
    fs::write(temp.path().join("target/generated.dowe"), "generated\n").expect("generated");

    let context = prepare_studio_context(&json!({
        "path": temp.path(),
        "query": "component",
        "selectedPaths": ["views/routes.dowe"]
    }))
    .await
    .expect("context");
    assert_eq!(context["protocolVersion"], 1);
    assert_eq!(context["profile"], "views");
    assert_eq!(context["requestType"], "clarify");
    assert_eq!(context["mode"], "dowe");
    assert_eq!(context["compilerVersion"], env!("CARGO_PKG_VERSION"));
    assert_eq!(context["workspaceId"].as_str().map(str::len), Some(64));
    let files = context["files"].as_array().expect("files");
    let paths = files
        .iter()
        .filter_map(|file| file["path"].as_str())
        .collect::<Vec<_>>();
    assert!(paths.contains(&"main.dowe"));
    assert!(paths.contains(&"views/routes.dowe"));
    assert!(paths.contains(&"views/components/card.dowe"));
    assert!(!paths.iter().any(|path| path.contains("target")));
    assert!(!paths.iter().any(|path| path.contains("AGENTS")));
    assert!(context["imports"].as_array().is_some_and(|imports| {
        imports
            .iter()
            .any(|value| value == "views/components/card.dowe")
    }));
    assert!(
        context["sourceFingerprint"]
            .as_str()
            .is_some_and(|value| value.len() == 64)
    );
    assert!(
        context["workspaceFingerprint"]
            .as_str()
            .is_some_and(|value| value.len() == 64)
    );
    assert!(temp.path().join(".dowe/studio-context-cache").is_dir());
    assert!(
        !context
            .to_string()
            .contains(&temp.path().to_string_lossy().to_string())
    );
    let compact_context = prepare_studio_context(&json!({
        "path": temp.path(),
        "query": "component",
        "selectedPaths": [],
        "detail": "compact"
    }))
    .await
    .expect("compact context");
    assert_eq!(compact_context["detail"], "compact");
    assert_eq!(compact_context["image"], "");
    assert!(
        compact_context["files"]
            .as_array()
            .is_some_and(|files| { files.iter().all(|file| file["content"] == "") })
    );
    let read_context = prepare_studio_context(&json!({
        "path": temp.path(),
        "query": "inspect context",
        "selectedPaths": [],
        "detail": "compact"
    }))
    .await
    .expect("read context");
    assert_eq!(read_context["requestType"], "context_read");
    let image_context = prepare_studio_context(&json!({
        "path": temp.path(),
        "query": "",
        "selectedPaths": [],
        "image": "data:image/png;base64,AA=="
    }))
    .await
    .expect("image context");
    assert_eq!(image_context["profile"], "viewReference");
    assert_eq!(image_context["requestType"], "vision_ui");
    assert_eq!(image_context["image"], "data:image/png;base64,AA==");
    assert!(
        prepare_studio_context(&json!({
            "path": temp.path(),
            "query": "",
            "selectedPaths": [],
            "image": "data:image/png;base64:not-base64"
        }))
        .await
        .is_err()
    );
    assert!(
        prepare_studio_context(&json!({
            "path": temp.path(),
            "query": "",
            "selectedPaths": ["../outside.dowe"]
        }))
        .await
        .is_err()
    );
}

