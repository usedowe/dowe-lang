fn dropzone_picker_route() -> ViewRoute {
    ViewRoute {
        id: "dropzone-picker".to_string(),
        route_path: "/dropzone-picker".to_string(),
        layout_tree: ViewNode::Children,
        page_tree: ViewNode::Dropzone {
            props: DropzoneProps {
                style: VariantProps {
                    label: Some("Assets".to_string()),
                    placeholder: Some("Choose files".to_string()),
                    variant: Some(ComponentVariant::Outlined),
                    color: Some(ColorFamily::Primary),
                    ..Default::default()
                },
                accept: Some("image/*".to_string()),
                multiple: true,
                max_size: Some(4096),
                size: ButtonSize::Md,
                name: Some("assets".to_string()),
                help_text: Some("Images only".to_string()),
                error_text: None,
                disabled: false,
            },
        },
        sections: Vec::new(),
        navigation_actions: Vec::new(),
    }
}

fn overlay_parity_route() -> ViewRoute {
    ViewRoute {
        id: "overlay-parity".to_string(),
        route_path: "/overlay-parity".to_string(),
        layout_tree: ViewNode::Children,
        page_tree: ViewNode::Box {
            props: StyleProps::default(),
            children: vec![
                ViewNode::Modal {
                    props: ModalProps {
                        style: VariantProps {
                            variant: Some(ComponentVariant::Outlined),
                            color: Some(ColorFamily::Warning),
                            ..Default::default()
                        },
                        open: "modal01".to_string(),
                        on_close: None,
                        disable_overlay_close: false,
                        hide_close_button: false,
                    },
                    header: vec![text("Settings")],
                    body: vec![text("Body")],
                    footer: Vec::new(),
                },
                ViewNode::AlertDialog {
                    props: AlertDialogProps {
                        style: VariantProps {
                            variant: Some(ComponentVariant::Solid),
                            color: Some(ColorFamily::Warning),
                            ..Default::default()
                        },
                        open: "alert01".to_string(),
                        title: "Archive?".to_string(),
                        description: "Archive this project.".to_string(),
                        confirm_text: "Archive".to_string(),
                        cancel_text: "Cancel".to_string(),
                        on_confirm: None,
                        on_cancel: None,
                        loading: false,
                    },
                },
            ],
        },
        sections: Vec::new(),
        navigation_actions: Vec::new(),
    }
}
