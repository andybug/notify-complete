mod config;

use clap::{AppSettings, Parser, ValueHint};
use humantime::format_duration;
use notify_rust::Notification;
use std::process::{Command, ExitStatus};
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

    #[clap(short, long, help = "Title of the notification.")]
    summary: Option<String>,

    #[clap(short, long, help = "Notification contents.")]
    body: Option<String>,

    #[clap(required = true, multiple_values = true, value_hint = ValueHint::CommandWithArguments, name = "cmd-with-args")]
    cmd: Vec<String>,
}

fn update_conf_from_args(conf: &mut config::Config, args: &Args) {
    if args.summary.is_some() {
        conf.profile.summary = String::from(args.summary.as_ref().unwrap());
    }

    if args.body.is_some() {
        conf.profile.body = String::from(args.body.as_ref().unwrap());
    }
}

fn send_notification(conf: &config::Config, duration: Duration, status: ExitStatus) {
    let duration_str = format_duration(duration).to_string();

    let mut body = String::from(conf.profile.body.as_str());
    body.push('\n');
    body.push_str(&format!("Result: {}\n", status.code().unwrap()));
    body.push_str(&format!("Completed in {}", duration_str));

    let result = Notification::new()
        .summary(conf.profile.summary.as_str())
        .body(body.as_str())
        .appname("notify-complete")
        .show();

    match result {
        Ok(handle) => println!("id = {}", handle.id()),
        Err(e) => println!("{}", e),
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let mut conf = config::get_config(args.profile.as_str());

    update_conf_from_args(&mut conf, &args);

    let start = Instant::now();

    let mut child = Command::new(args.cmd[0].as_str())
        .args(&args.cmd[1..])
        .spawn()
        .expect("Error creating child process");

    let child_result = child.wait().expect("Error waiting on child process");
    // using as_secs here to reduce the precision for humantime
    let elapsed_sec = Duration::from_secs((Instant::now() - start).as_secs());

    send_notification(&conf, elapsed_sec, child_result);

    Ok(())
}
