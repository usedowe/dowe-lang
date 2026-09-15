#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CarouselGeometryContract {
    pub content_gap: u16,
    pub viewport_padding: u16,
    pub vertical_viewport_height: u16,
    pub slide_fraction_percent: u16,
    pub slide_max_width: Option<u16>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CarouselVisualContract {
    pub accent: ColorToken,
    pub title: ColorToken,
    pub content: ColorToken,
    /// Alpha used for the IconButton outline on every renderer.
    pub control_border_alpha: f32,
    pub indicator_inactive_alpha: f32,
}

impl CarouselVisualContract {
    pub fn for_scheme(scheme: ColorFamily) -> Self {
        Self {
            accent: scheme.color_token(),
            title: scheme.title_token(),
            content: scheme.text_token(),
            control_border_alpha: IconButtonVisualContract::standard().border_alpha,
            indicator_inactive_alpha: 0.26,
        }
    }
}

impl CarouselGeometryContract {
    pub const fn for_variant(variant: CarouselVariant) -> Self {
        let (slide_fraction_percent, slide_max_width) = match variant {
            CarouselVariant::Masonry => (72, Some(200)),
            CarouselVariant::Sticky => (88, Some(672)),
            CarouselVariant::Stories => (82, Some(384)),
            CarouselVariant::SmartStack => (80, Some(352)),
            CarouselVariant::CardStack => (84, Some(448)),
            CarouselVariant::Flipbook => (88, Some(480)),
            CarouselVariant::Simple
            | CarouselVariant::Snapping
            | CarouselVariant::Rtl
            | CarouselVariant::Controls
            | CarouselVariant::Dots
            | CarouselVariant::Thumbnails
            | CarouselVariant::CoverFlow
            | CarouselVariant::Slideshow => (100, None),
        };
        Self {
            content_gap: 12,
            viewport_padding: 0,
            vertical_viewport_height: 448,
            slide_fraction_percent,
            slide_max_width,
        }
    }

    pub fn slide_width(self, available: f64, explicit: Option<u16>, count: u16, gap: u16) -> f64 {
        if let Some(width) = explicit {
            return f64::from(width);
        }
        if !available.is_finite() || available <= 0.0 {
            return 0.0;
        }
        if let Some(maximum) = self.slide_max_width {
            return (available * f64::from(self.slide_fraction_percent) / 100.0)
                .min(f64::from(maximum));
        }
        let count = f64::from(count.max(1));
        ((available - f64::from(gap) * (count - 1.0)) / count).max(0.0)
    }
}

impl CarouselProps {
    pub fn geometry_contract(&self) -> CarouselGeometryContract {
        CarouselGeometryContract::for_variant(self.variant)
    }

    pub fn visual_contract(&self) -> CarouselVisualContract {
        CarouselVisualContract::for_scheme(
            self.style.color.unwrap_or(ColorFamily::Primary),
        )
    }
}
