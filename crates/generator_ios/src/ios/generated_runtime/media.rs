fn swift_runtime_media() -> &'static str {
    concat!(
        include!("media_video_and_device.rs"),
        include!("media_audio.rs"),
        include!("media_images.rs"),
    )
}
