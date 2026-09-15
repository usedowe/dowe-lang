#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PaginationVisualContract {
    pub accent: ColorToken,
    pub inactive_alpha: f32,
    pub disabled_alpha: f32,
    pub control_border_alpha: f32,
}

impl PaginationVisualContract {
    pub fn for_scheme(scheme: ColorFamily) -> Self {
        Self {
            accent: scheme.color_token(),
            inactive_alpha: 0.28,
            disabled_alpha: IconButtonVisualContract::standard().disabled_alpha,
            control_border_alpha: IconButtonVisualContract::standard().border_alpha,
        }
    }
}

impl ToggleGroupProps {
    pub fn pagination_visual_contract(&self) -> PaginationVisualContract {
        PaginationVisualContract::for_scheme(self.style.color.unwrap_or(ColorFamily::Primary))
    }
}
