#[test]
fn lists_public_authoring_skills_without_workspace_skills() {
    let skills = public_skills();
    let ids = skills
        .iter()
        .map(|skill| skill.id.as_str())
        .collect::<Vec<_>>();
    let encoded = serde_json::to_string(&skills).expect("skills");

    assert_eq!(
        ids,
        [
            "core",
            "server",
            "native-ipc",
            "domain-modeling",
            "theme",
            "views"
        ]
    );
    assert!(skills.iter().all(|skill| skill.name.starts_with("dowe-")));
    assert!(skills.iter().all(|skill| skill.scope == "dowe-authoring"));
    assert!(
        skills
            .iter()
            .all(|skill| skill.path.starts_with("dowe-agent://skills/"))
    );
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
            "references/layouts.md",
            "references/composition.md",
            "references/blocks/index.json",
            "references/reference-ui.md",
            "references/components.md",
            "references/svg.md",
            "references/styles.md",
            "references/canvas.md",
            "references/table.md",
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
fn public_skills_follow_agent_skills_metadata_contract() {
    for skill in public_skills() {
        let document = get_public_skill(&skill.id, false).expect("skill document");
        let frontmatter = document
            .content
            .strip_prefix("---\n")
            .and_then(|content| content.split_once("\n---\n"))
            .map(|(header, _)| header)
            .expect("frontmatter");
        let name = frontmatter
            .lines()
            .find_map(|line| line.strip_prefix("name: "))
            .expect("name");
        let description = frontmatter
            .lines()
            .find_map(|line| line.strip_prefix("description: "))
            .expect("description");
        assert_eq!(name, skill.name);
        assert!(!name.is_empty() && name.len() <= 64);
        assert!(
            name.bytes()
                .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
        );
        assert!(!description.is_empty() && description.len() <= 1024);
        assert!(!document.content.contains("/Users/varb/"));
        assert!(!document.content.contains("/Users/varb/Work/dowe/"));
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
fn public_view_and_server_skills_connect_request_work() {
    let core = get_public_skill("core", false).expect("core");
    let server = get_public_skill("server", false).expect("server");
    let views = get_public_skill("views", false).expect("views");

    assert!(
        core.content
            .contains("load both `dowe-views` and `dowe-server`")
    );
    assert!(
        views
            .content
            .contains("load the companion `dowe-server` skill")
    );
    assert!(views.content.contains("request-to-route matrix"));
    assert!(views.content.contains("entities, migrations, Database"));
    assert!(
        server
            .content
            .contains("load the companion `dowe-views` skill")
    );
    assert!(server.content.contains("request-to-route matrix"));
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
        .join("src/embedded/examples/fullstack");

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
    assert!(full.content.contains("## Resource: references/svg.md"));
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
fn gets_the_focused_svg_skill_resource() {
    let resource =
        get_public_skill_resource("views", "references/svg.md").expect("svg resource");

    assert_eq!(resource.id, "views");
    assert_eq!(resource.name, "dowe-views");
    assert!(resource.content.contains("convert_svg"));
    assert!(resource.content.contains("format:\"source\""));
    assert!(resource.content.contains("Svg data:<reference>"));
    assert!(!resource.content.contains("/Users/"));
    assert!(!resource.content.contains("/agents/"));
}

#[test]
fn side_nav_skill_keeps_identity_on_the_root() {
    let layouts = get_public_skill_resource("views", "references/layouts.md")
        .expect("layouts resource");
    assert!(layouts.content.contains("item label:\"Home\" href:\"/\""));
    assert!(!layouts.content.contains("item id:"));

    let components = get_public_skill_resource("views", "references/components.md")
        .expect("components resource");
    assert!(components.content.contains("`item` entries do not accept `id`"));
}

