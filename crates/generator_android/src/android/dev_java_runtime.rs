fn dev_java_reactive_runtime() -> &'static str {
    concat!(
        include_str!("dev_java_runtime/svg.java"),
        include_str!("dev_java_runtime/models.java"),
        include_str!("dev_java_runtime/state.java"),
        include_str!("dev_java_runtime/styles-and-data.java"),
        include_str!("dev_java_runtime/stdlib.java"),
        include_str!("dev_java_runtime/actions-and-toast.java"),
        include_str!("dev_java_runtime/startup-and-network.java"),
        include_str!("dev_java_runtime/json.java"),
    )
}
