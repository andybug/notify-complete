mod config;

use clap::{AppSettings, Parser, ValueHint};
use humantime::format_duration;
use notify_rust::Notification;
use std::process::Command;
use std::time::{Duration, Instant};

// Runs a command and sends a notification upon completion
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None, setting = AppSettings::TrailingVarArg)]
struct Args {
    #[clap(
        short,
        long,
        default_value = "default",
        help = "The name of the profile to use for the notification."
    )]
    profile: String,

    #[clap(short, long, default_value = "", help = "Title of the notification.")]
    summary: String,

    #[clap(short, long, default_value = "", help = "Notification contents.")]
    body: String,

    #[clap(required = true, multiple_values = true, value_hint = ValueHint::CommandWithArguments, name = "cmd-with-args")]
    cmd: Vec<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let conf = config::get_config();
    let args = Args::parse();

    let profile = match conf.profiles.get(&args.profile) {
        Some(p) => p,
        None => {
            eprintln!("Unknown profile '{}'. Using default profile.", args.profile);
            conf.profiles.get(&"default".to_string()).unwrap()
        }
    };

    let summary = match args.summary.len() > 0 {
        true => args.summary,
        false => String::from(&profile.summary),
    };

    let mut body = match args.body.len() > 0 {
        true => args.body,
        false => String::from(&profile.body),
    };

    let start = Instant::now();

    let mut child = Command::new(args.cmd[0].as_str())
        .args(&args.cmd[1..])
        .spawn()
        .expect("Error creating child process");

    let child_result = child.wait().expect("Error waiting on child process");

    // using as_secs here to reduce the precision for humantime
    let elapsed_sec = Duration::from_secs((Instant::now() - start).as_secs());

    let success_str = match child_result.success() {
        true => "successfully completed",
        false => "errored",
    };

    let duration_str = format_duration(elapsed_sec).to_string();

    body.push('\n');
    body.push_str(&format!("Command {} in {}", success_str, duration_str));

    let result = Notification::new()
        .summary(&summary)
        .body(&body)
        .appname("notify-complete")
        .show();

    match result {
        Ok(handle) => println!("id = {}", handle.id()),
        Err(e) => println!("{}", e),
    }

    Ok(())
}
