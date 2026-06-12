use std::time::Duration;

use nclock_screensaver::AppConfig;

fn main() {
    let config = AppConfig {
        grace_period: Duration::from_secs(30),
        maximum_period: Duration::from_mins(15),
        wallpaper: None,
        locker: None,
    };

    nclock_screensaver::run(&config);
}
