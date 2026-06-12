use femtovg::renderer::OpenGl;
use glutin::context::PossiblyCurrentContext;
use glutin::surface::{GlSurface, Surface, WindowSurface};
use wayland_client::protocol::wl_output::WlOutput;
use wayland_client::protocol::wl_surface::WlSurface;
use wayland_protocols_wlr::layer_shell::v1::client::zwlr_layer_surface_v1::ZwlrLayerSurfaceV1;

use nclock_config::AnimationConfig;
use nclock_core::{AppCanvas, AppState, Size};

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
            let size = configured.canvas.physical_size_nonzero(self.scale_factor);
            let scale_factor = self.scale_factor as u32;
            if scale_factor > 1 {
                self.surface.set_buffer_scale(scale_factor as i32);
            }
            configured
                .surface
                .resize(&configured.context, size.width, size.height);
            configured.canvas.reapply_size(self.scale_factor);
            configured.pending_resize = false;
        }

        configured.canvas.render(animation_config, app_state);
        configured
            .surface
            .swap_buffers(&configured.context)
            .expect("could not swap buffers");
    }
}

pub struct ConfiguredSurface {
    pub context: PossiblyCurrentContext,
    pub surface: Surface<WindowSurface>,
    pub canvas: AppCanvas<OpenGl>,
    pub pending_resize: bool,
}

impl ConfiguredSurface {
    pub fn update_size(&mut self, width: u32, height: u32) {
        self.canvas.update_size(Size::new(width, height));
        self.pending_resize = true;
    }
}
