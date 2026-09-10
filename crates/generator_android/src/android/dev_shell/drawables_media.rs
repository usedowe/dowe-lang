fn dev_activity_drawables_media() -> &'static str {
    concat!(
        include!("drawables_background_and_shadows.rs"),
        include!("drawables_video_controls.rs"),
        include!("drawables_images_and_avatars.rs"),
        include!("drawables_device_and_iframe.rs"),
        include!("drawables_media_layouts.rs"),
    )
}
