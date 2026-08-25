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

#[test]
fn crud_generates_a_modal_editorial_dashboard() {
    let temp = TempDir::new().expect("tempdir");
    init_project(temp.path(), InitProjectOptions::new(ProjectTemplate::Crud)).expect("init");

    let layout =
        fs::read_to_string(temp.path().join("views/layouts/app.dowe")).expect("app layout");
    let page = fs::read_to_string(temp.path().join("views/pages/home.dowe")).expect("home page");
    let theme = fs::read_to_string(temp.path().join("theme.dowe")).expect("theme");

    assert!(theme.contains("primary"));

    assert!(layout.contains("Scaffold boxed:true"));
    assert!(layout.contains("AppBar boxed:true"));
    assert!(layout.contains("import session from \"@/views/store/session\""));
    assert!(layout.contains("signal sessionLoading value:true"));
    assert!(layout.contains("request res method:\"GET\" route:\"/api/auth/session\""));
    assert!(layout.contains("headers:{ Authorization:session.authorization }"));
    assert!(layout.contains("set session value:res.data"));
    assert!(layout.contains("reset session"));
    assert!(layout.contains("Splash bind:sessionLoading"));
    assert!(layout.contains("Icon name:\"svg-spinners:ring-resize\""));
    assert!(page.contains("EDITORIAL WORKSPACE"));
    assert!(page.contains("Latest stories"));
    assert!(page.contains("Table data:blogs"));
    assert!(page.contains("signal blogsLoading value:true"));
    assert!(page.contains("init\n    request res method:\"GET\" route:\"/api/blogs\""));
    assert!(!page.contains("route:\"/api/blogs\" autoload:true"));
    assert!(page.contains("Splash bind:blogsLoading"));
    assert!(page.contains("Icon name:\"svg-spinners:3-dots-bounce\""));
    assert!(!page.contains("Grid columns:{ xs:1 md:4 } gap:4"));
    for removed_copy in [
        "DATA MODEL",
        "2 tables",
        "Users and blogs stay explicit.",
        "ACCESS",
        "JWT",
        "Publishing requires a verified session.",
        "OWNERSHIP",
        "Scoped",
        "Writers can edit only their work.",
        "RUNTIME",
        "Rust",
        "One backend for every generated target.",
    ] {
        assert!(!page.contains(&format!("\"{removed_copy}\"")));
    }
    assert_eq!(page.matches("  Modal open:").count(), 4);
    for modal in [
        "registerModalOpen",
        "loginModalOpen",
        "createModalOpen",
        "editModalOpen",
    ] {
        assert!(page.contains(&format!("Modal open:{modal}")));
        assert!(page.contains(&format!("set {modal} value:false")));
    }

    let first_modal = page.find("  Modal open:").expect("first modal");
    let dashboard = &page[..first_modal];
    for control in ["Input ", "Password ", "Textarea "] {
        assert!(
            !dashboard.contains(control),
            "{control} rendered outside Modal"
        );
    }
    assert!(page.contains("toast value:{ type:\"success\""));

    assert!(theme.contains("fonts default:\"manrope\""));
    assert!(theme.contains("Card variant:\"solid\" scheme:\"surface\""));
    assert!(theme.contains("Button variant:\"solid\" scheme:\"primary\""));
    assert!(theme.contains("Avatar variant:\"solid\" scheme:\"primary\""));
    assert!(theme.contains("Chip variant:\"solid\" scheme:\"primary\""));
    for role in [
        "primary color:",
        "secondary color:",
        "accent color:",
        "muted color:",
        "background color:",
        "surface color:",
        "success color:",
        "info color:",
        "warning color:",
        "danger color:",
    ] {
        assert!(theme.contains(role), "theme missing {role}");
    }
}

#[test]
fn every_template_separates_view_and_server_modules() {
    for options in materialized_options() {
        let temp = TempDir::new().expect("tempdir");
        init_project(temp.path(), options).expect("init");

        for forbidden in [
            "routes",
            "pages",
            "layouts",
            "store",
            "types",
            "handlers",
            "middleware",
            "middlewares",
            "config",
            "migrations",
        ] {
            assert!(
                !temp.path().join(forbidden).exists(),
                "{} generated root {forbidden}",
                options.template().as_str()
            );
        }

        let main = fs::read_to_string(temp.path().join("main.dowe")).expect("main");
        assert!(main.contains("@/views/"));
        assert!(main.contains("@/server/"));
    }
}

