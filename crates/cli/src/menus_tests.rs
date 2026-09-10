#[cfg(test)]
mod tests {
    use super::{
        codegraph_commands, database_commands, deploy_target_default_index,
        dev_target_default_states, harness_commands, root_commands, should_prompt_simulator_quit,
    };
    use dowe_deploy::{DeploySurface, DeployTarget, deploy_targets_for_surface};
    use dowe_runtime::{DevTarget, DevTargetSelection, HostOs};

    #[test]
    fn root_menu_contains_root_cli_workflows() {
        assert_eq!(
            root_commands(),
            [
                "dev",
                "agent",
                "ai",
                "build",
                "cache",
                "codegraph",
                "d1",
                "database",
                "deploy",
                "icons",
                "init",
                "login",
                "queue",
                "test",
                "uninstall",
                "upgrade",
                "vector",
                "version"
            ]
        );
    }

    #[test]
    fn deploy_menu_contains_surface_targets() {
        assert_eq!(
            deploy_targets_for_surface(DeploySurface::Web),
            [
                DeployTarget::Dowe,
                DeployTarget::Docker,
                DeployTarget::CloudflarePages,
                DeployTarget::Vercel
            ]
        );
        assert_eq!(
            deploy_targets_for_surface(DeploySurface::Server),
            [
                DeployTarget::Dowe,
                DeployTarget::Docker,
                DeployTarget::Ssh,
                DeployTarget::Cloudflare,
                DeployTarget::Vercel
            ]
        );
        assert_eq!(
            deploy_targets_for_surface(DeploySurface::Android),
            [DeployTarget::Android]
        );
        assert_eq!(
            deploy_targets_for_surface(DeploySurface::Ios),
            [DeployTarget::Ios]
        );
    }

    #[test]
    fn deploy_target_menu_restores_saved_target_or_uses_first_available() {
        let targets = [
            DeployTarget::Dowe,
            DeployTarget::Docker,
            DeployTarget::Cloudflare,
        ];

        assert_eq!(
            deploy_target_default_index(&targets, Some(DeployTarget::Docker)),
            1
        );
        assert_eq!(
            deploy_target_default_index(&targets, Some(DeployTarget::Vercel)),
            0
        );
        assert_eq!(deploy_target_default_index(&targets, None), 0);
    }

    #[test]
    fn harness_menu_contains_interactive_safe_commands() {
        assert_eq!(harness_commands(), ["init", "check", "status"]);
    }

    #[test]
    fn codegraph_menu_contains_interactive_safe_commands() {
        assert_eq!(
            codegraph_commands(),
            ["build", "check", "report", "baseline"]
        );
    }

    #[test]
    fn database_menu_contains_all_database_commands() {
        assert_eq!(
            database_commands(),
            [
                "start",
                "create-account",
                "init",
                "list",
                "inspect",
                "query",
                "index",
                "compact",
                "bench",
                "migrate",
                "seeders"
            ]
        );
    }

    #[test]
    fn dev_target_menu_uses_supplied_defaults() {
        let targets = [
            DevTarget::Server,
            DevTarget::Web,
            DevTarget::Desktop,
            DevTarget::Android,
        ];
        let defaults =
            DevTargetSelection::new([DevTarget::Desktop, DevTarget::Android], HostOs::Linux)
                .expect("defaults");

        assert_eq!(
            dev_target_default_states(&targets, &defaults),
            [false, false, true, true]
        );
    }

    #[test]
    fn dev_target_menu_restores_persisted_server_selection() {
        let targets = [DevTarget::Server, DevTarget::Web, DevTarget::Desktop];
        let defaults =
            DevTargetSelection::new([DevTarget::Server, DevTarget::Desktop], HostOs::Linux)
                .expect("defaults");

        assert_eq!(
            dev_target_default_states(&targets, &defaults),
            [true, false, true]
        );
    }

    #[test]
    fn simulator_quit_prompt_requires_a_mobile_target() {
        let mobile =
            DevTargetSelection::new([DevTarget::Android], HostOs::Linux).expect("mobile selection");
        let web = DevTargetSelection::new([DevTarget::Web], HostOs::Linux).expect("web selection");

        assert!(should_prompt_simulator_quit(&mobile));
        assert!(!should_prompt_simulator_quit(&web));
    }
}
