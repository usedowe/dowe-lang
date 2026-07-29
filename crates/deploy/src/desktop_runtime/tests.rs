use super::{append_payload, validate_executable, validate_runtime};
use crate::model::BuildTarget;

#[test]
fn appends_and_validates_dowe_desktop_payloads() {
    let windows = append_payload(b"MZruntime", b"payload");
    let linux = append_payload(b"\x7fELFruntime", b"payload");

    validate_executable(BuildTarget::Windows, &windows).expect("Windows bundle");
    validate_executable(BuildTarget::Linux, &linux).expect("Linux bundle");
}

#[test]
fn rejects_wrong_runtime_formats_and_payload_hashes() {
    let mut executable = append_payload(b"MZruntime", b"payload");
    executable[9] ^= 1;

    assert!(validate_runtime(BuildTarget::Linux, b"MZruntime").is_err());
    assert!(validate_executable(BuildTarget::Windows, &executable).is_err());
}
