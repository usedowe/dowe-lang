use super::{
    InitProjectOptions, ProjectTemplate, TemplateFile, available_project_templates,
    has_dowe_project_marker, init_project, write_project_files,
};
use crate::{DevServerTargets, start_dev_servers};
use std::fs;
use std::path::Path;
use tempfile::TempDir;

fn compile_template(root: &Path) {
    compile_template_project(root);
}

fn compile_template_project(root: &Path) -> dowe_compiler::CompiledProject {
    let root = root.to_path_buf();
    std::thread::Builder::new()
        .stack_size(16 * 1024 * 1024)
        .spawn(move || dowe_compiler::compile_dev(root))
        .expect("spawn template compiler")
        .join()
        .expect("join template compiler")
        .expect("compile template")
}

fn materialized_options() -> [InitProjectOptions; 2] {
    [
        InitProjectOptions::new(ProjectTemplate::Blank),
        InitProjectOptions::new(ProjectTemplate::Crud),
    ]
}

#[test]
fn init_choices_use_canonical_order() {
    let names = available_project_templates()
        .iter()
        .map(|template| template.as_str())
        .collect::<Vec<_>>();

    assert_eq!(names, ["crud", "blank"]);
    assert_eq!(
        available_project_templates()
            .iter()
            .map(|template| template.label())
            .collect::<Vec<_>>(),
        ["CRUD", "blank"]
    );
}

#[test]
fn main_entrypoint_marks_an_existing_dowe_project() {
    let temp = TempDir::new().expect("tempdir");
    assert!(!has_dowe_project_marker(temp.path()));

    fs::write(temp.path().join("theme.dowe"), "design\n").expect("theme");
    assert!(!has_dowe_project_marker(temp.path()));

    fs::write(temp.path().join("main.dowe"), "main\n").expect("main");
    assert!(has_dowe_project_marker(temp.path()));
}

#[test]
fn every_project_template_enables_dowe_format_on_save() {
    for options in materialized_options() {
        let temp = TempDir::new().expect("tempdir");
        init_project(temp.path(), options).expect("init");
        let settings =
            fs::read_to_string(temp.path().join(".zed/settings.json")).expect("zed settings");
        let settings: serde_json::Value = serde_json::from_str(&settings).expect("valid settings");

        assert_eq!(
            settings.pointer("/languages/Dowe/formatter"),
            Some(&serde_json::Value::String("language_server".to_string()))
        );
        assert_eq!(
            settings.pointer("/languages/Dowe/format_on_save"),
            Some(&serde_json::Value::String("on".to_string()))
        );
        assert_eq!(
            settings.pointer("/languages/Dowe/preferred_line_length"),
            Some(&serde_json::Value::Number(100.into()))
        );
    }
}

#[test]
fn every_project_template_generates_grouped_theme_colors() {
    for options in materialized_options() {
        let temp = TempDir::new().expect("tempdir");
        init_project(temp.path(), options).expect("init");
        let theme = fs::read_to_string(temp.path().join("theme.dowe")).expect("theme");

        assert!(theme.contains("colors:\n"), "{theme}");
        assert!(theme.contains("primary color:\""), "{theme}");
        assert!(theme.contains("text:\""), "{theme}");
        assert!(theme.contains("title:\""), "{theme}");
        assert!(!theme.contains("primaryText:"), "{theme}");
        assert!(!theme.contains("primaryTitle:"), "{theme}");
        compile_template(temp.path());
    }
}

#[test]
fn custom_app_identity_is_written_to_main_for_each_template() {
    for template in [ProjectTemplate::Blank, ProjectTemplate::Crud] {
        let temp = TempDir::new().expect("tempdir");
        init_project(
            temp.path(),
            InitProjectOptions::new(template).with_app_identity("My App", "com.example.myapp"),
        )
        .expect("init");
        let main = fs::read_to_string(temp.path().join("main.dowe")).expect("main");
        assert!(main.contains("app name:\"My App\" bundle:\"com.example.myapp\""));
        assert!(main.contains(
            "main\n  app name:\"My App\" bundle:\"com.example.myapp\"\n  views:"
        ));
    }
}

