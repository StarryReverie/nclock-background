use std::ffi::c_void;
use std::num::NonZeroU32;
use std::ptr::NonNull;

use femtovg::renderer::OpenGl;
use femtovg::{Canvas, Color};
use glutin::config::{Config as GlConfig, ConfigTemplateBuilder};
use glutin::context::{ContextAttributesBuilder, NotCurrentGlContext as _};
use glutin::display::{Display as GlDisplay, DisplayApiPreference, GlDisplay as _};
use glutin::surface::{GlSurface as _, SurfaceAttributesBuilder, WindowSurface};
use raw_window_handle::{
    RawDisplayHandle, RawWindowHandle, WaylandDisplayHandle, WaylandWindowHandle,
};
use wayland_client::{Connection, Proxy};

use nclock_config::AnimationConfig;
use nclock_core::AppState;

use crate::output::{ConfiguredSurface, Output};

pub struct OpenGlContext {
    display: GlDisplay,
    config: GlConfig,
}

impl OpenGlContext {
    pub fn create(connection: &Connection) -> OpenGlContext {
        let display_ptr = connection.backend().display_ptr();
        let raw_display_handle = {
            let handle = WaylandDisplayHandle::new(
                NonNull::new(display_ptr as *mut c_void).expect("display ptr is null"),
            );
            RawDisplayHandle::Wayland(handle)
        };
        let display = unsafe {
            GlDisplay::new(raw_display_handle, DisplayApiPreference::Egl)
                .expect("could not create EGL display")
        };
        let config = unsafe {
            display
                .find_configs(ConfigTemplateBuilder::new().build())
                .expect("could not find EGL configs")
                .next()
                .expect("no suitable EGL config")
        };

        OpenGlContext { display, config }
    }

    pub fn init_for_output(&self, output: &Output, width: u32, height: u32) -> ConfiguredSurface {
        let scale_factor = output.scale_factor as u32;
        if scale_factor > 1 {
            output.surface.set_buffer_scale(scale_factor as i32);
        }

        let surface_ptr = output.surface.id().as_ptr().cast::<c_void>();
        let window_handle =
            WaylandWindowHandle::new(NonNull::new(surface_ptr).expect("surface ptr is null"));
        let raw_handle = RawWindowHandle::Wayland(window_handle);

        let surface_attrs = SurfaceAttributesBuilder::<WindowSurface>::new().build(
            raw_handle,
            NonZeroU32::new((width * scale_factor).max(1)).unwrap(),
            NonZeroU32::new((height * scale_factor).max(1)).unwrap(),
        );

        let surface = unsafe {
            self.display
                .create_window_surface(&self.config, &surface_attrs)
                .expect("could not create EGL surface")
        };

        let context_attrs = ContextAttributesBuilder::new().build(None);
        let context = unsafe {
            self.display
                .create_context(&self.config, &context_attrs)
                .expect("could not create OpenGL context")
                .make_current(&surface)
                .expect("could not make OpenGL context current")
        };

        let renderer = unsafe {
            OpenGl::new_from_function_cstr(|s| self.display.get_proc_address(s).cast())
                .expect("could not create renderer")
        };

        let mut canvas = Canvas::new(renderer).expect("could not create canvas");
        let font_id = canvas
            .add_font_mem(include_bytes!("../assets/Lato-Regular.ttf"))
            .expect("could not load font");

        ConfiguredSurface {
            context,
            surface,
            canvas,
            font_id,
            width,
            height,
            pending_resize: false,
        }
    }

    pub fn render_output(
        &self,
        output: &mut Output,
        animation_config: &AnimationConfig,
        app_state: &AppState,
    ) {
        let Some(surface) = &mut output.configured else {
            return;
        };

        if surface.pending_resize {
            let (pw, ph) = surface.physical_size_nonzero(output.scale_factor);
            let scale_factor = output.scale_factor as u32;
            if scale_factor > 1 {
                output.surface.set_buffer_scale(scale_factor as i32);
            }
            surface.surface.resize(&surface.context, pw, ph);
            surface.pending_resize = false;
        }

        let (pw, ph) = surface.physical_size(output.scale_factor);
        surface.canvas.set_size(pw, ph, output.scale_factor);
        surface.canvas.clear_rect(0, 0, pw, ph, Color::black());

        nclock_core::render(
            &mut surface.canvas,
            (pw as f32, ph as f32),
            animation_config,
            app_state,
            surface.font_id,
        );

        surface.canvas.flush();
        surface
            .surface
            .swap_buffers(&surface.context)
            .expect("could not swap buffers");
    }
}
