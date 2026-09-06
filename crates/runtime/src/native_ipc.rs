use crate::handlers::execute_native_ipc_action;
use crate::{RuntimeError, RuntimeResult};
use dowe_compiler::{CompiledProject, NativeIpcTarget};
use serde_json::Value;

pub async fn invoke_native_function(
    project: &CompiledProject,
    target: NativeIpcTarget,
    function: &str,
    args: Value,
) -> RuntimeResult<Value> {
    let callable = project.native_ipc.find(target, function).ok_or_else(|| {
        RuntimeError::new(format!(
            "native IPC function `{function}` is not registered"
        ))
    })?;
    if let Some(result) = crate::native_directory::invoke(&project.root, function, &args).await {
        return result;
    }
    execute_native_ipc_action(project, &callable.action, args).await
}
