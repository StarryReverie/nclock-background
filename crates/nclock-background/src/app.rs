use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use calloop::EventLoop;
use nix::sys::signal::{SaFlags, SigAction, SigHandler, SigSet, Signal};
use time::{OffsetDateTime, PrimitiveDateTime};
use wayland_client::globals::GlobalListContents;
use wayland_client::protocol::wl_compositor::WlCompositor;
use wayland_client::protocol::wl_keyboard::{Event as WlKeyboardEvent, KeyState, WlKeyboard};
use wayland_client::protocol::wl_output::{Event as WlOutputEvent, WlOutput};
use wayland_client::protocol::wl_pointer::{ButtonState, Event as WlPointerEvent, WlPointer};
use wayland_client::protocol::wl_registry::{Event as WlRegistryEvent, WlRegistry};
use wayland_client::protocol::wl_seat::{Capability, Event as WlSeatEvent, WlSeat};
use wayland_client::protocol::wl_surface::{Event as WlSurfaceEvent, WlSurface};
use wayland_client::{Connection, Dispatch, QueueHandle, WEnum, delegate_noop};
use wayland_protocols_wlr::layer_shell::v1::client::zwlr_layer_shell_v1::ZwlrLayerShellV1;
use wayland_protocols_wlr::layer_shell::v1::client::zwlr_layer_surface_v1::{
    Event as ZwlrLayerSurfaceV1Event, ZwlrLayerSurfaceV1,
};

use nclock_config::AppConfig;
use nclock_core::AppState;

use crate::opengl::OpenGlContext;
use crate::wayland::WaylandContext;

const FINALIZATION_NOTIFICATION_STR: &'static str = "finalizing";

static FINALIZATION_REQUESTED: AtomicBool = AtomicBool::new(false);

pub struct App {
    config: AppConfig,
    state: AppState,

    wayland: WaylandContext<Self>,
    opengl: OpenGlContext,

    should_finalize: bool,
    exit_deadline: Option<Instant>,
}

impl App {
    pub fn run(config: AppConfig) {
        let mut event_loop = EventLoop::try_new().expect("could not create event loop");

        let wayland = WaylandContext::create(&event_loop, &config.layer);
        let opengl = OpenGlContext::create(wayland.connection());

        let initial_instant = Instant::now();
        let initial_time = OffsetDateTime::now_local().unwrap();
        let utc_offset = initial_time.offset();
        let initial_time = PrimitiveDateTime::new(initial_time.date(), initial_time.time());

        let mut app = Self {
            config,
            state: AppState::new(initial_time, initial_instant, utc_offset),
            wayland,
            opengl,
            should_finalize: false,
            exit_deadline: None,
        };

        install_finalization_request_handler();

        loop {
            let frame_start = Instant::now();
            let _ = event_loop.dispatch(Some(Duration::ZERO), &mut app);

            if FINALIZATION_REQUESTED.load(Ordering::Acquire) {
                app.should_finalize = true;
            }

            if app.should_finalize {
                if let Some(exit_deadline) = app.exit_deadline
                    && exit_deadline < frame_start
                {
                    break;
                } else if app.exit_deadline.is_none() {
                    if app.config.ipc.notify_finalization {
                        println!("{FINALIZATION_NOTIFICATION_STR}");
                    }
                    if app.config.ipc.exit_delay.is_zero() {
                        break;
                    } else {
                        app.exit_deadline = Some(frame_start + app.config.ipc.exit_delay);
                    }
                }
            }

            app.state.refresh_current_instant();
            let interval = if app.state.is_high_motion() {
                Duration::from_millis(1000 / 60)
            } else {
                Duration::from_millis(1000 / 24)
            };
            app.render_all();

            let remaining = interval.saturating_sub(frame_start.elapsed());
            std::thread::sleep(remaining);
        }
    }

    fn handle_configure(&mut self, output_name: u32, width: u32, height: u32) {
        let gl = &self.opengl;
        let is_new = self
            .wayland
            .handle_configure(output_name, width, height, |output, w, h| {
                gl.init_for_output(output, w, h)
            });

        if is_new {
            self.state.refresh_current_instant();
            self.render_all();
        }
    }

    fn handle_closed(&mut self, output_name: u32) {
        self.wayland.handle_closed(output_name);
    }

    fn render_all(&mut self) {
        let animation_config = &self.config.animation;
        let state = &self.state;
        self.wayland.for_each_output_mut(|output| {
            output.render(animation_config, state);
        });
    }
}

delegate_noop!(App: WlCompositor);
delegate_noop!(App: ZwlrLayerShellV1);

impl Dispatch<WlSeat, ()> for App {
    fn event(
        state: &mut Self,
        proxy: &WlSeat,
        event: WlSeatEvent,
        _data: &(),
        _conn: &Connection,
        qh: &QueueHandle<Self>,
    ) {
        if let WlSeatEvent::Capabilities { capabilities } = event
            && let WEnum::Value(caps) = capabilities
        {
            if caps.contains(Capability::Keyboard) {
                state.wayland.set_keyboard(proxy.get_keyboard(qh, ()));
            }
            if caps.contains(Capability::Pointer) {
                state.wayland.set_pointer(proxy.get_pointer(qh, ()));
            }
        }
    }
}

impl Dispatch<WlKeyboard, ()> for App {
    fn event(
        state: &mut Self,
        _proxy: &WlKeyboard,
        event: WlKeyboardEvent,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        if let WlKeyboardEvent::Key {
            state: key_state, ..
        } = event
            && key_state == WEnum::Value(KeyState::Pressed)
            && state.config.layer.exit_on_input
        {
            state.should_finalize = true;
        }
    }
}

impl Dispatch<WlPointer, ()> for App {
    fn event(
        state: &mut Self,
        _proxy: &WlPointer,
        event: WlPointerEvent,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        if let WlPointerEvent::Button {
            state: btn_state, ..
        } = event
            && btn_state == WEnum::Value(ButtonState::Pressed)
            && state.config.layer.exit_on_input
        {
            state.should_finalize = true;
        }
    }
}

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
                state
                    .wayland
                    .bind_output(name, version, &state.config.layer);
            }
            WlRegistryEvent::Global {
                name,
                interface,
                version,
            } if interface == "wl_seat" && state.config.layer.exit_on_input => {
                state.wayland.bind_seat(name, version);
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
            state.wayland.set_scale_factor(output_name, factor as f32);
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

fn install_finalization_request_handler() {
    extern "C" fn handle_sigusr1(_: i32) {
        FINALIZATION_REQUESTED.store(true, Ordering::Release);
    }

    let action = SigAction::new(
        SigHandler::Handler(handle_sigusr1),
        SaFlags::empty(),
        SigSet::empty(),
    );

    unsafe {
        if nix::sys::signal::sigaction(Signal::SIGUSR1, &action).is_err() {
            eprintln!(
                "nclock-background: failed to install SIGUSR1 handler for finalization request"
            );
        }
    }
}
