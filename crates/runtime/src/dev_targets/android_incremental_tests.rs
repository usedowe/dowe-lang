use super::{
    CACHE_SCHEMA, CacheManifest, CacheUse, CachedSource, SourceFingerprint, StagedBuildFailure,
    cache_keys_to_prune, copy_cache_entry, full_plan, plan_incremental,
    publish_cache_entry_if_current, retryable_if_incremental, should_retry_full,
};
use std::collections::BTreeMap;
use std::fs;
use std::sync::{Arc, Mutex};
use tempfile::TempDir;

#[test]
fn cache_hit_has_no_compilation_work() {
    let current = sources([
        ("DoweDevActivity.java", "core", false),
        ("DoweDevRouteHome.java", "home", true),
    ]);
    let manifest = manifest(&current);

    let plan = plan_incremental(&current, Some(&manifest), true);

    assert!(!plan.full_rebuild);
    assert!(plan.compile.is_empty());
    assert!(plan.remove.is_empty());
}

#[test]
fn route_change_compiles_only_changed_route() {
    let previous = sources([
        ("DoweDevActivity.java", "core", false),
        ("DoweDevRouteHome.java", "old", true),
        ("DoweDevRouteAbout.java", "about", true),
    ]);
    let current = sources([
        ("DoweDevActivity.java", "core", false),
        ("DoweDevRouteHome.java", "new", true),
        ("DoweDevRouteAbout.java", "about", true),
    ]);

    let plan = plan_incremental(&current, Some(&manifest(&previous)), true);

    assert!(!plan.full_rebuild);
    assert_eq!(plan.compile, ["DoweDevRouteHome.java"]);
    assert!(plan.remove.is_empty());
}

#[test]
fn removed_route_drops_only_removed_outputs() {
    let previous = sources([
        ("DoweDevActivity.java", "core", false),
        ("DoweDevRouteHome.java", "home", true),
        ("DoweDevRouteAbout.java", "about", true),
    ]);
    let current = sources([
        ("DoweDevActivity.java", "core", false),
        ("DoweDevRouteHome.java", "home", true),
    ]);

    let plan = plan_incremental(&current, Some(&manifest(&previous)), true);

    assert!(!plan.full_rebuild);
    assert!(plan.compile.is_empty());
    assert_eq!(plan.remove, ["DoweDevRouteAbout.java"]);
}

#[test]
fn core_change_forces_full_rebuild() {
    let previous = sources([
        ("DoweDevActivity.java", "old", false),
        ("DoweDevLayout0.java", "layout", true),
        ("DoweDevRouteHome.java", "home", true),
    ]);
    let core_changed = sources([
        ("DoweDevActivity.java", "new", false),
        ("DoweDevLayout0.java", "layout", true),
        ("DoweDevRouteHome.java", "home", true),
    ]);

    let core = plan_incremental(&core_changed, Some(&manifest(&previous)), true);

    assert_eq!(core, full_plan(&core_changed));
}

#[test]
fn layout_change_compiles_only_changed_layout() {
    let previous = sources([
        ("DoweDevActivity.java", "core", false),
        ("DoweDevLayout0.java", "old", true),
        ("DoweDevRouteHome.java", "home", true),
    ]);
    let current = sources([
        ("DoweDevActivity.java", "core", false),
        ("DoweDevLayout0.java", "new", true),
        ("DoweDevRouteHome.java", "home", true),
    ]);

    let plan = plan_incremental(&current, Some(&manifest(&previous)), true);

    assert!(!plan.full_rebuild);
    assert_eq!(plan.compile, ["DoweDevLayout0.java"]);
    assert!(plan.remove.is_empty());
}

#[test]
fn incomplete_or_missing_cache_forces_full_rebuild() {
    let current = sources([
        ("DoweDevActivity.java", "core", false),
        ("DoweDevRouteHome.java", "home", true),
    ]);
    let manifest = manifest(&current);

    assert_eq!(plan_incremental(&current, None, false), full_plan(&current));
    assert_eq!(
        plan_incremental(&current, Some(&manifest), false),
        full_plan(&current)
    );
}

#[test]
fn manifest_records_every_class_and_dex_output() {
    let current = sources([("DoweDevActivity.java", "core", false)]);
    let manifest = manifest(&current);
    let encoded = serde_json::to_vec(&manifest).expect("manifest json");
    let decoded: CacheManifest = serde_json::from_slice(&encoded).expect("manifest decode");
    let source = &decoded.sources["DoweDevActivity.java"];

    assert_eq!(source.classes, ["dev/dowe/generated/DoweDevActivity.class"]);
    assert_eq!(source.dex_files, ["dex/core/DoweDevActivity.dex"]);
}

