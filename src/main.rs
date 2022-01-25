mod config;

use std::process::Command;
use std::time::{Duration, Instant};
use notify_rust::Notification;
use clap::{AppSettings, Parser, ValueHint};
use humantime::format_duration;

// Runs a command and sends a notification upon completion
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None, setting = AppSettings::TrailingVarArg)]
struct Args {
    #[clap(short, long, default_value = "default", help = "The name of the profile to use for the notification.")]
    profile: String,
    
    #[clap(short, long, help = "Title of the notification.")]
    summary: String,

    #[clap(short, long, help = "Notification contents.")]
    body: String,

    #[clap(required = true, multiple_values = true, value_hint = ValueHint::CommandWithArguments, name = "cmd-with-args")]
    cmd: Vec<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let conf = config::get_config();
    let args = Args::parse();

    match conf.profiles.get(&args.profile) {
        Some(_) => println!("Matched profile '{}'", args.profile),
        None => {
            eprintln!("Unknown profile '{}'", args.profile);
            return Ok(()); // TODO: real error here
        }
    };

    let start = Instant::now();

    let mut child = Command::new(args.cmd[0].as_str()).args(&args.cmd[1..]).spawn().expect("don't error pos");
    child.wait().expect("command failed");

    let elapsed_sec = Duration::from_secs((Instant::now() - start).as_secs());

    let mut body = String::from("<i>Completed in</i> ");
    body.push_str(&format_duration(elapsed_sec).to_string());
    body.push('\n');
    body.push_str(args.body.as_str());


    let result = Notification::new()
        .summary(&args.summary)
        .body(&body)
        .appname("notify-complete")
        .show();
    
    match result {
        Ok(handle) => println!("id = {}", handle.id()),
        Err(e) => println!("{}", e),
    }

    Ok(())
}