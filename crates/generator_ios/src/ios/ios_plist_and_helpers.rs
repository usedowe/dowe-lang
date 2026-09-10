fn info_plist(
    font_families: &BTreeSet<FontFamily>,
    default_locale: Option<&str>,
    app_name: &str,
    app_bundle: &str,
    uses_motion: bool,
    uses_video: bool,
    uses_camera: bool,
    uses_microphone: bool,
    has_app_icon: bool,
) -> String {
    let fonts = font_families
        .iter()
        .filter(|font| font.catalog_entry().package_assets)
        .flat_map(|font| {
            font.catalog_entry()
                .weights
                .iter()
                .map(|weight| format!("        <string>Fonts/{}.ttf</string>", weight.asset_stem))
                .collect::<Vec<_>>()
        })
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>()
        .join("\n");
    let motion = if uses_motion {
        "    <key>NSMotionUsageDescription</key>\n    <string>Use device motion to control interactive Canvas scenes.</string>\n"
    } else {
        ""
    };
    let background_playback = if uses_video {
        "    <key>UIBackgroundModes</key>\n    <array>\n        <string>audio</string>\n    </array>\n"
    } else {
        ""
    };
    let camera_usage = if uses_camera {
        "    <key>NSCameraUsageDescription</key>\n    <string>Use the camera to capture a photo.</string>\n"
    } else {
        ""
    };
    let microphone_usage = if uses_microphone {
        "    <key>NSMicrophoneUsageDescription</key>\n    <string>Use the microphone to record audio.</string>\n"
    } else {
        ""
    };
    let app_icon = if has_app_icon {
        r#"    <key>CFBundleIconName</key>
    <string>AppIcon</string>
    <key>CFBundleIcons</key>
    <dict>
        <key>CFBundlePrimaryIcon</key>
        <dict>
            <key>CFBundleIconFiles</key>
            <array>
                <string>AppIcon60x60</string>
            </array>
            <key>CFBundleIconName</key>
            <string>AppIcon</string>
        </dict>
    </dict>
    <key>CFBundleIcons~ipad</key>
    <dict>
        <key>CFBundlePrimaryIcon</key>
        <dict>
            <key>CFBundleIconFiles</key>
            <array>
                <string>AppIcon60x60</string>
                <string>AppIcon76x76</string>
            </array>
            <key>CFBundleIconName</key>
            <string>AppIcon</string>
        </dict>
    </dict>
"#
    } else {
        ""
    };
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<plist version="1.0">
<dict>
    <key>CFBundleDevelopmentRegion</key>
    <string>{}</string>
    <key>CFBundleDisplayName</key>
    <string>{}</string>
    <key>CFBundleIdentifier</key>
    <string>{}</string>
    <key>CFBundleExecutable</key>
    <string>DoweIosApp</string>
    <key>CFBundleName</key>
    <string>{}</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleShortVersionString</key>
    <string>0.1.0</string>
    <key>CFBundleVersion</key>
    <string>1</string>
    <key>DoweIRSchemaVersion</key>
    <integer>{}</integer>
{app_icon}    <key>NSAppTransportSecurity</key>
    <dict>
        <key>NSAllowsLocalNetworking</key>
        <true/>
    </dict>
{motion}{background_playback}{camera_usage}{microphone_usage}    <key>UILaunchScreen</key>
    <dict/>
    <key>UIAppFonts</key>
    <array>
{fonts}
    </array>
    <key>CFBundleURLTypes</key>
    <array>
        <dict>
            <key>CFBundleURLName</key>
            <string>{}</string>
            <key>CFBundleURLSchemes</key>
            <array>
                <string>dowe-dev</string>
            </array>
        </dict>
    </array>
</dict>
</plist>
"#,
        default_locale.unwrap_or("en"),
        escape_xml(app_name),
        escape_xml(app_bundle),
        escape_xml(app_name),
        dowe_components::VIEW_IR_SCHEMA_VERSION,
        escape_xml(app_bundle)
    )
}

fn ios_canvas_motion(node: &ViewNode) -> bool {
    if matches!(node, ViewNode::Canvas { props } if props.on_motion.is_some()) {
        return true;
    }
    node_child_groups(node)
        .into_iter()
        .flatten()
        .any(ios_canvas_motion)
}

fn ios_video_playback(node: &ViewNode) -> bool {
    if matches!(node, ViewNode::Video { .. }) {
        return true;
    }
    node_child_groups(node)
        .into_iter()
        .flatten()
        .any(ios_video_playback)
}

fn ios_tree_has_camera(node: &ViewNode) -> bool {
    if matches!(node, ViewNode::Camera { .. }) {
        return true;
    }
    node_child_groups(node)
        .into_iter()
        .flatten()
        .any(ios_tree_has_camera)
}

fn ios_tree_has_microphone(node: &ViewNode) -> bool {
    if matches!(node, ViewNode::Microphone { .. }) {
        return true;
    }
    node_child_groups(node)
        .into_iter()
        .flatten()
        .any(ios_tree_has_microphone)
}

fn ios_tree_has_phone(node: &ViewNode) -> bool {
    if matches!(node, ViewNode::Phone { .. }) {
        return true;
    }
    node_child_groups(node)
        .into_iter()
        .flatten()
        .any(ios_tree_has_phone)
}

fn escape_xml(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}
