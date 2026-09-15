use super::{
    BadgeVisualContract, CarouselControlContract, CarouselControlPlacement, CarouselVisualContract,
    AccordionVisualContract, AlertVisualContract, AvatarVisualContract, BannerVisualContract, ButtonVisualContract, CardVisualContract, ChipVisualContract, DatePickerVisualContract, DividerVisualContract, EmptyVisualContract, FeedbackSurfaceVisualContract, FormControlVisualContract, InteractionStateVisualContract, LoadingVisualContract, MediaVisualContract, MenuVisualContract, NavigationShellVisualContract, ProgressVisualContract, SkeletonVisualContract, StepperVisualContract, TableVisualContract, TabsVisualContract, VisualizationVisualContract,
    ToggleGroupKind, ToggleGroupProps, VariantProps,
    PaginationControlContract, PaginationVariant, PaginationVisualContract, IconButtonVisualContract,
};

#[test]
fn chip_contract_keeps_native_metrics_explicit() {
    let chip = ChipVisualContract::for_size(ButtonSize::Md);
    assert_eq!((chip.height, chip.horizontal_padding, chip.text_size), (32, 16, 14));
    assert_eq!(chip.content_gap, 8);
    assert!((chip.close_alpha - 0.72).abs() < f32::EPSILON);
}

#[test]
fn surface_contracts_keep_divider_and_skeleton_metrics_explicit() {
    assert_eq!(DividerVisualContract::standard().thickness, 1);
    let skeleton = SkeletonVisualContract::standard();
    assert_eq!((skeleton.text_height, skeleton.default_radius, skeleton.pulse_duration_ms), (16, 6, 900));
    assert!((skeleton.pulse_alpha - 0.45).abs() < f32::EPSILON);
}

#[test]
fn feedback_contracts_keep_alert_and_empty_composition_explicit() {
    let alert = AlertVisualContract::standard();
    assert_eq!((alert.panel_padding, alert.content_gap, alert.title_size, alert.description_size, alert.button_gap, alert.close_button_size), (20, 16, 18, 14, 12, 24));
    let empty = EmptyVisualContract::standard();
    assert_eq!((empty.panel_padding, empty.content_gap, empty.icon_size), (24, 12, 112));
    assert_eq!((empty.title_size, empty.description_size, empty.action_horizontal_padding, empty.action_vertical_padding), (20, 14, 16, 9));
}

#[test]
fn button_contract_keeps_size_metrics_shared() {
    let md = ButtonVisualContract::for_size(ButtonSize::Md);
    assert_eq!((md.min_height, md.horizontal_padding, md.vertical_padding, md.text_size), (44, 16, 10, 14));
}

#[test]
fn navigation_contracts_keep_list_and_state_geometry_explicit() {
    assert_eq!(TabsVisualContract::standard(), TabsVisualContract { list_gap: 4, tab_horizontal_padding: 12, tab_vertical_padding: 8, indicator_thickness: 2 });
    assert_eq!(StepperVisualContract::standard().indicator_size, 32);
    assert_eq!(AccordionVisualContract::standard().header_min_height, 44);
}

#[test]
fn form_control_contract_keeps_field_metrics_shared() {
    assert_eq!(FormControlVisualContract::standard(), FormControlVisualContract { min_height: 40, horizontal_padding: 12, text_size: 14, radius: 8 });
}

#[test]
fn feedback_surface_contract_keeps_overlay_spacing_shared() {
    assert_eq!(FeedbackSurfaceVisualContract::standard(), FeedbackSurfaceVisualContract { panel_padding: 16, content_gap: 12, tooltip_padding_x: 12, tooltip_padding_y: 8 });
}

#[test]
fn card_contract_keeps_surface_metrics_shared() {
    assert_eq!(CardVisualContract::standard(), CardVisualContract { padding: 16, content_gap: 12, border_width: 1 });
}

#[test]
fn table_contract_keeps_density_metrics_shared() {
    assert_eq!(TableVisualContract::standard(), TableVisualContract { header_padding_y: 12, cell_padding: 16, text_size: 14, divider_width: 1 });
}

