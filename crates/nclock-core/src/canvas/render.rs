use std::f64::consts::{FRAC_1_PI, FRAC_PI_2, PI};
use std::time::Duration;

use femtovg::{Align, Baseline, Canvas, Color, FontId, LineCap, Paint, Path, Renderer, Solidity};

use nclock_config::AnimationConfig;

use crate::canvas::Size;
use crate::canvas::glyph_cache::GlyphCache;
use crate::state::{AppState, INTRO_ANIMATION_WAIT_DURATION};

pub struct RenderConfig {
    inner_radius: f64,
    lane_width: f64,
    lane_margin: f64,
}

impl RenderConfig {
    pub fn convert(config: &AnimationConfig, heigth: f64) -> Self {
        Self {
            inner_radius: config.relative_inner_radius * heigth,
            lane_width: config.relative_lane_width * heigth,
            lane_margin: config.relative_lane_margin * heigth,
        }
    }
}

pub fn render_clock(
    canvas: &mut Canvas<impl Renderer>,
    size: Size<f32>,
    state: &AppState,
    config: RenderConfig,
    glyph_cache: &mut GlyphCache,
) {
    if state.initial_instant().elapsed() < Duration::from_secs_f64(INTRO_ANIMATION_WAIT_DURATION) {
        return;
    }

    let clock_center = (size.width / 2.0, size.height / 2.0);
    let min_lane_length = config.lane_width * 2.0;

    let angles = state.angles();
    let labels = state.labels();

    for (lane_num, (&angle, label)) in angles.angles.iter().zip(labels.labels.iter()).enumerate() {
        let radius = calc_lane_radius(&config, lane_num as u8);
        let angle = angle.max(min_lane_length / radius);

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
            clock_center,
            radius,
            pointer_angle,
            label,
            glyph_cache,
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
    clock_center: (f32, f32),
    radius: f64,
    pointer_angle: f64,
    text: &str,
    glyph_cache: &GlyphCache,
) {
    let chars: Vec<char> = text.chars().collect();
    let mut offset = 0.0f64;
    let iter: Box<dyn DoubleEndedIterator<Item = &char>> = if pointer_angle.sin() <= 0.0 {
        Box::new(chars.iter().rev())
    } else {
        Box::new(chars.iter())
    };

    for ch in iter {
        let slot = match glyph_cache.get(*ch) {
            Some(s) => s,
            None => continue,
        };
        let delta = slot.width / radius;
        offset += delta;
        let theta = pointer_angle - offset + delta / 2.0;

        let x = clock_center.0 + (radius * theta.cos()) as f32;
        let y = clock_center.1 + (radius * theta.sin()) as f32;

        let rotation = if pointer_angle.sin() > 0.0 {
            theta + PI * 1.5
        } else {
            theta + FRAC_PI_2
        };

        let half_w = slot.slot_w / 2.0;
        let half_h = slot.slot_h / 2.0;
        let paint = Paint::image_tint(
            slot.image_id,
            -half_w,
            -half_h,
            slot.slot_w,
            slot.slot_h,
            0.0,
            Color::black(),
        );
        canvas.save_with(|canvas| {
            canvas.translate(x, y);
            canvas.rotate(rotation as f32);
            let mut path = Path::new();
            path.rect(-half_w, -half_h, slot.slot_w, slot.slot_h);
            canvas.fill_path(&path, &paint);
        });
    }
}

pub fn render_footer(
    canvas: &mut Canvas<impl Renderer>,
    size: Size<f32>,
    state: &AppState,
    font_id: FontId,
) {
    let font_size = size.height * 0.015;
    let paint = Paint::color(Color::rgb(160, 160, 160))
        .with_font(&[font_id])
        .with_font_size(font_size)
        .with_text_align(Align::Center)
        .with_text_baseline(Baseline::Bottom);

    let text = state.footer_text();
    let y = size.height - size.height * 0.02;
    canvas.fill_text(size.width / 2.0, y, &text, &paint).ok();
}
