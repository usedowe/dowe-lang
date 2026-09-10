fn bounded_text(value: &str, max_bytes: usize) -> String {
    let mut output = value.to_owned();
    if output.len() <= max_bytes {
        return output;
    }
    let mut end = max_bytes.min(output.len());
    while end > 0 && !output.is_char_boundary(end) {
        end -= 1;
    }
    output.truncate(end);
    output
}

/// Capture a bounded task baseline. The baseline is deliberately metadata-only
/// so it can be persisted in the session event stream without copying the
/// repository into model context. Review reads the affected files on demand.
pub(super) fn task_baseline(root: &Path) -> Value {
    const MAX_FILES: usize = 256;
    const MAX_FILE_BYTES: u64 = 2 * 1024 * 1024;
    let mut files = Vec::new();
    let redactor = super::Redactor::for_project(root);
    collect_snapshot_files(root, root, &mut files, MAX_FILES, MAX_FILE_BYTES, &redactor);
    files.sort_by(|left, right| left.0.cmp(&right.0));
    json!(files
        .into_iter()
        .map(|(path, fingerprint, size, preview)| json!({
            "path": path,
            "fingerprint": fingerprint,
            "size": size,
            "preview": preview,
        }))
        .collect::<Vec<_>>())
}

fn collect_snapshot_files(
    root: &Path,
    directory: &Path,
    files: &mut Vec<(String, String, u64, String)>,
    max_files: usize,
    max_file_bytes: u64,
    redactor: &super::Redactor,
) {
    if files.len() >= max_files {
        return;
    }
    let Ok(mut entries) = fs::read_dir(directory).map(|entries| entries.flatten().collect::<Vec<_>>()) else {
        return;
    };
    entries.sort_by_key(|entry| entry.path());
    for entry in entries {
        if files.len() >= max_files {
            break;
        }
        let path = entry.path();
        let relative = path.strip_prefix(root).unwrap_or(&path);
        let name = entry.file_name().to_string_lossy().to_string();
        let Ok(metadata) = fs::symlink_metadata(&path) else {
            continue;
        };
        if metadata.file_type().is_symlink() || snapshot_skip(relative, &name, metadata.is_dir()) {
            continue;
        }
        if metadata.is_dir() {
            collect_snapshot_files(root, &path, files, max_files, max_file_bytes, redactor);
            continue;
        }
        if !metadata.is_file() || metadata.len() > max_file_bytes {
            continue;
        }
        let Ok(bytes) = fs::read(&path) else {
            continue;
        };
        let relative = relative.to_string_lossy().replace('\\', "/");
        let fingerprint = super::digest(&bytes);
        let preview = String::from_utf8_lossy(&bytes[..bytes.len().min(2048)]).into_owned();
        files.push((
            relative,
            fingerprint,
            bytes.len() as u64,
            bounded_text(&redactor.text(&preview), 2048),
        ));
    }
}

fn snapshot_skip(relative: &Path, name: &str, is_dir: bool) -> bool {
    if matches!(name, ".git" | "target" | "node_modules" | ".DS_Store") {
        return true;
    }
    if name.starts_with(".env")
        || name.ends_with(".pem")
        || name.ends_with(".key")
        || name.ends_with(".p12")
        || relative.components().any(|component| {
            matches!(component, std::path::Component::Normal(value) if matches!(value.to_str(), Some("secrets" | "credentials")))
        })
    {
        return true;
    }
    if relative.starts_with(".dowe/codegraph")
        || relative.starts_with(".agents/capabilities")
        || relative.starts_with(".agents/sessions")
        || relative.starts_with(".agents/locks")
        || relative.starts_with(".agents/processes")
        || relative.starts_with(".agents/queues")
        || relative.starts_with(".agents/snapshots")
    {
        return true;
    }
    is_dir && name.starts_with('.') && name != ".agents" && name != ".dowe"
}

fn snapshot_map(value: &Value) -> BTreeMap<String, (String, u64, Option<String>)> {
    value
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|entry| {
            let path = entry["path"].as_str()?.to_owned();
            let fingerprint = entry["fingerprint"].as_str()?.to_owned();
            let size = entry["size"].as_u64().unwrap_or_default();
            let preview = entry["preview"].as_str().map(str::to_owned);
            Some((path, (fingerprint, size, preview)))
        })
        .collect()
}

fn read_review_file(root: &Path, path: &str) -> Option<String> {
    let relative = Path::new(path);
    if relative.is_absolute()
        || relative
            .components()
            .any(|component| component == std::path::Component::ParentDir)
    {
        return None;
    }
    fs::read_to_string(root.join(relative))
        .ok()
        .map(|value| bounded_text(&value, 8 * 1024))
}

