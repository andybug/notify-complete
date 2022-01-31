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
}

impl Config {
    fn default_config() -> Config {
        Config {
            icon: Config::default_icon(),
            message: Config::default_message(),
            timeout: Config::default_timeout(),
            title: Config::default_title(),
            urgency: Config::default_urgency(),
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

        Config {
            icon,
            message,
            timeout,
            title,
            urgency,
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

pub fn get_config(profile: &str) -> Config {
    let config_path = get_config_path();

    let config = match read_config_file(config_path.as_path()) {
        Some(toml_config) => Config::from_toml(profile, &toml_config),
        None => Config::default_config(),
    };

    return config;
}

#[cfg(test)]
mod tests {
    use notify_rust::{Timeout, Urgency};

    use super::{Config, TomlConfig, TomlProfile};

    #[test]
    fn config_defaults() {
        let tc = TomlConfig { profile: None };
        let c = Config::from_toml("doesn't matter", &tc);

        assert_eq!(c.icon, Config::default_icon());
        assert_eq!(c.message, Config::default_message());
        assert_eq!(c.timeout, Config::default_timeout());
        assert_eq!(c.title, Config::default_title());
        assert_eq!(c.urgency, Config::default_urgency());
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
    }

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
