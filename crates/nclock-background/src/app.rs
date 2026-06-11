use std::time::{Duration, Instant};

use calloop::EventLoop;
use calloop::timer::{TimeoutAction, Timer};
use time::{OffsetDateTime, PrimitiveDateTime};
use wayland_client::globals::GlobalListContents;
use wayland_client::protocol::wl_compositor::WlCompositor;
use wayland_client::protocol::wl_output::{Event as WlOutputEvent, WlOutput};
use wayland_client::protocol::wl_registry::{Event as WlRegistryEvent, WlRegistry};
use wayland_client::protocol::wl_surface::{Event as WlSurfaceEvent, WlSurface};
use wayland_client::{Connection, Dispatch, QueueHandle, delegate_noop};
use wayland_protocols_wlr::layer_shell::v1::client::zwlr_layer_shell_v1::ZwlrLayerShellV1;
use wayland_protocols_wlr::layer_shell::v1::client::zwlr_layer_surface_v1::{
    Event as ZwlrLayerSurfaceV1Event, ZwlrLayerSurfaceV1,
};

use nclock_config::AppConfig;
use nclock_core::AppState;

use crate::opengl::OpenGlContext;
use crate::wayland::WaylandContext;

const RENDER_INTERVAL: Duration = Duration::from_millis(16);

pub struct App {
    config: AppConfig,
    state: AppState,

    wayland: WaylandContext<Self>,
    opengl: OpenGlContext,
}

impl App {
    pub fn run(config: AppConfig) {
        let mut event_loop = EventLoop::try_new().expect("could not create event loop");
        event_loop
            .handle()
            .insert_source(
                Timer::from_duration(RENDER_INTERVAL),
                |_, _, app: &mut App| {
                    app.state.refresh_current_instant();
                    app.render_all();
                    TimeoutAction::ToDuration(RENDER_INTERVAL)
                },
            )
            .expect("could not insert timer");

        let wayland = WaylandContext::create(&event_loop, &config.layer);
        let opengl = OpenGlContext::create(&wayland.connection());

        let initial_instant = Instant::now();
        let initial_time = OffsetDateTime::now_local().unwrap();
        let utc_offset = initial_time.offset();
        let initial_time = PrimitiveDateTime::new(initial_time.date(), initial_time.time());

        let mut app = Self {
            config,
            state: AppState::new(initial_time, initial_instant, utc_offset),
            wayland,
            opengl,
        };

        event_loop
            .run(None, &mut app, |_| {})
            .expect("event loop error");
    }

    fn bind_output(&mut self, global_name: u32, version: u32) {
        self.wayland
            .bind_output(global_name, version, &self.config.layer);
    }

    fn handle_configure(&mut self, output_name: u32, width: u32, height: u32) {
        let first_configure = self.wayland.handle_configure(output_name, width, height);
        if first_configure {
            let output = self.wayland.outputs.get(&output_name).unwrap();
            let configured = self.opengl.init_for_output(output, width, height);
            self.wayland
                .outputs
                .get_mut(&output_name)
                .unwrap()
                .configured = Some(configured);
        }
    }

    fn handle_closed(&mut self, output_name: u32) {
        self.wayland.handle_closed(output_name);
    }

    fn render_all(&mut self) {
        for output in self.wayland.outputs.values_mut() {
            self.opengl
                .render_output(output, &self.config.animation, &self.state);
        }
    }
}

delegate_noop!(App: WlCompositor);
delegate_noop!(App: ZwlrLayerShellV1);

impl Dispatch<WlSurface, ()> for App {
    fn event(
        _state: &mut Self,
        _proxy: &WlSurface,
        _event: WlSurfaceEvent,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<WlRegistry, GlobalListContents> for App {
    fn event(
        state: &mut Self,
        _proxy: &WlRegistry,
        event: WlRegistryEvent,
        _data: &GlobalListContents,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        match event {
            WlRegistryEvent::Global {
                name,
                interface,
                version,
            } if interface == "wl_output" => {
                state.bind_output(name, version);
            }
            WlRegistryEvent::GlobalRemove { name } => {
                state.handle_closed(name);
            }
            _ => {}
        }
    }
}

impl Dispatch<WlOutput, u32> for App {
    fn event(
        state: &mut Self,
        _proxy: &WlOutput,
        event: WlOutputEvent,
        output_name: &u32,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        if let WlOutputEvent::Scale { factor } = event {
            if let Some(output) = state.wayland.outputs.get_mut(output_name) {
                output.scale_factor = factor as f32;
            }
        }
    }
}

impl Dispatch<ZwlrLayerSurfaceV1, u32> for App {
    fn event(
        state: &mut Self,
        proxy: &ZwlrLayerSurfaceV1,
        event: ZwlrLayerSurfaceV1Event,
        output_name: &u32,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        match event {
            ZwlrLayerSurfaceV1Event::Configure {
                serial,
                width,
                height,
            } => {
                proxy.ack_configure(serial);
                state.handle_configure(*output_name, width, height);
            }
            ZwlrLayerSurfaceV1Event::Closed => {
                state.handle_closed(*output_name);
            }
            _ => {}
        }
    }
}
