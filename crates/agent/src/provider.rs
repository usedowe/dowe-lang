use crate::auth::{AgentAuthStore, AgentCredential, expand_credential_value};
use crate::error::{AgentError, AgentResult};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::env;
use std::path::Path;
use std::collections::BTreeSet;

include!("provider_types_and_registry.rs");
include!("builtin_provider_catalog.rs");
include!("provider_resolution.rs");
include!("provider_helpers.rs");
include!("provider_tests.rs");
