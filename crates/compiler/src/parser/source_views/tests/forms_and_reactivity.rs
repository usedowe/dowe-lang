    #[test]
    fn parses_advanced_form_components_and_structural_children() {
        let tree = parse_page(
            r#"page advancedPage
  signal form value:{ role:"editor" notes:"" password:"" phone:"" pin:"" bio:"" avatar:"" }
  Box
    ComboBox bind:form.role label:"Role" placeholder:"Choose" clearable:true
      comboOption value:"admin" label:"Admin" description:"Full access"
      comboOption value:"editor" label:"Editor"
    CsvField label:"Import" buttonText:"Upload CSV"
      csvColumn name:"email" label:"Email"
    DragDrop label:"Tasks" direction:"horizontal"
      dragGroup id:"todo" title:"Todo"
        dragItem id:"draft" label:"Draft" description:"Prepare"
    Editor bind:form.notes label:"Notes" placeholder:"Write notes" minHeight:180
    ImageCropper bind:form.avatar label:"Avatar" shape:"circle"
    PasswordField bind:form.password label:"Password" hideStrength:false
    PhoneField bind:form.phone label:"Phone" country:"US"
    PinField bind:form.pin label:"Code" length:6 type:"number"
    Textarea bind:form.bio label:"Bio" rows:4 maxLength:160"#,
        )
        .expect("tree");
        let ViewNode::Scope { children, .. } = tree else {
            panic!("scope");
        };
        let ViewNode::Box { children, .. } = &children[0] else {
            panic!("box");
        };

        assert!(matches!(
            &children[0],
            ViewNode::ComboBox { props, options }
                if props.clearable
                    && props.style.element.bind.as_deref() == Some("form.role")
                    && options.len() == 2
                    && options[0].description.as_deref() == Some("Full access")
        ));
        assert!(matches!(
            &children[1],
            ViewNode::CsvField { props, columns }
                if props.button_text == "Upload CSV"
                    && columns.len() == 1
                    && columns[0].name == "email"
        ));
        assert!(matches!(
            &children[2],
            ViewNode::DragDrop { props, groups, .. }
                if props.direction.as_str() == "horizontal"
                    && groups.len() == 1
                    && groups[0].items[0].id == "draft"
        ));
        assert!(matches!(
            &children[3],
            ViewNode::Editor { props }
                if props.min_height == 180
                    && props.style.element.bind.as_deref() == Some("form.notes")
        ));
        assert!(matches!(
            &children[4],
            ViewNode::ImageCropper { props }
                if props.shape.as_str() == "circle"
                    && props.style.element.bind.as_deref() == Some("form.avatar")
        ));
        assert!(matches!(
            &children[5],
            ViewNode::PasswordField { props } if !props.hide_strength
        ));
        assert!(matches!(
            &children[6],
            ViewNode::PhoneField { props } if props.country.as_deref() == Some("US")
        ));
        assert!(matches!(
            &children[7],
            ViewNode::PinField { props }
                if props.length == 6 && props.kind.as_str() == "number"
        ));
        assert!(matches!(
            &children[8],
            ViewNode::Textarea { props } if props.rows == 4 && props.max_length == Some(160)
        ));
    }

    #[test]
    fn rejects_duplicate_request_path_forms() {
        let error = parse_page(
            r#"page blogsPage
  signal blogs value:[]
  fn load
    request GET "/api/blogs" route:"/api/blogs" update:blogs
  Box
    Text
      "Blogs""#,
        )
        .expect_err("error");

        assert!(error.to_string().contains("only one route path"));
    }

    #[test]
    fn rejects_text_prop_on_text_component() {
        let error = parse_page(
            r#"page blogsPage
  Text text:"Blogs""#,
        )
        .expect_err("error");

        assert!(error.to_string().contains("unknown prop `text`"));
    }

    #[test]
    fn rejects_unknown_and_incompatible_signal_paths() {
        let missing = parse_page(
            r#"page blogsPage
  signal blog value:{ title:"" visible:false }
  Box
    Input bind:blog.missing"#,
        )
        .expect_err("missing field");
        assert!(
            missing
                .to_string()
                .contains("unknown signal path `blog.missing`")
        );

        let incompatible = parse_page(
            r#"page blogsPage
  signal alert value:{ message:"" visible:false }
  Alert type:"info" message:alert.visible visible:alert.message"#,
        )
        .expect_err("incompatible field");
        assert!(
            incompatible
                .to_string()
                .contains("invalid signal path `alert.visible` in `message`: expected string")
        );
    }

    #[test]
    fn parses_reactive_button_visual_props_and_conditional_icon() {
        let page = parse_page(
            r#"page buttonPage
  signal variantChoice value:"solid"
  signal schemeChoice value:"primary"
  signal sizeChoice value:"md"
  signal roundedChoice value:"md"
  signal loadingChoice value:false
  signal startIconVisible value:true
  Button variant:variantChoice scheme:schemeChoice size:sizeChoice rounded:roundedChoice loading:loadingChoice iconStart:{ when:startIconVisible value:"add-circle" }
    "Create""#,
        )
        .expect("reactive button");
        let ViewNode::Scope { children, .. } = page else {
            panic!("scope")
        };
        let ViewNode::Button { props, .. } = &children[0] else {
            panic!("button")
        };
        assert_eq!(props.reactive.variant.as_deref(), Some("variantChoice"));
        assert_eq!(props.reactive.scheme.as_deref(), Some("schemeChoice"));
        assert_eq!(props.reactive.size.as_deref(), Some("sizeChoice"));
        assert_eq!(props.reactive.rounded.as_deref(), Some("roundedChoice"));
        assert_eq!(props.reactive.loading.as_deref(), Some("loadingChoice"));
        assert_eq!(
            props.reactive.icon_start_when.as_deref(),
            Some("startIconVisible")
        );
    }

    #[test]
    fn rejects_removed_button_icon_visibility_props() {
        let error = parse_page(
            r#"page buttonPage
  signal visible value:true
  Button iconStart:"add-circle" showIconStart:visible
    "Create""#,
        )
        .expect_err("removed prop");
        assert!(error.to_string().contains("unknown prop `showIconStart`"));
    }

    #[test]
    fn rejects_static_button_loading_values() {
        let error = parse_page(
            r#"page buttonPage
  Button loading:true
    "Create""#,
        )
        .expect_err("static loading");
        assert!(error.to_string().contains("loading"));
        assert!(error.to_string().contains("signal bool path"));
    }

    #[test]
    fn omits_empty_button_icon_strings() {
        let page = parse_page(
            r#"page buttonPage
  Button iconStart:"" iconEnd:""
    "Create""#,
        )
        .expect("empty icons");
        let ViewNode::Button { props, .. } = &page else {
            panic!("button")
        };
        assert!(props.icon_start.is_none());
        assert!(props.icon_end.is_none());
    }

    #[test]
    fn parses_numeric_conditional_button_icon() {
        let page = parse_page(
            r#"page buttonPage
  signal count value:11
  Button iconStart:{ when:count gt:10 value:"bell" }
    "Notifications""#,
        )
        .expect("numeric condition");
        let ViewNode::Scope { children, .. } = page else {
            panic!("scope")
        };
        let ViewNode::Button { props, .. } = &children[0] else {
            panic!("button")
        };
        let comparison = props
            .reactive
            .icon_start_comparison
            .as_ref()
            .expect("comparison");
        assert_eq!(comparison.operator.as_str(), ">");
        assert_eq!(comparison.value, "10");
    }

    #[test]
    fn rejects_i18n_with_reactive_text_child() {
        let error = parse_page(
            r#"page profilePage
  signal profile value:{ title:"" }
  Text i18n:"profile.title"
    "{profile.title}""#,
        )
        .expect_err("reactive fallback");

        assert!(
            error
                .to_string()
                .contains("`i18n` requires a static fallback text child")
        );
    }

    #[test]
    fn parses_show_visibility_conditions() {
        let tree = parse_page(
            r#"page readyPage
  signal isReady value:false
  signal rows value:[{ id:"1" ready:true }]
  Box show:{ xs:false md:true }
    Text show:isReady
      "Ready"
    each in:rows as:row key:row.id
      Text show:row.ready
        "Row""#,
        )
        .expect("tree");

        let ViewNode::Scope {
            signals, children, ..
        } = tree
        else {
            panic!("scope");
        };
        assert_eq!(signals[0].name, "isReady");

        let ViewNode::Box {
            props,
            children: box_children,
        } = &children[0]
        else {
            panic!("box");
        };
        match props.element.show.as_ref().expect("box show") {
            VisibilityCondition::Static(value) => {
                assert_eq!(value.entries.len(), 2);
                assert_eq!(value.entries[0].breakpoint, Breakpoint::Xs);
                assert!(!value.entries[0].value);
                assert_eq!(value.entries[1].breakpoint, Breakpoint::Md);
                assert!(value.entries[1].value);
            }
            VisibilityCondition::Signal(_) => panic!("static show"),
            VisibilityCondition::NumberComparison { .. } => panic!("static show"),
        }

        let ViewNode::Text { props, .. } = &box_children[0] else {
            panic!("text");
        };
        assert_eq!(
            props.style.element.show,
            Some(VisibilityCondition::Signal("isReady".to_string()))
        );

        let ViewNode::Each {
            children: row_children,
            ..
        } = &box_children[1]
        else {
            panic!("each");
        };
        let ViewNode::Text { props, .. } = &row_children[0] else {
            panic!("row text");
        };
        assert_eq!(
            props.style.element.show,
            Some(VisibilityCondition::Signal("row.ready".to_string()))
        );
    }

    #[test]
    fn parses_and_validates_numeric_show_condition() {
        let page = parse_page(
            r#"page countPage
  signal itemCount value:11
  Section show:{ when:itemCount gt:10 }
    Text
      "Many items""#,
        )
        .expect("numeric show");
        let ViewNode::Scope { children, .. } = page else {
            panic!("scope")
        };
        let ViewNode::Section { props, .. } = &children[0] else {
            panic!("section")
        };
        let Some(VisibilityCondition::NumberComparison { path, comparison }) =
            props.element.show.as_ref()
        else {
            panic!("numeric show")
        };
        assert_eq!(path, "itemCount");
        assert_eq!(comparison.operator.as_str(), ">");
        assert_eq!(comparison.value, "10");

        let error = parse_page(
            r#"page countPage
  signal label value:"many"
  Section show:{ when:label gt:10 }
    Text
      "Many items""#,
        )
        .expect_err("string comparison");
        assert!(
            error
                .to_string()
                .contains("invalid signal path `label` in `show.when`: expected number")
        );

        let error = parse_page(
            r#"page countPage
  signal itemCount value:11
  Section show:{ when:itemCount gt:10 lte:20 }
    Text
      "Many items""#,
        )
        .expect_err("multiple comparisons");
        assert!(
            error
                .to_string()
                .contains("show conditions accept one numeric comparator")
        );
    }

    #[test]
    fn parses_box_and_card_animation_props() {
        let tree = parse_page(
            r#"page motionPage
  Box animation:"fadeIn"
    Card animation:"slideUp"
      Text
        "Motion""#,
        )
        .expect("tree");

        let ViewNode::Box { props, children } = tree else {
            panic!("box");
        };
        assert_eq!(props.animation, Some(ViewAnimation::FadeIn));

        let ViewNode::Card { props, .. } = &children[0] else {
            panic!("card");
        };
        assert_eq!(props.style.animation, Some(ViewAnimation::SlideUp));
    }

    #[test]
    fn parses_section_background_props() {
        let tree = parse_page(
            r#"page landingPage
  Section id:"hero" background:{ xs:"soft" md:"aurora" } color:"onBackground" boxed:true p:8
    Text
      "Hero""#,
        )
        .expect("tree");

        let ViewNode::Section { props, children } = tree else {
            panic!("section");
        };
        assert_eq!(props.element.id.as_deref(), Some("hero"));
        assert_eq!(
            props.background.expect("background").entries[1].value,
            SectionBackground::Aurora
        );
        assert!(props.text.is_some());
        assert!(props.boxed);
        assert_eq!(children.len(), 1);
    }

    #[test]
    fn rejects_responsive_section_boxed_prop() {
        let error = parse_page(
            r#"page landingPage
  Section boxed:{ xs:true md:false }
    Text
      "Hero""#,
        )
        .expect_err("boxed");

        assert!(
            error
                .to_string()
                .contains("invalid value for prop `boxed`: expected boolean")
        );
    }

    #[test]
    fn rejects_invalid_show_visibility_conditions() {
        let non_bool = parse_page(
            r#"page readyPage
  signal profile value:{ name:"" }
  Text show:profile.name
    "Ready""#,
        )
        .expect_err("non bool");
        assert!(
            non_bool
                .to_string()
                .contains("invalid signal path `profile.name` in `show`: expected bool")
        );

        let responsive_string = parse_page(
            r#"page readyPage
  Text show:{ xs:"false" }
    "Ready""#,
        )
        .expect_err("responsive string");
        assert!(
            responsive_string
                .to_string()
                .contains("invalid value for prop `show`: expected boolean")
        );
    }
