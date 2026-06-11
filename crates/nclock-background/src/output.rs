use std::num::NonZeroU32;

use femtovg::renderer::OpenGl;
use femtovg::{Canvas, Color, FontId};
use glutin::context::PossiblyCurrentContext;
use glutin::surface::{GlSurface, Surface, WindowSurface};
use wayland_client::protocol::wl_output::WlOutput;
use wayland_client::protocol::wl_surface::WlSurface;
use wayland_protocols_wlr::layer_shell::v1::client::zwlr_layer_surface_v1::ZwlrLayerSurfaceV1;

use nclock_config::AnimationConfig;
use nclock_core::AppState;

pub struct Output {
    #[allow(unused)]
    pub output: WlOutput,
    pub surface: WlSurface,
    #[allow(unused)]
    pub layer_surface: ZwlrLayerSurfaceV1,
    pub scale_factor: f32,
    pub configured: Option<ConfiguredSurface>,
}

impl Output {
    pub fn render(&mut self, animation_config: &AnimationConfig, app_state: &AppState) {
        let configured = match &mut self.configured {
            Some(c) => c,
            None => return,
        };

        if configured.pending_resize {
            let (pw, ph) = configured.physical_size_nonzero(self.scale_factor);
            let scale = self.scale_factor as u32;
            if scale > 1 {
                self.surface.set_buffer_scale(scale as i32);
            }
            configured.surface.resize(&configured.context, pw, ph);
            configured.pending_resize = false;
        }

        let (pw, ph) = configured.physical_size(self.scale_factor);

        configured.canvas.set_size(pw, ph, self.scale_factor);
        configured.canvas.clear_rect(0, 0, pw, ph, Color::black());

        nclock_core::render(
            &mut configured.canvas,
            (pw as f32, ph as f32),
            animation_config,
            app_state,
            configured.font_id,
        );

        configured.canvas.flush();
        configured
            .surface
            .swap_buffers(&configured.context)
            .expect("could not swap buffers");
    }
}

pub struct ConfiguredSurface {
    pub context: PossiblyCurrentContext,
    pub surface: Surface<WindowSurface>,
    pub canvas: Canvas<OpenGl>,
    pub font_id: FontId,
    pub width: u32,
    pub height: u32,
    pub pending_resize: bool,
}

impl ConfiguredSurface {
    pub fn physical_size(&self, scale_factor: f32) -> (u32, u32) {
        let s = scale_factor as u32;
        (self.width * s, self.height * s)
    }

    pub fn physical_size_nonzero(&self, scale_factor: f32) -> (NonZeroU32, NonZeroU32) {
        let (pw, ph) = self.physical_size(scale_factor);
        (
            NonZeroU32::new(pw.max(1)).unwrap(),
            NonZeroU32::new(ph.max(1)).unwrap(),
        )
    }
}
