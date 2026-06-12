use std::f64::consts::{FRAC_1_PI, FRAC_PI_2, PI};
use std::num::NonZeroU32;
use std::time::Duration;

use femtovg::renderer::SurfacelessRenderer;
use femtovg::{Align, Baseline, Canvas, Color, FontId, LineCap, Paint, Path, Renderer, Solidity};

use nclock_config::AnimationConfig;

use crate::state::{AppState, INTRO_ANIMATION_WAIT_DURATION};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Size<T> {
    pub width: T,
    pub height: T,
}

impl<T> Size<T> {
    pub fn new(width: T, height: T) -> Self {
        Self { width, height }
    }

    pub fn convert_with<F, U>(self, conv: F) -> Size<U>
    where
        F: Fn(T) -> U,
    {
        Size::new(conv(self.width), conv(self.height))
    }
}

pub struct AppCanvas<R: Renderer> {
    canvas: Canvas<R>,
    font_id: FontId,
    logical_size: Size<u32>,
    physical_size: Size<u32>,
}

impl<R: Renderer> AppCanvas<R> {
    pub fn new(
        mut canvas: Canvas<R>,
        font_id: FontId,
        logical_size: Size<u32>,
        scale_factor: f32,
    ) -> Self {
        let physical_width = logical_size.width * scale_factor as u32;
        let physical_height = logical_size.height * scale_factor as u32;
        canvas.set_size(physical_width, physical_height, scale_factor);
        Self {
            canvas,
            font_id,
            logical_size,
            physical_size: Size::new(physical_width, physical_height),
        }
    }

    pub fn update_size(&mut self, size: Size<u32>) {
        self.logical_size = size;
    }

    pub fn physical_size(&self, scale_factor: f32) -> Size<u32> {
        let s = scale_factor as u32;
        Size::new(self.logical_size.width * s, self.logical_size.height * s)
    }

    pub fn physical_size_nonzero(&self, scale_factor: f32) -> Size<NonZeroU32> {
        let size = self.physical_size(scale_factor);
        Size::new(
            NonZeroU32::new(size.width.max(1)).unwrap(),
            NonZeroU32::new(size.height.max(1)).unwrap(),
        )
    }

    pub fn reapply_size(&mut self, scale_factor: f32) {
        self.physical_size.width = self.logical_size.width * scale_factor as u32;
        self.physical_size.height = self.logical_size.height * scale_factor as u32;
        self.canvas.set_size(
            self.physical_size.width,
            self.physical_size.height,
            scale_factor,
        );
    }
}

impl<R: SurfacelessRenderer> AppCanvas<R> {
    pub fn render(&mut self, config: &AnimationConfig, state: &AppState) {
        let config = RenderConfig::convert(config, self.physical_size.height as f64);
        self.canvas.clear_rect(
            0,
            0,
            self.physical_size.width,
            self.physical_size.height,
            Color::black(),
        );
        render_clock(
            &mut self.canvas,
            self.physical_size.convert_with(|x| x as f32),
            state,
            self.font_id,
            config,
        );
        render_footer(
            &mut self.canvas,
            self.physical_size.convert_with(|x| x as f32),
            state,
            self.font_id,
        );
        self.canvas.flush();
    }
}

struct RenderConfig {
    inner_radius: f64,
    lane_width: f64,
    lane_margin: f64,
}

impl RenderConfig {
    fn convert(config: &AnimationConfig, heigth: f64) -> Self {
        Self {
            inner_radius: config.relative_inner_radius * heigth,
            lane_width: config.relative_lane_width * heigth,
            lane_margin: config.relative_lane_margin * heigth,
        }
    }
}

fn render_clock(
    canvas: &mut Canvas<impl Renderer>,
    size: Size<f32>,
    state: &AppState,
    font_id: FontId,
    config: RenderConfig,
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
    clock_center: (f32, f32),
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
            .measure_text(0.0, 0.0, ch.to_string(), &paint)
            .unwrap();
        let char_width = metrics.width() as f64;
        let delta = char_width / radius;
        offset += delta;
        let theta = pointer_angle - offset + delta / 2.0;

        let x = clock_center.0 + (radius * theta.cos()) as f32;
        let y = clock_center.1 + (radius * theta.sin()) as f32;

        let rotation = if pointer_angle.sin() > 0.0 {
            theta + PI * 1.5
        } else {
            theta + FRAC_PI_2
        };

        canvas.save_with(|canvas| {
            canvas.translate(x, y);
            canvas.rotate(rotation as f32);
            canvas.fill_text(0.0, 0.0, ch.to_string(), &paint).ok();
        });
    }
}

fn render_footer(
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
