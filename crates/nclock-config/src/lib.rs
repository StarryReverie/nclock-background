#[derive(Debug, Clone)]
pub struct AppConfig {
    pub animation: AnimationConfig,
}

#[derive(Debug, Clone)]
pub struct AnimationConfig {
    pub relative_inner_radius: f64,
    pub relative_lane_width: f64,
    pub relative_lane_margin: f64,
}
