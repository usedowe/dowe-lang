use crate::error::{DoweError, DoweResult};
use crate::model::ViewPlatform;
use dowe_generator_web::{WebOutput, render_page_document_with_icons};
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;
use std::sync::Arc;

#[derive(Default)]
pub(super) struct ProjectIconTargets {
    pub web_favicon: bool,
    pub web_apple_touch: bool,
    pub desktop_macos: bool,
    pub desktop_windows: bool,
    pub desktop_linux: bool,
    pub ios: bool,
    pub android: bool,
}

impl ProjectIconTargets {
    pub fn detect(root: &Path) -> DoweResult<Self> {
        let icons = root.join("icons");
        reject_icon_root_symlink(&icons)?;
        Ok(Self {
            web_favicon: regular_icon_file(&icons, "web/favicon-32x32.png")?,
            web_apple_touch: regular_icon_file(&icons, "web/apple-touch-icon.png")?,
            desktop_macos: regular_icon_file(&icons, "desktop/icon.icns")?,
            desktop_windows: regular_icon_file(&icons, "desktop/icon.ico")?,
            desktop_linux: regular_icon_file(&icons, "desktop/icon.png")?,
            ios: regular_icon_file(&icons, "ios/AppIcon.appiconset/Contents.json")?
                && regular_icon_file(&icons, "ios/AppIcon.png")?,
            android: regular_icon_file(&icons, "android/mipmap-anydpi-v26/ic_launcher.xml")?
                && regular_icon_file(&icons, "android/mipmap-mdpi/ic_launcher.png")?,
        })
    }
}

fn reject_icon_root_symlink(path: &Path) -> DoweResult<()> {
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(DoweError::at_path(path, error.to_string())),
    };
    if metadata.file_type().is_symlink() {
        return Err(DoweError::at_path(
            path,
            "icon output cannot contain symlinks",
        ));
    }
    Ok(())
}

fn regular_icon_file(root: &Path, relative: &str) -> DoweResult<bool> {
    let mut current = root.to_path_buf();
    for component in Path::new(relative).components() {
        let std::path::Component::Normal(value) = component else {
            return Ok(false);
        };
        current.push(value);
        let metadata = match fs::symlink_metadata(&current) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(false),
            Err(error) => return Err(DoweError::at_path(&current, error.to_string())),
        };
        if metadata.file_type().is_symlink() {
            return Err(DoweError::at_path(
                &current,
                "icon output cannot contain symlinks",
            ));
        }
    }
    Ok(current.is_file())
}

pub(super) fn apply_web_icon_documents(
    web: &mut WebOutput,
    previous: Option<&WebOutput>,
    icons: &ProjectIconTargets,
) {
    let favicon = icons.web_favicon.then_some("/icons/web/favicon-32x32.png");
    let apple_touch = icons
        .web_apple_touch
        .then_some("/icons/web/apple-touch-icon.png");
    for page in &mut web.pages {
        if previous.is_some_and(|previous| {
            previous
                .pages
                .iter()
                .any(|candidate| Arc::ptr_eq(candidate, page))
        }) {
            continue;
        }
        let page = Arc::make_mut(page);
        page.html_document = render_page_document_with_icons(page, favicon, apple_touch);
    }
}

pub(super) fn sync_project_icons(
    root: &Path,
    icons: &ProjectIconTargets,
    selected_platforms: Option<&BTreeSet<ViewPlatform>>,
) -> DoweResult<()> {
    let source = root.join("icons");
    if platform_selected(selected_platforms, ViewPlatform::Web) {
        sync_optional_directory(
            &source.join("web"),
            &root.join(".dowe/web/icons/web"),
            icons.web_favicon,
        )?;
    }
    if platform_selected(selected_platforms, ViewPlatform::Desktop) {
        sync_optional_directory(
            &source.join("web"),
            &root.join(".dowe/apps/desktop/web/icons/web"),
            icons.web_favicon,
        )?;
        sync_optional_file(
            &source.join("desktop/icon.icns"),
            &root.join(".dowe/apps/desktop/macos/icon.icns"),
            icons.desktop_macos,
        )?;
        sync_optional_file(
            &source.join("desktop/icon.ico"),
            &root.join(".dowe/apps/desktop/windows/icon.ico"),
            icons.desktop_windows,
        )?;
        sync_optional_file(
            &source.join("desktop/icon.png"),
            &root.join(".dowe/apps/desktop/linux/icon.png"),
            icons.desktop_linux,
        )?;
    }
    if platform_selected(selected_platforms, ViewPlatform::Ios) {
        sync_ios_icons(root, &source, icons.ios)?;
    }
    if platform_selected(selected_platforms, ViewPlatform::Android) {
        sync_android_icons(root, &source, icons.android)?;
    }
    Ok(())
}

