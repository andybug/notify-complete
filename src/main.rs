mod config;

use crate::config::Config;
use humantime::format_duration;
use notify_rust::Notification;
use std::process::{self, Command, ExitStatus};
use std::time::{Duration, Instant};

fn send_notification(conf: &config::Config, duration: Duration, status: ExitStatus) {
    let duration_str = format_duration(duration).to_string();

    let mut message = String::from(conf.message.as_str());
    message.push('\n');
    message.push_str(&format!("Result: {}\n", status.code().unwrap()));
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
        Err(e) => println!("{}", e),
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let conf = Config::new();

    let start = Instant::now();

    let mut child = Command::new(conf.command[0].as_str())
        .args(&conf.command[1..])
        .spawn()
        .expect("Error creating child process");

    let child_result = child.wait().expect("Error waiting on child process");
    // using as_secs here to reduce the precision for humantime
    let elapsed_sec = Duration::from_secs((Instant::now() - start).as_secs());

    send_notification(&conf, elapsed_sec, child_result);

    match child_result.code() {
        Some(code) => match code {
            0 => Ok(()),
            _ => process::exit(code),
        },
        None => {
            eprintln!("notify-complete: Child killed by signal");
            Ok(())
        }
    }
}
