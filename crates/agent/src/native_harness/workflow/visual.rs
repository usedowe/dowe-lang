use crate::{AgentError, AgentResult};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PageArchitecturePlan {
    pub viewport: VisualViewport,
    pub layout: VisualNode,
    pub pages: Vec<VisualPage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VisualViewport {
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VisualPage {
    pub id: String,
    pub name: String,
    pub bounds: VisualBounds,
    #[serde(default)]
    pub sections: Vec<VisualSection>,
    #[serde(default)]
    pub responsive: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VisualSection {
    pub id: String,
    pub name: String,
    pub bounds: VisualBounds,
    #[serde(default)]
    pub components: Vec<VisualNode>,
    #[serde(default)]
    pub repeat: Option<String>,
    #[serde(default)]
    pub responsive: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VisualNode {
    pub id: String,
    pub kind: String,
    pub name: String,
    pub bounds: VisualBounds,
    #[serde(default)]
    pub children: Vec<VisualNode>,
    #[serde(default)]
    pub repeat: Option<String>,
    #[serde(default)]
    pub responsive: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VisualBounds {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

impl PageArchitecturePlan {
    pub fn validate(&self) -> AgentResult<()> {
        if !(1..=4096).contains(&self.viewport.width)
            || !(1..=4096).contains(&self.viewport.height)
            || self.pages.len() > 64
        {
            return Err(AgentError::new(
                "visual plan viewport or page count is out of bounds",
            ));
        }
        validate_node(&self.layout, "layout", &self.viewport)?;
        for page in &self.pages {
            bounded_id(&page.id)?;
            bounded_text(&page.name, 256)?;
            validate_bounds(&page.bounds, &self.viewport)?;
            validate_responsive(&page.responsive)?;
            if page.sections.len() > 256 || page.responsive.len() > 32 {
                return Err(AgentError::new(
                    "visual page sections or responsive rules are unbounded",
                ));
            }
            for section in &page.sections {
                bounded_id(&section.id)?;
                bounded_text(&section.name, 256)?;
                validate_bounds(&section.bounds, &self.viewport)?;
                validate_responsive(&section.responsive)?;
                if section.components.len() > 256 || section.responsive.len() > 32 {
                    return Err(AgentError::new(
                        "visual section components or responsive rules are unbounded",
                    ));
                }
                if let Some(repeat) = &section.repeat {
                    validate_repeat(repeat)?;
                }
                for component in &section.components {
                    validate_node(component, "component", &self.viewport)?;
                }
            }
        }
        Ok(())
    }
}

/// Builds a conservative architecture from pixels when a screenshot is
/// available. This is structural evidence, not OCR: labels and intent remain
/// the planner's responsibility. Horizontal color bands become Sections and
/// repeated bands are represented with an explicit `each` template.
pub(crate) fn infer_page_architecture(
    width: u32,
    height: u32,
    rgba: &[u8],
) -> AgentResult<PageArchitecturePlan> {
    let pixel_count = usize::try_from(width)
        .ok()
        .and_then(|width| usize::try_from(height).ok().map(|height| width * height))
        .ok_or_else(|| AgentError::new("visual pixels exceed architecture limits"))?;
    if !(1..=4096).contains(&width)
        || !(1..=4096).contains(&height)
        || rgba.len() != pixel_count.saturating_mul(4)
    {
        return Err(AgentError::new(
            "visual pixels do not match the reference viewport",
        ));
    }
    let row_signatures = (0..height as usize)
        .map(|row| {
            let start = row * width as usize * 4;
            let pixels = &rgba[start..start + width as usize * 4];
            let (red, green, blue) = pixels.chunks_exact(4).fold((0u64, 0u64, 0u64), |sum, px| {
                (
                    sum.0 + u64::from(px[0]),
                    sum.1 + u64::from(px[1]),
                    sum.2 + u64::from(px[2]),
                )
            });
            let count = u64::from(width.max(1));
            [red / count, green / count, blue / count]
        })
        .collect::<Vec<_>>();
    let mut bands = Vec::new();
    let mut start = 0usize;
    for row in 1..=row_signatures.len() {
        let changed = row == row_signatures.len()
            || row_signatures[row]
                .iter()
                .zip(row_signatures[row - 1].iter())
                .map(|(left, right)| left.abs_diff(*right))
                .sum::<u64>()
                > 72;
        if changed && row.saturating_sub(start) >= 4 {
            bands.push((start, row));
            start = row;
        }
    }
    if bands.is_empty() {
        bands.push((0, height as usize));
    }
    let repeated_height = bands
        .iter()
        .map(|(start, end)| end - start)
        .find(|size| *size >= 4 && bands.iter().filter(|(a, b)| b - a == *size).count() >= 3);
    let sections = bands
        .iter()
        .enumerate()
        .map(|(index, (top, bottom))| VisualSection {
            id: format!("section-{index}"),
            name: format!("Visual band {}", index + 1),
            bounds: VisualBounds {
                x: 0,
                y: *top as u32,
                width,
                height: bottom.saturating_sub(*top) as u32,
            },
            components: infer_components(width, *top, *bottom, rgba, index),
            repeat: repeated_height
                .filter(|size| bottom - top == *size)
                .map(|_| "each in:repeated_visual_regions as:item key:item.index".into()),
            responsive: vec!["xs:single-column".into(), "md:reference-width".into()],
        })
        .collect();
    let plan = PageArchitecturePlan {
        viewport: VisualViewport { width, height },
        layout: VisualNode {
            id: "detected-layout".into(),
            kind: "layout".into(),
            name: "Screenshot-derived layout".into(),
            bounds: VisualBounds {
                x: 0,
                y: 0,
                width,
                height,
            },
            children: Vec::new(),
            repeat: None,
            responsive: vec!["xs:fluid".into(), "md:reference-viewport".into()],
        },
        pages: vec![VisualPage {
            id: "detected-page".into(),
            name: "Screenshot-derived page".into(),
            bounds: VisualBounds {
                x: 0,
                y: 0,
                width,
                height,
            },
            sections,
            responsive: vec!["xs:single-column".into(), "md:reference-layout".into()],
        }],
    };
    plan.validate()?;
    Ok(plan)
}

fn validate_node(node: &VisualNode, expected: &str, viewport: &VisualViewport) -> AgentResult<()> {
    bounded_id(&node.id)?;
    bounded_text(&node.kind, 64)?;
    bounded_text(&node.name, 256)?;
    if node.kind != expected {
        return Err(AgentError::new(format!(
            "visual node {} must be a {expected}",
            node.id
        )));
    }
    validate_bounds(&node.bounds, viewport)?;
    validate_responsive(&node.responsive)?;
    if node.children.len() > 256 || node.responsive.len() > 32 {
        return Err(AgentError::new(
            "visual node children or responsive rules are unbounded",
        ));
    }
    if let Some(repeat) = &node.repeat {
        validate_repeat(repeat)?;
    }
    for child in &node.children {
        validate_node(child, "component", viewport)?;
    }
    Ok(())
}

fn infer_components(
    width: u32,
    top: usize,
    bottom: usize,
    rgba: &[u8],
    section_index: usize,
) -> Vec<VisualNode> {
    let height = bottom.saturating_sub(top).max(1);
    let mut columns = Vec::new();
    let mut start = 0usize;
    let row_stride = width as usize * 4;
    for column in 1..=width as usize {
        let changed = column == width as usize
            || (top..bottom).any(|row| {
                let left =
                    &rgba[row * row_stride + (column - 1) * 4..row * row_stride + column * 4];
                let right_start = row * row_stride + column * 4;
                let right = &rgba[right_start..right_start + 4];
                left.iter()
                    .zip(right)
                    .map(|(a, b)| a.abs_diff(*b) as u32)
                    .sum::<u32>()
                    > 96
            });
        if changed && column.saturating_sub(start) >= 4 {
            columns.push((start, column));
            start = column;
        }
    }
    if columns.is_empty() {
        columns.push((0, width as usize));
    }
    if columns.len() > 256 {
        columns.clear();
        columns.push((0, width as usize));
    }
    columns
        .into_iter()
        .enumerate()
        .map(|(index, (left, right))| VisualNode {
            id: format!("section-{section_index}-component-{index}"),
            kind: "component".into(),
            name: format!("Detected visual component {}", index + 1),
            bounds: VisualBounds {
                x: left as u32,
                y: top as u32,
                width: right.saturating_sub(left) as u32,
                height: height as u32,
            },
            children: Vec::new(),
            repeat: None,
            responsive: vec!["xs:stack".into(), "md:preserve-reference-axis".into()],
        })
        .collect()
}

fn validate_repeat(value: &str) -> AgentResult<()> {
    if value.len() > 256 || !value.contains("each") {
        return Err(AgentError::new(
            "visual repetition must describe an each template",
        ));
    }
    Ok(())
}

fn validate_responsive(values: &[String]) -> AgentResult<()> {
    if values.len() > 32 {
        return Err(AgentError::new("visual responsive rules are unbounded"));
    }
    for value in values {
        let (breakpoint, rule) = value
            .split_once(':')
            .ok_or_else(|| AgentError::new("visual responsive rules require breakpoint:rule"))?;
        if !matches!(breakpoint, "xs" | "sm" | "md" | "lg" | "xl")
            || rule.trim().is_empty()
            || rule.len() > 192
            || rule.chars().any(char::is_control)
        {
            return Err(AgentError::new("visual responsive rule is invalid"));
        }
    }
    Ok(())
}

fn validate_bounds(bounds: &VisualBounds, viewport: &VisualViewport) -> AgentResult<()> {
    if bounds.width == 0
        || bounds.height == 0
        || bounds.x.saturating_add(bounds.width) > viewport.width
        || bounds.y.saturating_add(bounds.height) > viewport.height
    {
        return Err(AgentError::new(
            "visual bounds exceed the reference viewport",
        ));
    }
    Ok(())
}

fn bounded_id(value: &str) -> AgentResult<()> {
    if value.is_empty()
        || value.len() > 128
        || !value
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || "-_:.".contains(c))
    {
        return Err(AgentError::new("visual id is invalid or out of bounds"));
    }
    Ok(())
}

fn bounded_text(value: &str, max: usize) -> AgentResult<()> {
    if value.trim().is_empty() || value.len() > max || value.chars().any(char::is_control) {
        return Err(AgentError::new("visual text is invalid or out of bounds"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn node(kind: &str) -> VisualNode {
        VisualNode {
            id: "hero".into(),
            kind: kind.into(),
            name: "Hero component".into(),
            bounds: VisualBounds {
                x: 0,
                y: 0,
                width: 800,
                height: 300,
            },
            children: Vec::new(),
            repeat: None,
            responsive: vec!["xs:stack".into(), "md:split".into()],
        }
    }

    #[test]
    fn page_architecture_plan_validates_hierarchy_bounds_and_each_templates() {
        let plan = PageArchitecturePlan {
            viewport: VisualViewport {
                width: 800,
                height: 600,
            },
            layout: VisualNode {
                kind: "layout".into(),
                ..node("layout")
            },
            pages: vec![VisualPage {
                id: "home".into(),
                name: "Home".into(),
                bounds: VisualBounds {
                    x: 0,
                    y: 0,
                    width: 800,
                    height: 600,
                },
                responsive: vec!["xs:single-column".into()],
                sections: vec![VisualSection {
                    id: "features".into(),
                    name: "Features".into(),
                    bounds: VisualBounds {
                        x: 0,
                        y: 300,
                        width: 800,
                        height: 300,
                    },
                    repeat: Some("each in:features as:feature key:feature.id".into()),
                    responsive: vec!["xs:one-column".into()],
                    components: vec![node("component")],
                }],
            }],
        };
        assert!(plan.validate().is_ok());
    }

    #[test]
    fn page_architecture_plan_rejects_out_of_view_bounds() {
        let plan = PageArchitecturePlan {
            viewport: VisualViewport {
                width: 100,
                height: 100,
            },
            layout: VisualNode {
                kind: "layout".into(),
                ..node("layout")
            },
            pages: Vec::new(),
        };
        assert!(plan.validate().is_err());
    }

    #[test]
    fn pixel_inference_creates_bounded_sections_and_repetition() {
        let mut pixels = vec![255; 20 * 40 * 4];
        for band in [4..12, 16..24, 28..36] {
            for row in band {
                for pixel in pixels[(row * 20 * 4)..((row + 1) * 20 * 4)].chunks_exact_mut(4) {
                    pixel.copy_from_slice(&[20, 40, 80, 255]);
                }
            }
        }
        let plan = infer_page_architecture(20, 40, &pixels).unwrap();
        assert!(plan.pages[0].sections.len() >= 3);
        assert!(
            plan.pages[0]
                .sections
                .iter()
                .any(|section| section.repeat.is_some())
        );
    }
}
