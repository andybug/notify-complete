use clap::Parser;
use notify_rust::{Timeout, Urgency};
use serde_derive::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};
use std::vec::Vec;

#[derive(Debug, Deserialize)]
struct TomlProfile {
    name: String,
    icon: Option<String>,
    message: Option<String>,
    timeout: Option<String>,
    title: Option<String>,
    urgency: Option<String>,
    command: Option<String>,
}

#[derive(Debug, Deserialize)]
struct TomlConfig {
    profile: Option<Vec<TomlProfile>>,
}

pub struct Config {
    pub icon: String,
    pub message: String,
    pub timeout: Timeout,
    pub title: String,
    pub urgency: Urgency,
    pub command: Vec<String>,
}

impl Config {
    fn default_config() -> Config {
        Config {
            icon: Config::default_icon(),
            message: Config::default_message(),
            timeout: Config::default_timeout(),
            title: Config::default_title(),
            urgency: Config::default_urgency(),
            command: Config::default_command(),
        }
    }

    fn default_icon() -> String {
        String::new()
    }

    fn default_message() -> String {
        String::new()
    }

    fn default_title() -> String {
        String::from("Command completed")
    }

    fn default_timeout() -> Timeout {
        Timeout::Default
    }

    fn default_urgency() -> Urgency {
        Urgency::Normal
    }

    fn default_command() -> Vec<String> {
        vec![]
    }

    pub fn new() -> Config {
        let args = std::env::args().collect();
        let config_path = get_config_path();
        let toml_config = read_config_file(config_path.as_path());

        return Config::new_from(args, &toml_config);
    }

    //fn new_from(arguments: &mut dyn std::iter::Iterator<Item = String>, toml_config: &Option<TomlConfig>) -> Config {
    fn new_from(arguments: Vec<String>, toml_config: &Option<TomlConfig>) -> Config {
        let args = args::Args::parse_from(arguments);

        let mut conf = match toml_config {
            Some(tc) => {
                if tc.profile.is_none() {
                    Config::default_config()
                } else {
                    Config::from_toml(args.get_profile(), tc)
                }
            }
            None => Config::default_config(),
        };

        if args.title.is_some() {
            conf.title = String::from(args.title.as_ref().unwrap());
        }

        if args.message.is_some() {
            conf.message = String::from(args.message.as_ref().unwrap());
        }

        if args.timeout.is_some() {
            conf.timeout = Config::parse_timeout(args.timeout.as_ref().unwrap().as_str());
        }

        if args.urgency.is_some() {
            conf.urgency = Config::parse_urgency(args.urgency.as_ref().unwrap().as_str());
        }

        conf.command = args.command;
        conf
    }

    pub fn parse_timeout(timeout: &str) -> Timeout {
        match timeout {
            "default" => Timeout::Default,
            "never" => Timeout::Never,
            _ => match timeout.parse::<u32>() {
                Ok(ms) => Timeout::Milliseconds(ms),
                Err(_) => {
                    eprintln!("notify-complete: Error parsing timeout value '{}'", timeout);
                    Timeout::Default
                }
            },
        }
    }

    pub fn parse_urgency(urgency: &str) -> Urgency {
        match urgency {
            "low" => Urgency::Low,
            "normal" => Urgency::Normal,
            "critical" => Urgency::Critical,
            _ => {
                eprintln!("notify-complete: Invalid urgency value '{}'", urgency);
                Urgency::Normal
            }
        }
    }

    fn parse_command(command: &str) -> Vec<String> {
        let components = command.split_whitespace();
        let mut cmd_vec = Vec::new();
        for component in components {
            cmd_vec.push(String::from(component));
        }
        cmd_vec
    }

    fn from_toml(profile: &str, toml: &TomlConfig) -> Config {
        if toml.profile.is_none() {
            // no profiles in config file, return default config
            return Config::default_config();
        }

        for toml_profile in toml.profile.as_ref().unwrap() {
            if profile == toml_profile.name {
                return Config::from_toml_profile(toml_profile);
            }
        }

        return Config::default_config();
    }

    fn from_toml_profile(profile: &TomlProfile) -> Config {
        let icon = match &profile.icon {
            Some(icon) => String::from(icon),
            None => Config::default_icon(),
        };

        let message = match &profile.message {
            Some(message) => String::from(message),
            None => Config::default_message(),
        };

        let timeout = match profile.timeout.as_ref() {
            Some(t) => Config::parse_timeout(t.as_str()),
            None => Config::default_timeout(),
        };

        let title = match &profile.title {
            Some(title) => String::from(title),
            None => Config::default_title(),
        };

        let urgency = match profile.urgency.as_ref() {
            Some(u) => Config::parse_urgency(u.as_str()),
            None => Config::default_urgency(),
        };

        let command = match &profile.command.as_ref() {
            Some(c) => Config::parse_command(c.as_str()),
            None => Config::default_command(),
        };

        Config {
            icon,
            message,
            timeout,
            title,
            urgency,
            command,
        }
    }
}

