    use super::*;
    use std::fs;
    use std::path::Path;
    use tempfile::TempDir;

    fn managed_skills() -> Vec<ManagedAgentSkill> {
        vec![ManagedAgentSkill {
            name: "dowe-core".to_string(),
            files: vec![
                ManagedAgentSkillFile {
                    path: "SKILL.md".to_string(),
                    content: "---\nname: dowe-core\ndescription: Core Dowe authoring.\n---\n"
                        .to_string(),
                },
                ManagedAgentSkillFile {
                    path: "references/project.md".to_string(),
                    content: "# Project\n".to_string(),
                },
            ],
        }]
    }

    #[test]
    fn init_project_harness_creates_project_agent_files() {
        let temp = TempDir::new().expect("tempdir");
        fs::create_dir_all(temp.path().join("src")).expect("src");

        let report =
            init_project_harness(temp.path(), InitOptions::default()).expect("init report");

        assert_eq!(report.created.len(), 4);
        assert!(temp.path().join(".agents/AGENTS.md").exists());
        assert!(temp.path().join(".agents/manifest.json").exists());
        assert!(temp.path().join(".agents/harnesses/tdd.md").exists());
        assert!(temp.path().join(".agents/plans").exists());
        assert!(!temp.path().join("agents").exists());
        assert!(!temp.path().join(".dowe").exists());

        let manifest = read_manifest(temp.path()).expect("manifest");

        assert_eq!(manifest.schema_version, "1");
        assert_eq!(manifest.mode, HarnessMode::Project);
        assert_eq!(manifest.agent_root, ".agents");
        assert_eq!(manifest.generated_evidence_root, ".dowe/agent-harnesses");
        assert_eq!(
            manifest.source_roots,
            [
                "main.dowe",
                "theme.dowe",
                ".env.example",
                "server",
                "types",
                "views"
            ]
        );
        assert!(manifest.tdd_required);
        assert!(
            manifest
                .validation_commands
                .iter()
                .any(|command| command.id == "harness-check")
        );
        assert!(
            manifest
                .validation_commands
                .iter()
                .any(|command| command.id == "codegraph-check")
        );
    }

    #[test]
    fn init_agent_project_creates_root_instructions_and_harness() {
        let temp = TempDir::new().expect("tempdir");

        let report = init_agent_project_with_skills(
            temp.path(),
            InitOptions::default(),
            &managed_skills(),
        )
        .expect("agent init");
        let root_agents = fs::read_to_string(temp.path().join("AGENTS.md")).expect("root agents");

        assert_eq!(report.created.len(), 8);
        assert!(temp.path().join("AGENTS.md").is_file());
        assert!(temp.path().join("CLAUDE.md").is_file());
        assert!(temp.path().join(".agents/AGENTS.md").is_file());
        assert!(temp.path().join(".agents/manifest.json").is_file());
        assert!(temp.path().join(".agents/harnesses/tdd.md").is_file());
        assert!(temp.path().join(".agents/plans").is_dir());
        assert!(
            temp.path()
                .join(".agents/skills/dowe-core/SKILL.md")
                .is_file()
        );
        assert!(
            temp.path()
                .join(".agents/skills/dowe-core/references/project.md")
                .is_file()
        );
        assert!(root_agents.contains(".agents/skills"));
        assert!(root_agents.contains("Spec -> Contract -> Tests"));
        assert!(root_agents.contains("CodeGraph"));
        assert!(root_agents.contains("Agent Harness"));
        assert!(!root_agents.contains("dowe agent context project --json"));
        assert!(!root_agents.contains("dowe agent examples search"));
        assert!(!root_agents.contains("dowe agent skills"));
        assert!(!root_agents.contains("dowe agent chat"));
        assert!(!root_agents.contains("Node.js"));
        assert!(!root_agents.contains("Tailwind"));
        let root_claude =
            fs::read_to_string(temp.path().join("CLAUDE.md")).expect("root claude");
        assert!(root_claude.contains("AGENTS.md"));
        assert!(!temp.path().join(".dowe").exists());

        let manifest = read_manifest(temp.path()).expect("manifest");
        assert_eq!(manifest.managed_skills, vec!["dowe-core"]);

        let repeated = init_agent_project_with_skills(
            temp.path(),
            InitOptions::default(),
            &managed_skills(),
        )
        .expect("repeated agent init");
        assert!(repeated.created.is_empty());
        assert_eq!(repeated.preserved.len(), 8);
        assert!(!temp.path().join(".dowe").exists());
    }

    #[test]
    fn harness_check_warns_when_project_agent_version_differs() {
        let temp = TempDir::new().expect("tempdir");
        init_agent_project_with_skills(temp.path(), InitOptions::default(), &managed_skills())
            .expect("agent init");
        let path = temp.path().join(".agents/manifest.json");
        let content = fs::read_to_string(&path).expect("manifest");
        fs::write(
            &path,
            content.replace(env!("CARGO_PKG_VERSION"), "0.0.0"),
        )
        .expect("stale manifest");

        let report = check_harness(temp.path()).expect("check");
        let diagnostic = report
            .diagnostics
            .iter()
            .find(|diagnostic| diagnostic.code == "dowe_version_mismatch")
            .expect("version warning");

        assert_eq!(diagnostic.severity, DiagnosticSeverity::Warning);
        assert!(diagnostic.action.contains("verify `dowe version`"));
        assert!(!report.has_errors());
    }

    #[test]
    fn update_managed_skills_preserves_project_skills() {
        let temp = TempDir::new().expect("tempdir");
        let mut initial_skills = managed_skills();
        initial_skills[0]
            .files
            .retain(|file| file.path == "SKILL.md");
        initial_skills.push(ManagedAgentSkill {
            name: "dowe-canvas".to_string(),
            files: vec![ManagedAgentSkillFile {
                path: "SKILL.md".to_string(),
                content: "---\nname: dowe-canvas\ndescription: Old canvas skill.\n---\n".to_string(),
            }],
        });
        init_agent_project_with_skills(
            temp.path(),
            InitOptions::default(),
            &initial_skills,
        )
        .expect("init");
        fs::create_dir_all(temp.path().join(".agents/skills/project-review")).expect("project");
        fs::write(
            temp.path().join(".agents/skills/project-review/SKILL.md"),
            "project skill",
        )
        .expect("project skill");
        fs::write(
            temp.path().join(".agents/skills/dowe-core/SKILL.md"),
            "outdated",
        )
        .expect("outdated");
        fs::create_dir_all(temp.path().join(".agents/skills/dowe-core/references"))
            .expect("references");
        fs::write(
            temp.path()
                .join(".agents/skills/dowe-core/references/obsolete.md"),
            "obsolete",
        )
        .expect("obsolete");

        init_agent_project_with_skills(
            temp.path(),
            InitOptions {
                update_existing: true,
            },
            &managed_skills(),
        )
        .expect("update");

        assert!(
            fs::read_to_string(temp.path().join(".agents/skills/dowe-core/SKILL.md"))
                .expect("managed")
                .contains("Core Dowe authoring")
        );
        assert!(
            temp.path()
                .join(".agents/skills/dowe-core/references/project.md")
                .is_file()
        );
        assert!(
            !temp
                .path()
                .join(".agents/skills/dowe-core/references/obsolete.md")
                .exists()
        );
        assert!(!temp.path().join(".agents/skills/dowe-canvas").exists());
        assert_eq!(
            fs::read_to_string(temp.path().join(".agents/skills/project-review/SKILL.md"))
                .expect("project"),
            "project skill"
        );
    }

    #[test]
    fn managed_skill_paths_reject_traversal_before_root_instructions_are_written() {
        let temp = TempDir::new().expect("tempdir");
        let skills = vec![ManagedAgentSkill {
            name: "dowe-core".to_string(),
            files: vec![ManagedAgentSkillFile {
                path: "../../outside.md".to_string(),
                content: "outside".to_string(),
            }],
        }];

        let error = init_agent_project_with_skills(
            temp.path(),
            InitOptions::default(),
            &skills,
        )
        .expect_err("traversal");

        assert!(error.to_string().contains("managed skill path"));
        assert!(!temp.path().join("AGENTS.md").exists());
        assert!(!temp.path().join("outside.md").exists());
    }

    #[cfg(unix)]
    #[test]
    fn managed_skill_files_reject_symlinks() {
        use std::os::unix::fs::symlink;

        let temp = TempDir::new().expect("tempdir");
        let outside = TempDir::new().expect("outside");
        fs::create_dir_all(temp.path().join(".agents/skills/dowe-core")).expect("skill dir");
        let target = outside.path().join("SKILL.md");
        fs::write(&target, "outside").expect("outside skill");
        symlink(
            &target,
            temp.path().join(".agents/skills/dowe-core/SKILL.md"),
        )
        .expect("symlink");

        let error = init_agent_project_with_skills(
            temp.path(),
            InitOptions::default(),
            &managed_skills(),
        )
        .expect_err("skill symlink");

        assert!(error.to_string().contains("must not be a symlink"));
        assert_eq!(fs::read_to_string(target).expect("outside"), "outside");
        assert!(!temp.path().join("AGENTS.md").exists());
    }

    #[test]
    fn managed_agent_init_rejects_dowe_mode_without_creating_project_files() {
        let temp = TempDir::new().expect("tempdir");
        fs::create_dir_all(temp.path().join("agents")).expect("agents");
        fs::write(temp.path().join("AGENTS.md"), "# Dowe workspace\n").expect("root agents");
        fs::write(temp.path().join("agents/README.md"), "# Agents\n").expect("agents");

        let error = init_agent_project_with_skills(
            temp.path(),
            InitOptions::default(),
            &managed_skills(),
        )
        .expect_err("Dowe mode");

        assert!(error.to_string().contains("Dowe mode uses /agents"));
        assert!(!temp.path().join(".agents").exists());
    }

    #[test]
    fn init_agent_project_preserves_and_explicitly_updates_root_instructions() {
        let temp = TempDir::new().expect("tempdir");
        fs::write(temp.path().join("AGENTS.md"), "user instructions").expect("root agents");
        fs::write(temp.path().join("CLAUDE.md"), "user claude instructions")
            .expect("root claude");

        let preserved = init_agent_project(temp.path(), InitOptions::default()).expect("preserve");
        assert_eq!(
            fs::read_to_string(temp.path().join("AGENTS.md")).expect("preserved"),
            "user instructions"
        );
        assert_eq!(
            fs::read_to_string(temp.path().join("CLAUDE.md")).expect("preserved"),
            "user claude instructions"
        );
        assert!(
            preserved
                .preserved
                .iter()
                .any(|file| file.path == "AGENTS.md")
        );

        let updated = init_agent_project(
            temp.path(),
            InitOptions {
                update_existing: true,
            },
        )
        .expect("update");
        assert!(
            fs::read_to_string(temp.path().join("AGENTS.md"))
                .expect("updated")
                .contains("# Dowe Project Agent")
        );
        assert!(
            fs::read_to_string(temp.path().join("CLAUDE.md"))
                .expect("updated")
                .contains("AGENTS.md")
        );
        assert!(
            updated
                .created
                .iter()
                .any(|file| file.path == "AGENTS.md")
        );
    }

    #[cfg(unix)]
    #[test]
    fn init_agent_project_rejects_root_agents_symlink() {
        use std::os::unix::fs::symlink;

        let temp = TempDir::new().expect("tempdir");
        let outside = TempDir::new().expect("outside");
        let target = outside.path().join("AGENTS.md");
        fs::write(&target, "outside").expect("outside agents");
        symlink(&target, temp.path().join("AGENTS.md")).expect("symlink");

        let error = init_agent_project(temp.path(), InitOptions::default()).expect_err("symlink");

        assert!(error.to_string().contains("AGENTS.md must not be a symlink"));
        assert_eq!(fs::read_to_string(target).expect("outside"), "outside");
    }

    #[cfg(unix)]
    #[test]
    fn init_agent_project_rejects_root_claude_symlink() {
        use std::os::unix::fs::symlink;

        let temp = TempDir::new().expect("tempdir");
        let outside = TempDir::new().expect("outside");
        let target = outside.path().join("CLAUDE.md");
        fs::write(&target, "outside").expect("outside claude");
        symlink(&target, temp.path().join("CLAUDE.md")).expect("symlink");

        let error = init_agent_project(temp.path(), InitOptions::default()).expect_err("symlink");

        assert!(error.to_string().contains("CLAUDE.md must not be a symlink"));
        assert_eq!(fs::read_to_string(target).expect("outside"), "outside");
        assert!(!temp.path().join("AGENTS.md").exists());
        assert!(!temp.path().join(".agents").exists());
    }
