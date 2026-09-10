fn swift_runtime_avatar_chat() -> &'static str {
    concat!(
        include!("avatar_components.rs"),
        include!("chat_models_and_box.rs"),
        include!("chat_interactions.rs"),
    )
}