fn get_config_path() -> PathBuf {
    const CONFIG_DIR: &str = "notify-complete";
    const CONFIG_FILE: &str = "config.toml";

    let mut config_path = match dirs::config_dir() {
        Some(dir) => dir,
        None => {
            let default_path = format!("~/.config/{}/{}", CONFIG_DIR, CONFIG_FILE);
            eprintln!(
                "Could not determine user config directory -- using {}",
                default_path
            );
            return PathBuf::from(default_path);
        }
    };

    config_path.push(CONFIG_DIR);
    config_path.push(CONFIG_FILE);
    return config_path;
}

fn read_config_file(path: &Path) -> Option<TomlConfig> {
    if !path.is_file() {
        // no config provided
        return None;
    }

    let contents = match fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!(
                "Error reading config file '{}': {}",
                path.to_str().unwrap(),
                e
            );
            return None;
        }
    };

    let conf: TomlConfig = match toml::from_str(&contents) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error parsing config file: {}", e);
            return None;
        }
    };

    Some(conf)
}

#[cfg(test)]
mod toml_tests {
    use super::{Config, TomlConfig, TomlProfile};
    use notify_rust::{Timeout, Urgency};

    #[test]
    fn config_defaults() {
        let tc = TomlConfig { profile: None };
        let c = Config::from_toml("doesn't matter", &tc);

        assert_eq!(c.icon, Config::default_icon());
        assert_eq!(c.message, Config::default_message());
        assert_eq!(c.timeout, Config::default_timeout());
        assert_eq!(c.title, Config::default_title());
        assert_eq!(c.urgency, Config::default_urgency());
        assert_eq!(c.command, Config::default_command());
    }

    #[test]
    fn profile_defaults() {
        let tp = TomlProfile {
            name: "test".to_string(),
            icon: None,
            message: None,
            timeout: None,
            title: None,
            urgency: None,
            command: None,
        };

        let tc = TomlConfig {
            profile: Some(vec![tp]),
        };

        let c = Config::from_toml("test", &tc);

        assert_eq!(c.icon, Config::default_icon());
        assert_eq!(c.message, Config::default_message());
        assert_eq!(c.timeout, Config::default_timeout());
        assert_eq!(c.title, Config::default_title());
        assert_eq!(c.urgency, Config::default_urgency());
        assert_eq!(c.command, Config::default_command());
    }

    #[test]
    fn profile_not_found() {
        let tp = TomlProfile {
            name: "test".to_string(),
            icon: None,
            message: None,
            timeout: None,
            title: None,
            urgency: None,
            command: None,
        };

        let tc = TomlConfig {
            profile: Some(vec![tp]),
        };

        let c = Config::from_toml("does not exist", &tc);

        assert_eq!(c.icon, Config::default_icon());
        assert_eq!(c.message, Config::default_message());
        assert_eq!(c.timeout, Config::default_timeout());
        assert_eq!(c.title, Config::default_title());
        assert_eq!(c.urgency, Config::default_urgency());
        assert_eq!(c.command, Config::default_command());
    }

    #[test]
    fn profile_values() {
        let tp = TomlProfile {
            name: "test".to_string(),
            icon: Some("icon".to_string()),
            message: Some("message".to_string()),
            timeout: Some("5000".to_string()),
            title: Some("title".to_string()),
            urgency: Some("critical".to_string()),
            command: Some("echo hello".to_string()),
        };

        let tc = TomlConfig {
            profile: Some(vec![tp]),
        };

        let c = Config::from_toml("test", &tc);

        assert_eq!(c.icon, "icon");
        assert_eq!(c.message, "message");
        assert_eq!(c.timeout, Timeout::Milliseconds(5000));
        assert_eq!(c.title, "title");
        assert_eq!(c.urgency, Urgency::Critical);
        assert_eq!(c.command, vec!["echo", "hello"]);
    }
}

#[cfg(test)]
mod new_from_tests {
    use super::{Config, TomlConfig, TomlProfile};
    use notify_rust::{Timeout, Urgency};

    #[test]
    fn default_values() {
        let tc = TomlConfig { profile: None };
        let c = Config::new_from(
            vec!["notify-complete", "sleep", "1"]
                .iter()
                .map(|s| s.to_string())
                .collect(),
            &Some(tc),
        );

        assert_eq!(c.icon, Config::default_icon());
        assert_eq!(c.message, Config::default_message());
        assert_eq!(c.timeout, Config::default_timeout());
        assert_eq!(c.title, Config::default_title());
        assert_eq!(c.urgency, Config::default_urgency());
        assert_eq!(c.command, vec!["sleep", "1"]);
    }

