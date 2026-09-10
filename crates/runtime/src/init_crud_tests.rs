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
    assert_eq!(page.matches("  Modal bind:").count(), 4);
    for modal in [
        "registerModalOpen",
        "loginModalOpen",
        "createModalOpen",
        "editModalOpen",
    ] {
        assert!(page.contains(&format!("Modal bind:{modal}")));
        assert!(page.contains(&format!("set {modal} value:false")));
    }

    let first_modal = page.find("  Modal bind:").expect("first modal");
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
        init_project(temp.path(), options.clone()).expect("init");

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
        let report = init_project(temp.path(), options.clone().with_i18n(true)).expect("init");

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

