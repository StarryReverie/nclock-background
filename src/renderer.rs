use std::f64::consts::{FRAC_1_PI, FRAC_PI_2, PI};

use femtovg::{Canvas, Color, LineCap, Paint, Path, Renderer, Solidity};

use crate::app::AppConfig;
use crate::state::AppState;

struct RenderConfig {
    inner_radius: f64,
    lane_width: f64,
    lane_margin: f64,
}

impl RenderConfig {
    fn convert(config: &AppConfig, heigth: f64) -> Self {
        Self {
            inner_radius: config.inner_radius_frac * heigth,
            lane_width: config.lane_width_frac * heigth,
            lane_margin: config.lane_margin_frac * heigth,
        }
    }
}

pub fn render(
    canvas: &mut Canvas<impl Renderer>,
    (width, heigth): (f32, f32),
    config: &AppConfig,
    state: &AppState,
) {
    let config = RenderConfig::convert(&config, heigth as f64);
    let clock_center = (width / 2.0, heigth / 2.0);

    let angles = state.angles();

    for (lane_num, &angle) in angles.angles.iter().enumerate() {
        let mut path = Path::new();
        path.arc(
            clock_center.0,
            clock_center.1,
            calc_lane_radius(&config, lane_num as u8) as f32,
            (FRAC_PI_2 * 3.0) as f32,
            (FRAC_PI_2 * 3.0 + angle).rem_euclid(2.0 * PI) as f32,
            Solidity::Hole,
        );

        let paint = Paint::color(calc_rgb_from_angle(angle))
            .with_line_width(config.lane_width as f32)
            .with_line_cap(LineCap::Round);

        canvas.stroke_path(&path, &paint);
    }
}

fn calc_lane_radius(config: &RenderConfig, lane_num: u8) -> f64 {
    config.inner_radius + (config.lane_width + config.lane_margin) * (lane_num as f64)
}

fn calc_rgb_from_angle(angle: f64) -> Color {
    let hue = angle * 0.5 * FRAC_1_PI;
    Color::hsl(hue as f32, 0.75, 0.50)
}
