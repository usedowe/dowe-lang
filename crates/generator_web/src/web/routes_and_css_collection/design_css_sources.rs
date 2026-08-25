const DESIGN_RESET_CSS: &str = include_str!("design_css/reset.css");
const DESIGN_FOUNDATION_CSS: &str = include_str!("design_css/foundation.css");
const DESIGN_MOTION_CSS: &str = include_str!("design_css/motion.css");

const DESIGN_CONTENT_CSS: &[&str] = &[include_str!("design_css/content.css")];

const DESIGN_FORMS_BEFORE_CONTROL_CSS: &[&str] = &[
    include_str!("design_css/forms_groups.css"),
    include_str!("design_css/forms_color.css"),
    include_str!("design_css/forms_field.css"),
];

const DESIGN_FORMS_AFTER_CONTROL_CSS: &[&str] = &[
    include_str!("design_css/forms_input_select.css"),
    include_str!("design_css/forms_choices.css"),
    include_str!("design_css/forms_dates.css"),
    include_str!("design_css/forms_theme_slider.css"),
    include_str!("design_css/forms_upload.css"),
    include_str!("design_css/forms_combo_drag.css"),
    include_str!("design_css/forms_editing.css"),
    include_str!("design_css/forms_identity.css"),
];

const DESIGN_MEDIA_CSS: &[&str] = &[
    include_str!("design_css/media_playback.css"),
    include_str!("design_css/media_capture.css"),
];

const DESIGN_VISUALIZATION_CSS: &[&str] = &[
    include_str!("design_css/visualization_data.css"),
    include_str!("design_css/visualization_charts.css"),
    include_str!("design_css/visualization_canvas.css"),
    include_str!("design_css/visualization_diagram.css"),
];

const DESIGN_DISCLOSURE_CSS: &[&str] = &[
    include_str!("design_css/disclosure_accordion.css"),
    include_str!("design_css/disclosure_carousel_base.css"),
    include_str!("design_css/disclosure_carousel_variants.css"),
];

const DESIGN_FEEDBACK_CSS: &[&str] = &[
    include_str!("design_css/feedback_status.css"),
    include_str!("design_css/feedback_content.css"),
];

const DESIGN_NAVIGATION_CSS: &[&str] = &[
    include_str!("design_css/navigation_bars.css"),
    include_str!("design_css/navigation_menus.css"),
    include_str!("design_css/navigation_tabs.css"),
];

const DESIGN_OVERLAYS_CSS: &[&str] = &[
    include_str!("design_css/overlays_base.css"),
    include_str!("design_css/overlays_panels.css"),
];