#[test]
fn i18n_generates_complete_english_and_spanish_catalogs() {
    for options in materialized_options() {
        let temp = TempDir::new().expect("tempdir");
        let report = init_project(temp.path(), options.with_i18n(true)).expect("init");

        let en = fs::read_to_string(temp.path().join("i18n/en.dowe")).expect("english catalog");
        let es = fs::read_to_string(temp.path().join("i18n/es.dowe")).expect("spanish catalog");
        let page =
            fs::read_to_string(temp.path().join("views/pages/home.dowe")).expect("home page");
        let layout = temp
            .path()
            .join("views/layouts/app.dowe")
            .is_file()
            .then(|| {
                fs::read_to_string(temp.path().join("views/layouts/app.dowe"))
                    .expect("application layout")
            });

        assert!(report.i18n_enabled());
        assert!(en.starts_with("translations default:true\n"));
        assert!(es.starts_with("translations\n"));
        assert!(page.contains("i18n:\""));
        if options.template() == ProjectTemplate::Crud {
            assert!(page.contains("i18n:\"loading.blogs\""));
            assert!(
                layout
                    .as_deref()
                    .expect("CRUD layout")
                    .contains("i18n:\"loading.session\"")
            );
            assert!(en.contains("  loading\n"));
            assert!(en.contains("    session \"Validating your session\""));
            assert!(en.contains("    blogs \"Loading the latest stories\""));
        }
        assert!(!en.contains("translation key:"));
        assert_eq!(en.lines().count(), es.lines().count());
        compile_template(temp.path());
    }
}

#[test]
fn disabled_i18n_keeps_plain_view_source() {
    for options in materialized_options() {
        let temp = TempDir::new().expect("tempdir");
        let report = init_project(temp.path(), options).expect("init");
        let page =
            fs::read_to_string(temp.path().join("views/pages/home.dowe")).expect("home page");

        assert!(!report.i18n_enabled());
        assert!(!temp.path().join("i18n").exists());
        assert!(!page.contains("i18n:\""));
    }
}

#[tokio::test]
async fn crud_session_endpoint_revalidates_the_persisted_identity() {
    let temp = TempDir::new().expect("tempdir");
    init_project(temp.path(), InitProjectOptions::new(ProjectTemplate::Crud)).expect("init");
    let main_path = temp.path().join("main.dowe");
    let main = fs::read_to_string(&main_path)
        .expect("main")
        .replace("server port:8081", "server port:0");
    fs::write(main_path, main).expect("ephemeral server port");
    let project = compile_template_project(temp.path());
    let servers = start_dev_servers(
        project,
        DevServerTargets {
            backend: true,
            views: false,
            desktop: false,
        },
    )
    .await
    .expect("servers");
    let backend = format!("http://{}", servers.backend_addr.expect("backend address"));
    let client = reqwest::Client::new();

    let missing = client
        .get(format!("{backend}/api/auth/session"))
        .send()
        .await
        .expect("missing session");
    assert_eq!(missing.status(), reqwest::StatusCode::UNAUTHORIZED);

    let registration = client
        .post(format!("{backend}/api/auth/register"))
        .json(&serde_json::json!({
            "name": "Ada Lovelace",
            "email": "ada@example.com",
            "password": "analytical-engine"
        }))
        .send()
        .await
        .expect("register")
        .json::<serde_json::Value>()
        .await
        .expect("register json");
    let authorization = registration["data"]["authorization"]
        .as_str()
        .expect("authorization");
    let token = authorization
        .strip_prefix("Bearer ")
        .expect("bearer authorization");
    assert_eq!(token.len(), 26);
    assert!(dowe_id::validate_ulid(token).is_ok());

    let session = client
        .get(format!("{backend}/api/auth/session"))
        .header(reqwest::header::AUTHORIZATION, authorization)
        .send()
        .await
        .expect("session");
    assert_eq!(session.status(), reqwest::StatusCode::OK);
    let session = session
        .json::<serde_json::Value>()
        .await
        .expect("session json");
    assert_eq!(session["data"]["authenticated"], true);
    assert_eq!(session["data"]["guest"], false);
    assert_eq!(session["data"]["authorization"], authorization);
    assert_eq!(session["data"]["user"]["name"], "Ada Lovelace");
    assert_eq!(session["data"]["user"]["email"], "ada@example.com");

    let cache = dowe_cache::open_database(temp.path(), "dowe-sessions", true).expect("cache");
    assert!(
        cache
            .delete(&format!("session:{token}"))
            .expect("clear cache")
    );
    drop(cache);

    let rehydrated = client
        .get(format!("{backend}/api/auth/session"))
        .header(reqwest::header::AUTHORIZATION, authorization)
        .send()
        .await
        .expect("rehydrated session");
    assert_eq!(rehydrated.status(), reqwest::StatusCode::OK);

    let logout = client
        .post(format!("{backend}/api/auth/logout"))
        .header(reqwest::header::AUTHORIZATION, authorization)
        .send()
        .await
        .expect("logout");
    assert_eq!(logout.status(), reqwest::StatusCode::OK);

    let revoked = client
        .get(format!("{backend}/api/auth/session"))
        .header(reqwest::header::AUTHORIZATION, authorization)
        .send()
        .await
        .expect("revoked session");
    assert_eq!(revoked.status(), reqwest::StatusCode::UNAUTHORIZED);

    servers.shutdown().await.expect("shutdown");
}

