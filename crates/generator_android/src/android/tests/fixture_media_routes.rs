fn media_display_form_route() -> ViewRoute {
    ViewRoute {
        id: "components".to_string(),
        route_path: "/components".to_string(),
        layout_tree: ViewNode::Children,
        page_tree: ViewNode::Box {
            props: Default::default(),
            children: vec![
                ViewNode::Audio {
                    props: AudioProps {
                        style: VariantProps {
                            variant: Some(ComponentVariant::Solid),
                            color: Some(ColorFamily::Primary),
                            ..Default::default()
                        },
                        src: "https://cdn.pixabay.com/audio/2022/04/25/audio_5d61b5204f.mp3"
                            .to_string(),
                        subtitle: Some("Preview".to_string()),
                        avatar_src: None,
                    },
                },
                ViewNode::Image {
                    props: ImageProps {
                        style: VariantProps {
                            color: Some(ColorFamily::Secondary),
                            ..Default::default()
                        },
                        src: "https://example.com/photo.jpg".to_string(),
                        reactive_src: None,
                        alt: "Photo".to_string(),
                        aspect: ImageAspect::Square,
                        object_fit: ImageObjectFit::Cover,
                        loading: ImageLoading::Lazy,
                        hide_controls: true,
                    },
                },
                ViewNode::Accordion {
                    props: AccordionProps {
                        style: VariantProps {
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
                        variant: CarouselVariant::Snapping,
                        autoplay: false,
                        autoplay_interval: 3000,
                        disable_loop: false,
                        hide_controls: false,
                        hide_indicators: false,
                        show_navigation: true,
                        show_counter: true,
                        orientation: CarouselOrientation::Horizontal,
                        size: ButtonSize::Sm,
                        indicator_type: CarouselIndicatorType::Bar,
                        title: Some("Samples".to_string()),
                        slide_width: None,
                        slide_height: None,
                        slides_per_view: 1,
                        gap: 8,
                    },
                    slides: vec![CarouselSlide {
                        id: "one".to_string(),
                        children: vec![text("First")],
                    }],
                },
                ViewNode::Checkbox {
                    props: CheckboxProps {
                        style: VariantProps {
                            color: Some(ColorFamily::Success),
                            label: Some("Accept".to_string()),
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
                            color: Some(ColorFamily::Primary),
                            label: Some("Theme".to_string()),
                            label_floating: true,
                            element: ElementProps {
                                bind: Some("themeColor".to_string()),
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        value: "#3366ff".to_string(),
                        size: ButtonSize::Md,
                        name: None,
                        help_text: None,
                        error_text: None,
                        show_hex: true,
                        show_rgb: false,
                        show_cmyk: false,
                        show_oklch: false,
                    },
                },
                ViewNode::Date {
                    props: DateProps {
                        style: VariantProps {
                            color: Some(ColorFamily::Warning),
                            label: Some("Ship date".to_string()),
                            label_floating: true,
                            element: ElementProps {
                                bind: Some("shipDate".to_string()),
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        value: Some("2026-06-05".to_string()),
                        size: ButtonSize::Lg,
                        name: None,
                        help_text: None,
                        error_text: None,
                        min: None,
                        max: None,
                    },
                },
                ViewNode::DateRange {
                    props: DateRangeProps {
                        style: VariantProps {
                            color: Some(ColorFamily::Danger),
                            label: Some("Range".to_string()),
                            label_floating: true,
                            ..Default::default()
                        },
                        start: Some("startDate".to_string()),
                        end: Some("endDate".to_string()),
                        start_value: Some("2026-06-01".to_string()),
                        end_value: Some("2026-06-08".to_string()),
                        size: ButtonSize::Sm,
                        name: None,
                        help_text: None,
                        error_text: None,
                        min: None,
                        max: None,
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
                        size: ButtonSize::Md,
                        orientation: RadioGroupOrientation::Horizontal,
                        presentation: RadioGroupPresentation::List,
                        name: Some("plan".to_string()),
                        info: None,
                        error: None,
                    },
                    options: vec![RadioOption {
                        value: "basic".to_string(),
                        label: "Basic".to_string(),
                        description: None,
                        icon: None,
                        disabled: false,
                    }],
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
                        name: None,
                        label_left: Some("Off".to_string()),
                        label_right: Some("On".to_string()),
                    },
                },
            ],
        },
        sections: Vec::new(),
        navigation_actions: Vec::new(),
    }
}

