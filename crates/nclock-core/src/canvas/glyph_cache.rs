use std::collections::HashMap;

use femtovg::{
    Align, Baseline, Canvas, Color, FontId, ImageFlags, ImageId, Paint, PixelFormat, RenderTarget,
    Renderer,
};

#[derive(Clone, Copy)]
pub struct GlyphSlot {
    pub image_id: ImageId,
    pub width: f64,
    pub slot_w: f32,
    pub slot_h: f32,
}

pub struct GlyphCache {
    font_id: Option<FontId>,
    font_size: f32,
    glyphs: HashMap<char, GlyphSlot>,
}

impl GlyphCache {
    pub fn new() -> Self {
        Self {
            font_id: None,
            font_size: 0.0,
            glyphs: HashMap::new(),
        }
    }

    pub fn ensure<R: Renderer>(
        &mut self,
        canvas: &mut Canvas<R>,
        font_id: FontId,
        font_size: f32,
        chars: &[char],
        screen_dpi: f32,
    ) {
        let changed =
            (self.font_size - font_size).abs() > f32::EPSILON || self.font_id != Some(font_id);
        if changed {
            for glyph in self.glyphs.values_mut() {
                canvas.delete_image(glyph.image_id);
            }
            self.glyphs.clear();
            self.font_id = Some(font_id);
            self.font_size = font_size;
        }

        let measure_paint = Paint::color(Color::black())
            .with_font(&[font_id])
            .with_font_size(font_size);
        let render_paint = Paint::color(Color::white())
            .with_font(&[font_id])
            .with_font_size(font_size)
            .with_text_align(Align::Center)
            .with_text_baseline(Baseline::Middle);

        for &ch in chars {
            if self.glyphs.contains_key(&ch) {
                continue;
            }

            let metrics = canvas
                .measure_text(0.0, 0.0, ch.to_string(), &measure_paint)
                .expect("could not measure character");

            let char_width = metrics.width() as f64;

            let slot_w = if let Some(g) = metrics.glyphs.first() {
                (g.width.ceil() as u32 + 12).max(32)
            } else {
                (char_width.ceil() as u32 + 12).max(32)
            };
            let slot_h = if let Some(g) = metrics.glyphs.first() {
                (g.height.ceil() as u32 + 12).max(32)
            } else {
                ((font_size + 4.0).ceil() as u32 + 12).max(32)
            };

            let image_id = canvas
                .create_image_empty(
                    slot_w as usize,
                    slot_h as usize,
                    PixelFormat::Rgba8,
                    ImageFlags::PREMULTIPLIED | ImageFlags::FLIP_Y,
                )
                .expect("could not create glyph image");

            canvas.set_size(slot_w, slot_h, screen_dpi);
            canvas.set_render_target(RenderTarget::Image(image_id));
            canvas.clear_rect(0, 0, slot_w, slot_h, Color::rgba(0, 0, 0, 0));

            let slot_cx = slot_w as f32 / 2.0;
            let slot_cy = slot_h as f32 / 2.0;
            canvas
                .fill_text(slot_cx, slot_cy, ch.to_string(), &render_paint)
                .ok();

            canvas.set_render_target(RenderTarget::Screen);

            self.glyphs.insert(
                ch,
                GlyphSlot {
                    image_id,
                    width: char_width,
                    slot_w: slot_w as f32,
                    slot_h: slot_h as f32,
                },
            );
        }
    }

    pub fn get(&self, ch: char) -> Option<&GlyphSlot> {
        self.glyphs.get(&ch)
    }
}
