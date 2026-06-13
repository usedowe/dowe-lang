fn media_display_form_tree() -> ViewNode {
    ViewNode::Box {
        props: StyleProps::default(),
        children: vec![
            ViewNode::Audio {
                props: AudioProps {
                    style: VariantProps {
                        variant: Some(ComponentVariant::Soft),
                        color: Some(ColorFamily::Primary),
                        ..Default::default()
                    },
                    src: "https://cdn.pixabay.com/audio/2022/04/25/audio_5d61b5204f.mp3"
                        .to_string(),
                    subtitle: Some("Preview".to_string()),
                    avatar_src: Some("https://example.com/avatar.png".to_string()),
                },
            },
            ViewNode::Image {
                props: ImageProps {
                    style: VariantProps {
                        color: Some(ColorFamily::Secondary),
                        ..Default::default()
                    },
                    src: "https://example.com/photo.jpg".to_string(),
                    alt: "Photo".to_string(),
                    aspect: ImageAspect::Square,
                    object_fit: ImageObjectFit::Contain,
                    loading: ImageLoading::Eager,
                    hide_controls: false,
                },
            },
            ViewNode::Accordion {
                props: AccordionProps {
                    style: VariantProps {
                        variant: Some(ComponentVariant::Outlined),
                        color: Some(ColorFamily::Surface),
                        ..Default::default()
                    },
                    multiple: true,
                },
                items: vec![AccordionItem {
                    id: "intro".to_string(),
                    label: "Intro".to_string(),
                    disabled: false,
                    default_open: true,
                    children: vec![text("Intro body")],
                }],
            },
            ViewNode::Carousel {
                props: CarouselProps {
                    style: VariantProps {
                        color: Some(ColorFamily::Info),
                        ..Default::default()
                    },
                    autoplay: true,
                    autoplay_interval: 4000,
                    disable_loop: false,
                    hide_controls: false,
                    hide_indicators: false,
                    show_navigation: true,
                    show_counter: true,
                    orientation: CarouselOrientation::Horizontal,
                    size: ButtonSize::Sm,
                    indicator_type: CarouselIndicatorType::Dot,
                    title: Some("Samples".to_string()),
                    slide_width: Some(240),
                    slide_height: None,
                    slides_per_view: 1,
                    gap: 12,
                },
                slides: vec![
                    CarouselSlide {
                        id: "one".to_string(),
                        children: vec![text("First")],
                    },
                    CarouselSlide {
                        id: "two".to_string(),
                        children: vec![text("Second")],
                    },
                ],
            },
            ViewNode::Checkbox {
                props: CheckboxProps {
                    style: VariantProps {
                        color: Some(ColorFamily::Success),
                        label: Some("I accept".to_string()),
                        element: ElementProps {
                            bind: Some("accepted".to_string()),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    checked: true,
                    disabled: false,
                    name: Some("accepted".to_string()),
                },
            },
            ViewNode::Color {
                props: ColorProps {
                    style: VariantProps {
                        variant: Some(ComponentVariant::Outlined),
                        color: Some(ColorFamily::Primary),
                        label: Some("Theme".to_string()),
                        element: ElementProps {
                            bind: Some("themeColor".to_string()),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    value: "#3366ff".to_string(),
                    size: ButtonSize::Md,
                    name: Some("themeColor".to_string()),
                    help_text: Some("Pick a color".to_string()),
                    error_text: None,
                    show_hex: true,
                    show_rgb: true,
                    show_cmyk: false,
                    show_oklch: false,
                },
            },
            ViewNode::Date {
                props: DateProps {
                    style: VariantProps {
                        variant: Some(ComponentVariant::Outlined),
                        color: Some(ColorFamily::Warning),
                        label: Some("Ship date".to_string()),
                        element: ElementProps {
                            bind: Some("shipDate".to_string()),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    value: Some("2026-06-05".to_string()),
                    size: ButtonSize::Md,
                    name: Some("shipDate".to_string()),
                    help_text: None,
                    error_text: None,
                    min: Some("2026-01-01".to_string()),
                    max: Some("2026-12-31".to_string()),
                },
            },
            ViewNode::DateRange {
                props: DateRangeProps {
                    style: VariantProps {
                        variant: Some(ComponentVariant::Outlined),
                        color: Some(ColorFamily::Danger),
                        label: Some("Range".to_string()),
                        ..Default::default()
                    },
                    start: Some("startDate".to_string()),
                    end: Some("endDate".to_string()),
                    start_value: Some("2026-06-01".to_string()),
                    end_value: Some("2026-06-08".to_string()),
                    size: ButtonSize::Md,
                    name: Some("range".to_string()),
                    help_text: None,
                    error_text: None,
                    min: Some("2026-01-01".to_string()),
                    max: Some("2026-12-31".to_string()),
                },
            },
            ViewNode::RadioGroup {
                props: RadioGroupProps {
                    style: VariantProps {
                        color: Some(ColorFamily::Muted),
                        label: Some("Plan".to_string()),
                        element: ElementProps {
                            bind: Some("choice".to_string()),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    size: ButtonSize::Lg,
                    name: Some("plan".to_string()),
                    info: Some("Choose one".to_string()),
                    error: None,
                },
                options: vec![
                    RadioOption {
                        value: "basic".to_string(),
                        label: "Basic".to_string(),
                        disabled: false,
                    },
                    RadioOption {
                        value: "pro".to_string(),
                        label: "Pro".to_string(),
                        disabled: true,
                    },
                ],
            },
            ViewNode::Toggle {
                props: ToggleProps {
                    style: VariantProps {
                        color: Some(ColorFamily::Secondary),
                        label: Some("Enabled".to_string()),
                        element: ElementProps {
                            bind: Some("accepted".to_string()),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    checked: true,
                    disabled: false,
                    name: Some("enabled".to_string()),
                    label_left: Some("Off".to_string()),
                    label_right: Some("On".to_string()),
                },
            },
        ],
    }
}
