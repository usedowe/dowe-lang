#[cfg(test)]
mod tests {
    use super::{IOS_INCREMENTAL_MODULE_NAME, IosHotModuleSnapshot, IosIncrementalWorkspace};
    use dowe_compiler::GeneratedFile;
    use std::fs;
    use std::path::{Path, PathBuf};
    use tempfile::tempdir;

    #[test]
    fn captures_ios_hot_sources_and_materializes_objc_revision() {
        let first_files = hot_module_files("pages", "route-a", "host");
        let second_files = hot_module_files("pages", "route-b", "host");
        let first = IosHotModuleSnapshot::from_generated_files(
            &first_files,
            "arm64-apple-ios17.0-simulator",
            b"swift-a",
        )
        .expect("snapshot");
        let second = IosHotModuleSnapshot::from_generated_files(
            &second_files,
            "arm64-apple-ios17.0-simulator",
            b"swift-a",
        )
        .expect("snapshot");

        assert_eq!(first.sources.len(), 3);
        assert!(
            first
                .sources
                .iter()
                .any(|source| source.relative_path == Path::new("DowePages.swift"))
        );
        assert_eq!(
            first.sources[2].relative_path,
            Path::new("dev/DoweIosViewModule.swift")
        );
        assert_ne!(first.version, second.version);
        let first_factory = factory_content(&first);
        let second_factory = factory_content(&second);
        assert!(first_factory.contains(&format!(
            "@objc(DoweIosDevModuleController_{})",
            first.version
        )));
        assert!(second_factory.contains(&format!(
            "@objc(DoweIosDevModuleController_{})",
            second.version
        )));
        assert!(!first_factory.contains("__DOWE_IOS_SOURCE_REVISION__"));
        assert!(first_factory.contains("@_cdecl(\"dowe_create_root_view_controller\")"));
        let first_pages = first
            .sources
            .iter()
            .find(|source| source.relative_path == Path::new("DowePages.swift"))
            .expect("first pages");
        let second_pages = second
            .sources
            .iter()
            .find(|source| source.relative_path == Path::new("DowePages.swift"))
            .expect("second pages");
        assert_eq!(first_pages.content, second_pages.content);
    }

    #[test]
    fn versions_module_and_cache_with_target_toolchain_sdk_and_host_abi() {
        let files = hot_module_files("pages", "route", "host-a");
        let source_change = hot_module_files("pages", "changed", "host-a");
        let host_change = hot_module_files("pages", "route", "host-b");
        let baseline = IosHotModuleSnapshot::from_generated_files(
            &files,
            "arm64-apple-ios17.0-simulator",
            b"swift-xcode-sdk-a",
        )
        .expect("baseline");
        let changed_source = IosHotModuleSnapshot::from_generated_files(
            &source_change,
            "arm64-apple-ios17.0-simulator",
            b"swift-xcode-sdk-a",
        )
        .expect("source");
        let changed_target = IosHotModuleSnapshot::from_generated_files(
            &files,
            "x86_64-apple-ios17.0-simulator",
            b"swift-xcode-sdk-a",
        )
        .expect("target");
        let changed_toolchain = IosHotModuleSnapshot::from_generated_files(
            &files,
            "arm64-apple-ios17.0-simulator",
            b"swift-xcode-sdk-b",
        )
        .expect("toolchain");
        let changed_host = IosHotModuleSnapshot::from_generated_files(
            &host_change,
            "arm64-apple-ios17.0-simulator",
            b"swift-xcode-sdk-a",
        )
        .expect("host");

        assert_ne!(baseline.version, changed_source.version);
        assert_eq!(baseline.cache_key, changed_source.cache_key);
        assert_ne!(baseline.version, changed_target.version);
        assert_ne!(baseline.cache_key, changed_target.cache_key);
        assert_ne!(baseline.version, changed_toolchain.version);
        assert_ne!(baseline.cache_key, changed_toolchain.cache_key);
        assert_ne!(baseline.version, changed_host.version);
        assert_ne!(baseline.cache_key, changed_host.cache_key);
    }

    #[test]
    fn prepares_incremental_sources_objects_and_dependency_map() {
        let temp = tempdir().expect("tempdir");
        let files = hot_module_files("pages", "route", "host");
        let snapshot = IosHotModuleSnapshot::from_generated_files(
            &files,
            "arm64-apple-ios17.0-simulator",
            b"swift-a",
        )
        .expect("snapshot");

        let workspace =
            IosIncrementalWorkspace::prepare(temp.path(), &snapshot).expect("workspace");
        let output_map: serde_json::Value =
            serde_json::from_slice(&fs::read(&workspace.output_map).expect("output map"))
                .expect("json");
        let first = workspace
            .sources
            .iter()
            .find(|source| source.source.ends_with("DowePages.swift"))
            .expect("pages source");

        assert_eq!(IOS_INCREMENTAL_MODULE_NAME, "DoweIosViewModule");
        assert_eq!(fs::read_to_string(&first.source).expect("source"), "pages");
        assert_eq!(
            output_map[""]["swift-dependencies"],
            workspace.master_dependencies.to_string_lossy().as_ref()
        );
        assert_eq!(
            output_map[first.source.to_string_lossy().as_ref()]["object"],
            first.object.to_string_lossy().as_ref()
        );
        assert_eq!(
            output_map[first.source.to_string_lossy().as_ref()]["swift-dependencies"],
            first.swift_dependencies.to_string_lossy().as_ref()
        );
    }

