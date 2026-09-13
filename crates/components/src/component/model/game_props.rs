#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameProps {
    pub style: StyleProps,
    pub renderer: GameRenderer,
    pub scene: Option<String>,
    pub world: Option<String>,
    pub camera: Option<String>,
    pub controls: GameControls,
    pub move_speed: u16,
    pub turn_speed: u16,
    pub view_width: u16,
    pub view_height: u16,
    pub fit: CanvasFit,
    pub fps: u8,
    pub autoplay: bool,
    pub background: CanvasBackground,
    pub pixelated: bool,
    pub label: String,
    pub on_pointer: Option<String>,
    pub on_key: Option<String>,
    pub on_fire: Option<String>,
    pub on_motion: Option<String>,
    pub motion_rate: u8,
    pub socket: Option<String>,
    pub socket_binding: bool,
    pub send: Option<String>,
    pub status: Option<String>,
    pub on_open: Option<String>,
    pub on_message: Option<String>,
    pub on_close: Option<String>,
    pub on_error: Option<String>,
    pub reconnect: bool,
    pub reconnect_delay: u16,
}

impl GameProps {
    pub fn canvas_props(&self) -> CanvasProps {
        CanvasProps {
            style: self.style.clone(),
            is_draw: false,
            scene: self.scene.clone().unwrap_or_default(),
            layer_bind: None,
            selected_layer: None,
            view_width: self.view_width,
            view_height: self.view_height,
            fit: self.fit,
            fps: self.fps,
            autoplay: self.autoplay,
            background: self.background,
            pixelated: self.pixelated,
            label: self.label.clone(),
            on_pointer: self.on_pointer.clone(),
            on_key: self.on_key.clone(),
            on_motion: self.on_motion.clone(),
            motion_rate: self.motion_rate,
            draw: false,
            draw_mode: DrawMode::Pen.as_str().to_string(),
            draw_mode_binding: false,
            on_layer_add: None,
            on_layer_change: None,
            on_layer_remove: None,
            on_layer_select: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameRenderer {
    Canvas2d,
    Raycast3d,
}

impl GameRenderer {
    pub fn from_name(value: &str) -> Option<Self> {
        match value {
            "canvas2d" => Some(Self::Canvas2d),
            "raycast3d" => Some(Self::Raycast3d),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Canvas2d => "canvas2d",
            Self::Raycast3d => "raycast3d",
        }
    }

    pub fn all() -> &'static [Self] {
        &[Self::Canvas2d, Self::Raycast3d]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameControls {
    None,
    Doom,
}

impl GameControls {
    pub fn from_name(value: &str) -> Option<Self> {
        match value {
            "none" => Some(Self::None),
            "doom" => Some(Self::Doom),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Doom => "doom",
        }
    }

    pub fn all() -> &'static [Self] {
        &[Self::None, Self::Doom]
    }
}
