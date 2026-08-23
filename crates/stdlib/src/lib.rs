mod evaluation;
mod helpers;
mod model;
mod registry;
mod svg;
mod validation;

pub use evaluation::evaluate;
pub use model::{
    StdlibArgument, StdlibCall, StdlibError, StdlibErrorCode, StdlibResult, StdlibReturnKind,
    StdlibSignature, StdlibSurface, StdlibValue,
};
pub use registry::{
    functions, is_stdlib_function, is_stdlib_namespace, namespaces, signature, signatures,
};
pub use validation::{reference_paths, validate_call};

#[cfg(test)]
mod tests;
