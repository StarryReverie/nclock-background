mod glyph_cache;
mod render;
mod size;

pub use size::Size;

use std::num::NonZeroU32;

use femtovg::renderer::SurfacelessRenderer;
use femtovg::{Canvas, Color, FontId, Renderer};

use nclock_config::AnimationConfig;

use crate::canvas::glyph_cache::GlyphCache;
use crate::canvas::render::{RenderConfig, render_clock, render_footer};
use crate::state::AppState;

pub struct AppCanvas<R: Renderer> {
    canvas: Canvas<R>,
    font_id: FontId,
    logical_size: Size<u32>,
    physical_size: Size<u32>,
    glyph_cache: GlyphCache,
}

impl<R: Renderer> AppCanvas<R> {
    pub fn new(
        mut canvas: Canvas<R>,
        font_id: FontId,
        logical_size: Size<u32>,
        scale_factor: f32,
    ) -> Self {
        let physical_size = Size::new(
            logical_size.width * scale_factor as u32,
            logical_size.height * scale_factor as u32,
        );
        canvas.set_size(physical_size.width, physical_size.height, scale_factor);
        Self {
            canvas,
            font_id,
            logical_size,
            physical_size,
            glyph_cache: GlyphCache::new(),
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
    pub fn render(&mut self, animation_config: &AnimationConfig, state: &AppState) {
        let pw = self.physical_size.width;
        let ph = self.physical_size.height;
        let render_config = RenderConfig::convert(animation_config, ph as f64);

        let lane_font_size = (animation_config.relative_lane_width * ph as f64 * 0.4) as f32;
        let labels = state.labels();
        let all_chars: Vec<char> = labels.labels.iter().flat_map(|s| s.chars()).collect();
        let screen_scale = self.physical_size.width as f32 / self.logical_size.width as f32;
        self.glyph_cache.ensure(
            &mut self.canvas,
            self.font_id,
            lane_font_size,
            &all_chars,
            screen_scale,
        );

        self.canvas.set_size(pw, ph, screen_scale);
        self.canvas.clear_rect(0, 0, pw, ph, Color::black());

        render_clock(
            &mut self.canvas,
            self.physical_size.convert_with(|x| x as f32),
            state,
            render_config,
            &mut self.glyph_cache,
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
