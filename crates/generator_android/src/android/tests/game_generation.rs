#[test]
fn generates_android_game_with_socket_runtime_for_compose_and_dev() {
    let game = dowe_components::game_component_node(vec![
        ComponentProp {
            name: "scene".into(),
            value: PropValue::String("scene".into()),
        },
        ComponentProp {
            name: "label".into(),
            value: PropValue::String("Network game".into()),
        },
        ComponentProp {
            name: "socket".into(),
            value: PropValue::String("/game".into()),
        },
        ComponentProp {
            name: "send".into(),
            value: PropValue::String("outbound".into()),
        },
        ComponentProp {
            name: "status".into(),
            value: PropValue::String("connection".into()),
        },
    ])
    .expect("game");
    let route = ViewRoute {
        id: "game".into(),
        route_path: "/game".into(),
        layout_tree: ViewNode::Children,
        page_tree: game,
        sections: Vec::new(),
        navigation_actions: Vec::new(),
    };
    let output = generate_android(
        &[route],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let views = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("DowePages.kt"))
        .expect("views");
    for contract in [
        "private fun DoweGame(",
        "private class DoweGameSocket(",
        "okhttp3.OkHttpClient()",
        "DoweGame(state = state, scenePath = \"scene\"",
        "socketPath = \"/game\"",
    ] {
        assert!(
            views.content.contains(contract),
            "missing Compose Game contract: {contract}"
        );
    }
    let gradle = output
        .files
        .iter()
        .find(|file| file.relative_path.ends_with("app/build.gradle.kts"))
        .expect("gradle");
    assert!(gradle
        .content
        .contains("com.squareup.okhttp3:okhttp:4.12.0"));

    let dev = dev_java_source(&output);
    for contract in [
        "private View doweGame(",
        "private final class DoweGameSocket",
        "import java.net.Socket;",
        "doweGame(\"scene\"",
        "\"/game\"",
        "private final class DoweRaycastView",
    ] {
        assert!(
            dev.content.contains(contract),
            "missing launcher Game contract: {contract}"
        );
    }
    assert!(!dev.content.contains("okhttp3."));
    assert!(dev
        .content
        .contains("\"pen\", null, null, null, null, null, null, null, borderWidth"));
    assert!(!dev
        .content
        .contains("\"pen\", null, null, null, null, null, null, null, null, borderWidth"));
}

#[test]
fn generates_android_raycast3d_game_runtime_for_compose_and_dev() {
    let game = dowe_components::game_component_node(vec![
        ComponentProp { name: "renderer".into(), value: PropValue::String("raycast3d".into()) },
        ComponentProp { name: "world".into(), value: PropValue::String("world".into()) },
        ComponentProp { name: "camera".into(), value: PropValue::String("camera".into()) },
        ComponentProp { name: "controls".into(), value: PropValue::String("doom".into()) },
        ComponentProp { name: "onFire".into(), value: PropValue::String("fire".into()) },
        ComponentProp { name: "label".into(), value: PropValue::String("Dowe Fortress".into()) },
    ])
    .expect("raycast game");
    let route = ViewRoute {
        id: "fortress".into(),
        route_path: "/fortress".into(),
        layout_tree: ViewNode::Children,
        page_tree: game,
        sections: Vec::new(),
        navigation_actions: Vec::new(),
    };
    let output = generate_android(
        &[route],
        &FontConfig::default(),
        &DesignConfig::default(),
        &[],
    );
    let views = output.files.iter().find(|file| file.relative_path.ends_with("DowePages.kt")).expect("views");
    for contract in [
        "renderer = \"raycast3d\"",
        "worldPath = \"world\"",
        "cameraPath = \"camera\"",
        "controls = \"doom\"",
        "private fun DoweRaycastGame(",
    ] {
        assert!(views.content.contains(contract), "missing raycast Compose contract: {contract}");
    }
    let dev = dev_java_source(&output);
    for contract in [
        "doweGame(null, \"raycast3d\"",
        "doweRaycastGame(worldPath, cameraPath",
        "private final class DoweRaycastView",
        "moveByTouch",
    ] {
        assert!(dev.content.contains(contract), "missing raycast launcher contract: {contract}");
    }
}
