use std::process::{Child, Command};
use std::sync::Mutex;
use std::sync::mpsc::RecvTimeoutError;
use std::time::{Duration, Instant};

use nix::sys::signal::Signal;
use nix::unistd::Pid;

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub grace_period: Duration,
    pub maximum_period: Duration,
    pub wallpaper: Option<(String, Vec<String>)>,
    pub locker: Option<(String, Vec<String>)>,
}

#[derive(Debug, Clone)]
enum AppEvent {
    SubprocessExited,
    MaximumTimerExpired,
}

pub fn run(config: &AppConfig) {
    let start = Instant::now();

    let subprocess = Mutex::new(spawn_wallpaper(config.wallpaper.as_ref()));
    let pid = subprocess.lock().unwrap().id();

    let (events_tx, events) = std::sync::mpsc::channel();
    let events_tx2 = events_tx.clone();

    let (cancel_tx, can_exit) = std::sync::mpsc::channel();
    let cancel = Mutex::new(can_exit);

    std::thread::scope(|s| {
        s.spawn(|| match subprocess.lock().unwrap().wait() {
            Ok(status) => {
                eprintln!(
                    "nclock-screensaver: nclock-background subprocess terminated with {status}"
                );
                let _ = events_tx.send(AppEvent::SubprocessExited);
            }
            Err(err) => {
                eprintln!(
                    "nclock-screensaver: failed to wait for nclock-background subprocess: {err}"
                );
            }
        });

        s.spawn(|| {
            if let Err(RecvTimeoutError::Timeout) =
                cancel.lock().unwrap().recv_timeout(config.maximum_period)
            {
                let _ = events_tx2.send(AppEvent::MaximumTimerExpired);
            }
        });

        let should_run_locker = match events.recv() {
            Ok(AppEvent::SubprocessExited) => {
                let _ = cancel_tx.send(());
                if start.elapsed() > config.grace_period {
                    eprintln!("nclock-screensaver: grace period has already expired, spawn locker");
                    true
                } else {
                    false
                }
            }
            Ok(AppEvent::MaximumTimerExpired) | Err(_) => {
                eprintln!(
                    "nclock-screensaver: maximum period expired, stop screensaver with SIGTERM and spawn locker"
                );
                let _ = nix::sys::signal::kill(Pid::from_raw(pid as i32), Signal::SIGTERM);
                let _ = cancel_tx.send(());
                true
            }
        };

        if should_run_locker {
            run_locker(config.locker.as_ref());
        }
    });
}

fn spawn_wallpaper(cmd: Option<&(String, Vec<String>)>) -> Child {
    let res = Command::new(cmd.map_or("nclock-background", |(bin, _)| bin))
        .arg("--exit-on-input")
        .args(["--layer", "overlay"])
        .args(cmd.map_or(&[][..], |(_, args)| &args[..]))
        .spawn();
    match res {
        Ok(child) => child,
        Err(err) => {
            eprintln!("nclock-screensaver: failed to spawn nclock-background subprocess: {err}");
            std::process::exit(1);
        }
    }
}

fn run_locker(cmd: Option<&(String, Vec<String>)>) {
    if let Some((bin, args)) = cmd {
        let res = Command::new(bin)
            .args(args)
            .spawn()
            .and_then(|mut s| s.wait());
        if let Err(err) = res {
            eprintln!("nclock-screensaver: failed to run locker subprocess: {err}");
        }
    };
}
