use std::fs;
use std::path::Path;
use std::sync::Mutex;
use tempfile::TempDir;

static ENV_LOCK: Mutex<()> = Mutex::new(());

#[test]
fn compiles_the_self_contained_fullstack_example() {
    let _guard = ENV_LOCK.lock().expect("env lock");
    let temp = TempDir::new().expect("tempdir");
    let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../skill-data/examples/fullstack");
    copy_tree(&source, temp.path());
    fs::copy(temp.path().join(".env.example"), temp.path().join(".env")).expect("env");

    let jwt = std::env::var_os("JWT_SECRET");
    let provider = std::env::var_os("PROVIDER_BASE_URL");
    unsafe {
        std::env::set_var("JWT_SECRET", "test-secret");
        std::env::set_var("PROVIDER_BASE_URL", "https://provider.example");
    }
    let root = temp.path().to_path_buf();
    let result = std::thread::Builder::new()
        .stack_size(16 * 1024 * 1024)
        .spawn(move || dowe_compiler::compile_dev(root))
        .expect("compiler thread")
        .join()
        .expect("compiler thread result");
    unsafe {
        match jwt {
            Some(value) => std::env::set_var("JWT_SECRET", value),
            None => std::env::remove_var("JWT_SECRET"),
        }
        match provider {
            Some(value) => std::env::set_var("PROVIDER_BASE_URL", value),
            None => std::env::remove_var("PROVIDER_BASE_URL"),
        }
    }
    result.expect("fullstack example");
}

#[test]
fn compiles_the_reference_ui_example() {
    let _guard = ENV_LOCK.lock().expect("env lock");
    let temp = TempDir::new().expect("tempdir");
    let source =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../skill-data/examples/reference-ui");
    copy_tree(&source, temp.path());
    fs::copy(temp.path().join(".env.example"), temp.path().join(".env")).expect("env");

    let root = temp.path().to_path_buf();
    let result = std::thread::Builder::new()
        .stack_size(16 * 1024 * 1024)
        .spawn(move || dowe_compiler::compile_dev(root))
        .expect("compiler thread")
        .join()
        .expect("compiler thread result");

    result.expect("reference UI example");
}

fn copy_tree(source: &Path, target: &Path) {
    fs::create_dir_all(target).expect("target");
    for entry in fs::read_dir(source).expect("source") {
        let entry = entry.expect("entry");
        let target_path = target.join(entry.file_name());
        if entry.path().is_dir() {
            copy_tree(&entry.path(), &target_path);
        } else {
            fs::copy(entry.path(), target_path).expect("copy");
        }
    }
}