#[test]
fn custom_app_identity_rejects_partial_values() {
    let temp = TempDir::new().expect("tempdir");
    let error = init_project(
        temp.path(),
        InitProjectOptions::new(ProjectTemplate::Blank).with_app_identity("", "com.example.app"),
    )
    .expect_err("invalid identity");
    assert!(error.to_string().contains("must both be non-empty"));
}

#[test]
fn blank_template_writes_hello_page_and_endpoint() {
    let temp = TempDir::new().expect("tempdir");
    let report =
        init_project(temp.path(), InitProjectOptions::new(ProjectTemplate::Blank)).expect("init");

    assert_eq!(report.template(), ProjectTemplate::Blank);
    assert_eq!(
        fs::read_to_string(temp.path().join(".gitignore")).expect("gitignore"),
        ".dowe\n.env\n.env.live\n.env.stage\n.env.uat\n"
    );
    assert!(temp.path().join(".env.example").is_file());
    assert!(temp.path().join(".env").is_file());
    assert!(temp.path().join(".env.live").is_file());
    assert!(temp.path().join(".env.stage").is_file());
    assert!(temp.path().join(".env.uat").is_file());
    assert!(
        fs::read_to_string(temp.path().join(".env.example"))
            .expect("environment example")
            .contains("DOWE_DEPLOY_ACCESS_PASSWORD=")
    );
    assert!(temp.path().join("main.dowe").is_file());
    assert!(temp.path().join("views/routes/view.dowe").is_file());
    assert!(temp.path().join("views/pages/home.dowe").is_file());
    assert!(temp.path().join("server/endpoints.dowe").is_file());
    assert!(temp.path().join("server/handlers/hello.dowe").is_file());
    assert!(!temp.path().join("views/layouts/app.dowe").exists());
    assert!(!temp.path().join("server/migrations").exists());
    assert!(!temp.path().join("migrations").exists());
    let theme = fs::read_to_string(temp.path().join("theme.dowe")).expect("theme");
    assert!(theme.contains("primary"));
    assert!(theme.contains("secondary"));
    assert!(
        fs::read_to_string(temp.path().join("views/pages/home.dowe"))
            .expect("home")
            .contains("Hello Dowe")
    );
    assert!(
        fs::read_to_string(temp.path().join("server/handlers/hello.dowe"))
            .expect("handler")
            .contains("Hello Dowe")
    );

    compile_template(temp.path());
}