fn platform_selected(
    selected_platforms: Option<&BTreeSet<ViewPlatform>>,
    platform: ViewPlatform,
) -> bool {
    selected_platforms
        .map(|platforms| platforms.contains(&platform))
        .unwrap_or(true)
}

fn sync_ios_icons(root: &Path, source: &Path, enabled: bool) -> DoweResult<()> {
    sync_optional_directory(
        &source.join("ios/AppIcon.appiconset"),
        &root.join(".dowe/apps/ios/Assets.xcassets/AppIcon.appiconset"),
        enabled,
    )?;
    sync_optional_file(
        &source.join("ios/AppIcon.png"),
        &root.join(".dowe/apps/ios/AppIcon.png"),
        enabled,
    )
}

fn sync_android_icons(root: &Path, source: &Path, enabled: bool) -> DoweResult<()> {
    let source = source.join("android");
    let resources = root.join(".dowe/apps/android/app/src/main/res");
    for density in ["mdpi", "hdpi", "xhdpi", "xxhdpi", "xxxhdpi"] {
        for name in ["ic_launcher.png", "ic_launcher_round.png"] {
            sync_optional_file(
                &source.join(format!("mipmap-{density}/{name}")),
                &resources.join(format!("mipmap-{density}/{name}")),
                enabled,
            )?;
        }
        sync_optional_file(
            &source.join(format!("drawable-{density}/ic_launcher_foreground.png")),
            &resources.join(format!("drawable-{density}/ic_launcher_foreground.png")),
            enabled,
        )?;
    }
    for relative in [
        "mipmap-anydpi-v26/ic_launcher.xml",
        "mipmap-anydpi-v26/ic_launcher_round.xml",
        "drawable/ic_launcher_background.xml",
    ] {
        sync_optional_file(&source.join(relative), &resources.join(relative), enabled)?;
    }
    Ok(())
}

fn sync_optional_directory(source: &Path, destination: &Path, enabled: bool) -> DoweResult<()> {
    remove_path(destination)?;
    if enabled {
        copy_directory(source, destination)?;
    }
    Ok(())
}

fn sync_optional_file(source: &Path, destination: &Path, enabled: bool) -> DoweResult<()> {
    remove_path(destination)?;
    if !enabled || !source.is_file() {
        return Ok(());
    }
    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| DoweError::at_path(parent, error.to_string()))?;
    }
    fs::copy(source, destination).map_err(|error| DoweError::at_path(source, error.to_string()))?;
    Ok(())
}

fn copy_directory(source: &Path, destination: &Path) -> DoweResult<()> {
    let metadata = fs::symlink_metadata(source)
        .map_err(|error| DoweError::at_path(source, error.to_string()))?;
    if metadata.file_type().is_symlink() {
        return Err(DoweError::at_path(
            source,
            "icon output cannot contain symlinks",
        ));
    }
    fs::create_dir_all(destination)
        .map_err(|error| DoweError::at_path(destination, error.to_string()))?;
    let mut entries = fs::read_dir(source)
        .map_err(|error| DoweError::at_path(source, error.to_string()))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| DoweError::at_path(source, error.to_string()))?;
    entries.sort_by_key(|entry| entry.file_name());
    for entry in entries {
        let source_path = entry.path();
        let file_type = entry
            .file_type()
            .map_err(|error| DoweError::at_path(&source_path, error.to_string()))?;
        if file_type.is_symlink() {
            return Err(DoweError::at_path(
                &source_path,
                "icon output cannot contain symlinks",
            ));
        }
        let destination_path = destination.join(entry.file_name());
        if file_type.is_dir() {
            copy_directory(&source_path, &destination_path)?;
        } else if file_type.is_file() {
            fs::copy(&source_path, &destination_path)
                .map_err(|error| DoweError::at_path(&source_path, error.to_string()))?;
        }
    }
    Ok(())
}

fn remove_path(path: &Path) -> DoweResult<()> {
    if path.is_dir() {
        fs::remove_dir_all(path).map_err(|error| DoweError::at_path(path, error.to_string()))?;
    } else if path.exists() {
        fs::remove_file(path).map_err(|error| DoweError::at_path(path, error.to_string()))?;
    }
    Ok(())
}
