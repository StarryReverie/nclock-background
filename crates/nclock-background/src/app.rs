use std::num::NonZeroU32;
use std::time::Instant;

use femtovg::renderer::OpenGl;
use femtovg::{Canvas, Color, FontId};
use glutin::config::ConfigTemplateBuilder;
use glutin::context::{ContextAttributesBuilder, PossiblyCurrentContext};
use glutin::display::{Display, GetGlDisplay};
use glutin::prelude::{GlDisplay, NotCurrentGlContext};
use glutin::surface::{GlSurface, Surface, SurfaceAttributesBuilder, WindowSurface};
use glutin_winit::DisplayBuilder;
use time::{OffsetDateTime, PrimitiveDateTime};
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::raw_window_handle::HasWindowHandle;
use winit::window::{Fullscreen, Window, WindowId};

use nclock_config::AppConfig;
use nclock_core::AppState;

pub struct AppContext {
    window: Window,
    context: PossiblyCurrentContext,
    surface: Surface<WindowSurface>,
    canvas: Canvas<OpenGl>,
    font_id: FontId,
}

pub struct App {
    context: Option<AppContext>,
    config: AppConfig,
    state: AppState,
}

impl Drop for App {
    fn drop(&mut self) {
        if let Some(context) = self.context.take() {
            drop(context.canvas);
            drop(context.surface);
            drop(context.context);
            drop(context.window);
        }
    }
}

impl App {
    pub fn new(config: AppConfig) -> Self {
        let initial_instant = Instant::now();

        let time = OffsetDateTime::now_local().unwrap();
        let initial_time = PrimitiveDateTime::new(time.date(), time.time());
        let utc_offset = time.offset();

        Self {
            context: None,
            config,
            state: AppState::new(initial_time, initial_instant, utc_offset),
        }
    }

    fn render(&mut self) {
        let Some(context) = &mut self.context else {
            return;
        };

        let canvas = &mut context.canvas;
        let font_id = context.font_id;

        let size = context.window.inner_size();
        let scale_factor = context.window.scale_factor() as f32;
        canvas.set_size(size.width, size.height, scale_factor);
        canvas.clear_rect(0, 0, size.width, size.height, Color::black());

        nclock_core::render(
            canvas,
            (size.width as f32, size.height as f32),
            &self.config.animation,
            &self.state,
            font_id,
        );

        canvas.flush();
        context
            .surface
            .swap_buffers(&context.context)
            .expect("could not swap buffers");
    }
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
        let mut canvas = Canvas::new(renderer).expect("could not create canvas");

        let font_data = include_bytes!("../assets/Lato-Regular.ttf");
        let font_id = canvas
            .add_font_mem(font_data)
            .expect("could not load embedded font");

        self.context = Some(AppContext {
            window,
            context,
            surface,
            canvas,
            font_id,
        })
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::RedrawRequested => {
                self.state.refresh_current_instant();
                self.render();
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(context) = &self.context {
            context.window.request_redraw();
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
        .with_title("Night Clock");

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
