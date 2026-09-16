use dowe_codegraph::*;
use std::{fs, process::Command};

#[test]
fn generation_process_worker() {
    let Some(root) = std::env::var_os("DOWE_GRAPH_TEST_ROOT") else {
        return;
    };
    if std::env::var_os("DOWE_GRAPH_TEST_HOLD").is_some() {
        use std::io::Write;
        let file = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(std::path::Path::new(&root).join(".dowe/codegraph.lock"))
            .unwrap();
        file.lock().unwrap();
        println!("graph-lock-held");
        std::io::stdout().flush().unwrap();
        loop {
            std::thread::park();
        }
    }
    for _ in 0..8 {
        let snapshot = refresh_persistent_codegraph(&root).unwrap();
        assert_eq!(snapshot.manifest.revision, 1);
        assert_eq!(
            read_persistent_codegraph(&root).unwrap().generation,
            snapshot.generation
        );
        let result = query_persistent_codegraph(
            &root,
            CodeGraphQuery {
                text: "main.py".into(),
                limit: 10,
                depth: 0,
            },
        )
        .unwrap();
        assert!(
            result
                .nodes
                .iter()
                .any(|node| node.path.as_deref() == Some("main.py"))
        );
    }
}

#[test]
fn busy_store_is_bounded_and_process_exit_releases_ownership() {
    use std::io::{BufRead, BufReader};
    let root = tempfile::tempdir().unwrap();
    refresh_persistent_codegraph(root.path()).unwrap();
    let mut child = Command::new(std::env::current_exe().unwrap())
        .args(["--exact", "generation_process_worker", "--nocapture"])
        .env("DOWE_GRAPH_TEST_ROOT", root.path())
        .env("DOWE_GRAPH_TEST_HOLD", "1")
        .stdout(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    let stdout = child.stdout.take().unwrap();
    let ready = BufReader::new(stdout)
        .lines()
        .any(|line| line.unwrap().contains("graph-lock-held"));
    assert!(ready);
    let (result, read) = std::thread::scope(|scope| {
        let reader = scope.spawn(|| read_persistent_codegraph(root.path()));
        (
            refresh_persistent_codegraph(root.path()),
            reader.join().unwrap(),
        )
    });
    child.kill().unwrap();
    child.wait().unwrap();
    assert!(result.unwrap_err().to_string().contains("busy"));
    assert!(read.unwrap_err().to_string().contains("busy"));
    assert!(refresh_persistent_codegraph(root.path()).is_ok());
    assert!(root.path().join(".dowe/codegraph.lock").is_file());
}

#[test]
fn query_uses_its_snapshot_instead_of_reopening_the_path_index() {
    let root = tempfile::tempdir().unwrap();
    fs::write(root.path().join("main.py"), "print(1)\n").unwrap();
    let snapshot = refresh_persistent_codegraph(root.path()).unwrap();
    fs::write(
        root.path()
            .join(".dowe/codegraph")
            .join(snapshot.generation.unwrap())
            .join("index.json"),
        "not an index",
    )
    .unwrap();
    let result = query_persistent_codegraph(
        root.path(),
        CodeGraphQuery {
            text: "main.py".into(),
            limit: 10,
            depth: 0,
        },
    )
    .unwrap();
    assert!(
        result
            .nodes
            .iter()
            .any(|node| node.path.as_deref() == Some("main.py"))
    );
}

#[test]
fn missing_read_is_non_mutating_and_oversized_pointer_is_rejected() {
    let root = tempfile::tempdir().unwrap();
    assert!(read_persistent_codegraph(root.path()).is_err());
    assert!(!root.path().join(".dowe").exists());
    refresh_persistent_codegraph(root.path()).unwrap();
    fs::write(root.path().join(".dowe/codegraph/CURRENT"), "x".repeat(257)).unwrap();
    assert!(read_persistent_codegraph(root.path()).is_err());
}

#[test]
fn processes_publish_and_read_one_consistent_generation() {
    let root = tempfile::tempdir().unwrap();
    fs::write(root.path().join("main.py"), "print(1)\n").unwrap();
    let mut children = (0..4)
        .map(|_| {
            Command::new(std::env::current_exe().unwrap())
                .args(["--exact", "generation_process_worker", "--nocapture"])
                .env("DOWE_GRAPH_TEST_ROOT", root.path())
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped())
                .spawn()
                .unwrap()
        })
        .collect::<Vec<_>>();
    let outputs = children
        .drain(..)
        .map(|child| child.wait_with_output().unwrap())
        .collect::<Vec<_>>();
    for output in outputs {
        assert!(
            output.status.success(),
            "{}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
    let snapshot = read_persistent_codegraph(root.path()).unwrap();
    assert_eq!(snapshot.manifest.revision, 1);
    assert_eq!(
        fs::read_dir(root.path().join(".dowe/codegraph"))
            .unwrap()
            .filter_map(Result::ok)
            .filter(|entry| entry
                .file_name()
                .to_string_lossy()
                .starts_with("generation-"))
            .count(),
        1
    );
}

#[cfg(unix)]
#[test]
fn pointer_and_generation_symlinks_are_rejected_without_touching_targets() {
    use std::os::unix::fs::symlink;
    let root = tempfile::tempdir().unwrap();
    let outside = tempfile::tempdir().unwrap();
    fs::write(root.path().join("main.py"), "print(1)\n").unwrap();
    let snapshot = refresh_persistent_codegraph(root.path()).unwrap();
    let store = root.path().join(".dowe/codegraph");
    let pointer = store.join("CURRENT");
    let original = fs::read(&pointer).unwrap();
    fs::remove_file(&pointer).unwrap();
    fs::write(outside.path().join("pointer"), &original).unwrap();
    symlink(outside.path().join("pointer"), &pointer).unwrap();
    assert!(read_persistent_codegraph(root.path()).is_err());
    fs::remove_file(&pointer).unwrap();
    fs::write(&pointer, "../escape\n").unwrap();
    assert!(read_persistent_codegraph(root.path()).is_err());
    fs::write(&pointer, &original).unwrap();
    let nodes = store.join(snapshot.generation.unwrap()).join("nodes.json");
    fs::remove_file(&nodes).unwrap();
    symlink(outside.path().join("absent"), &nodes).unwrap();
    assert!(read_persistent_codegraph(root.path()).is_err());
    assert!(!outside.path().join("absent").exists());
}
