use nclock_screensaver::{App, AppConfig};
use winit::event_loop::EventLoop;

fn main() {
    let event_loop = EventLoop::new().expect("could not create event loop");
    let mut app = App::new(AppConfig {
        inner_radius_frac: 0.1,
        lane_width_frac: 0.045,
        lane_margin_frac: 0.015,
    });
    event_loop.run_app(&mut app).unwrap();
}
