use std::time::Duration;

use clap::Parser;

use nclock_screensaver::AppConfig;

fn main() {
    let cli = Cli::parse();

    let config = AppConfig {
        grace_period: Duration::from_secs(cli.grace_period_secs),
        maximum_period: Duration::from_secs(cli.maximum_period_secs),
        wallpaper: cli.wallpaper_cmd.as_deref().map(parse_cmd),
        locker: cli.locker_cmd.as_deref().map(parse_cmd),
    };

    nclock_screensaver::run(&config);
}

fn parse_cmd(s: &str) -> (String, Vec<String>) {
    let parts = shlex::split(s).unwrap_or_else(|| {
        eprintln!("nclock-screensaver: failed to parse command string: {s:?}");
        std::process::exit(2);
    });
    let mut iter = parts.into_iter();
    let bin = iter.next().unwrap_or_else(|| {
        eprintln!("nclock-screensaver: empty command string");
        std::process::exit(2);
    });
    (bin, iter.collect())
}

#[derive(Parser)]
#[command(author, version, about)]
struct Cli {
    /// How long after screensaver start the user can exit without spawning the locker.
    #[arg(long, default_value = "30")]
    grace_period_secs: u64,

    /// Maximum idle time before forcefully stopping the screensaver and spawning the locker.
    #[arg(long, default_value = "900")]
    maximum_period_secs: u64,

    /// Wallpaper command with arguments, e.g. `"nclock-background --exit-delay-ms 1000"`. Some
    /// extra arguments are automatically prepended.
    #[arg(long)]
    wallpaper_cmd: Option<String>,

    /// Locker command with arguments, e.g. `"swaylock --grace 5"`. If omitted, no locker is
    /// spawned on timeout.
    #[arg(long)]
    locker_cmd: Option<String>,
}
