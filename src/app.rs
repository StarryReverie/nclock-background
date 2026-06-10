use std::num::NonZeroU32;

use femtovg::renderer::OpenGl;
use femtovg::{Canvas, Color};
use glutin::config::ConfigTemplateBuilder;
use glutin::context::{ContextAttributesBuilder, PossiblyCurrentContext};
use glutin::display::{Display, GetGlDisplay};
use glutin::prelude::{GlDisplay, NotCurrentGlContext};
use glutin::surface::{GlSurface, Surface, SurfaceAttributesBuilder, WindowSurface};
use glutin_winit::DisplayBuilder;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::raw_window_handle::HasWindowHandle;
use winit::window::{Fullscreen, Window, WindowId};

pub struct App {
    context: Option<AppContext>,
}

impl App {
    pub fn new() -> Self {
        Self { context: None }
    }
}

pub struct AppContext {
    window: Window,
    context: PossiblyCurrentContext,
    surface: Surface<WindowSurface>,
    canvas: Canvas<OpenGl>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.context.is_some() {
            return;
        }

        let (window, context, display, surface) = create_window(event_loop);

        let renderer = unsafe {
            OpenGl::new_from_function_cstr(|s| display.get_proc_address(s).cast())
                .expect("could not create renderer")
        };
        let canvas = Canvas::new(renderer).expect("could not create canvas");

        self.context = Some(AppContext {
            window,
            context,
            surface,
            canvas,
        })
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        let Some(context) = self.context.as_mut() else {
            return;
        };

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::RedrawRequested => {
                render(context);
            }
            _ => {}
        }
    }
}

fn create_window(
    event_loop: &ActiveEventLoop,
) -> (
    Window,
    PossiblyCurrentContext,
    Display,
    Surface<WindowSurface>,
) {
    let window_attrs = Window::default_attributes()
        .with_fullscreen(Some(Fullscreen::Borderless(None)))
        .with_title("Night Clock Screensaver");

    let (window, config) = DisplayBuilder::new()
        .with_window_attributes(Some(window_attrs))
        .build(event_loop, ConfigTemplateBuilder::new(), |mut configs| {
            configs.next().unwrap()
        })
        .expect("could not create display");
    let window = window.expect("could not create window");
    let display = config.display();

    let handle = window
        .window_handle()
        .expect("could not get raw window handle")
        .as_raw();
    let context_attrs = ContextAttributesBuilder::new().build(Some(handle));
    let context = unsafe {
        display
            .create_context(&config, &context_attrs)
            .expect("could not create OpenGL context")
    };

    let handle = window
        .window_handle()
        .expect("could not get raw window handle")
        .as_raw();
    let surface_attrs = SurfaceAttributesBuilder::<WindowSurface>::new().build(
        handle,
        NonZeroU32::new(window.inner_size().width).expect("window width should not be 0"),
        NonZeroU32::new(window.inner_size().height).expect("window height should not be 0"),
    );
    let surface = unsafe {
        display
            .create_window_surface(&config, &surface_attrs)
            .expect("could not create rendering surface")
    };

    let context = context
        .make_current(&surface)
        .expect("could not set current OpenGL context");

    (window, context, display, surface)
}

fn render(context: &mut AppContext) {
    let size = context.window.inner_size();
    context.canvas.set_size(
        size.width,
        size.height,
        context.window.scale_factor() as f32,
    );
    context
        .canvas
        .clear_rect(0, 0, size.width, size.height, Color::black());

    context.canvas.flush();
    context
        .surface
        .swap_buffers(&context.context)
        .expect("could not swap buffers");
}
