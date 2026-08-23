use crate::{
    StdlibCall, StdlibError, StdlibResult, StdlibReturnKind, StdlibSurface, StdlibValue, signature,
};

pub fn validate_call(call: &StdlibCall, surface: StdlibSurface) -> StdlibResult<StdlibReturnKind> {
    let Some(signature) = signature(&call.namespace, &call.function) else {
        return Err(StdlibError::unsupported(format!(
            "unsupported stdlib function `{}`",
            call.name()
        )));
    };
    if matches!(surface, StdlibSurface::Views) && call.namespace == "id" {
        return Err(StdlibError::unsupported(
            "ULID generation is available only on the server",
        ));
    }
    if matches!(surface, StdlibSurface::Views) && call.namespace == "date" && call.function == "now"
    {
        return Ok(signature.return_kind);
    }
    let allowed = signature
        .required
        .iter()
        .chain(signature.optional.iter())
        .copied()
        .collect::<Vec<_>>();
    for name in signature.required {
        if !call.args.iter().any(|arg| arg.name == *name) {
            return Err(StdlibError::invalid_argument(format!(
                "`{}` requires argument `{name}`",
                call.name()
            )));
        }
    }
    for arg in &call.args {
        if !allowed.iter().any(|name| *name == arg.name) {
            return Err(StdlibError::invalid_argument(format!(
                "`{}` does not accept argument `{}`",
                call.name(),
                arg.name
            )));
        }
    }
    Ok(signature.return_kind)
}

pub fn reference_paths(call: &StdlibCall) -> Vec<String> {
    let mut output = Vec::new();
    for arg in &call.args {
        collect_references(&arg.value, &mut output);
    }
    output
}

fn collect_references(value: &StdlibValue, output: &mut Vec<String>) {
    match value {
        StdlibValue::Reference(value) => output.push(value.clone()),
        StdlibValue::Array(values) => {
            for value in values {
                collect_references(value, output);
            }
        }
        StdlibValue::Object(entries) => {
            for (_, value) in entries {
                collect_references(value, output);
            }
        }
        StdlibValue::Null
        | StdlibValue::Bool(_)
        | StdlibValue::Number(_)
        | StdlibValue::String(_) => {}
    }
}
