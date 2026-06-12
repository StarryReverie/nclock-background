use std::collections::HashMap;

use calloop::EventLoop;
use calloop_wayland_source::WaylandSource;
use wayland_client::globals::GlobalListContents;
use wayland_client::protocol::wl_compositor::WlCompositor;
use wayland_client::protocol::wl_output::WlOutput;
use wayland_client::protocol::wl_registry::WlRegistry;
use wayland_client::protocol::wl_surface::WlSurface;
use wayland_client::{Connection, Dispatch, QueueHandle};
use wayland_protocols_wlr::layer_shell::v1::client::zwlr_layer_shell_v1::{
    Layer, ZwlrLayerShellV1,
};
use wayland_protocols_wlr::layer_shell::v1::client::zwlr_layer_surface_v1::{
    Anchor, KeyboardInteractivity, ZwlrLayerSurfaceV1,
};

use nclock_config::{Layer as AppLayer, LayerConfig};

use crate::output::{ConfiguredSurface, Output};

pub struct WaylandContext<A> {
    connection: Connection,
    registry: WlRegistry,
    compositor: WlCompositor,
    queue_handle: QueueHandle<A>,
    layer_shell: ZwlrLayerShellV1,
    outputs: HashMap<u32, Output>,
}

impl<A> WaylandContext<A>
where
    A: Dispatch<WlCompositor, ()>,
    A: Dispatch<WlOutput, u32>,
    A: Dispatch<WlRegistry, GlobalListContents>,
    A: Dispatch<WlSurface, ()>,
    A: Dispatch<ZwlrLayerShellV1, ()>,
    A: Dispatch<ZwlrLayerSurfaceV1, u32>,
    A: 'static,
{
    pub fn create(event_loop: &EventLoop<'_, A>, layer_config: &LayerConfig) -> Self {
        let connection =
            Connection::connect_to_env().expect("could not connect to Wayland compositor");

        let (globals, event_queue) = wayland_client::globals::registry_queue_init::<A>(&connection)
            .expect("could not initialize registry");
        let queue_handle = event_queue.handle();

        let registry = globals.registry().clone();
        let compositor: WlCompositor = globals
            .bind(&queue_handle, 1..=6, ())
            .expect("wl_compositor protocol not available");
        let layer_shell: ZwlrLayerShellV1 = globals
            .bind(&queue_handle, 1..=5, ())
            .expect("zwlr_layer_shell_v1 protocol not available");

        WaylandSource::new(connection.clone(), event_queue)
            .insert(event_loop.handle())
            .expect("could not insert Wayland event source");

        let mut context = WaylandContext {
            connection,
            registry,
            compositor,
            layer_shell,
            queue_handle,
            outputs: Default::default(),
        };

        globals.contents().with_list(|globals| {
            for global in globals {
                if global.interface == "wl_output" {
                    context.bind_output(global.name, global.version, layer_config);
                }
            }
        });

        context
    }

    pub fn connection(&self) -> &Connection {
        &self.connection
    }

    pub fn bind_output(&mut self, global_name: u32, version: u32, layer_config: &LayerConfig) {
        if self.outputs.contains_key(&global_name) {
            return;
        }

        let output = self
            .registry
            .bind(global_name, version, &self.queue_handle, global_name);

        let surface = self.compositor.create_surface(&self.queue_handle, ());

        let layer = match layer_config.layer {
            AppLayer::Background => Layer::Background,
            AppLayer::Bottom => Layer::Bottom,
            AppLayer::Top => Layer::Top,
            AppLayer::Overlay => Layer::Overlay,
        };

        let layer_surface = self.layer_shell.get_layer_surface(
            &surface,
            Some(&output),
            layer,
            layer_config.namespace.clone(),
            &self.queue_handle,
            global_name,
        );

        layer_surface.set_anchor(Anchor::all());
        layer_surface.set_exclusive_zone(-1);
        layer_surface.set_keyboard_interactivity(KeyboardInteractivity::None);
        layer_surface.set_size(0, 0);

        surface.commit();

        self.outputs.insert(
            global_name,
            Output {
                output,
                surface,
                layer_surface,
                scale_factor: 1.0,
                configured: None,
            },
        );
    }

    pub fn handle_configure(
        &mut self,
        output_name: u32,
        width: u32,
        height: u32,
        init: impl FnOnce(&Output, u32, u32) -> ConfiguredSurface,
    ) {
        let Some(output) = self.outputs.get_mut(&output_name) else {
            return;
        };

        if let Some(configured) = &mut output.configured {
            configured.update_size(width, height);
            return;
        }

        let configured = init(output, width, height);
        output.configured = Some(configured);
    }

    pub fn handle_closed(&mut self, output_name: u32) {
        self.outputs.remove(&output_name);
    }

    pub fn set_scale_factor(&mut self, output_name: &u32, factor: f32) {
        if let Some(output) = self.outputs.get_mut(output_name) {
            output.scale_factor = factor;
            if let Some(c) = &mut output.configured {
                c.pending_resize = true;
            }
        }
    }

    pub fn for_each_output_mut(&mut self, mut f: impl FnMut(&mut Output)) {
        for output in self.outputs.values_mut() {
            f(output);
        }
    }
}
