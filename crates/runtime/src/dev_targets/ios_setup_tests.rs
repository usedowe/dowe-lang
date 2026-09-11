use super::*;
use serde_json::json;
use std::collections::VecDeque;

fn inventory(devices: bool, runtime: bool) -> Value {
    json!({
        "devices": {"com.apple.CoreSimulator.SimRuntime.iOS-26-5": if devices {
            vec![json!({"name":"Dowe iPhone", "udid":"created", "state":"Shutdown", "isAvailable":true})]
        } else { vec![] }},
        "runtimes": if runtime { vec![json!({
            "identifier":"com.apple.CoreSimulator.SimRuntime.iOS-26-5", "version":"26.5",
            "isAvailable":true, "supportedDeviceTypes":[{
                "identifier":"com.apple.CoreSimulator.SimDeviceType.iPhone-17", "name":"iPhone 17", "productFamily":"iPhone"
            }]
        })] } else { vec![] }
    })
}

#[test]
fn ios_setup_reuses_available_simulators_without_mutations() {
    let mut calls = Vec::new();
    let options = ensure_ios_simulators_with("arm64", |config| {
        calls.push(config.args);
        Ok(serde_json::to_vec(&inventory(true, true)).unwrap())
    })
    .unwrap();
    assert_eq!(options.len(), 1);
    assert_eq!(calls, [vec!["simctl", "list", "-j"]]);
}

#[test]
fn ios_setup_downloads_runtime_then_creates_and_verifies_simulator() {
    let mut responses = VecDeque::from([
        serde_json::to_vec(&inventory(false, false)).unwrap(),
        Vec::new(),
        serde_json::to_vec(&inventory(false, true)).unwrap(),
        b"created\n".to_vec(),
        serde_json::to_vec(&inventory(true, true)).unwrap(),
    ]);
    let mut calls = Vec::new();
    let options = ensure_ios_simulators_with("arm64", |config| {
        calls.push(config);
        Ok(responses.pop_front().expect("bounded commands"))
    })
    .unwrap();
    assert_eq!(options[0].udid(), "created");
    assert_eq!(calls[1].args, ["xcodebuild", "-downloadPlatform", "iOS"]);
    assert_eq!(calls[1].options.stdout, StreamMode::Inherit);
    assert_eq!(
        calls[3].args,
        [
            "simctl",
            "create",
            "Dowe iPhone 17 (iOS 26.5)",
            "com.apple.CoreSimulator.SimDeviceType.iPhone-17",
            "com.apple.CoreSimulator.SimRuntime.iOS-26-5"
        ]
    );
    assert!(responses.is_empty());
}

#[test]
fn ios_setup_does_not_retry_failed_download_or_create_a_device() {
    let mut count = 0;
    let error = ensure_ios_simulators_with("arm64", |_| {
        count += 1;
        if count == 1 {
            Ok(serde_json::to_vec(&inventory(false, false)).unwrap())
        } else {
            Err(RuntimeError::new("download: disk full"))
        }
    })
    .unwrap_err();
    assert!(error.to_string().contains("disk full"));
    assert_eq!(count, 2);
}

#[test]
fn ios_setup_reuses_runtime_and_rejects_unverified_creation() {
    let mut count = 0;
    let error = ensure_ios_simulators_with("arm64", |config| {
        count += 1;
        assert!(!config.args.iter().any(|arg| arg == "-downloadPlatform"));
        if count == 2 {
            Ok(b"missing".to_vec())
        } else {
            Ok(serde_json::to_vec(&inventory(false, true)).unwrap())
        }
    })
    .unwrap_err();
    assert_eq!(count, 3);
    assert!(error.to_string().contains("created simulator"));
}

#[test]
fn ios_setup_selects_numeric_versions_and_compatible_iphone_types() {
    let value = json!({"runtimes":[
        {"identifier":"com.apple.CoreSimulator.SimRuntime.iOS-26-9", "version":"26.9", "isAvailable":true},
        {"identifier":"com.apple.CoreSimulator.SimRuntime.iOS-26-10", "version":"26.10", "isAvailable":true},
        {"identifier":"com.apple.CoreSimulator.SimRuntime.iOS-27", "version":"27.0", "isAvailable":true, "supportedArchitectures":["x86_64"]}
    ], "devicetypes":[
        {"identifier":"too-new", "name":"iPhone New", "productFamily":"iPhone", "minRuntimeVersionString":"27.0"},
        {"identifier":"tablet", "name":"iPad", "productFamily":"iPad"},
        {"identifier":"phone", "name":"iPhone", "productFamily":"iPhone", "minRuntimeVersionString":"17.0", "maxRuntimeVersionString":"26.255.255"}
    ]});
    let (runtime, device) = ios_simulator_template(&value, "arm64").unwrap();
    assert_eq!(runtime["version"], "26.10");
    assert_eq!(device["identifier"], "phone");
}