    #[test]
    fn profile() {
        let tp = TomlProfile {
            name: "test".to_string(),
            icon: Some("icon".to_string()),
            message: Some("message".to_string()),
            timeout: Some("5000".to_string()),
            title: Some("title".to_string()),
            urgency: Some("critical".to_string()),
            command: None,
        };

        let tc = TomlConfig {
            profile: Some(vec![tp]),
        };

        let c = Config::new_from(
            vec!["notify-complete", "-p", "test", "echo", "test"]
                .iter()
                .map(|s| s.to_string())
                .collect(),
            &Some(tc),
        );

        assert_eq!(c.icon, "icon");
        assert_eq!(c.message, "message");
        assert_eq!(c.timeout, Timeout::Milliseconds(5000));
        assert_eq!(c.title, "title");
        assert_eq!(c.urgency, Urgency::Critical);
        assert_eq!(c.command, vec!["echo", "test"]);
    }
}

#[cfg(test)]
mod value_parsing_tests {
    use super::Config;
    use notify_rust::{Timeout, Urgency};

    #[test]
    fn timeout_value_default() {
        let timeout = Config::parse_timeout("default");
        assert_eq!(timeout, Timeout::Default);
    }

    #[test]
    fn timeout_value_never() {
        let timeout = Config::parse_timeout("never");
        assert_eq!(timeout, Timeout::Never);
    }

    #[test]
    fn timeout_value_negative() {
        // if timeout < 0, print and error and use the default
        let timeout = Config::parse_timeout("-1");
        assert_eq!(timeout, Config::default_timeout());
    }

    #[test]
    fn timeout_value_ms() {
        let timeout = Config::parse_timeout("3000");
        assert_eq!(timeout, Timeout::Milliseconds(3000));
    }

    #[test]
    fn urgency_value_invalid() {
        let urgency = Config::parse_urgency("invalid");
        assert_eq!(urgency, Config::default_urgency());
    }

    #[test]
    fn urgency_value_low() {
        let urgency = Config::parse_urgency("low");
        assert_eq!(urgency, Urgency::Low);
    }

    #[test]
    fn urgency_value_normal() {
        let urgency = Config::parse_urgency("normal");
        assert_eq!(urgency, Urgency::Normal);
    }

    #[test]
    fn urgency_value_critical() {
        let urgency = Config::parse_urgency("critical");
        assert_eq!(urgency, Urgency::Critical);
    }
}

mod args {
    use clap::{AppSettings, Parser, ValueHint};

    // Runs a command and sends a notification upon completion
    #[derive(Parser, Debug)]
    #[clap(author, version, about, long_about = None, setting = AppSettings::TrailingVarArg)]
    pub struct Args {
        #[clap(
            short,
            long,
            default_value = "default",
            help = "The name of the profile to use for the notification."
        )]
        profile: String,

        #[clap(short, long, help = "Title of the notification.")]
        pub title: Option<String>,

        #[clap(short, long, help = "Notification contents.")]
        pub message: Option<String>,

        #[clap(
            short = 'o',
            long,
            help = "Notification timeout in ms or 'never'/'default'."
        )]
        pub timeout: Option<String>,

        #[clap(short, long, help = "Notification urgency (low, normal, critical)")]
        pub urgency: Option<String>,

        #[clap(required = true, multiple_values = true, value_hint = ValueHint::CommandWithArguments, name = "cmd-with-args")]
        pub command: Vec<String>,
    }

    impl Args {
        pub fn get_profile(&self) -> &str {
            self.profile.as_str()
        }
    }

    #[cfg(test)]
    mod tests {
        use clap::StructOpt;

        use super::Args;

        #[test]
        fn args_no_cmd() {
            let args = vec!["notify-complete"];
            let result = Args::try_parse_from(args);
            assert!(result.is_err());
        }

        #[test]
        fn args_cmd_only() {
            let args = vec!["notify-complete", "fake-cmd"];
            let parsed = Args::parse_from(args);

            assert_eq!(parsed.profile, "default");
            assert_eq!(parsed.title, None);
            assert_eq!(parsed.message, None);
            assert_eq!(parsed.timeout, None);
            assert_eq!(parsed.urgency, None);
            assert_eq!(parsed.command, vec!["fake-cmd"]);
        }

        #[test]
        fn args_all() {
            let args = vec![
                "notify-complete",
                "-p",
                "test-profile",
                "-t",
                "Unit test",
                "-m",
                "This is a unit test.",
                "-o",
                "never",
                "-u",
                "low",
                "fake-cmd",
                "--option",
                "yes",
            ];
            let parsed = Args::parse_from(args);

            assert_eq!(parsed.profile, "test-profile");
            assert_eq!(parsed.title.unwrap(), "Unit test");
            assert_eq!(parsed.message.unwrap(), "This is a unit test.");
            assert_eq!(parsed.timeout.unwrap(), "never");
            assert_eq!(parsed.urgency.unwrap(), "low");
            assert_eq!(parsed.command, vec!["fake-cmd", "--option", "yes"]);
        }
    }
}