#[test]
fn prune_keeps_active_and_newest_inactive_toolchain() {
    let entries = vec![
        cache_use('a', 1),
        cache_use('b', 4),
        cache_use('c', 3),
        cache_use('d', 2),
    ];

    let pruned = cache_keys_to_prune(&entries, &"a".repeat(64), 1);

    assert_eq!(pruned, ["c".repeat(64), "d".repeat(64)]);
}

#[test]
fn staging_copy_does_not_share_mutable_metadata() {
    let temp = TempDir::new().expect("temp dir");
    let entry = temp.path().join("entry");
    let staging = temp.path().join("staging");
    fs::create_dir_all(entry.join("classes")).expect("cache entry");
    fs::write(entry.join("manifest.json"), "active-manifest").expect("active manifest");
    fs::write(entry.join("last-used"), "active-version").expect("active marker");
    fs::write(entry.join("classes/Activity.class"), "class").expect("cached class");

    copy_cache_entry(&entry, &staging).expect("staging copy");
    fs::write(staging.join("manifest.json"), "staged-manifest").expect("staged manifest");
    fs::write(staging.join("last-used"), "staged-version").expect("staged marker");

    assert_eq!(
        fs::read_to_string(entry.join("manifest.json")).expect("manifest"),
        "active-manifest"
    );
    assert_eq!(
        fs::read_to_string(entry.join("last-used")).expect("marker"),
        "active-version"
    );
    assert!(staging.join("classes/Activity.class").is_file());
}

#[test]
fn superseded_revision_cannot_swap_staged_cache() {
    let temp = TempDir::new().expect("temp dir");
    let key = "a".repeat(64);
    let entry = temp.path().join(&key);
    let staging = temp.path().join("staging");
    fs::create_dir_all(&entry).expect("active cache");
    fs::create_dir_all(&staging).expect("staged cache");
    fs::write(entry.join("marker"), "active").expect("active marker");
    fs::write(staging.join("marker"), "staged").expect("staged marker");
    let revision = crate::dev_modules::DevModuleRevision::new(1, Arc::new(Mutex::new(2)));

    let published = publish_cache_entry_if_current(temp.path(), &key, &staging, Some(&revision))
        .expect("guarded publish");

    assert!(!published);
    assert_eq!(
        fs::read_to_string(entry.join("marker")).expect("marker"),
        "active"
    );
    assert!(staging.join("marker").is_file());
}

#[test]
fn dex_failure_retries_once_with_a_clean_full_plan() {
    let current = sources([
        ("DoweDevActivity.java", "core", false),
        ("DoweDevRouteHome.java", "new", true),
    ]);
    let previous = sources([
        ("DoweDevActivity.java", "core", false),
        ("DoweDevRouteHome.java", "old", true),
    ]);
    let incremental = plan_incremental(&current, Some(&manifest(&previous)), true);
    let full = full_plan(&current);
    let failure = || StagedBuildFailure {
        error: crate::error::RuntimeError::new("truncated dex shard"),
        retry_full: true,
    };

    assert!(should_retry_full(&incremental, &failure()));
    assert!(!should_retry_full(&full, &failure()));

    let incremental_javac = retryable_if_incremental::<()>(
        Err(crate::error::RuntimeError::new("truncated class shard")),
        &incremental,
    )
    .expect_err("incremental javac failure");
    let full_javac = retryable_if_incremental::<()>(
        Err(crate::error::RuntimeError::new("full javac failure")),
        &full,
    )
    .expect_err("full javac failure");

    assert!(should_retry_full(&incremental, &incremental_javac));
    assert!(!should_retry_full(&full, &full_javac));
}

fn sources<const N: usize>(values: [(&str, &str, bool); N]) -> BTreeMap<String, SourceFingerprint> {
    values
        .into_iter()
        .map(|(path, digest, shard)| {
            (
                path.to_string(),
                SourceFingerprint {
                    digest: digest.to_string(),
                    shard,
                },
            )
        })
        .collect()
}

fn manifest(values: &BTreeMap<String, SourceFingerprint>) -> CacheManifest {
    CacheManifest {
        schema: CACHE_SCHEMA,
        toolchain: "toolchain".to_string(),
        sources: values
            .iter()
            .map(|(path, source)| {
                let stem = path.trim_end_matches(".java");
                (
                    path.clone(),
                    CachedSource {
                        digest: source.digest.clone(),
                        classes: vec![format!("dev/dowe/generated/{stem}.class")],
                        dex_key: source.digest.clone(),
                        dex_files: vec![format!("dex/{}/{stem}.dex", source.digest)],
                    },
                )
            })
            .collect(),
    }
}

fn cache_use(value: char, last_used: u128) -> CacheUse {
    CacheUse {
        key: value.to_string().repeat(64),
        last_used,
    }
}
