#[derive(Debug, Clone, Copy, PartialEq)]
pub struct InteractionStateVisualContract {
    pub disabled_alpha: f32,
    pub focus_ring_width: u16,
    pub focus_ring_alpha: f32,
    pub pressed_scale: f32,
    pub error_ring_width: u16,
}

impl InteractionStateVisualContract {
    pub const fn standard() -> Self { Self { disabled_alpha: 0.5, focus_ring_width: 2, focus_ring_alpha: 0.24, pressed_scale: 0.94, error_ring_width: 1 } }
}
