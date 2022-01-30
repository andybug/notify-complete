use notify_rust::{Timeout, Urgency};
use serde_derive::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};
use std::vec::Vec;

#[derive(Debug, Deserialize)]
struct TomlProfile {
    name: String,
    body: Option<String>,
    icon: Option<String>,
    summary: Option<String>,
    timeout: Option<String>,
    urgency: Option<String>,
}

#[derive(Debug, Deserialize)]
struct TomlConfig {
    profile: Option<Vec<TomlProfile>>,
}

pub struct Config {
    pub body: String,
    pub icon: String,
    pub summary: String,
    pub timeout: Timeout,
    pub urgency: Urgency,
}

impl Config {
    fn default_config() -> Config {
        Config {
            body: Config::default_body(),
            icon: Config::default_icon(),
            summary: Config::default_summary(),
            timeout: Config::default_timeout(),
            urgency: Config::default_urgency(),
        }
    }

    fn default_body() -> String {
        String::new()
    }

    fn default_icon() -> String {
        String::new()
    }

    fn default_summary() -> String {
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
        let body = match &profile.body {
            Some(body) => String::from(body),
            None => Config::default_body(),
        };

        let icon = match &profile.icon {
            Some(icon) => String::from(icon),
            None => Config::default_icon(),
        };

        let summary = match &profile.summary {
            Some(summary) => String::from(summary),
            None => Config::default_summary(),
        };

        let timeout = match profile.timeout.as_ref() {
            Some(t) => Config::parse_timeout(t.as_str()),
            None => Config::default_timeout(),
        };

        let urgency = match &profile.urgency {
            Some(urgency) => match urgency.as_str() {
                "low" => Urgency::Low,
                "normal" => Urgency::Normal,
                "critical" => Urgency::Critical,
                _ => {
                    eprintln!("Invalid urgency setting '{}'", urgency);
                    Config::default_urgency()
                }
            },
            None => Config::default_urgency(),
        };

        Config {
            body,
            icon,
            summary,
            timeout,
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

        assert_eq!(c.body, Config::default_body());
        assert_eq!(c.icon, Config::default_icon());
        assert_eq!(c.summary, Config::default_summary());
        assert_eq!(c.timeout, Config::default_timeout());
        assert_eq!(c.urgency, Config::default_urgency());
    }

    #[test]
    fn profile_defaults() {
        let tp = TomlProfile {
            name: "test".to_string(),
            body: None,
            icon: None,
            summary: None,
            timeout: None,
            urgency: None,
        };

        let tc = TomlConfig {
            profile: Some(vec![tp]),
        };

        let c = Config::from_toml("test", &tc);

        assert_eq!(c.body, Config::default_body());
        assert_eq!(c.icon, Config::default_icon());
        assert_eq!(c.summary, Config::default_summary());
        assert_eq!(c.timeout, Config::default_timeout());
        assert_eq!(c.urgency, Config::default_urgency());
    }

    #[test]
    fn profile_not_found() {
        let tp = TomlProfile {
            name: "test".to_string(),
            body: None,
            icon: None,
            summary: None,
            timeout: None,
            urgency: None,
        };

        let tc = TomlConfig {
            profile: Some(vec![tp]),
        };

        let c = Config::from_toml("does not exist", &tc);

        assert_eq!(c.body, Config::default_body());
        assert_eq!(c.icon, Config::default_icon());
        assert_eq!(c.summary, Config::default_summary());
        assert_eq!(c.timeout, Config::default_timeout());
        assert_eq!(c.urgency, Config::default_urgency());
    }

    #[test]
    fn profile_values() {
        let tp = TomlProfile {
            name: "test".to_string(),
            body: Some("body".to_string()),
            icon: Some("icon".to_string()),
            summary: Some("summary".to_string()),
            timeout: Some("5000".to_string()),
            urgency: Some("critical".to_string()),
        };

        let tc = TomlConfig {
            profile: Some(vec![tp]),
        };

        let c = Config::from_toml("test", &tc);

        assert_eq!(c.body, "body");
        assert_eq!(c.icon, "icon");
        assert_eq!(c.summary, "summary");
        assert_eq!(c.timeout, Timeout::Milliseconds(5000));
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
        let tp = TomlProfile {
            name: "test".to_string(),
            body: None,
            icon: None,
            summary: None,
            timeout: None,
            urgency: Some("invalid".to_string()),
        };

        let tc = TomlConfig {
            profile: Some(vec![tp]),
        };

        // this should print an error and return default
        let c = Config::from_toml("test", &tc);
        assert_eq!(c.urgency, Config::default_urgency());
    }

    #[test]
    fn urgency_value_low() {
        let tp = TomlProfile {
            name: "test".to_string(),
            body: None,
            icon: None,
            summary: None,
            timeout: None,
            urgency: Some("low".to_string()),
        };

        let tc = TomlConfig {
            profile: Some(vec![tp]),
        };

        let c = Config::from_toml("test", &tc);
        assert_eq!(c.urgency, Urgency::Low);
    }

    #[test]
    fn urgency_value_normal() {
        let tp = TomlProfile {
            name: "test".to_string(),
            body: None,
            icon: None,
            summary: None,
            timeout: None,
            urgency: Some("normal".to_string()),
        };

        let tc = TomlConfig {
            profile: Some(vec![tp]),
        };

        let c = Config::from_toml("test", &tc);
        assert_eq!(c.urgency, Urgency::Normal);
    }

    #[test]
    fn urgency_value_critical() {
        let tp = TomlProfile {
            name: "test".to_string(),
            body: None,
            icon: None,
            summary: None,
            timeout: None,
            urgency: Some("critical".to_string()),
        };

        let tc = TomlConfig {
            profile: Some(vec![tp]),
        };

        let c = Config::from_toml("test", &tc);
        assert_eq!(c.urgency, Urgency::Critical);
    }
}
