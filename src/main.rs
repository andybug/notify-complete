mod config;

use crate::config::Config;
use humantime::format_duration;
use notify_rust::Notification;
use std::process::{self, Command};
use std::time::{Duration, Instant};

fn send_notification(conf: &config::Config, _exit_code: i32, duration: Duration) {
    let duration_str = format_duration(duration).to_string();

    let mut message = String::from(conf.message.as_str());
    message.push('\n');
    message.push_str(&format!("Completed in {}", duration_str));

    let mut notification = Notification::new();
    notification.summary(conf.title.as_str());
    notification.body(message.as_str());
    notification.timeout(conf.timeout);
    notification.appname("notify-complete");

    #[cfg(all(unix, not(target_os = "macos")))]
    notification.urgency(conf.urgency);

    let result = notification.show();

    match result {
        Ok(_) => (),
        Err(e) => eprintln!("notify-complete: Error sending notification: {}", e),
    }
}

fn spawn_child(conf: &Config) -> (i32, Duration) {
    let start = Instant::now();

    let mut child = Command::new(conf.command[0].as_str())
        .args(&conf.command[1..])
        .spawn()
        .expect("notify-complete: Error creating child process");

    let child_result = child
        .wait()
        .expect("notify-complete: Error waiting on child process");

    // using as_secs here to reduce the precision
    let elapsed_sec = Duration::from_secs((Instant::now() - start).as_secs());

    let exit_code = match child_result.code() {
        Some(code) => code,
        None => {
            eprintln!("notify-complete: Child killed by signal");
            // since the child was killed and didn't exit normally, exit with an error
            1
        }
    };

    (exit_code, elapsed_sec)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let conf = Config::new();
    let (exit_code, elapsed_sec) = spawn_child(&conf);
    send_notification(&conf, exit_code, elapsed_sec);

    match exit_code {
        0 => Ok(()),
        _ => process::exit(exit_code),
    }
}
