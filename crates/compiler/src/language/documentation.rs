use crate::language::completion::{component_value_completions, props_for_component};
use dowe_components::{BuiltinComponent, ColorFamily};
use dowe_stdlib::{StdlibReturnKind, StdlibSignature};


include!("documentation_catalog.rs");
include!("documentation_server.rs");
include!("documentation_components.rs");
include!("documentation_props.rs");