#[test]
fn avatar_contract_keeps_group_composition_shared() {
    assert_eq!(AvatarVisualContract::standard(), AvatarVisualContract { group_overlap: 12, indicator_border: 3, counter_border: 3 });
}

#[test]
fn loading_contract_keeps_spinner_metrics_shared() {
    assert_eq!(LoadingVisualContract::standard(), LoadingVisualContract { spinner_size: 18, stroke_width: 2, disabled_alpha: 0.5 });
}

#[test]
fn media_contract_keeps_surface_controls_shared() {
    assert_eq!(MediaVisualContract::standard(), MediaVisualContract { default_radius: 8, control_size: 40, overlay_alpha: 48 });
}

#[test]
fn visualization_contract_keeps_data_surfaces_shared() {
    assert_eq!(VisualizationVisualContract::standard(), VisualizationVisualContract { container_padding: 16, grid_line_width: 1, empty_min_height: 180 });
}

#[test]
fn navigation_shell_contract_keeps_shell_geometry_shared() {
    assert_eq!(NavigationShellVisualContract::standard(), NavigationShellVisualContract { item_min_height: 40, item_padding: 12, rail_width: 72, item_gap: 4 });
}

#[test]
fn menu_contract_keeps_option_geometry_shared() {
    assert_eq!(MenuVisualContract::standard(), MenuVisualContract { option_min_height: 40, option_padding: 12, menu_gap: 4, min_width: 192 });
}

#[test]
fn date_picker_contract_keeps_calendar_geometry_shared() {
    assert_eq!(DatePickerVisualContract::standard(), DatePickerVisualContract { cell_size: 32, grid_gap: 4, nav_size: 32 });
}

#[test]
fn progress_contract_keeps_track_geometry_shared() {
    assert_eq!(ProgressVisualContract::standard(), ProgressVisualContract { track_height: 6, radius: 999, disabled_alpha: 50 });
}

#[test]
fn banner_contract_keeps_inline_feedback_geometry_shared() {
    assert_eq!(BannerVisualContract::standard(), BannerVisualContract { padding: 16, content_gap: 12, action_height: 40, disabled_alpha: 50 });
}

#[test]
fn interaction_state_contract_keeps_feedback_shared() {
    assert_eq!(InteractionStateVisualContract::standard(), InteractionStateVisualContract { disabled_alpha: 0.5, focus_ring_width: 2, focus_ring_alpha: 0.24, pressed_scale: 0.94, error_ring_width: 1 });
}

#[test]
fn navigation_controls_share_border_alpha_across_targets() {
    assert!((CarouselVisualContract::for_scheme(ColorFamily::Primary).control_border_alpha - 0.24).abs() < f32::EPSILON);
    let pagination = PaginationVisualContract::for_scheme(ColorFamily::Primary);
    assert!((pagination.control_border_alpha - 0.24).abs() < f32::EPSILON);
}

#[test]
fn icon_button_contract_centralizes_shared_state_alphas() {
    assert_eq!(
        IconButtonVisualContract::standard(),
        IconButtonVisualContract { border_alpha: 0.24, disabled_alpha: 0.42, focus_ring_alpha: 0.24 }
    );
}


#[test]
fn resolves_carousel_control_contract_from_size() {
    let slide = carousel_slide_component(
        vec![string_prop("id", "one")],
        vec![text_node("Slide").expect("text")],
    )
    .expect("slide");
    let node =
        carousel_component_node(vec![string_prop("size", "lg")], vec![slide]).expect("carousel");
    let ViewNode::Carousel { props, .. } = node else {
        panic!("carousel");
    };
    let contract = props.control_contract();
    assert_eq!(
        contract.navigation_placement,
        CarouselControlPlacement::OverlayStage
    );
    assert_eq!(
        contract.controls_placement,
        CarouselControlPlacement::BelowTrack
    );
    assert_eq!(contract.navigation_size, 40);
    assert_eq!(contract.navigation_inset, 16);
    assert_eq!(contract.control_size, 32);
    assert_eq!(contract.control_gap, 8);
    assert_eq!(contract.indicator_gap, 8);
    assert_eq!(contract.indicator_height, 10);
    assert_eq!(contract.indicator_inactive_width, 40);
    assert_eq!(contract.indicator_active_width, 64);
    assert_eq!(contract.indicator_dot_size, 10);
    assert_eq!(contract.indicator_dot_active_scale_percent, 125);

    let sm = CarouselControlContract::for_size(ButtonSize::Sm);
    assert_eq!(sm.indicator_height, 6);
    assert_eq!(sm.indicator_inactive_width, 24);
    assert_eq!(sm.indicator_active_width, 32);
}

