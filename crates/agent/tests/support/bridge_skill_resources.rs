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
            .all(|example| example.source_path.starts_with("dowe-agent://examples/"))
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
    assert_eq!(context.skills.len(), 6);
    assert!(!encoded.contains("PRIVATE_WORKSPACE_SKILL"));
    assert!(!encoded.contains("agents/skills/private"));
    assert!(!encoded.contains("\"Home\""));
    assert!(!temp.path().join(".dowe").exists());
}

