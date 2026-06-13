use std::time::Duration;

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub animation: AnimationConfig,
    pub layer: LayerConfig,
    pub ipc: IpcConfig,
}

#[derive(Debug, Clone)]
pub struct AnimationConfig {
    pub relative_inner_radius: f64,
    pub relative_lane_width: f64,
    pub relative_lane_margin: f64,
    pub hue_start: f32,
    pub hue_end: f32,
}

#[derive(Debug, Clone)]
pub struct LayerConfig {
    pub layer: Layer,
    pub namespace: String,
    pub exit_on_input: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Layer {
    Background,
    Bottom,
    Top,
    Overlay,
}

#[derive(Debug, Clone)]
pub struct IpcConfig {
    pub exit_delay: Duration,
    pub notify_finalization: bool,
}
