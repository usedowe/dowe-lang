use dowe_agent::{
    get_public_skill, get_public_skill_resource, handle_mcp_message, init_dowe_project,
    init_external_agent_project, project_context, public_skills, search_public_examples,
    summarize_codegraph_for, update_external_agent_project,
};
use dowe_agent_harness::{InitOptions, init_project_harness};
use dowe_components::BuiltinComponent;
use dowe_runtime::{InitProjectOptions, ProjectTemplate};
use serde_json::Value;
use std::fs;
use tempfile::TempDir;

const VIEW_COMPONENT_REFERENCE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../skill-data/dowe-views/references/components.md"
));

const VIEW_BLOCK_INDEX: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../skill-data/dowe-views/references/blocks/index.json"
));

#[test]
fn initializes_template_and_agent_bundle_as_one_project() {
    let temp = TempDir::new().expect("tempdir");

    let report = init_dowe_project(temp.path(), InitProjectOptions::new(ProjectTemplate::Blank))
        .expect("init");

    assert_eq!(report.project.template(), ProjectTemplate::Blank);
    assert!(temp.path().join("main.dowe").is_file());
    assert!(temp.path().join("AGENTS.md").is_file());
    assert!(temp.path().join("CLAUDE.md").is_file());
    assert!(temp.path().join(".agents/manifest.json").is_file());
    assert_eq!(
        fs::read_dir(temp.path().join(".agents/skills"))
            .expect("skills")
            .count(),
        5
    );
    dowe_compiler::compile_dev(temp.path()).expect("compile");
}

#[test]
fn every_dowe_project_template_includes_the_managed_agent_bundle() {
    for options in [
        InitProjectOptions::new(ProjectTemplate::Blank),
        InitProjectOptions::new(ProjectTemplate::Crud),
    ] {
        let temp = TempDir::new().expect("tempdir");

        init_dowe_project(temp.path(), options).expect("init");

        assert!(temp.path().join("AGENTS.md").is_file());
        assert!(temp.path().join("CLAUDE.md").is_file());
        assert!(temp.path().join(".agents/manifest.json").is_file());
        assert_eq!(
            fs::read_dir(temp.path().join(".agents/skills"))
                .expect("skills")
                .count(),
            5
        );
    }
}

#[test]
fn project_initialization_rejects_all_conflicts_before_writing() {
    let temp = TempDir::new().expect("tempdir");
    fs::write(temp.path().join("AGENTS.md"), "user-owned").expect("agents");

    let error = init_dowe_project(temp.path(), InitProjectOptions::new(ProjectTemplate::Blank))
        .expect_err("conflict");

    assert!(error.to_string().contains("AGENTS.md"));
    assert!(!temp.path().join("main.dowe").exists());
    assert!(!temp.path().join(".agents").exists());
}

#[test]
fn confirmed_reinstall_replaces_managed_project_and_agent_files() {
    let temp = TempDir::new().expect("tempdir");
    init_dowe_project(temp.path(), InitProjectOptions::new(ProjectTemplate::Blank)).expect("init");
    fs::write(temp.path().join("main.dowe"), "stale main").expect("main");
    fs::write(temp.path().join("AGENTS.md"), "stale agents").expect("agents");
    fs::write(temp.path().join("notes.md"), "keep").expect("notes");

    let report = init_dowe_project(
        temp.path(),
        InitProjectOptions::new(ProjectTemplate::Blank).with_reinstall(true),
    )
    .expect("reinstall");

    assert!(report.project.reinstalled());
    assert_ne!(
        fs::read_to_string(temp.path().join("main.dowe")).expect("main"),
        "stale main"
    );
    assert_ne!(
        fs::read_to_string(temp.path().join("AGENTS.md")).expect("agents"),
        "stale agents"
    );
    assert_eq!(
        fs::read_to_string(temp.path().join("notes.md")).expect("notes"),
        "keep"
    );
}

#[cfg(unix)]
#[test]
fn confirmed_reinstall_rejects_agent_symlinks_before_replacing_project_files() {
    use std::os::unix::fs::symlink;

    let temp = TempDir::new().expect("tempdir");
    let outside = TempDir::new().expect("outside");
    let outside_agents = outside.path().join("AGENTS.md");
    fs::write(temp.path().join("main.dowe"), "existing main").expect("main");
    fs::write(&outside_agents, "outside agents").expect("outside agents");
    symlink(&outside_agents, temp.path().join("AGENTS.md")).expect("symlink");

    let error = init_dowe_project(
        temp.path(),
        InitProjectOptions::new(ProjectTemplate::Blank).with_reinstall(true),
    )
    .expect_err("error");

    assert!(error.to_string().contains("AGENTS.md"));
    assert_eq!(
        fs::read_to_string(temp.path().join("main.dowe")).expect("main"),
        "existing main"
    );
    assert_eq!(
        fs::read_to_string(&outside_agents).expect("outside agents"),
        "outside agents"
    );
}

