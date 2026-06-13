use std::io::Read;
use std::process::{Child, Command, Stdio};
use std::sync::Mutex;
use std::sync::mpsc::RecvTimeoutError;
use std::time::{Duration, Instant};

use nix::sys::signal::Signal;
use nix::unistd::Pid;

const FINALIZATION_NOTIFICATION_STR: &'static str = "finalizing";

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub grace_period: Duration,
    pub maximum_period: Duration,
    pub wallpaper: Option<(String, Vec<String>)>,
    pub locker: Option<(String, Vec<String>)>,
}

#[derive(Debug, Clone)]
enum AppEvent {
    SubprocessFinalizingOrExited,
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
        s.spawn(|| {
            let subprocess = &mut subprocess.lock().unwrap();

            if let Some(stdout) = &mut subprocess.stdout {
                let mut buf = [0u8; FINALIZATION_NOTIFICATION_STR.as_bytes().len()];
                if stdout.read_exact(&mut buf[..]).is_ok()
                    && buf == FINALIZATION_NOTIFICATION_STR.as_bytes()
                {
                    eprintln!("nclock-screensaver: nclock-background subprocess is finalizing");
                    let _ = events_tx.send(AppEvent::SubprocessFinalizingOrExited);
                }
            }

            match subprocess.wait() {
                Ok(status) => {
                    eprintln!(
                        "nclock-screensaver: nclock-background subprocess terminated with {status}"
                    );
                    let _ = events_tx.send(AppEvent::SubprocessFinalizingOrExited);
                }
                Err(err) => {
                    eprintln!(
                        "nclock-screensaver: failed to wait for nclock-background subprocess: {err}"
                    );
                }
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
            Ok(AppEvent::SubprocessFinalizingOrExited) => {
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
                    "nclock-screensaver: maximum period expired, stop screensaver with SIGUSR1 and spawn locker"
                );
                // Request nclock-screensaver to do graceful finalization by sending SIGUSR1.
                let _ = nix::sys::signal::kill(Pid::from_raw(pid as i32), Signal::SIGUSR1);
                let _ = cancel_tx.send(());
                true
            }
        };

        if should_run_locker {
            run_locker(config.locker.as_ref());
        } else {
            // Kill nclock-screensaver right now to prevent exit delay.
            let _ = nix::sys::signal::kill(Pid::from_raw(pid as i32), Signal::SIGTERM);
        }
    });
}

fn spawn_wallpaper(cmd: Option<&(String, Vec<String>)>) -> Child {
    let res = Command::new(cmd.map_or("nclock-background", |(bin, _)| bin))
        .stdout(Stdio::piped())
        .arg("--exit-on-input")
        .args(["--layer", "overlay"])
        .args(["--exit-delay-ms", "1000"])
        .arg("--notify-finalization")
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
