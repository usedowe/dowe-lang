use std::process::{Command, Stdio};
use std::io::Write;

#[test]
fn rpc_correlates_requests_and_rejects_mutation() {
    let binary = env!("CARGO_BIN_EXE_dowe");
    let mut child = Command::new(binary)
        .arg("agent").arg("rpc")
        .stdin(Stdio::piped()).stdout(Stdio::piped()).spawn().unwrap();
    let mut input = child.stdin.take().unwrap();
    input.write_all(b"{\"id\":\"one\",\"command\":\"execute\",\"sessionId\":\"0123456789abcdef0123456789abcdef\"}\n").unwrap();
    input.write_all(b"{\"id\":2,\"command\":\"unknown\",\"sessionId\":\"0123456789abcdef0123456789abcdef\"}\n").unwrap();
    drop(input);
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());
    let text = String::from_utf8(output.stdout).unwrap();
    let lines = text.lines().collect::<Vec<_>>();
    assert_eq!(lines.len(), 2);
    let first: serde_json::Value = serde_json::from_str(lines[0]).unwrap();
    let second: serde_json::Value = serde_json::from_str(lines[1]).unwrap();
    assert_eq!(first["id"], "one");
    assert_eq!(first["error"]["code"], "mutation_rejected");
    assert_eq!(second["id"], 2);
    assert_eq!(second["error"]["code"], "unknown_command");
}
