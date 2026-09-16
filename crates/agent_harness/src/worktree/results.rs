use super::snapshot::{FileState, MAX_BYTES, Snapshot};
use super::*;
use crate::AllowedEditSurface;
use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Clone)]
pub struct WorkerResult {
    root: PathBuf,
    base: String,
    task_id: String,
    contributions: Contributions,
}

#[derive(PartialEq, Eq)]
struct Contribution {
    ancestors: BTreeSet<String>,
    changes: Snapshot,
}

type Contributions = BTreeMap<String, Arc<Contribution>>;

impl WorkerResult {
    pub fn task_id(&self) -> &str {
        &self.task_id
    }
    pub fn changed_files(&self) -> Vec<&str> {
        self.contributions[&self.task_id]
            .changes
            .keys()
            .map(String::as_str)
            .collect()
    }
}

pub fn capture_worker_result(
    root: &Path,
    worker: &IsolatedWorktree,
    task_id: &str,
    dependencies: &[WorkerResult],
    scopes: &[AllowedEditSurface],
) -> HarnessResult<WorkerResult> {
    if task_id.is_empty()
        || task_id.len() > 128
        || !task_id
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"-_".contains(&b))
    {
        return Err(HarnessError::new("invalid worker task ID"));
    }
    let root = root.canonicalize()?;
    ensure_git_root(&root)?;
    let base = git(&root, ["rev-parse", "HEAD"])?.stdout.trim().to_owned();
    if worker.base_revision != base {
        return Err(HarnessError::new("worker result has a stale Git base"));
    }
    let mut contributions = merge(&root, &base, dependencies)?;
    if contributions.contains_key(task_id) {
        return Err(HarnessError::new(
            "task ID is already present in dependency ancestry",
        ));
    }
    let baseline = resolve(&root, &base, &contributions)?;
    let current = snapshot::capture(&root, worker)?;
    let paths = baseline
        .keys()
        .chain(current.keys())
        .collect::<BTreeSet<_>>();
    let mut changes = Snapshot::new();
    for path in paths {
        let before = baseline.get(path).unwrap_or(&FileState::Unchanged);
        let after = current.get(path).unwrap_or(&FileState::Unchanged);
        if before != after {
            if !scopes.iter().any(|scope| scope.allows(path)) {
                return Err(HarnessError::new(format!(
                    "worker change is outside its declared scope: {path}"
                )));
            }
            changes.insert(path.clone(), after.clone());
        }
    }
    let ancestors = contributions.keys().cloned().collect();
    contributions.insert(
        task_id.into(),
        Arc::new(Contribution { ancestors, changes }),
    );
    check_bounds(&contributions)?;
    Ok(WorkerResult {
        root,
        base,
        task_id: task_id.into(),
        contributions,
    })
}

pub fn create_worktree_from_results(
    root: &Path,
    id: &str,
    results: &[WorkerResult],
) -> HarnessResult<IsolatedWorktree> {
    let root = root.canonicalize()?;
    ensure_git_root(&root)?;
    let base = git(&root, ["rev-parse", "HEAD"])?.stdout.trim().to_owned();
    let contributions = merge(&root, &base, results)?;
    let desired = resolve(&root, &base, &contributions)?;
    let worker = create_isolated_worktree(&root, id)?;
    let populated = snapshot::populate(&worker, &desired).and_then(|_| {
        if snapshot::capture(&root, &worker)? != desired {
            return Err(HarnessError::new(
                "materialized worker snapshot differs from its result",
            ));
        }
        Ok(())
    });
    if let Err(error) = populated {
        return Err(HarnessError::new(format!(
            "{error}; generated worktree retained at {}",
            worker.path
        )));
    }
    Ok(worker)
}

fn merge(root: &Path, base: &str, results: &[WorkerResult]) -> HarnessResult<Contributions> {
    if results.len() > 256 {
        return Err(HarnessError::new("too many worker results"));
    }
    let mut merged: Contributions = BTreeMap::new();
    for result in results {
        if result.root != root || result.base != base {
            return Err(HarnessError::new(
                "worker results have different roots or Git bases",
            ));
        }
        for (id, contribution) in &result.contributions {
            if let Some(existing) = merged.get(id) {
                if existing != contribution {
                    return Err(HarnessError::new(format!(
                        "different worker results reuse task ID: {id}"
                    )));
                }
            } else {
                merged.insert(id.clone(), Arc::clone(contribution));
            }
        }
        check_bounds(&merged)?;
    }
    Ok(merged)
}