    #[test]
    fn preserves_current_objects_and_removes_obsolete_units() {
        let temp = tempdir().expect("tempdir");
        let mut first_files = hot_module_files("first", "route", "host");
        first_files.push(generated("apps/ios/Second.swift", "second", "ios"));
        let first = IosHotModuleSnapshot::from_generated_files(
            &first_files,
            "arm64-apple-ios17.0-simulator",
            b"swift-a",
        )
        .expect("snapshot");
        let workspace = IosIncrementalWorkspace::prepare(temp.path(), &first).expect("workspace");
        for source in &workspace.sources {
            fs::write(&source.object, source.source.to_string_lossy().as_bytes()).expect("object");
            fs::write(&source.swift_dependencies, b"deps").expect("dependencies");
        }
        fs::write(&workspace.dependency_graph, b"priors").expect("priors");
        assert!(workspace.is_complete());
        let first_source = workspace
            .sources
            .iter()
            .find(|source| source.source.ends_with("DowePages.swift"))
            .expect("first source");
        let obsolete = workspace
            .sources
            .iter()
            .find(|source| source.source.ends_with("Second.swift"))
            .expect("obsolete source");
        let first_object = first_source.object.clone();
        let obsolete_object = obsolete.object.clone();
        let obsolete_source = obsolete.source.clone();
        let second_files = hot_module_files("changed", "route", "host");
        let second = IosHotModuleSnapshot::from_generated_files(
            &second_files,
            "arm64-apple-ios17.0-simulator",
            b"swift-a",
        )
        .expect("snapshot");

        let next = IosIncrementalWorkspace::prepare(temp.path(), &second).expect("workspace");

        assert!(first_object.is_file());
        assert!(!obsolete_object.exists());
        assert!(!obsolete_source.exists());
        let next_pages = next
            .sources
            .iter()
            .find(|source| source.source.ends_with("DowePages.swift"))
            .expect("pages");
        assert_eq!(
            fs::read_to_string(&next_pages.source).expect("source"),
            "changed"
        );
    }

    #[test]
    fn invalidates_and_bounds_incremental_toolchain_caches() {
        let temp = tempdir().expect("tempdir");
        let files = hot_module_files("pages", "route", "host");
        let target = "arm64-apple-ios17.0-simulator";
        let first =
            IosHotModuleSnapshot::from_generated_files(&files, target, b"swift-a").expect("first");
        let second =
            IosHotModuleSnapshot::from_generated_files(&files, target, b"swift-b").expect("second");
        let third =
            IosHotModuleSnapshot::from_generated_files(&files, target, b"swift-c").expect("third");
        let first_key = first.cache_key.clone();
        let second_key = second.cache_key.clone();
        let third_key = third.cache_key.clone();

        IosIncrementalWorkspace::prepare(temp.path(), &first).expect("first workspace");
        IosIncrementalWorkspace::prepare(temp.path(), &second).expect("second workspace");
        IosIncrementalWorkspace::prepare(temp.path(), &third).expect("third workspace");

        let root = temp.path().join(".dowe/dev/ios/incremental");
        let retained = fs::read_dir(&root)
            .expect("caches")
            .filter_map(Result::ok)
            .filter(|entry| entry.path().is_dir())
            .map(|entry| entry.file_name().to_string_lossy().to_string())
            .collect::<Vec<_>>();
        assert_eq!(retained.len(), 2);
        assert!(retained.contains(&third_key));
        assert!(retained.contains(&first_key) || retained.contains(&second_key));
        assert_ne!(first_key, second_key);
        assert_ne!(second_key, third_key);
    }

    fn hot_module_files(pages: &str, route: &str, host: &str) -> Vec<GeneratedFile> {
        vec![
            generated("apps/ios/DoweIosApp.swift", "app", "ios"),
            generated("apps/ios/DowePages.swift", pages, "ios"),
            generated("apps/ios/DowePageIndexView.swift", route, "ios"),
            generated("apps/ios/dev/DoweIosDevHost.swift", host, "ios"),
            generated(
                "apps/ios/dev/DoweIosViewModule.swift",
                "@objc(DoweIosDevModuleController___DOWE_IOS_SOURCE_REVISION__)\n@_cdecl(\"dowe_create_root_view_controller\")",
                "ios",
            ),
            generated("apps/android/App.java", "android", "android-dev"),
        ]
    }

    fn factory_content(snapshot: &IosHotModuleSnapshot) -> &str {
        &snapshot
            .sources
            .iter()
            .find(|source| source.relative_path == Path::new("dev/DoweIosViewModule.swift"))
            .expect("factory")
            .content
    }

    fn generated(path: &str, content: &str, target: &str) -> GeneratedFile {
        GeneratedFile {
            relative_path: PathBuf::from(path),
            content: content.to_string(),
            kind: "test".to_string(),
            target: target.to_string(),
        }
    }
}
