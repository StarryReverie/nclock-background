use std::num::NonZeroU32;

use femtovg::renderer::OpenGl;
use femtovg::{Canvas, FontId};
use glutin::context::PossiblyCurrentContext;
use glutin::surface::{Surface, WindowSurface};
use wayland_client::protocol::wl_output::WlOutput;
use wayland_client::protocol::wl_surface::WlSurface;
use wayland_protocols_wlr::layer_shell::v1::client::zwlr_layer_surface_v1::ZwlrLayerSurfaceV1;

pub struct Output {
    #[allow(unused)]
    pub output: WlOutput,
    pub surface: WlSurface,
    #[allow(unused)]
    pub layer_surface: ZwlrLayerSurfaceV1,
    pub scale_factor: f32,
    pub configured: Option<ConfiguredSurface>,
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