/// Build a bounded review packet from the task baseline and confirmed
/// operation receipts. The reviewer does not receive repository-wide history
/// by default, and changes outside the tool ledger are surfaced as external
/// changes instead of silently disappearing.
fn review_change_context(root: &Path, session: &HarnessSession) -> String {
    const MAX_BYTES: usize = 48 * 1024;
    const MAX_FILE_BYTES: usize = 8 * 1024;
    let starts = session
        .events
        .iter()
        .enumerate()
        .filter_map(|(index, event)| (event["event"] == "task_started").then_some(index))
        .collect::<Vec<_>>();
    // The current marker belongs to `/review`; the immediately preceding
    // marker bounds the task whose confirmed changes are being reviewed.
    let boundary = starts
        .iter()
        .rev()
        .nth(1)
        .copied()
        .unwrap_or(0);
    let redactor = super::Redactor::for_project(root);
    #[derive(Clone)]
    struct Change {
        path: String,
        before: String,
        after: String,
        before_fingerprint: Option<String>,
        after_fingerprint: Option<String>,
        external: bool,
    }
    let mut changes = Vec::new();
    let mut baseline_available = false;

    // Prefer the durable baseline captured at the beginning of the task. It
    // lets review detect edits, additions, deletions and renames even when a
    // provider used shell or returned a partial tool batch.
    let baseline = starts
        .iter()
        .rev()
        .nth(1)
        .and_then(|index| session.events.get(*index))
        .map(|event| snapshot_map(&event["baseline"]));
    if let Some(baseline) = baseline {
        baseline_available = true;
        let current = snapshot_map(&task_baseline(root));
        let paths = baseline.keys().chain(current.keys()).cloned().collect::<BTreeSet<_>>();
        for path in paths {
            let before_entry = baseline.get(&path);
            let after_entry = current.get(&path);
            if before_entry.map(|entry| &entry.0) == after_entry.map(|entry| &entry.0) {
                continue;
            }
            let before_fingerprint = before_entry.map(|entry| entry.0.clone());
            let after_fingerprint = after_entry.map(|entry| entry.0.clone());
            let before = before_entry
                .and_then(|entry| entry.2.clone())
                .map(|value| redactor.text(&value))
                .unwrap_or_else(|| {
                    before_fingerprint
                        .as_deref()
                        .map(|fingerprint| format!("<baseline unavailable; sha256:{fingerprint}>"))
                        .unwrap_or_else(|| "<missing>".into())
                });
            let after = after_entry
                .and_then(|_| read_review_file(root, &path))
                .map(|value| redactor.text(&value))
                .unwrap_or_else(|| "<missing>".into());
            changes.push(Change {
                path,
                before,
                after,
                before_fingerprint,
                after_fingerprint,
                external: false,
            });
        }
    }

    let known_paths = changes
        .iter()
        .map(|change| change.path.clone())
        .collect::<BTreeSet<_>>();
    let mut receipt_paths = BTreeSet::new();
    for finished in session
        .events
        .iter()
        .skip(boundary)
        .filter(|event| event["event"] == "operation_finished" && event["failed"] != true)
    {
        let receipt = &finished["receipt"];
        let Some(path) = receipt["path"].as_str() else {
            continue;
        };
        receipt_paths.insert(path.to_owned());
        if known_paths.contains(path) {
            continue;
        }
        let call_id = finished["call_id"].as_str();
        let Some(details) = session.events.iter().skip(boundary).rev().find_map(|event| {
            (event["event"] == "approval_required"
                && call_id.is_some_and(|id| event["approval"]["call"]["id"] == id))
            .then_some(&event["approval"]["details"])
        }) else {
            continue;
        };
        let before = details["before"]
            .as_str()
            .map(|value| redactor.text(&bounded_text(value, MAX_FILE_BYTES)))
            .unwrap_or_else(|| "<missing>".into());
        let after = details["after"]
            .as_str()
            .map(|value| redactor.text(&bounded_text(value, MAX_FILE_BYTES)))
            .or_else(|| {
                HarnessTools::checked_path(root, path)
                    .ok()
                    .and_then(|safe| fs::read_to_string(safe).ok())
                    .map(|value| redactor.text(&bounded_text(&value, MAX_FILE_BYTES)))
            })
            .unwrap_or_else(|| "<unavailable>".into());
        changes.push(Change {
            path: path.to_owned(),
            before,
            after,
            before_fingerprint: details["before_sha256"].as_str().map(str::to_owned),
            after_fingerprint: receipt["afterFingerprint"].as_str().map(str::to_owned),
            external: baseline_available,
        });
    }
    if baseline_available {
        for change in &mut changes {
            change.external = !receipt_paths.contains(&change.path)
                && !change.path.starts_with(".agents/capabilities/");
        }
    }
    let mut output = String::new();
    let mut rendered = BTreeSet::new();
    for (index, change) in changes.iter().enumerate() {
        if rendered.contains(&index) {
            continue;
        }
        let mut label = change.path.clone();
        let mut related = None;
        if let Some((other_index, other)) = changes.iter().enumerate().find(|(other_index, other)| {
            *other_index != index
                && change.before == "<missing>"
                && other.after == "<missing>"
                && ((change.after_fingerprint.is_some()
                    && other.before_fingerprint == change.after_fingerprint)
                    || (change.after != "<unavailable>" && other.before == change.after))
        }) {
            label = format!("rename {} -> {}", other.path, change.path);
            related = Some(other_index);
        }
        let kind = if related.is_some() {
            "renamed"
        } else if change.before == "<missing>" {
            "added"
        } else if change.after == "<missing>" {
            "deleted"
        } else {
            "modified"
        };
        let origin = if change.external { " external" } else { "" };
        let entry = format!("\n--- {label} ({kind}{origin}) ---\n- before:\n{}\n- after:\n{}\n", change.before, change.after);
        if output.len().saturating_add(entry.len()) > MAX_BYTES {
            output.push_str("\n[remaining changed files omitted at review budget]\n");
            break;
        }
        output.push_str(&entry);
        rendered.insert(index);
        if let Some(other_index) = related {
            rendered.insert(other_index);
        }
    }
    output
}
