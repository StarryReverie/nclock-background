use nclock_screensaver::App;
use winit::event_loop::EventLoop;

fn main() {
    let event_loop = EventLoop::new().expect("could not create event loop");
    let mut app = App::new();
    event_loop.run_app(&mut app).unwrap();
}