#[test]
fn lists_public_authoring_skills_without_workspace_skills() {
    let skills = public_skills();
    let ids = skills
        .iter()
        .map(|skill| skill.id.as_str())
        .collect::<Vec<_>>();
    let encoded = serde_json::to_string(&skills).expect("skills");

    assert_eq!(ids, ["core", "server", "domain-modeling", "theme", "views"]);
    assert!(skills.iter().all(|skill| skill.name.starts_with("dowe-")));
    assert!(skills.iter().all(|skill| skill.scope == "dowe-authoring"));
    assert!(skills.iter().all(|skill| {
        skill
            .resources
            .iter()
            .all(|resource| !resource.ends_with("documentation.md"))
    }));
    let views = skills
        .iter()
        .find(|skill| skill.id == "views")
        .expect("views skill");
    assert_eq!(
        views.resources,
        [
            "references/views.md",
            "references/composition.md",
            "references/blocks/index.json",
            "references/reference-ui.md",
            "references/components.md",
            "references/styles.md",
            "references/canvas.md",
            "scripts/visual_qa.py",
            "scripts/visual_qa_blueprint.py",
            "scripts/visual_qa_png.py"
        ]
    );
    assert!(views.description.contains("reference-driven UI"));
    assert!(views.description.contains("visual fidelity"));
    assert!(views.description.contains("without screenshot crops"));
    let domain = skills
        .iter()
        .find(|skill| skill.id == "domain-modeling")
        .expect("domain-modeling skill");
    assert_eq!(
        domain.resources,
        [
            "references/workflow.md",
            "references/pos.md",
            "references/crm.md",
            "references/ecommerce.md",
            "references/reservations.md"
        ]
    );
    assert!(domain.description.contains("business descriptions"));
    let domain_document = get_public_skill("domain-modeling", true)
        .expect("full domain-modeling skill")
        .content;
    for marker in [
        "description -> modules -> entities -> relations -> invariants -> permissions -> workflows -> endpoints -> seeders -> views",
        "# Point-of-sale blueprint",
        "# Customer relationship management blueprint",
        "# Ecommerce blueprint",
        "# Reservations and resource scheduling blueprint",
    ] {
        assert!(
            domain_document.contains(marker),
            "missing domain marker {marker}"
        );
    }
    assert!(!encoded.contains("/agents/skills"));
    assert!(!encoded.contains("dowe-dev-artifacts"));
    for skill in &skills {
        let full = get_public_skill(&skill.id, true).expect("full skill");
        assert!(!full.content.contains("Node.js"));
        assert!(!full.content.contains("Tailwind"));
    }
}

#[test]
fn every_resource_named_by_a_public_skill_is_embedded() {
    for skill in public_skills() {
        let compact = get_public_skill(&skill.id, false).expect("compact skill");
        let full = get_public_skill(&skill.id, true).expect("full skill");
        let named_resources = compact
            .content
            .split('`')
            .filter(|fragment| {
                (fragment.starts_with("references/")
                    && (fragment.ends_with(".md") || fragment.ends_with(".json")))
                    || (fragment.starts_with("scripts/") && fragment.ends_with(".py"))
            })
            .collect::<std::collections::BTreeSet<_>>();
        let embedded_resources = skill
            .resources
            .iter()
            .map(String::as_str)
            .collect::<std::collections::BTreeSet<_>>();

        assert_eq!(
            embedded_resources, named_resources,
            "{} names a resource that is not embedded",
            skill.name
        );
        for resource in &skill.resources {
            assert!(
                full.content.contains(&format!("## Resource: {resource}")),
                "{} full document omits {resource}",
                skill.name
            );
        }
    }
}

#[test]
fn public_skills_teach_canonical_view_and_server_directories() {
    let core = get_public_skill("core", true).expect("core");
    let server = get_public_skill("server", true).expect("server");
    let views = get_public_skill("views", true).expect("views");

    assert!(
        core.content
            .contains("Frontend modules belong under `views`")
    );
    assert!(
        core.content
            .contains("backend modules belong under `server`")
    );
    assert!(server.content.contains("under `server/`"));
    assert!(views.content.contains("under `views/`"));
}

#[test]
fn public_skills_group_related_entities_into_bounded_modules() {
    let domain = get_public_skill("domain-modeling", true).expect("domain modeling");
    let server = get_public_skill("server", true).expect("server");

    assert!(
        domain
            .content
            .contains("Default to one source file per cohesive bounded module")
    );
    assert!(
        domain
            .content
            .contains("Do not generate one file per entity by default")
    );
    assert!(server.content.contains("user-entities.dowe"));
    assert!(
        server
            .content
            .contains("import Users, UserRoles from \"@/server/entities/user-entities\"")
    );
}

