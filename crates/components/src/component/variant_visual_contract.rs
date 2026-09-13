#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VariantVisualRoles {
    pub background: ColorToken,
    pub content: ColorToken,
    pub title: ColorToken,
    pub border: Option<ColorToken>,
}

pub fn variant_visual_roles(
    variant: ComponentVariant,
    scheme: ColorFamily,
) -> VariantVisualRoles {
    let color = scheme.color_token();
    match variant {
        ComponentVariant::Solid => VariantVisualRoles {
            background: color,
            content: scheme.text_token(),
            title: scheme.title_token(),
            border: None,
        },
        ComponentVariant::Outlined => VariantVisualRoles {
            background: ColorToken::Transparent,
            content: color,
            title: color,
            border: Some(color),
        },
        ComponentVariant::Line | ComponentVariant::Ghost => {
            let (content, title) = match scheme {
                ColorFamily::Background | ColorFamily::Surface => {
                    (scheme.text_token(), scheme.title_token())
                }
                _ => (color, color),
            };
            VariantVisualRoles {
                background: ColorToken::Transparent,
                content,
                title,
                border: None,
            }
        }
    }
}

pub fn card_variant_visual_roles(
    variant: ComponentVariant,
    scheme: ColorFamily,
) -> VariantVisualRoles {
    match variant {
        ComponentVariant::Solid => variant_visual_roles(variant, scheme),
        ComponentVariant::Outlined => {
            let (background, title) = if scheme == ColorFamily::Background {
                (
                    ColorToken::Background,
                    ColorToken::BackgroundTitle,
                )
            } else {
                (
                    ColorToken::Surface,
                    ColorToken::SurfaceTitle,
                )
            };
            VariantVisualRoles {
                background,
                content: scheme.color_token(),
                title,
                border: Some(scheme.color_token()),
            }
        }
        ComponentVariant::Line | ComponentVariant::Ghost => {
            variant_visual_roles(variant, scheme)
        }
    }
}
