#[cfg(test)]
mod tests {
    use super::docker_prompt_defaults;
    use super::parse_deploy_options;
    use super::should_auto_publish;
    use dowe_deploy::DeployEnvironment;
    use dowe_deploy::DeploySurface;
    use dowe_deploy::DeployTarget;
    use dowe_deploy::DockerDeployPreferences;
    use std::path::PathBuf;

    #[test]
    fn parses_cloudflare_publish_options() {
        let args = vec![
            "--target".to_string(),
            "cloudflare".to_string(),
            "--name".to_string(),
            "docs-app".to_string(),
            "--publish".to_string(),
            "--dry-run".to_string(),
        ];
        let options = parse_deploy_options(&args, PathBuf::from("/project"))
            .expect("parse")
            .expect("options");

        assert_eq!(options.target, DeployTarget::Cloudflare);
        assert_eq!(options.name.as_deref(), Some("docs-app"));
        assert!(options.publish);
        assert!(options.dry_run);
    }

    #[test]
    fn parses_stage_environment() {
        let args = vec![
            "--target".to_string(),
            "cloudflare-pages".to_string(),
            "--environment".to_string(),
            "stage".to_string(),
        ];

        let options = parse_deploy_options(&args, PathBuf::from("/project"))
            .expect("parse")
            .expect("options");

        assert_eq!(options.environment, DeployEnvironment::Stage);
    }

    #[test]
    fn parses_ssh_publish_options_without_password() {
        let args = vec![
            "--target".to_string(),
            "ssh".to_string(),
            "--host".to_string(),
            "server.example.com".to_string(),
            "--user".to_string(),
            "deploy".to_string(),
            "--key-file".to_string(),
            "/keys/deploy".to_string(),
            "--publish".to_string(),
        ];

        let options = parse_deploy_options(&args, PathBuf::from("/project"))
            .expect("parse")
            .expect("options");

        assert_eq!(options.target, DeployTarget::Ssh);
        assert_eq!(options.ssh_host.as_deref(), Some("server.example.com"));
        assert_eq!(options.ssh_user.as_deref(), Some("deploy"));
        assert_eq!(options.ssh_key_file, Some(PathBuf::from("/keys/deploy")));
        assert!(options.publish);
    }

    #[test]
    fn parses_docker_registry_and_image() {
        let args = vec![
            "--target".to_string(),
            "docker".to_string(),
            "--registry".to_string(),
            "ghcr.io/dowe".to_string(),
            "--image".to_string(),
            "docs-app:stable".to_string(),
            "--dry-run".to_string(),
        ];
        let options = parse_deploy_options(&args, PathBuf::from("/project"))
            .expect("parse")
            .expect("options");

        assert_eq!(options.target, DeployTarget::Docker);
        assert_eq!(options.registry.as_deref(), Some("ghcr.io/dowe"));
        assert_eq!(options.image.as_deref(), Some("docs-app:stable"));
        assert!(options.dry_run);
    }

    #[test]
    fn docker_menu_defaults_to_saved_project_preferences() {
        let saved = DockerDeployPreferences::new("ghcr.io/acme", "clinic-web:stage");
        let defaults = docker_prompt_defaults(
            PathBuf::from("/project").as_path(),
            None,
            None,
            Some(&saved),
        );

        assert_eq!(
            defaults,
            ("ghcr.io/acme".to_string(), "clinic-web:stage".to_string())
        );
    }

    #[test]
    fn explicit_docker_flags_override_saved_project_preferences() {
        let saved = DockerDeployPreferences::new("ghcr.io/acme", "clinic-web:stage");
        let defaults = docker_prompt_defaults(
            PathBuf::from("/project").as_path(),
            Some("docker.io"),
            Some("clinic-web"),
            Some(&saved),
        );

        assert_eq!(
            defaults,
            ("docker.io".to_string(), "clinic-web".to_string())
        );
    }

    #[test]
    fn parses_android_store_track() {
        let args = vec![
            "--target".to_string(),
            "android".to_string(),
            "--track".to_string(),
            "beta".to_string(),
            "--publish".to_string(),
        ];
        let options = parse_deploy_options(&args, PathBuf::from("/project"))
            .expect("parse")
            .expect("options");

        assert_eq!(options.target, DeployTarget::Android);
        assert_eq!(options.track.as_deref(), Some("beta"));
        assert!(options.publish);
    }

    #[test]
    fn leaves_target_to_menu_without_args() {
        assert!(
            parse_deploy_options(&[], PathBuf::from("/project"))
                .expect("parse")
                .is_none()
        );
    }

    #[test]
    fn defaults_web_surface_to_cloudflare_pages() {
        let options = super::parse_deploy_flags(
            &["--publish".to_string()],
            PathBuf::from("/project"),
            Some(DeployTarget::CloudflarePages),
        )
        .expect("options");

        assert_eq!(options.target, DeployTarget::CloudflarePages);
        assert!(options.publish);
    }

    #[test]
    fn interactive_web_pages_deploy_publishes_automatically() {
        assert!(should_auto_publish(
            DeploySurface::Web,
            DeployTarget::CloudflarePages
        ));
        assert!(should_auto_publish(
            DeploySurface::Server,
            DeployTarget::Cloudflare
        ));
        assert!(should_auto_publish(
            DeploySurface::Server,
            DeployTarget::Vercel
        ));
        assert!(should_auto_publish(
            DeploySurface::Web,
            DeployTarget::Vercel
        ));
        assert!(!should_auto_publish(
            DeploySurface::Server,
            DeployTarget::Docker
        ));
        assert!(!should_auto_publish(
            DeploySurface::Web,
            DeployTarget::Docker
        ));
        assert!(should_auto_publish(
            DeploySurface::Android,
            DeployTarget::Android
        ));
        assert!(should_auto_publish(DeploySurface::Ios, DeployTarget::Ios));
        assert!(should_auto_publish(
            DeploySurface::Server,
            DeployTarget::Ssh
        ));
        assert!(!should_auto_publish(DeploySurface::Web, DeployTarget::Dowe));
    }

    #[test]
    fn docker_supports_server_and_web_surfaces() {
        assert!(DeployTarget::Docker.supports_surface(DeploySurface::Server));
        assert!(DeployTarget::Docker.supports_surface(DeploySurface::Web));
        assert!(!DeployTarget::Docker.supports_surface(DeploySurface::Android));
    }

    #[test]
    fn dowe_cloud_selection_reports_coming_soon() {
        assert!(super::announce_dowe_cloud_coming_soon(DeployTarget::Dowe));
        assert!(!super::announce_dowe_cloud_coming_soon(
            DeployTarget::Docker
        ));
        assert_eq!(
            super::DOWE_CLOUD_COMING_SOON_MESSAGE,
            "Dowe Cloud deployment is coming soon. This option is not fully implemented yet and will be improved in a future release."
        );
    }
}