#[test]
fn fullstack_skill_example_separates_frontend_and_backend_source() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../skill-data/examples/fullstack");

    assert!(root.join("views/types/blog.dowe").is_file());
    assert!(!root.join("types").exists());
    for entry in fs::read_dir(&root).expect("example root") {
        let entry = entry.expect("example entry");
        let name = entry.file_name();
        let name = name.to_string_lossy();
        assert!(
            matches!(
                name.as_ref(),
                ".env.example" | "main.dowe" | "theme.dowe" | "server" | "views"
            ),
            "unexpected fullstack root entry {name}"
        );
    }
}

#[test]
fn installs_and_updates_compiled_public_skills() {
    let temp = TempDir::new().expect("tempdir");

    init_external_agent_project(temp.path()).expect("init");

    let installed = temp.path().join(".agents/skills/dowe-views");
    let installed_skill = fs::read_to_string(installed.join("SKILL.md")).expect("skill");
    assert_eq!(
        installed_skill.trim_end(),
        get_public_skill("views", false)
            .expect("public skill")
            .content
    );
    let installed_views =
        fs::read_to_string(installed.join("references/views.md")).expect("views reference");
    assert!(installed_views.contains(r#""{blog.title}""#));
    assert!(installed_views.contains(r#"`"blog.title"` is literal text"#));
    let installed_components = fs::read_to_string(installed.join("references/components.md"))
        .expect("components reference");
    assert_eq!(installed_components, VIEW_COMPONENT_REFERENCE);
    for resource in [
        "references/views.md",
        "references/composition.md",
        "references/blocks/index.json",
        "references/reference-ui.md",
        "references/components.md",
        "references/styles.md",
        "references/canvas.md",
        "scripts/visual_qa.py",
        "scripts/visual_qa_blueprint.py",
        "scripts/visual_qa_png.py",
    ] {
        assert!(installed.join(resource).is_file(), "missing {resource}");
    }
    assert_eq!(
        fs::read_dir(temp.path().join(".agents/skills"))
            .expect("skills")
            .count(),
        5
    );
    let manifest: Value = serde_json::from_str(
        &fs::read_to_string(temp.path().join(".agents/manifest.json")).expect("manifest"),
    )
    .expect("manifest json");
    assert_eq!(manifest["doweVersion"], env!("CARGO_PKG_VERSION"));
    assert_eq!(
        manifest["managedSkills"],
        serde_json::json!([
            "dowe-core",
            "dowe-domain-modeling",
            "dowe-server",
            "dowe-theme",
            "dowe-views"
        ])
    );

    fs::create_dir_all(temp.path().join(".agents/skills/project-domain")).expect("project skill");
    fs::write(
        temp.path().join(".agents/skills/project-domain/SKILL.md"),
        "project-owned",
    )
    .expect("project skill");
    fs::write(installed.join("SKILL.md"), "stale").expect("stale");

    update_external_agent_project(temp.path()).expect("update");

    assert!(
        fs::read_to_string(installed.join("SKILL.md"))
            .expect("updated")
            .starts_with("---\nname: dowe-views\n")
    );
    assert_eq!(
        fs::read_to_string(temp.path().join(".agents/skills/project-domain/SKILL.md"))
            .expect("project skill"),
        "project-owned"
    );
}

#[test]
fn update_requires_an_initialized_agent_project() {
    let temp = TempDir::new().expect("tempdir");

    let error = update_external_agent_project(temp.path()).expect_err("missing init");

    assert!(error.to_string().contains("dowe agent init"));
}

#[test]
fn gets_compact_and_full_view_skill_documents() {
    let compact = get_public_skill("views", false).expect("compact");
    let full = get_public_skill("views", true).expect("full");

    assert_eq!(compact.id, "views");
    assert!(compact.content.starts_with("---\nname: dowe-views\n"));
    assert!(compact.content.contains("`references/components.md`"));
    assert!(!compact.content.contains("## Resource: references/views.md"));
    assert!(
        !compact
            .content
            .contains("## Resource: references/components.md")
    );
    assert!(full.content.contains("## Resource: references/views.md"));
    assert!(
        full.content
            .contains("## Resource: references/composition.md")
    );
    assert!(
        full.content
            .contains("## Resource: references/blocks/index.json")
    );
    assert!(
        full.content
            .contains("## Resource: references/reference-ui.md")
    );
    assert!(
        full.content
            .contains("## Resource: references/components.md")
    );
    assert!(full.content.contains("## Resource: references/styles.md"));
    assert!(full.content.contains("## Resource: references/canvas.md"));
    assert!(full.content.contains("## Resource: scripts/visual_qa.py"));
    assert!(
        full.content
            .contains("## Resource: scripts/visual_qa_blueprint.py")
    );
    assert!(
        full.content
            .contains("## Resource: scripts/visual_qa_png.py")
    );
    assert!(!full.content.contains("docs/views/"));
    assert!(!full.content.contains("dowe-docs/"));
    assert!(full.content.len() > compact.content.len());
    assert!(full.content.contains("Section boxed:true"));
    assert!(full.content.contains("Scaffold boxed:true"));
    assert!(full.content.contains("## Hero sections"));
    assert!(full.content.contains("## Landing-page section sequence"));
    assert!(full.content.contains("`96rem` on web"));
    assert!(full.content.contains("`1536` logical units"));
    assert_eq!(full.content.matches("`96rem` on web").count(), 2);
}

#[test]
fn block_index_is_valid_compact_and_self_contained() {
    let index: Value = serde_json::from_str(VIEW_BLOCK_INDEX).expect("block index json");

    assert_eq!(index["schemaVersion"], 1);
    assert_eq!(index["corpus"]["documentationPages"], 94);
    assert_eq!(index["corpus"]["blockEntries"], 75);
    assert_eq!(index["blocks"].as_array().expect("blocks").len(), 75);
    assert!(
        index["blocks"]
            .as_array()
            .expect("blocks")
            .iter()
            .any(|block| block["id"] == "hero/centered-media")
    );
    assert!(
        index["blocks"]
            .as_array()
            .expect("blocks")
            .iter()
            .any(|block| block["kind"] == "server")
    );
    assert!(!VIEW_BLOCK_INDEX.contains("dowe-docs/"));
    assert!(!VIEW_BLOCK_INDEX.contains("/Users/"));
    assert!(!VIEW_BLOCK_INDEX.contains("data:image/"));
}

#[test]
fn gets_one_declared_public_skill_resource() {
    let resource =
        get_public_skill_resource("views", "references/styles.md").expect("styles resource");

    assert_eq!(resource.id, "views");
    assert_eq!(resource.name, "dowe-views");
    assert_eq!(resource.path, "references/styles.md");
    assert!(
        resource
            .content
            .contains("# Style and design-system reference")
    );
    assert!(!resource.content.contains("# Canvas reference"));

    let traversal = get_public_skill_resource("views", "../SKILL.md").expect_err("traversal");
    let unknown =
        get_public_skill_resource("views", "references/missing.md").expect_err("unknown resource");
    assert!(
        traversal
            .to_string()
            .contains("unknown public Dowe skill resource")
    );
    assert!(
        unknown
            .to_string()
            .contains("unknown public Dowe skill resource")
    );
}

#[test]
fn view_skill_requires_faithful_reference_driven_composition() {
    let compact = get_public_skill("views", false).expect("compact views skill");
    let full = get_public_skill("views", true).expect("full views skill");
    let compact_content = compact
        .content
        .to_ascii_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");

    assert!(compact_content.contains("inventory"));
    assert!(compact_content.contains("preserve visible"));
    assert!(compact_content.contains("reference viewport"));
    assert!(
        full.content
            .contains("even when the route graph has one page")
    );
    assert!(full.content.contains("When an original is unavailable"));
    assert!(full.content.contains("AppBar and Footer"));
}

#[test]
fn view_skill_requires_dowe_native_reference_reconstruction() {
    let compact = get_public_skill("views", false).expect("compact views skill");
    let full = get_public_skill("views", true).expect("full views skill");
    let compact_content = compact
        .content
        .to_ascii_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");

    assert!(
        compact_content
            .contains("never use the reference image or crops derived from it as assets")
    );
    assert!(full.content.contains("validation evidence, never as"));
    assert!(
        full.content
            .contains("Rebuild navigation, headings, text, controls, cards, lists, metrics")
    );
    assert!(full.content.contains("independently obtained"));
    assert!(full.content.contains("rasterization, or recomposition"));
}

#[test]
fn view_skill_requires_semantic_ownership_and_collection_modeling() {
    let compact = get_public_skill("views", false).expect("compact views skill");
    let full = get_public_skill("views", true).expect("full views skill");

    assert!(
        compact
            .content
            .contains("composition map with ordered bands")
    );
    assert!(
        compact
            .content
            .contains("never copy sibling Cards or list units")
    );
    assert!(full.content.contains("never invent `MenuBar`"));
    assert!(
        full.content
            .contains("Fixed copy and records visible in the reference")
    );
    assert!(
        full.content
            .contains("Data loaded, filtered, paged, appended, or replaced")
    );
    assert!(
        full.content
            .contains("Reusable components do not accept dynamic caller inputs")
    );
    assert!(full.content.contains("page-only"));
    assert!(full.content.contains("theme unchanged"));
    assert!(full.content.contains("grouped `colors:` form"));
    assert!(
        compact
            .content
            .contains("layout, page, reusable component, or")
    );
    assert!(
        compact.content.contains(
            "Generate a theme or modify its colors only when the user explicitly requests"
        )
    );
}

#[test]
fn view_skill_reserves_box_for_advanced_layer_planes() {
    let compact = get_public_skill("views", false).expect("compact views skill");
    let full = get_public_skill("views", true).expect("full views skill");
    let index: Value = serde_json::from_str(VIEW_BLOCK_INDEX).expect("block index json");
    let auth_rules = index["families"]["auth"]["authoringRules"]
        .as_array()
        .expect("auth authoring rules")
        .iter()
        .map(|rule| rule.as_str().expect("auth rule"))
        .collect::<Vec<_>>()
        .join(" ");

    assert!(compact.content.contains("Begin with no `Box` nodes"));
    assert!(compact.content.contains("fixed viewport layer"));
    assert!(compact.content.contains("one responsive source tree"));
    assert!(
        full.content
            .contains("Do not use empty Boxes as Grid gutters")
    );
    assert!(
        full.content.contains(
            "Do not wrap `Input`, `Password`, `Phone`, `Pin`, `Button`, `Image`, or `Svg`"
        )
    );
    assert!(
        full.content
            .contains("Separate mobile and desktop form trees")
    );
    assert!(auth_rules.contains("without individual Box wrappers"));
    assert!(auth_rules.contains("advanced auth-layout layer plane"));
}

#[test]
fn view_skill_rejects_direct_same_kind_layout_nesting() {
    let compact = get_public_skill("views", false).expect("compact views skill");
    let composition = get_public_skill_resource("views", "references/composition.md")
        .expect("composition resource");
    let reference_ui = get_public_skill_resource("views", "references/reference-ui.md")
        .expect("reference UI resource");

    assert!(
        compact
            .content
            .contains("Never generate a `Grid` as a direct child of another `Grid`")
    );
    assert!(
        compact
            .content
            .contains("direct child of another `Flex`; flatten")
    );
    assert!(
        composition
            .content
            .contains("Direct same-kind layout nesting is a forbidden generated pattern")
    );
    assert!(
        composition
            .content
            .contains("| `Grid` directly inside `Grid` |")
    );
    assert!(
        composition
            .content
            .contains("| `Flex` directly inside `Flex` |")
    );
    assert!(
        reference_ui
            .content
            .contains("No direct `Grid`-in-`Grid` or `Flex`-in-`Flex` wrapper remains")
    );
}

#[test]
fn view_skill_keeps_horizontal_and_vertical_shell_navigation_separate() {
    let compact = get_public_skill("views", false).expect("compact views skill");
    let components = get_public_skill_resource("views", "references/components.md")
        .expect("components resource");
    let composition = get_public_skill_resource("views", "references/composition.md")
        .expect("composition resource");
    let views = get_public_skill_resource("views", "references/views.md").expect("views resource");
    let reference_ui = get_public_skill_resource("views", "references/reference-ui.md")
        .expect("reference UI resource");
    let example = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../skill-data/examples/reference-ui/views/components/site-navigation.dowe"
    ));

    assert!(
        compact
            .content
            .contains("`NavMenu` is horizontal shell navigation")
    );
    assert!(
        compact
            .content
            .contains("put a vertical `SideNav` in its `body`")
    );
    assert!(
        compact
            .content
            .contains("AppBar `start`, `center`, and `end` already lay out their direct children")
    );
    assert!(
        components
            .content
            .contains("never use it as the body of a `Drawer`, `Sidebar`")
    );
    assert!(
        components
            .content
            .contains("AppBar regions already provide a horizontal flex row")
    );
    assert!(
        composition
            .content
            .contains("mount a prop-free reusable `SideNav`")
    );
    assert!(
        composition
            .content
            .contains("Do not write `start > Flex` merely to place `Logo` beside `IconButton`")
    );
    assert!(
        views
            .content
            .contains("`SideNav` is vertical and is the navigation child")
    );
    assert!(
        views
            .content
            .contains("`IconButton` as direct region children; never add a wrapper `Flex`")
    );
    assert!(
        reference_ui
            .content
            .contains("`Drawer` containing a vertical `SideNav` in `body`")
    );
    assert!(
        reference_ui
            .content
            .contains("Direct `IconButton` before `Brand` in AppBar `start`")
    );
    assert!(example.contains("SideNav"));
    assert!(!example.contains("NavMenu"));
}

#[test]
fn view_skill_reserves_translation_for_advanced_visual_layers() {
    let compact = get_public_skill("views", false).expect("compact views skill");
    let full = get_public_skill("views", true).expect("full views skill");
    let index: Value = serde_json::from_str(VIEW_BLOCK_INDEX).expect("block index json");
    let app_bar_rules = index["families"]["app-bar"]["authoringRules"]
        .as_array()
        .expect("app bar authoring rules")
        .iter()
        .map(|rule| rule.as_str().expect("app bar rule"))
        .collect::<Vec<_>>()
        .join(" ");

    assert!(
        compact
            .content
            .contains("Zero authored translations is the default")
    );
    assert!(
        compact
            .content
            .contains("Never translate `AppBar`, `Brand`, `NavMenu`, `Drawer`")
    );
    assert!(
        compact
            .content
            .contains("First solve one-axis placement with `Flex`")
    );
    assert!(
        compact
            .content
            .contains("solve shared tracks and responsive")
    );
    assert!(
        full.content
            .contains("Availability is not a layout recommendation")
    );
    assert!(full.content.contains("Screenshot measurements describe"));
    assert!(full.content.contains("Measured `x` and `y` bounds are QA"));
    assert!(app_bar_rules.contains("Do not use translateX or translateY on AppBar"));
}

#[test]
fn view_skill_requires_reference_blueprints_and_visual_qa() {
    let compact = get_public_skill("views", false).expect("compact views skill");
    let full = get_public_skill("views", true).expect("full views skill");
    let script = get_public_skill_resource("views", "scripts/visual_qa.py").expect("script");

    assert!(
        compact
            .content
            .contains(".dowe/visual-qa/<screen>/blueprint.json")
    );
    assert!(full.content.contains("observed` or `inferred"));
    assert!(full.content.contains("`regions[].bounds`"));
    assert!(
        full.content
            .contains("Loading, populated, empty, and error")
    );
    assert!(full.content.contains("## Accessibility review"));
    assert!(full.content.contains("## Theme extraction"));
    assert!(full.content.contains(
        "An image supplied to build or adapt a layout, page, reusable component, or `Section`"
    ));
    assert!(
        full.content
            .contains("keep the existing theme colors unchanged")
    );
    assert!(full.content.contains("kind:\"dynamic\""));
    assert!(full.content.contains("report.json"));
    assert!(full.content.contains("diff.png"));
    assert!(
        script
            .content
            .contains("commands.add_parser(\"self-test\")")
    );
    assert!(
        script
            .content
            .contains("[dowe, \"dev\", \"--target\", \"web\"]")
    );
}

#[test]
fn theme_skill_requires_semantic_reference_system_extraction() {
    let compact = get_public_skill("theme", false).expect("compact theme skill");
    let full = get_public_skill("theme", true).expect("full theme skill");

    assert!(
        compact
            .content
            .contains("extracting a repeated visual system")
    );
    assert!(compact.content.contains("anti-aliased shade"));
    assert!(full.content.contains("## Reference-system extraction"));
    assert!(full.content.contains("background color:"));
    assert!(full.content.contains("smallest semantic palette"));
    assert!(full.content.contains("does not authorize an"));
    assert!(full.content.contains("request to create or change a theme"));
    assert!(
        full.content
            .contains("not create one solely from the image")
    );
}

#[test]
fn theme_skill_documents_grouped_color_families() {
    let full = get_public_skill("theme", true).expect("full theme skill");

    assert!(full.content.contains("colors:\n"));
    assert!(full.content.contains("primary color:"));
    assert!(full.content.contains("softPrimary color:"));
    assert!(full.content.contains("`color`, `text`, and `title` props"));
    assert!(!full.content.contains("colors primary:"));
    assert!(full.content.contains("## Theme/page contract"));
    assert!(full.content.contains("page-only"));
    assert!(!full.content.contains("Migrating a legacy theme"));
    assert!(!full.content.contains("onPrimary"));
}

#[test]
fn full_view_skill_covers_every_builtin_component() {
    let full = get_public_skill("views", true).expect("full views skill");
    let names = BuiltinComponent::ALL
        .iter()
        .map(|component| component.as_str())
        .collect::<std::collections::BTreeSet<_>>();

    assert_eq!(names.len(), BuiltinComponent::ALL.len());

    for component in BuiltinComponent::ALL {
        let name = component.as_str();
        let entry = format!("`{name}`");
        assert!(
            VIEW_COMPONENT_REFERENCE.contains(&entry),
            "missing public component entry for {name}"
        );
        assert!(full.content.contains(&entry), "full skill omits {name}");
    }

    for name in [
        "Pagination",
        "appBar",
        "top",
        "start",
        "center",
        "end",
        "bottom",
        "main",
        "bottomBar",
        "overlays",
        "header",
        "body",
        "footer",
        "trigger",
        "item",
        "divider",
        "submenu",
        "megamenu",
        "group",
        "column",
        "icon",
        "mark",
        "marker",
        "waypoint",
        "slide",
    ] {
        let entry = format!("`{name}`");
        assert!(
            VIEW_COMPONENT_REFERENCE.contains(&entry),
            "missing contextual component entry for {name}"
        );
    }

    for private_path in ["/agents/", "/docs/", "/dowe-docs/", "/specs/"] {
        assert!(!VIEW_COMPONENT_REFERENCE.contains(private_path));
    }
}

#[test]
fn rejects_unknown_public_skill_without_falling_back_to_private_paths() {
    let error = get_public_skill("dowe-dev-artifacts", false).expect_err("unknown");
    let removed = get_public_skill("canvas", false).expect_err("removed");

    assert!(error.to_string().contains("unknown public Dowe skill"));
    assert!(removed.to_string().contains("unknown public Dowe skill"));
    assert!(!error.to_string().contains("/agents/skills"));
}

#[test]
fn searches_curated_examples_deterministically() {
    let result = search_public_examples("dashboard sidebar form", 5).expect("search");

    assert_eq!(result.query, "dashboard sidebar form");
    assert_eq!(result.terms, ["dashboard", "form", "sidebar"]);
    assert!(!result.results.is_empty());
    assert_eq!(result.results[0].id, "dashboard-layout");
    assert!(result.results.iter().all(|example| example.score > 0));
    assert!(
        result
            .results
            .iter()
            .all(|example| example.source_path.starts_with("skill-data/examples/"))
    );
    assert!(
        result
            .results
            .iter()
            .all(|example| !example.source_path.contains("/.dowe/"))
    );
    let paths = result
        .results
        .iter()
        .map(|example| example.source_path.as_str())
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(paths.len(), result.results.len());
}

#[test]
fn searches_reference_ui_examples() {
    let result = search_public_examples("reference-ui each signal states", 5).expect("search");

    assert_eq!(result.results[0].id, "reference-collections");
    assert!(
        result
            .results
            .iter()
            .any(|example| example.id == "reference-layout")
    );
    assert!(result.results[0].content.contains("each in:features"));
    assert!(
        result.results[0]
            .content
            .contains("signal metrics type:Metric[]")
    );
}

#[test]
fn ranks_project_codegraph_nodes_from_the_request() {
    let temp = TempDir::new().expect("tempdir");
    fs::create_dir_all(temp.path().join("views/pages")).expect("views");
    fs::create_dir_all(temp.path().join("server/handlers")).expect("server");
    fs::write(temp.path().join("main.dowe"), "main\n").expect("main");
    fs::write(
        temp.path().join("views/pages/billing.dowe"),
        "page BillingPage\n",
    )
    .expect("billing");
    fs::write(
        temp.path().join("server/handlers/health.dowe"),
        "handler health\n",
    )
    .expect("health");

    let summary =
        summarize_codegraph_for(temp.path(), "update the billing page", 8).expect("summary");
    let paths = summary
        .relevant_nodes
        .iter()
        .filter_map(|node| node.path.as_deref())
        .collect::<Vec<_>>();

    assert_eq!(paths.first().copied(), Some("views/pages/billing.dowe"));
    assert!(!paths.contains(&"server/handlers/health.dowe"));
}

#[test]
fn builds_compact_project_context_without_private_skill_content() {
    let temp = tempfile::tempdir().expect("tempdir");
    fs::create_dir_all(temp.path().join("views/pages")).expect("views");
    fs::create_dir_all(temp.path().join("agents/skills/private")).expect("private");
    fs::write(
        temp.path().join("main.dowe"),
        "main\n  app name:\"Example\" bundle:\"dev.example\"\n",
    )
    .expect("main");
    fs::write(
        temp.path().join("views/pages/home.dowe"),
        "page homePage\n  Text\n    \"Home\"\n",
    )
    .expect("page");
    fs::write(
        temp.path().join("agents/skills/private/SKILL.md"),
        "PRIVATE_WORKSPACE_SKILL",
    )
    .expect("private skill");
    init_project_harness(temp.path(), InitOptions::default()).expect("harness");

    let context = project_context(temp.path()).expect("context");
    let encoded = serde_json::to_string(&context).expect("context json");

    assert_eq!(context.mode, "project");
    assert_eq!(context.source_file_count, 2);
    assert_eq!(context.source_files, ["main.dowe", "views/pages/home.dowe"]);
    assert!(context.markers.contains(&"main.dowe".to_string()));
    assert!(
        context
            .markers
            .contains(&".agents/manifest.json".to_string())
    );
    assert_eq!(context.skills.len(), 5);
    assert!(!encoded.contains("PRIVATE_WORKSPACE_SKILL"));
    assert!(!encoded.contains("agents/skills/private"));
    assert!(!encoded.contains("\"Home\""));
    assert!(!temp.path().join(".dowe").exists());
}

#[test]
fn handles_mcp_initialize_tools_and_resources() {
    let temp = tempfile::tempdir().expect("tempdir");
    fs::write(temp.path().join("main.dowe"), "main\n").expect("main");
    let initialize = handle_mcp_message(
        temp.path(),
        r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"test","version":"1"}}}"#,
    )
    .expect("initialize")
    .expect("response");
    let tools = handle_mcp_message(
        temp.path(),
        r#"{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}"#,
    )
    .expect("tools")
    .expect("response");
    let resources = handle_mcp_message(
        temp.path(),
        r#"{"jsonrpc":"2.0","id":3,"method":"resources/list","params":{}}"#,
    )
    .expect("resources")
    .expect("response");
    let initialized: Value = serde_json::from_str(&initialize).expect("initialize json");
    let listed_tools: Value = serde_json::from_str(&tools).expect("tools json");
    let listed_resources: Value = serde_json::from_str(&resources).expect("resources json");

    assert_eq!(initialized["result"]["protocolVersion"], "2025-11-25");
    assert_eq!(initialized["result"]["serverInfo"]["name"], "dowe-agent");
    assert_eq!(
        listed_tools["result"]["tools"]
            .as_array()
            .expect("tools array")
            .len(),
        4
    );
    assert!(
        listed_resources["result"]["resources"]
            .as_array()
            .expect("resources array")
            .iter()
            .any(|resource| resource["uri"] == "dowe://skills/views")
    );
    assert!(
        listed_resources["result"]["resources"]
            .as_array()
            .expect("resources array")
            .iter()
            .any(|resource| { resource["uri"] == "dowe://skills/views/references/styles.md" })
    );
    assert!(
        listed_resources["result"]["resources"]
            .as_array()
            .expect("resources array")
            .iter()
            .all(|resource| !resource["uri"]
                .as_str()
                .is_some_and(|uri| uri.ends_with("/full")))
    );
}