#[test]
fn resolves_pagination_variants_and_shared_indicator_contract() {
    assert_eq!(
        PaginationVariant::from_name("default"),
        Some(PaginationVariant::Pages)
    );
    assert_eq!(
        PaginationVariant::from_name("controls"),
        Some(PaginationVariant::Controls)
    );
    assert_eq!(
        PaginationVariant::from_name("dots"),
        Some(PaginationVariant::Dots)
    );
    assert_eq!(
        PaginationVariant::from_name("bars"),
        Some(PaginationVariant::Bars)
    );
    assert_eq!(PaginationVariant::from_name("wheel"), None);
    assert_eq!(PaginationVariant::all().len(), 4);

    let sm = PaginationControlContract::for_size(ButtonSize::Sm);
    let md = PaginationControlContract::for_size(ButtonSize::Md);
    let lg = PaginationControlContract::for_size(ButtonSize::Lg);
    assert_eq!(sm.control_size, md.control_size);
    assert_eq!(sm.indicator_gap, md.indicator_gap);
    assert_eq!(
        (
            sm.indicator_height,
            sm.indicator_inactive_width,
            sm.indicator_active_width
        ),
        (6, 24, 32)
    );
    assert_eq!(
        (
            md.indicator_height,
            md.indicator_inactive_width,
            md.indicator_active_width
        ),
        (8, 32, 48)
    );
    assert_eq!(
        (
            lg.indicator_height,
            lg.indicator_inactive_width,
            lg.indicator_active_width
        ),
        (10, 40, 64)
    );
    assert_eq!(lg.indicator_dot_size, 10);
    assert_eq!(lg.indicator_dot_active_scale_percent, 125);

    let props = ToggleGroupProps {
        style: VariantProps {
            color: Some(ColorFamily::Success),
            ..Default::default()
        },
        kind: ToggleGroupKind::Pagination,
        pagination: None,
        value: None,
        selected: String::new(),
        multiple: false,
        size: ButtonSize::Md,
        wide: false,
        vertical: false,
        disabled: false,
        aria_label: None,
        on_change: None,
    };
    let visual = props.pagination_visual_contract();
    assert_eq!(visual.accent, ColorToken::Success);
    assert_eq!(visual.inactive_alpha, 0.28);
    assert_eq!(visual.disabled_alpha, 0.42);
}

#[test]
fn carousel_visual_contract_uses_scheme_roles_for_all_targets() {
    let slide = carousel_slide_component(
        vec![string_prop("id", "one")],
        vec![text_node("Slide").expect("text")],
    )
    .expect("slide");
    let node = carousel_component_node(
        vec![string_prop("scheme", "success")],
        vec![slide],
    )
    .expect("carousel");
    let ViewNode::Carousel { props, .. } = node else {
        panic!("carousel");
    };
    let visual = props.visual_contract();
    assert_eq!(visual.accent, ColorToken::Success);
    assert_eq!(visual.indicator_inactive_alpha, 0.26);
    assert_eq!(visual.title, ColorToken::SuccessTitle);
    assert_eq!(visual.content, ColorToken::SuccessText);
    assert_eq!(
        CarouselVisualContract::for_scheme(ColorFamily::Success),
        visual
    );
}

#[test]
fn badge_visual_contract_keeps_native_metrics_explicit() {
    let contract = BadgeVisualContract::standard();
    assert_eq!(
        (contract.font_size, contract.font_weight, contract.height),
        (12, 600, 20)
    );
    assert_eq!((contract.horizontal_padding, contract.vertical_padding), (6, 2));
}