fn resolve(root: &Path, base: &str, contributions: &Contributions) -> HarnessResult<Snapshot> {
    let mut writers: BTreeMap<&String, Vec<(&String, &FileState)>> = BTreeMap::new();
    for (id, contribution) in contributions {
        for (path, state) in &contribution.changes {
            writers.entry(path).or_default().push((id, state));
        }
    }
    let mut snapshot = Snapshot::new();
    for (path, writes) in writers {
        let latest = writes
            .iter()
            .filter(|(id, _)| {
                !writes
                    .iter()
                    .any(|(other, _)| contributions[*other].ancestors.contains(*id))
            })
            .collect::<Vec<_>>();
        let Some(_) = latest.first() else {
            return Err(HarnessError::new("invalid worker ancestry"));
        };
        let state = if latest.len() == 1 {
            (*latest[0].1).clone()
        } else {
            let base_bytes = git_bytes(root, ["show", &format!("{base}:{path}")])
                .ok()
                .map(|output| output.stdout);
            merge_latest_states(base_bytes.as_deref(), &latest).ok_or_else(|| {
                HarnessError::new(format!("independent worker results conflict at {path}"))
            })?
        };
        if state != FileState::Unchanged {
            snapshot.insert(path.clone(), state);
        }
    }
    Ok(snapshot)
}

fn merge_latest_states(
    base: Option<&[u8]>,
    latest: &[&(&String, &FileState)],
) -> Option<FileState> {
    let base = base?;
    let mut merged = latest.first()?.1.clone();
    for (_, candidate) in latest.iter().skip(1) {
        merged = merge_two_states(base, &merged, candidate)?;
    }
    Some(merged)
}

fn merge_two_states(base: &[u8], left: &FileState, right: &FileState) -> Option<FileState> {
    if left == right {
        return Some(left.clone());
    }
    if *left == FileState::Unchanged {
        return Some(right.clone());
    }
    if *right == FileState::Unchanged {
        return Some(left.clone());
    }
    let (
        FileState::Contents {
            bytes: left,
            executable: left_exec,
        },
        FileState::Contents {
            bytes: right,
            executable: right_exec,
        },
    ) = (left, right)
    else {
        return None;
    };
    if left_exec != right_exec || !base.is_ascii() || !left.is_ascii() || !right.is_ascii() {
        return None;
    }
    let base_lines = split_lines(base);
    let left_lines = split_lines(left);
    let right_lines = split_lines(right);
    let left_span = changed_span(&base_lines, &left_lines)?;
    let right_span = changed_span(&base_lines, &right_lines)?;
    if left_span.1 > right_span.0 && right_span.1 > left_span.0 {
        return None;
    }
    let (first_span, first_lines, second_span, second_lines) = if left_span.0 <= right_span.0 {
        (left_span, left_lines, right_span, right_lines)
    } else {
        (right_span, right_lines, left_span, left_lines)
    };
    let mut output = base_lines[..first_span.0].to_vec();
    output.extend_from_slice(&first_lines[first_span.2..first_span.3]);
    output.extend_from_slice(&base_lines[first_span.1..second_span.0]);
    output.extend_from_slice(&second_lines[second_span.2..second_span.3]);
    output.extend_from_slice(&base_lines[second_span.1..]);
    Some(FileState::Contents {
        bytes: output.concat().into_bytes().into(),
        executable: *left_exec,
    })
}

fn split_lines(bytes: &[u8]) -> Vec<String> {
    bytes
        .split_inclusive(|byte| *byte == b'\n')
        .map(|line| String::from_utf8_lossy(line).into_owned())
        .collect()
}

fn changed_span(base: &[String], changed: &[String]) -> Option<(usize, usize, usize, usize)> {
    let mut prefix = 0;
    while prefix < base.len() && prefix < changed.len() && base[prefix] == changed[prefix] {
        prefix += 1;
    }
    let mut base_end = base.len();
    let mut changed_end = changed.len();
    while base_end > prefix
        && changed_end > prefix
        && base[base_end - 1] == changed[changed_end - 1]
    {
        base_end -= 1;
        changed_end -= 1;
    }
    Some((prefix, base_end, prefix.min(changed_end), changed_end))
}

fn check_bounds(contributions: &Contributions) -> HarnessResult<()> {
    let files = contributions
        .values()
        .map(|c| c.changes.len())
        .sum::<usize>();
    let bytes = contributions
        .values()
        .flat_map(|c| c.changes.values())
        .map(FileState::len)
        .sum::<usize>();
    if contributions.len() > 256 || files > 4096 || bytes > MAX_BYTES {
        return Err(HarnessError::new(
            "worker result graph exceeds 256 tasks, 4096 contributions or 64 MiB",
        ));
    }
    Ok(())
}
