#[cfg(test)]
mod font_asset_tests {
    use super::{font_asset_source, font_assets_source_roots_for_executable, font_family_source};
    use std::fs;
    use std::path::Path;
    use tempfile::TempDir;

    #[test]
    fn puts_project_font_assets_first() {
        let temp = TempDir::new().expect("temp directory");
        let project_assets = temp.path().join("assets/fonts");
        let executable_assets = temp.path().join("bin/assets/fonts");
        fs::create_dir_all(&project_assets).expect("project assets");
        fs::create_dir_all(&executable_assets).expect("executable assets");

        let roots = font_assets_source_roots_for_executable(temp.path(), None);

        assert_eq!(roots[0], project_assets);
    }

    #[test]
    fn puts_assets_next_to_the_executable_after_project_assets() {
        let temp = TempDir::new().expect("temp directory");
        let executable_dir = temp.path().join("bin");
        let executable_assets = executable_dir.join("assets/fonts");
        fs::create_dir_all(&executable_assets).expect("executable assets");
        let executable = executable_dir.join("dowe.exe");
        fs::write(&executable, "binary").expect("executable");

        let roots = font_assets_source_roots_for_executable(temp.path(), Some(&executable));

        assert_eq!(roots[1], executable_assets);
        assert!(!Path::new("assets/fonts").is_absolute());
    }

    #[test]
    fn falls_back_to_packaged_family_when_project_font_directory_is_partial() {
        let temp = TempDir::new().expect("temp directory");
        let project_assets = temp.path().join("assets/fonts");
        let packaged_assets = temp.path().join("bin/assets/fonts");
        fs::create_dir_all(project_assets.join("inter")).expect("project family");
        fs::create_dir_all(packaged_assets.join("manrope")).expect("packaged family");
        let executable = temp.path().join("bin/dowe.exe");

        let roots = font_assets_source_roots_for_executable(temp.path(), Some(&executable));
        let resolved = font_family_source(&roots, "manrope");

        assert_eq!(resolved, packaged_assets.join("manrope"));
    }

    #[test]
    fn falls_back_to_packaged_font_file_when_project_family_is_partial() {
        let temp = TempDir::new().expect("temp directory");
        let project_assets = temp.path().join("assets/fonts");
        let packaged_assets = temp.path().join("bin/assets/fonts");
        fs::create_dir_all(project_assets.join("manrope")).expect("project family");
        fs::create_dir_all(packaged_assets.join("manrope")).expect("packaged family");
        fs::write(
            packaged_assets.join("manrope/manrope-regular.ttf"),
            "regular",
        )
        .expect("packaged font");
        let executable = temp.path().join("bin/dowe.exe");

        let roots = font_assets_source_roots_for_executable(temp.path(), Some(&executable));
        let resolved = font_asset_source(&roots, "manrope", "manrope-regular.ttf");

        assert_eq!(
            resolved,
            packaged_assets.join("manrope/manrope-regular.ttf")
        );
    }
}
