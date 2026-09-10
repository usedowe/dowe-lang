fn recover_cache_entry(root: &Path, key: &str) -> RuntimeResult<()> {
    let entry = root.join(key);
    let previous = root.join(format!(".{key}.previous"));
    if previous.exists() {
        if entry.exists() {
            fs::remove_dir_all(&previous)?;
        } else {
            fs::rename(&previous, &entry)?;
        }
    }
    for item in fs::read_dir(root)? {
        let item = item?;
        let name = item.file_name().to_string_lossy().to_string();
        if item.path().is_dir() && name.starts_with(&format!(".{key}.")) && name.ends_with(".tmp") {
            fs::remove_dir_all(item.path())?;
        }
    }
    Ok(())
}

fn publish_cache_entry(root: &Path, key: &str, staging: &Path) -> RuntimeResult<()> {
    let entry = root.join(key);
    let previous = root.join(format!(".{key}.previous"));
    if previous.exists() {
        fs::remove_dir_all(&previous)?;
    }
    if entry.exists() {
        fs::rename(&entry, &previous)?;
    }
    if let Err(error) = fs::rename(staging, &entry) {
        if previous.exists() && !entry.exists() {
            let _ = fs::rename(&previous, &entry);
        }
        return Err(error.into());
    }
    if previous.exists() {
        fs::remove_dir_all(previous)?;
    }
    Ok(())
}

fn publish_cache_entry_if_current(
    root: &Path,
    key: &str,
    staging: &Path,
    revision: Option<&DevModuleRevision>,
) -> RuntimeResult<bool> {
    let Some(revision) = revision else {
        publish_cache_entry(root, key, staging)?;
        return Ok(true);
    };
    let Some(result) = revision.run_if_current(|| publish_cache_entry(root, key, staging)) else {
        return Ok(false);
    };
    result?;
    Ok(true)
}

fn revision_is_current(revision: Option<&DevModuleRevision>) -> bool {
    revision.is_none_or(DevModuleRevision::is_current)
}

fn prune_toolchain_caches(root: &Path, active: &str) -> RuntimeResult<()> {
    let entries = fs::read_dir(root)?
        .filter_map(Result::ok)
        .filter(|entry| entry.path().is_dir())
        .filter_map(|entry| {
            let key = entry.file_name().to_string_lossy().to_string();
            is_cache_key(&key).then(|| CacheUse {
                last_used: fs::metadata(entry.path().join("last-used"))
                    .and_then(|metadata| metadata.modified())
                    .ok()
                    .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
                    .map(|duration| duration.as_nanos())
                    .unwrap_or_default(),
                key,
            })
        })
        .collect::<Vec<_>>();
    for key in cache_keys_to_prune(&entries, active, RETAINED_INACTIVE_TOOLCHAINS) {
        fs::remove_dir_all(root.join(key))?;
    }
    Ok(())
}

fn cache_keys_to_prune(entries: &[CacheUse], active: &str, retained: usize) -> Vec<String> {
    let mut inactive = entries
        .iter()
        .filter(|entry| entry.key != active)
        .cloned()
        .collect::<Vec<_>>();
    inactive.sort_by(|left, right| {
        right
            .last_used
            .cmp(&left.last_used)
            .then_with(|| right.key.cmp(&left.key))
    });
    inactive
        .into_iter()
        .skip(retained)
        .map(|entry| entry.key)
        .collect()
}

fn touch_cache_entry(entry: &Path, version: &str) -> RuntimeResult<()> {
    fs::write(entry.join("last-used"), version.as_bytes())?;
    Ok(())
}

fn staging_path(root: &Path, key: &str) -> PathBuf {
    static NEXT_ID: AtomicU64 = AtomicU64::new(1);
    let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
    root.join(format!(".{key}.{}.{}.tmp", std::process::id(), id))
}

fn is_cache_key(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn validate_relative_path(path: &Path, label: &str) -> RuntimeResult<()> {
    if path.as_os_str().is_empty()
        || path.is_absolute()
        || path
            .components()
            .any(|part| !matches!(part, Component::Normal(_)))
    {
        return Err(RuntimeError::new(format!(
            "{label} must use a safe relative path: {}",
            path.display()
        )));
    }
    Ok(())
}

