//! Small, deterministic Doc-as-Code knowledge map for project-local agents.
//!
//! The map is deliberately derived from confirmed file effects. It never calls a
//! provider and it never treats generated CodeGraph output as authoritative. A
//! failed update is reported to the harness as a warning so that a documentation
//! problem cannot turn a successful source edit into a failed edit.

include!("capability_map_runtime.rs");
include!("capability_map_tests.rs");
