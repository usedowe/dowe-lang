//! Shared notification contracts and a small durable outbox.
//!
//! Provider adapters belong to the runtime and native hosts. This crate owns the
//! values crossing those boundaries so local and remote notifications cannot
//! silently diverge.

include!("notification_contracts.rs");
include!("notification_store.rs");
include!("notification_helpers_and_tests.rs");
