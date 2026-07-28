    #[test]
    fn parses_media_display_and_form_components() {
        let tree = parse_page(
            r##"page componentsPage
  signal accepted value:false
  signal themeColor value:"#3366ff"
  signal shipDate value:"2026-06-05"
  signal startDate value:"2026-06-01"
  signal endDate value:"2026-06-08"
  signal choice value:"basic"
  Box
    Audio src:"https://cdn.pixabay.com/audio/2022/04/25/audio_5d61b5204f.mp3" subtitle:"Preview" avatarSrc:"https://example.com/avatar.png" scheme:"primary" variant:"soft"
    Image src:"https://example.com/photo.jpg" alt:"Photo" aspect:"square" objectFit:"contain" loading:"eager" hideControls:true scheme:"secondary"
    Accordion multiple:true variant:"outlined" scheme:"surface"
      item id:"intro" label:"Intro" defaultOpen:true
        Text
          "Intro body"
      item id:"details" label:"Details" disabled:true
        Text
          "Details body"
    Carousel title:"Samples" variant:"coverFlow" autoplay:true autoplayInterval:4000 showCounter:true orientation:"horizontal" size:"sm" indicatorType:"dot" slidesPerView:2 gap:12 scheme:"info"
      slide id:"one"
        Text
          "First"
      slide id:"two"
        Text
          "Second"
    Checkbox bind:accepted checked:true label:"I accept" name:"accepted" scheme:"success"
    Color bind:themeColor value:"#3366ff" label:"Theme" showHex:true showRgb:true showCmyk:true showOklch:true scheme:"primary"
    Date bind:shipDate value:"2026-06-05" label:"Ship date" min:"2026-01-01" max:"2026-12-31" scheme:"warning"
    DateRange start:startDate end:endDate startValue:"2026-06-01" endValue:"2026-06-08" label:"Range" scheme:"danger"
    RadioGroup bind:choice label:"Plan" size:"lg" info:"Choose one" scheme:"muted"
      item value:"basic" label:"Basic"
      item value:"pro" label:"Pro" disabled:true
    Toggle bind:accepted checked:true label:"Enabled" labelLeft:"Off" labelRight:"On" name:"enabled" scheme:"secondary""##,
        )
        .expect("tree");

        let ViewNode::Scope { children, .. } = tree else {
            panic!("scope");
        };
        let ViewNode::Box {
            children: box_children,
            ..
        } = &children[0]
        else {
            panic!("box");
        };
        assert_eq!(box_children.len(), 10);

        let ViewNode::Audio { props } = &box_children[0] else {
            panic!("audio");
        };
        assert_eq!(props.subtitle.as_deref(), Some("Preview"));
        assert_eq!(props.style.color, Some(ColorFamily::Primary));

        let ViewNode::Image { props } = &box_children[1] else {
            panic!("image");
        };
        assert_eq!(props.aspect, ImageAspect::Square);
        assert_eq!(props.object_fit, ImageObjectFit::Contain);
        assert_eq!(props.loading, ImageLoading::Eager);
        assert!(props.hide_controls);

        let ViewNode::Accordion { props, items } = &box_children[2] else {
            panic!("accordion");
        };
        assert!(props.multiple);
        assert_eq!(props.style.variant, Some(ComponentVariant::Outlined));
        assert_eq!(props.style.color, Some(ColorFamily::Surface));
        assert_eq!(items.len(), 2);
        assert!(items[0].default_open);
        assert!(items[1].disabled);

        let ViewNode::Carousel { props, slides } = &box_children[3] else {
            panic!("carousel");
        };
        assert!(props.autoplay);
        assert_eq!(props.variant, CarouselVariant::CoverFlow);
        assert_eq!(props.autoplay_interval, 4000);
        assert_eq!(props.orientation, CarouselOrientation::Horizontal);
        assert_eq!(props.indicator_type, CarouselIndicatorType::Dot);
        assert_eq!(props.slides_per_view, 2);
        assert_eq!(props.gap, 12);
        assert_eq!(slides.len(), 2);

        let ViewNode::Checkbox { props } = &box_children[4] else {
            panic!("checkbox");
        };
        assert_eq!(props.style.element.bind.as_deref(), Some("accepted"));
        assert_eq!(props.style.label.as_deref(), Some("I accept"));
        assert!(props.checked);

        let ViewNode::Color { props } = &box_children[5] else {
            panic!("color");
        };
        assert_eq!(props.style.element.bind.as_deref(), Some("themeColor"));
        assert!(props.show_hex && props.show_rgb && props.show_cmyk && props.show_oklch);

        let ViewNode::Date { props } = &box_children[6] else {
            panic!("date");
        };
        assert_eq!(props.style.element.bind.as_deref(), Some("shipDate"));
        assert_eq!(props.value.as_deref(), Some("2026-06-05"));
        assert_eq!(props.min.as_deref(), Some("2026-01-01"));

        let ViewNode::DateRange { props } = &box_children[7] else {
            panic!("date range");
        };
        assert_eq!(props.start.as_deref(), Some("startDate"));
        assert_eq!(props.end.as_deref(), Some("endDate"));
        assert_eq!(props.start_value.as_deref(), Some("2026-06-01"));

        let ViewNode::RadioGroup { props, options } = &box_children[8] else {
            panic!("radio group");
        };
        assert_eq!(props.style.element.bind.as_deref(), Some("choice"));
        assert_eq!(props.size, ButtonSize::Lg);
        assert_eq!(options.len(), 2);
        assert!(options[1].disabled);

        let ViewNode::Toggle { props } = &box_children[9] else {
            panic!("toggle");
        };
        assert_eq!(props.style.element.bind.as_deref(), Some("accepted"));
        assert_eq!(props.label_left.as_deref(), Some("Off"));
        assert_eq!(props.label_right.as_deref(), Some("On"));
        assert!(props.checked);
    }

    #[test]
    fn parses_theme_fab_slider_and_dropzone_components() {
        let tree = parse_page(
            r##"page controlsPage
  signal volume value:40
  fn create
    reset volume
  Box
    ToggleTheme variant:"soft" scheme:"secondary" size:"sm" lightLabel:"Light mode" darkLabel:"Dark mode"
    SelectTheme label:"Theme palette" placeholder:"Choose a palette" variant:"outlined" scheme:"surface" size:"sm"
    Fab position:"top-left" fixed:false offsetX:6 offsetY:8 icon:"settings" label:"Actions" variant:"soft" scheme:"primary" size:"lg" onClick:create
      fabAction label:"Docs" icon:"link" href:"#top" navigate:"replace" scheme:"info"
      fabAction label:"Create" icon:"plus" onClick:create scheme:"success"
    Slider bind:volume min:0 max:100 step:5 label:"Volume" name:"volume" hideLabel:false scheme:"warning" size:"lg"
    Dropzone accept:"image/*" multiple:false maxSize:2048 name:"images" label:"Images" helpText:"PNG only" placeholder:"Drop images" disabled:false variant:"outlined" scheme:"surface" size:"sm""##,
        )
        .expect("tree");

        let ViewNode::Scope { children, .. } = tree else {
            panic!("scope");
        };
        let ViewNode::Box {
            children: box_children,
            ..
        } = &children[0]
        else {
            panic!("box");
        };
        assert_eq!(box_children.len(), 5);

        let ViewNode::ToggleTheme { props } = &box_children[0] else {
            panic!("theme toggle");
        };
        assert_eq!(props.light_label, "Light mode");
        assert_eq!(props.dark_label, "Dark mode");
        assert_eq!(props.style.variant, Some(ComponentVariant::Soft));
        assert_eq!(props.style.color, Some(ColorFamily::Secondary));

        let ViewNode::SelectTheme { props } = &box_children[1] else {
            panic!("theme select");
        };
        assert_eq!(props.label, "Theme palette");
        assert_eq!(props.placeholder, "Choose a palette");
        assert!(props.themes.is_empty());
        assert_eq!(props.default_theme, "light");
        assert_eq!(props.style.variant, Some(ComponentVariant::Outlined));
        assert_eq!(props.style.color, Some(ColorFamily::Surface));

        let ViewNode::Fab { props, actions } = &box_children[2] else {
            panic!("fab");
        };
        assert_eq!(props.position, OverlayCornerPosition::TopLeft);
        assert!(!props.fixed);
        assert_eq!(props.icon, ViewIcon::Settings);
        assert_eq!(props.style.variant, Some(ComponentVariant::Soft));
        assert_eq!(props.style.color, Some(ColorFamily::Primary));
        assert_eq!(actions.len(), 2);
        assert_eq!(actions[0].label, "Docs");
        assert_eq!(actions[0].icon, ViewIcon::Link);
        assert_eq!(actions[0].color, ColorFamily::Info);
        assert!(matches!(
            actions[0].navigation,
            Some(NavigationAction::Section { .. })
        ));
        assert_eq!(actions[1].on_click.as_deref(), Some("create"));
        assert_eq!(actions[1].color, ColorFamily::Success);

        let ViewNode::Slider { props } = &box_children[3] else {
            panic!("slider");
        };
        assert_eq!(props.style.element.bind.as_deref(), Some("volume"));
        assert_eq!(props.value, "0");
        assert_eq!(props.min, "0");
        assert_eq!(props.max, "100");
        assert_eq!(props.step.as_deref(), Some("5"));
        assert_eq!(props.style.label.as_deref(), Some("Volume"));
        assert_eq!(props.style.color, Some(ColorFamily::Warning));
        assert_eq!(props.size, ButtonSize::Lg);

        let ViewNode::Dropzone { props } = &box_children[4] else {
            panic!("dropzone");
        };
        assert_eq!(props.accept.as_deref(), Some("image/*"));
        assert!(!props.multiple);
        assert_eq!(props.max_size, Some(2048));
        assert_eq!(props.name.as_deref(), Some("images"));
        assert_eq!(props.help_text.as_deref(), Some("PNG only"));
        assert_eq!(props.style.placeholder.as_deref(), Some("Drop images"));
        assert_eq!(props.style.variant, Some(ComponentVariant::Outlined));
        assert_eq!(props.style.color, Some(ColorFamily::Surface));
        assert_eq!(props.size, ButtonSize::Sm);
    }

