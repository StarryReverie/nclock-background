use std::f64::consts::{FRAC_1_PI, FRAC_PI_2, PI};

use femtovg::{Align, Baseline, Canvas, Color, FontId, LineCap, Paint, Path, Renderer, Solidity};

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
    font_id: FontId,
) {
    let config = RenderConfig::convert(&config, heigth as f64);
    let clock_center = (width / 2.0, heigth / 2.0);

    let angles = state.angles();
    let labels = state.labels();

    for (lane_num, (&angle, label)) in angles.angles.iter().zip(labels.labels.iter()).enumerate() {
        let radius = calc_lane_radius(&config, lane_num as u8);

        let mut path = Path::new();
        path.arc(
            clock_center.0,
            clock_center.1,
            radius as f32,
            (FRAC_PI_2 * 3.0) as f32,
            (FRAC_PI_2 * 3.0 + angle).rem_euclid(2.0 * PI) as f32,
            Solidity::Hole,
        );

        let paint = Paint::color(calc_rgb_from_angle(angle))
            .with_line_width(config.lane_width as f32)
            .with_line_cap(LineCap::Round);

        canvas.stroke_path(&path, &paint);

        let pointer_angle = FRAC_PI_2 * 3.0 + angle;
        render_text_on_lane(
            canvas,
            clock_center.0,
            clock_center.1,
            radius,
            pointer_angle,
            label,
            font_id,
            config.lane_width,
        );
    }
}

fn calc_lane_radius(config: &RenderConfig, lane_num: u8) -> f64 {
    config.inner_radius + (config.lane_width + config.lane_margin) * (lane_num as f64)
}

fn calc_rgb_from_angle(angle: f64) -> Color {
    let hue = angle * 0.5 * FRAC_1_PI;
    Color::hsl(hue as f32, 0.75, 0.50)
}

fn render_text_on_lane(
    canvas: &mut Canvas<impl Renderer>,
    cx: f32,
    cy: f32,
    radius: f64,
    pointer_angle: f64,
    text: &str,
    font_id: FontId,
    lane_width: f64,
) {
    let font_size = (lane_width * 0.4) as f32;
    let paint = Paint::color(Color::black())
        .with_font(&[font_id])
        .with_font_size(font_size)
        .with_text_align(Align::Center)
        .with_text_baseline(Baseline::Middle);

    let chars: Vec<char> = text.chars().collect();
    let mut offset = 0.0f64;
    let iter: Box<dyn DoubleEndedIterator<Item = &char>> = if pointer_angle.sin() <= 0.0 {
        Box::new(chars.iter().rev())
    } else {
        Box::new(chars.iter())
    };

    for ch in iter {
        let metrics = canvas
            .measure_text(0.0, 0.0, &ch.to_string(), &paint)
            .unwrap();
        let char_width = metrics.width() as f64;
        let delta = char_width / radius;
        offset += delta;
        let theta = pointer_angle - offset + delta / 2.0;

        let x = cx + (radius * theta.cos()) as f32;
        let y = cy + (radius * theta.sin()) as f32;

        let rotation = if pointer_angle.sin() > 0.0 {
            theta + PI * 1.5
        } else {
            theta + FRAC_PI_2
        };

        canvas.save_with(|canvas| {
            canvas.translate(x, y);
            canvas.rotate(rotation as f32);
            canvas.fill_text(0.0, 0.0, &ch.to_string(), &paint).ok();
        });
    }
}
