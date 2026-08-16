    #[test]
    fn rejects_color_prop_for_new_view_components() {
        for source in [
            r#"page componentsPage
  Audio src:"https://example.com/audio.mp3" color:"primary""#,
            r#"page componentsPage
  Accordion color:"primary"
    item id:"one" label:"One"
      Text
        "Body""#,
            r#"page componentsPage
  Checkbox color:"primary""#,
            r#"page componentsPage
  RadioGroup color:"primary"
    item value:"one" label:"One""#,
            r#"page componentsPage
  ToggleTheme color:"primary""#,
            r#"page componentsPage
  Fab color:"primary""#,
            r#"page componentsPage
  Slider color:"primary""#,
            r#"page componentsPage
  Dropzone color:"primary""#,
            r#"page componentsPage
  RichText
    mark text:"Launch" color:"primary""#,
            r#"page componentsPage
  Record name:"voice" color:"primary""#,
            r#"page componentsPage
  ToggleGroup color:"primary"
    item id:"one" label:"One""#,
            r#"page componentsPage
  Collapsible label:"Details" color:"primary"
    Text
      "Body""#,
            r#"page componentsPage
  Countdown target:"2030-01-01T00:00:00Z" color:"primary""#,
            r#"page componentsPage
  Map color:"primary""#,
        ] {
            let error = parse_page(source).expect_err("color prop");
            assert!(error.to_string().contains("use `scheme`"));
        }
    }

    #[test]
    fn rejects_invalid_capture_component_props() {
        let facing = parse_page(
            r#"page capturePage
  Camera facing:"sideways""#,
        )
        .expect_err("camera facing");
        assert!(facing
            .to_string()
            .contains("invalid value for prop `facing`: expected user or environment"));

        let duration = parse_page(
            r#"page capturePage
  Microphone maxDuration:0"#,
        )
        .expect_err("microphone duration");
        assert!(duration
            .to_string()
            .contains("invalid value for prop `maxDuration`: expected positive integer"));
    }

    #[test]
    fn rejects_quoted_rich_text_title_mode() {
        let error = parse_page(
            r#"page componentsPage
  RichText title:"true"
    mark text:"Launch" style:"mark" scheme:"primary""#,
        )
        .expect_err("static boolean");

        assert!(
            error
                .to_string()
                .contains("invalid value for prop `title`: expected boolean")
        );
    }

    #[test]
    fn rejects_color_prop_for_display_and_overlay_components() {
        let error = parse_page(
            r#"page overlayPage
  Avatar color:"primary""#,
        )
        .expect_err("color");

        assert!(
            error
                .to_string()
                .contains("unknown prop `color` on `Avatar`; use `scheme` for visual family")
        );
    }

    #[test]
    fn rejects_invalid_drawer_open_signal() {
        let missing = parse_page(
            r#"page navPage
  Drawer
    Text
      "Navigation""#,
        )
        .expect_err("open");
        assert!(
            missing
                .to_string()
                .contains("invalid value for prop `open`: expected signal bool path")
        );

        let wrong_type = parse_page(
            r#"page navPage
  signal title value:"Navigation"
  Drawer open:title
    Text
      "Navigation""#,
        )
        .expect_err("bool");
        assert!(
            wrong_type
                .to_string()
                .contains("invalid signal path `title` in `open`: expected bool")
        );

        let quoted = parse_page(
            r#"page navPage
  signal drawerOpen value:false
  Drawer open:"drawerOpen"
    Text
      "Navigation""#,
        )
        .expect_err("bare signal path");
        assert!(
            quoted
                .to_string()
                .contains("invalid value for prop `open`: expected signal bool path")
        );

        let duplicate = parse_page(
            r#"page navPage
  signal drawerOpen value:false
  Drawer open:drawerOpen
    body
      Text
        "Primary"
    body
      Text
        "Duplicate""#,
        )
        .expect_err("duplicate body");
        assert!(
            duplicate
                .to_string()
                .contains("duplicate `body` region in Drawer")
        );
    }

    #[test]
    fn rejects_unquoted_static_component_prop_strings() {
        let fill_error = parse_page(
            r#"page iconPage
  Svg viewBox:"0 0 24 24"
    Path d:"M0 0h24v24H0z" fill:none"#,
        )
        .expect_err("fill error");
        assert!(
            fill_error
                .to_string()
                .contains("invalid value for prop `fill`: expected quoted static string literal")
        );

        let option_error = parse_page(
            r#"page formPage
  Select label:"Role"
    Option value:admin label:"Administrator""#,
        )
        .expect_err("option error");
        assert!(
            option_error
                .to_string()
                .contains("invalid value for prop `value`: expected quoted static string literal")
        );

        let variant_error = parse_page(
            r#"page visualPage
  Input variant:outlined scheme:primary"#,
        )
        .expect_err("variant error");
        assert!(
            variant_error.to_string().contains(
                "invalid value for prop `variant`: expected quoted static string literal"
            )
        );

        let color_error = parse_page(
            r#"page visualPage
  Svg viewBox:"0 0 24 24" color:tertiary
    Path d:"M0 0h24v24H0z""#,
        )
        .expect_err("color error");
        assert!(
            color_error
                .to_string()
                .contains("invalid value for prop `color`: expected quoted static string literal")
        );

        let message_error = parse_page(
            r#"page alertPage
  Alert type:"info" message:Saved"#,
        )
        .expect_err("message error");
        assert!(
            message_error.to_string().contains(
                "invalid value for prop `message`: expected quoted static string literal"
            )
        );
    }

    #[test]
    fn requires_quoted_static_text_children() {
        let quoted = parse_page(
            r#"page copyPage
  Box
    Text
      "Dowe compiles your code directly into fast native code."
    Title
      "Dashboard"
    Button href:"/docs"
      "Open docs""#,
        )
        .expect("quoted text children");

        let ViewNode::Box {
            children: box_children,
            ..
        } = &quoted
        else {
            panic!("box");
        };
        assert!(matches!(box_children[0], ViewNode::Text { .. }));
        assert!(matches!(box_children[1], ViewNode::Title { .. }));
        assert!(matches!(box_children[2], ViewNode::Button { .. }));

        for source in [
            r#"page copyPage
  Text
    Dowe compiles your code directly into fast native code."#,
            r#"page copyPage
  Title
    header"#,
            r#"page copyPage
  Button
    Open docs"#,
        ] {
            let error = parse_page(source).expect_err("unquoted text child");
            assert!(
                error
                    .to_string()
                    .contains("must be a quoted static string literal")
            );
        }
    }

    #[test]
    fn rejects_path_outside_svg() {
        let error = parse_page(
            r#"page iconPage
  Path d:"M0 0""#,
        )
        .expect_err("error");

        assert!(
            error
                .to_string()
                .contains("Path can only be used inside Svg")
        );
    }

    #[test]
    fn rejects_non_path_svg_children() {
        let error = parse_page(
            r#"page iconPage
  Svg viewBox:"0 0 24 24"
    Text
      "Bad""#,
        )
        .expect_err("error");

        assert!(error.to_string().contains("Svg only accepts Path children"));
    }

    #[test]
    fn parses_view_function_signature() {
        let tree = parse_page(
            r#"type Appointment
  startsAt:string

page appointmentsPage
  signal appointment type:Appointment value:{ startsAt:"" }
  signal appointments type:Appointment[] value:[]
  fn create params:{ appointment:Appointment } return:"boolean"
    request POST route:"/api/appointments" body:appointment update:appointments
  Button onClick:create
    "Create""#,
        )
        .expect("view function");
        let ViewNode::Scope { actions, .. } = tree else {
            panic!("scope");
        };

        assert_eq!(actions[0].name, "create");
        assert_eq!(actions[0].params[0].name, "appointment");
        assert_eq!(actions[0].params[0].type_name, "Appointment");
        assert_eq!(
            actions[0]
                .return_type
                .as_ref()
                .map(|value| value.type_name.as_str()),
            Some("boolean")
        );
    }

    #[test]
    fn rejects_legacy_view_action() {
        let error = parse_page(
            r#"page appointmentsPage
  action create
    reset appointment
  Text
    "Appointments""#,
        )
        .expect_err("legacy action");

        assert!(
            error
                .to_string()
                .contains("`action` was replaced by `fn <name>` in views")
        );
    }
