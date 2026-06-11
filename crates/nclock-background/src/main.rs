use winit::event_loop::EventLoop;

use nclock_background::App;
use nclock_config::{AnimationConfig, AppConfig};

fn main() {
    let config = AppConfig {
        animation: AnimationConfig {
            relative_inner_radius: 0.1,
            relative_lane_width: 0.045,
            relative_lane_margin: 0.015,
        },
    };

    let event_loop = EventLoop::new().expect("could not create event loop");
    let mut app = App::new(config);
    event_loop.run_app(&mut app).unwrap();
}