#[test]
fn crud_writes_auth_owned_blogs_and_layered_server_modules() {
    let temp = TempDir::new().expect("tempdir");
    let report =
        init_project(temp.path(), InitProjectOptions::new(ProjectTemplate::Crud)).expect("init");

    assert_eq!(report.template(), ProjectTemplate::Crud);

    for path in [
        "server/handlers/users-handler.dowe",
        "server/handlers/blogs-handler.dowe",
        "server/services/users-service.dowe",
        "server/services/blogs-service.dowe",
        "server/repositories/users-repository.dowe",
        "server/repositories/blogs-repository.dowe",
        "server/entities/users-entity.dowe",
        "server/entities/blogs-entity.dowe",
        "server/entities/sessions-entity.dowe",
        "server/types/auth-types.dowe",
        "server/types/blogs-types.dowe",
    ] {
        assert!(
            temp.path().join(path).is_file(),
            "missing generated file {path}"
        );
    }

    let database =
        fs::read_to_string(temp.path().join("server/config/database.dowe")).expect("database");
    let env_example = fs::read_to_string(temp.path().join(".env.example")).expect("env example");
    let users_entity = fs::read_to_string(temp.path().join("server/entities/users-entity.dowe"))
        .expect("users entity");
    let blogs_entity = fs::read_to_string(temp.path().join("server/entities/blogs-entity.dowe"))
        .expect("blogs entity");
    let sessions_entity =
        fs::read_to_string(temp.path().join("server/entities/sessions-entity.dowe"))
            .expect("sessions entity");
    let routes = fs::read_to_string(temp.path().join("server/endpoints.dowe")).expect("routes");
    let blogs = fs::read_to_string(temp.path().join("server/handlers/blogs-handler.dowe"))
        .expect("blogs handler");
    let users = fs::read_to_string(temp.path().join("server/handlers/users-handler.dowe"))
        .expect("users handler");
    let blogs_service = fs::read_to_string(temp.path().join("server/services/blogs-service.dowe"))
        .expect("blogs service");
    let users_repository = fs::read_to_string(
        temp.path()
            .join("server/repositories/users-repository.dowe"),
    )
    .expect("users repository");
    let middleware =
        fs::read_to_string(temp.path().join("server/middlewares/auth.dowe")).expect("auth");

    assert!(database.contains("import Users from \"@/server/entities/users-entity\""));
    assert!(database.contains("import Blogs from \"@/server/entities/blogs-entity\""));
    assert!(database.contains("import Sessions from \"@/server/entities/sessions-entity\""));
    assert!(database.contains("provider:\"dowe\""));
    assert!(!database.contains("entity Users"));
    assert!(!database.contains("entity Blogs"));
    assert!(users_entity.contains("entity Users"));
    assert!(blogs_entity.contains("entity Blogs"));
    assert!(sessions_entity.contains("entity Sessions"));
    assert!(database.contains("entities:[Users Blogs Sessions] seeders:[]"));
    assert!(database.contains("cache appCache provider:\"dowe\""));
    assert!(env_example.contains("CACHE_HOST="));
    assert!(!env_example.contains("JWT_SECRET"));
    assert!(!temp.path().join("server/migrations").exists());
    assert!(routes.contains("@/server/middlewares/auth"));
    assert!(routes.contains("path:\"/api/auth\""));
    assert!(routes.contains("path:\"/register\""));
    assert!(routes.contains("path:\"/login\""));
    assert!(routes.contains("get path:\"/session\" handler:getSession"));
    assert!(routes.contains("path:\"/api/blogs\""));
    assert!(routes.contains("path:\"/:id\""));
    assert!(blogs.contains("createBlogService result args:"));
    assert!(!blogs.contains("let result = createBlogService"));
    assert!(!blogs.contains("handler createBlog async"));
    assert!(!blogs.contains("handler getSession"));
    assert!(!blogs.contains("conn:appDb."));
    assert!(!blogs.contains("query "));
    assert!(blogs_service.contains("updateBlogRepository updated args:"));
    assert!(!blogs_service.contains("let updated = updateBlogRepository"));
    assert!(!blogs_service.contains("response "));
    assert!(users.contains("registerUserService result args:"));
    assert!(!users.contains("handler registerUser async"));
    assert!(users.contains("handler getSession\n"));
    assert!(!users.contains("conn:appDb."));
    assert!(!users.contains("query "));
    assert!(!users.contains("jwt "));
    assert!(users_repository.contains("conn:appDb.insert table:\"users\""));
    assert!(users_repository.contains("id session source:\"ulid\""));
    assert!(
        users_repository.contains("str sessionKey source:\"join\" values:[\"session\" args.id]")
    );
    assert!(!users_repository.contains("let "));
    assert!(users_repository.contains("kv cached conn:appCache.set key:sessionKey"));
    assert!(!blogs.contains("return response"));
    assert!(!users.contains("return response"));
    assert!(middleware.contains("authorization:req.header.Authorization"));
    assert!(
        middleware
            .contains("session verified cache:appCache database:appDb token:token maxAge:2592000")
    );
    assert!(!middleware.contains("let verified = session.verify"));
    assert!(!middleware.contains("return response"));
    assert!(!middleware.contains("jwt "));
    compile_template(temp.path());
}

