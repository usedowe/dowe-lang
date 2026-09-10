#[cfg(test)]
mod project_asset_tests {
    use super::{
        asset_content_type, dev_client_script, safe_inspector_source_path, safe_project_asset_path,
        studio_preview_client_script,
    };
    use std::fs;
    use std::path::Path;
    use tempfile::TempDir;

    #[test]
    fn resolves_regular_project_assets_and_rejects_traversal() {
        let temp = TempDir::new().expect("tempdir");
        fs::create_dir_all(temp.path().join("icons/web")).expect("icons");
        fs::write(temp.path().join("icons/web/favicon-32x32.png"), "png").expect("asset");

        assert!(safe_project_asset_path(temp.path(), "icons", "web/favicon-32x32.png").is_some());
        assert!(safe_project_asset_path(temp.path(), "icons", "../main.dowe").is_none());
        assert!(safe_project_asset_path(temp.path(), "icons", "/etc/passwd").is_none());
        assert_eq!(asset_content_type(Path::new("favicon.png")), "image/png");
    }

    #[test]
    fn inspector_client_is_only_included_for_dev_web_output() {
        let inspector = dev_client_script(true, Some("http://127.0.0.1:8081/_dowe/dev/server/"));
        assert!(inspector.contains("Dowe inspect"));
        assert!(inspector.contains("setPointerCapture"));
        assert!(inspector.contains("dowe-inspector-position"));
        assert!(inspector.contains("left:\"16px\""));
        assert!(inspector.contains("right:\"auto\""));
        assert!(inspector.contains("Number.isFinite(top)"));
        assert!(inspector.contains("dowe-inspector-enabled"));
        assert!(inspector.contains("dowe-inspector-hidden"));
        assert!(inspector.contains("dowe-inspector-panel-open"));
        assert!(inspector.contains("KeyD"));
        assert!(inspector.contains("KeyR"));
        assert!(inspector.contains("#1f3a5f"));
        assert!(inspector.contains("#6bc670"));
        assert!(inspector.contains("rgb(31,58,95)"));
        assert!(!inspector.contains("__DOWE_INSPECTOR_ICON_SVG__"));
        assert!(inspector.contains("function solarIcon"));
        assert!(inspector.contains("Open Dowe Server Inspector"));
        assert!(inspector.contains("http://127.0.0.1:8081/_dowe/dev/server/"));
        assert!(inspector.contains("aria-label"));
        assert!(inspector.contains("Routes"));
        assert!(inspector.contains("Show details"));
        assert!(inspector.contains("loadManifest();"));
        assert!(inspector.contains("DOWE_STUDIO_CHANNEL"));
        assert!(inspector.contains("studio:dev:event"));
        assert!(!inspector.contains("inspectorPreview"));
        let studio_preview = studio_preview_client_script();
        assert!(studio_preview.contains("DOWE_STUDIO_PREVIEW"));
        assert!(studio_preview.contains("studio:view:hover"));
        assert!(studio_preview.contains("studio:view:selected"));
        assert!(studio_preview.contains("studio:builder:dragover"));
        assert!(studio_preview.contains("studio:builder:drop"));
        assert!(studio_preview.contains("application/x-dowe-builder"));
        assert!(studio_preview.contains("studioPreviewShowHover"));
        assert!(studio_preview.contains("if(!node)return null"));
        assert!(studio_preview.contains("studioPreviewLoadManifest"));
        assert!(studio_preview.contains("document.addEventListener(\"mousemove\",event=>{const node"));
        assert!(studio_preview.contains("nonce:doweStudioPreviewNonce"));
        assert!(!studio_preview.contains("Dowe Devtools"));
        let studio_host = include_str!("../studio_inspector_client.js");
        assert!(studio_host.contains("Dowe Studio Inspector"));
        assert!(studio_host.contains("Pasa el cursor"));
        assert!(studio_host.contains("studioInspectorSendToChat"));
        assert!(studio_host.contains("Send to chat"));
        assert!(studio_host.contains("sendButton.onclick"));
        assert!(!studio_host.contains("studioInspectorSendToChat(message.payload.node)"));
        assert!(studio_host.contains("data-dowe-studio-builder-item"));
        assert!(studio_host.contains("application/x-dowe-builder"));
        assert!(studio_host.contains("studio:builder:drop"));
        assert!(studio_host.contains("studioInspectorStageBuilderDrop"));
        assert!(studio_host.contains("studioInspector.selected"));
        assert!(studio_host.contains("functionName,args"));
        assert!(studio_host.contains("stageStudioChanges"));
        assert!(studio_host.contains("changePlan"));
        assert!(studio_host.contains("Builder change review"));
        assert!(studio_host.contains("#studio-chat"));
        assert!(studio_host.contains("applyStudioChanges"));
        assert!(studio_host.contains("rejectStudioChanges"));
        assert!(studio_host.contains("studio:view:hover"));
        assert!(!studio_host.contains("Dowe Devtools"));
        assert!(!inspector.contains("<iframe"));
        assert!(dev_client_script(true, None).contains("const SERVER_INSPECTOR_URL=null;"));
        assert!(!dev_client_script(false, None).contains("Dowe inspect"));
        assert!(!dev_client_script(false, None).contains("Inspector"));
        assert!(safe_inspector_source_path("views/pages/home.dowe"));
        assert!(!safe_inspector_source_path(""));
        assert!(!safe_inspector_source_path("../main.dowe"));
        assert!(!safe_inspector_source_path("/tmp/main.dowe"));
    }

    #[cfg(unix)]
    #[test]
    fn rejects_symlinked_project_assets() {
        let temp = TempDir::new().expect("tempdir");
        fs::create_dir_all(temp.path().join("icons")).expect("icons");
        fs::write(temp.path().join("outside.png"), "outside").expect("outside");
        std::os::unix::fs::symlink(
            temp.path().join("outside.png"),
            temp.path().join("icons/favicon.png"),
        )
        .expect("symlink");

        assert!(safe_project_asset_path(temp.path(), "icons", "favicon.png").is_none());
    }
}