#[test]
fn handles_mcp_tool_calls_and_notifications() {
    let temp = tempfile::tempdir().expect("tempdir");
    fs::write(temp.path().join("main.dowe"), "main\n").expect("main");
    let response = handle_mcp_message(
        temp.path(),
        r#"{"jsonrpc":"2.0","id":"search","method":"tools/call","params":{"name":"dowe_examples_search","arguments":{"query":"dashboard sidebar form","limit":3}}}"#,
    )
    .expect("call")
    .expect("response");
    let notification = handle_mcp_message(
        temp.path(),
        r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#,
    )
    .expect("notification");
    let payload: Value = serde_json::from_str(&response).expect("response json");

    assert_eq!(payload["id"], "search");
    assert_eq!(payload["result"]["isError"], false);
    assert!(
        payload["result"]["structuredContent"]["results"]
            .as_array()
            .expect("results")
            .iter()
            .any(|result| result["id"] == "dashboard-layout")
    );
    assert!(notification.is_none());
}

#[test]
fn handles_mcp_public_skill_resource_tool_call() {
    let temp = tempfile::tempdir().expect("tempdir");
    let response = handle_mcp_message(
        temp.path(),
        r#"{"jsonrpc":"2.0","id":"resource","method":"tools/call","params":{"name":"dowe_skills_get","arguments":{"id":"views","resource":"references/styles.md"}}}"#,
    )
    .expect("call")
    .expect("response");
    let payload: Value = serde_json::from_str(&response).expect("response json");

    assert_eq!(payload["result"]["isError"], false);
    assert_eq!(
        payload["result"]["structuredContent"]["path"],
        "references/styles.md"
    );
    assert!(
        payload["result"]["structuredContent"]["content"]
            .as_str()
            .expect("content")
            .contains("# Style and design-system reference")
    );
    assert!(
        !payload["result"]["structuredContent"]["content"]
            .as_str()
            .expect("content")
            .contains("# Canvas reference")
    );
}

#[test]
fn reads_mcp_skill_resource_and_recovers_from_invalid_json() {
    let temp = tempfile::tempdir().expect("tempdir");
    let resource = handle_mcp_message(
        temp.path(),
        r#"{"jsonrpc":"2.0","id":4,"method":"resources/read","params":{"uri":"dowe://skills/views/full"}}"#,
    )
    .expect("resource")
    .expect("response");
    let invalid = handle_mcp_message(temp.path(), "{")
        .expect("invalid")
        .expect("parse response");
    let resource_payload: Value = serde_json::from_str(&resource).expect("resource json");
    let invalid_payload: Value = serde_json::from_str(&invalid).expect("invalid json");

    assert!(
        resource_payload["result"]["contents"][0]["text"]
            .as_str()
            .expect("text")
            .contains("## Resource: references/views.md")
    );
    assert!(
        resource_payload["result"]["contents"][0]["text"]
            .as_str()
            .expect("text")
            .contains("Every page starts with `Section`")
    );
    assert_eq!(invalid_payload["error"]["code"], -32700);
}