#[test]
fn init_rejects_conflicts_without_partial_writes() {
    let temp = TempDir::new().expect("tempdir");
    fs::write(temp.path().join(".gitignore"), "user").expect("gitignore");

    let error = init_project(temp.path(), InitProjectOptions::new(ProjectTemplate::Blank))
        .expect_err("error");

    assert!(error.to_string().contains(".gitignore"));
    assert!(!temp.path().join("main.dowe").exists());
}

#[test]
fn init_rejects_existing_zed_settings_without_partial_writes() {
    let temp = TempDir::new().expect("tempdir");
    fs::create_dir_all(temp.path().join(".zed")).expect("zed directory");
    fs::write(temp.path().join(".zed/settings.json"), "{}").expect("zed settings");

    let error = init_project(temp.path(), InitProjectOptions::new(ProjectTemplate::Blank))
        .expect_err("error");

    assert!(error.to_string().contains(".zed/settings.json"));
    assert!(!temp.path().join(".gitignore").exists());
    assert!(!temp.path().join("main.dowe").exists());
}

#[test]
fn confirmed_reinstall_replaces_managed_files_and_preserves_unrelated_files() {
    let temp = TempDir::new().expect("tempdir");
    fs::write(temp.path().join("main.dowe"), "user main").expect("main");
    fs::write(temp.path().join("notes.md"), "keep").expect("notes");

    let report = init_project(
        temp.path(),
        InitProjectOptions::new(ProjectTemplate::Blank).with_reinstall(true),
    )
    .expect("reinstall");

    assert!(report.reinstalled());
    assert_ne!(
        fs::read_to_string(temp.path().join("main.dowe")).expect("main"),
        "user main"
    );
    assert_eq!(
        fs::read_to_string(temp.path().join("notes.md")).expect("notes"),
        "keep"
    );
}

#[cfg(unix)]
#[test]
fn confirmed_reinstall_rejects_managed_symlinks_before_writing() {
    use std::os::unix::fs::symlink;

    let temp = TempDir::new().expect("tempdir");
    let outside = TempDir::new().expect("outside");
    let outside_main = outside.path().join("main.dowe");
    fs::write(&outside_main, "outside").expect("outside main");
    symlink(&outside_main, temp.path().join("main.dowe")).expect("symlink");

    let error = init_project(
        temp.path(),
        InitProjectOptions::new(ProjectTemplate::Blank).with_reinstall(true),
    )
    .expect_err("error");

    assert!(error.to_string().contains("main.dowe"));
    assert_eq!(
        fs::read_to_string(&outside_main).expect("outside main"),
        "outside"
    );
    assert!(!temp.path().join(".gitignore").exists());
}

#[test]
fn init_rejects_unsafe_template_paths() {
    let temp = TempDir::new().expect("tempdir");
    let files = [TemplateFile::new("../outside.dowe", "bad")];
    let error = write_project_files(
        temp.path(),
        InitProjectOptions::new(ProjectTemplate::Blank),
        &files,
    )
    .expect_err("error");

    assert!(error.to_string().contains("unsafe init template path"));
    assert!(!temp.path().join("../outside.dowe").exists());
}
