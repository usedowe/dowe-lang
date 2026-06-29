use crate::error::RuntimeResult;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};

pub(super) fn ios_app_cache_key(
    ios_root: &Path,
    target: &str,
    toolchain_signature: &[u8],
) -> RuntimeResult<String> {
    let mut files = ios_input_files(ios_root)?;
    files.sort();
    let mut digest = Sha256::new();
    update_digest(&mut digest, target.as_bytes());
    update_digest(&mut digest, toolchain_signature);
    for path in files {
        let relative = path
            .strip_prefix(ios_root)
            .expect("iOS input belongs to generated root");
        update_digest(&mut digest, relative.to_string_lossy().as_bytes());
        update_digest(&mut digest, &fs::read(path)?);
    }
    Ok(format!("{:x}", digest.finalize()))
}

pub(super) fn cached_ios_app(project_root: &Path, cache_key: &str) -> Option<PathBuf> {
    let bundle = ios_cache_entry(project_root, cache_key).join("DoweIosApp.app");
    bundle.join("DoweIosApp").is_file().then_some(bundle)
}

pub(super) fn publish_ios_app(
    project_root: &Path,
    cache_key: &str,
    bundle: &Path,
) -> RuntimeResult<PathBuf> {
    let cache_root = ios_cache_root(project_root);
    let cache_entry = ios_cache_entry(project_root, cache_key);
    if let Some(cached) = cached_ios_app(project_root, cache_key) {
        return Ok(cached);
    }
    fs::create_dir_all(&cache_root)?;
    if cache_entry.exists() {
        fs::remove_dir_all(&cache_entry)?;
    }
    let staging = cache_root.join(format!(".{cache_key}.{}.tmp", std::process::id()));
    if staging.exists() {
        fs::remove_dir_all(&staging)?;
    }
    fs::create_dir_all(&staging)?;
    fs::rename(bundle, staging.join("DoweIosApp.app"))?;
    match fs::rename(&staging, &cache_entry) {
        Ok(()) => {}
        Err(_) if cached_ios_app(project_root, cache_key).is_some() => {
            fs::remove_dir_all(staging)?;
        }
        Err(error) if cache_entry.exists() => {
            if cached_ios_app(project_root, cache_key).is_some() {
                fs::remove_dir_all(staging)?;
            } else {
                fs::remove_dir_all(&cache_entry)?;
                fs::rename(&staging, &cache_entry).map_err(|_| error)?;
            }
        }
        Err(error) => return Err(error.into()),
    }
    Ok(cached_ios_app(project_root, cache_key).expect("published iOS cache entry"))
}

fn ios_cache_root(project_root: &Path) -> PathBuf {
    project_root.join(".dowe/dev/ios/cache")
}

fn ios_cache_entry(project_root: &Path, cache_key: &str) -> PathBuf {
    ios_cache_root(project_root).join(cache_key)
}

fn ios_input_files(root: &Path) -> RuntimeResult<Vec<PathBuf>> {
    let mut files = Vec::new();
    collect_files(root, &mut files)?;
    Ok(files)
}

fn collect_files(root: &Path, files: &mut Vec<PathBuf>) -> RuntimeResult<()> {
    for entry in fs::read_dir(root)? {
        let path = entry?.path();
        if path.is_dir() {
            collect_files(&path, files)?;
        } else if path.is_file() {
            files.push(path);
        }
    }
    Ok(())
}

fn update_digest(digest: &mut Sha256, value: &[u8]) {
    digest.update(value.len().to_le_bytes());
    digest.update(value);
}

#[cfg(test)]
mod tests {
    use super::{cached_ios_app, ios_app_cache_key, publish_ios_app};
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn ios_cache_key_changes_with_inputs_target_and_toolchain() {
        let temp = tempdir().expect("tempdir");
        let ios_root = temp.path().join(".dowe/apps/ios");
        fs::create_dir_all(ios_root.join("Fonts")).expect("ios root");
        fs::write(ios_root.join("DoweIosApp.swift"), "struct App {}").expect("swift");
        fs::write(ios_root.join("Info.plist"), "plist").expect("plist");
        fs::write(ios_root.join("Fonts/Inter.ttf"), "font").expect("font");

        let original = ios_app_cache_key(&ios_root, "arm64-apple-ios17.0-simulator", b"swift-1")
            .expect("cache key");
        let target_changed =
            ios_app_cache_key(&ios_root, "x86_64-apple-ios17.0-simulator", b"swift-1")
                .expect("cache key");
        let toolchain_changed =
            ios_app_cache_key(&ios_root, "arm64-apple-ios17.0-simulator", b"swift-2")
                .expect("cache key");
        fs::write(ios_root.join("Fonts/Inter.ttf"), "updated-font").expect("font");
        let input_changed =
            ios_app_cache_key(&ios_root, "arm64-apple-ios17.0-simulator", b"swift-1")
                .expect("cache key");

        assert_ne!(original, target_changed);
        assert_ne!(original, toolchain_changed);
        assert_ne!(original, input_changed);
    }

    #[test]
    fn publishes_and_reuses_complete_ios_app_bundle() {
        let temp = tempdir().expect("tempdir");
        let bundle = temp.path().join(".dowe/dev/ios/build/1/DoweIosApp.app");
        fs::create_dir_all(&bundle).expect("bundle");
        fs::write(bundle.join("DoweIosApp"), "binary").expect("binary");
        fs::write(bundle.join("Info.plist"), "plist").expect("plist");

        assert!(cached_ios_app(temp.path(), "key").is_none());

        let published = publish_ios_app(temp.path(), "key", &bundle).expect("publish");
        let cached = cached_ios_app(temp.path(), "key").expect("cached bundle");

        assert_eq!(published, cached);
        assert_eq!(
            fs::read_to_string(cached.join("DoweIosApp")).expect("binary"),
            "binary"
        );
        assert!(!bundle.exists());
    }

    #[test]
    fn replaces_incomplete_ios_cache_entry() {
        let temp = tempdir().expect("tempdir");
        let stale = temp.path().join(".dowe/dev/ios/cache/key/DoweIosApp.app");
        fs::create_dir_all(&stale).expect("stale bundle");
        fs::write(stale.join("Info.plist"), "stale").expect("stale plist");
        let bundle = temp.path().join(".dowe/dev/ios/build/1/DoweIosApp.app");
        fs::create_dir_all(&bundle).expect("bundle");
        fs::write(bundle.join("DoweIosApp"), "binary").expect("binary");
        fs::write(bundle.join("Info.plist"), "plist").expect("plist");

        let published = publish_ios_app(temp.path(), "key", &bundle).expect("publish");

        assert_eq!(
            fs::read_to_string(published.join("DoweIosApp")).expect("binary"),
            "binary"
        );
        assert_eq!(
            fs::read_to_string(published.join("Info.plist")).expect("plist"),
            "plist"
        );
        assert!(!bundle.exists());
    }
}
