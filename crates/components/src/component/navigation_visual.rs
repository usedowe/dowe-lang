#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TabsVisualContract {
    pub list_gap: u16,
    pub tab_horizontal_padding: u16,
    pub tab_vertical_padding: u16,
    pub indicator_thickness: u16,
}

impl TabsVisualContract {
    pub const fn standard() -> Self { Self { list_gap: 4, tab_horizontal_padding: 12, tab_vertical_padding: 8, indicator_thickness: 2 } }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StepperVisualContract {
    pub indicator_size: u16,
    pub connector_thickness: u16,
    pub item_gap: u16,
}

impl StepperVisualContract {
    pub const fn standard() -> Self { Self { indicator_size: 32, connector_thickness: 2, item_gap: 12 } }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AccordionVisualContract {
    pub header_min_height: u16,
    pub header_padding: u16,
    pub content_padding: u16,
    pub item_gap: u16,
}

impl AccordionVisualContract {
    pub const fn standard() -> Self { Self { header_min_height: 44, header_padding: 16, content_padding: 16, item_gap: 0 } }
}
