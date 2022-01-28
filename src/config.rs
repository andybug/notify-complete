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
    timeout: Option<i32>,
    urgency: Option<String>,
}

#[derive(Debug, Deserialize)]
struct TomlConfig {
    profile: Option<Vec<TomlProfile>>,
}

pub struct Profile {
    pub name: String,
    pub body: String,
    pub icon: String,
    pub summary: String,
    pub timeout: Timeout,
    pub urgency: Urgency,
}

pub struct Config {
    pub profile: Profile,
}

impl Profile {
    fn default_profile() -> Profile {
        Profile {
            name: String::from("default"),
            body: Profile::default_body(),
            icon: Profile::default_icon(),
            summary: Profile::default_summary(),
            timeout: Profile::default_timeout(),
            urgency: Profile::default_urgency(),
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

    fn from_toml(other: &TomlProfile) -> Profile {
        let name = String::from(&other.name);

        let body = match &other.body {
            Some(body) => String::from(body),
            None => Profile::default_body(),
        };

        let icon = match &other.icon {
            Some(icon) => String::from(icon),
            None => Profile::default_icon(),
        };

        let summary = match &other.summary {
            Some(summary) => String::from(summary),
            None => Profile::default_summary(),
        };

        let timeout = match &other.timeout {
            Some(timeout) => match *timeout {
                -1 => Timeout::Default,
                0 => Timeout::Never,
                _ => {
                    if *timeout < 0 {
                        eprintln!("Invalid timeout value {}", timeout);
                        Profile::default_timeout()
                    } else {
                        Timeout::Milliseconds(*timeout as u32)
                    }
                }
            },
            None => Profile::default_timeout(),
        };

        let urgency = match &other.urgency {
            Some(urgency) => match urgency.as_str() {
                "low" => Urgency::Low,
                "normal" => Urgency::Normal,
                "critical" => Urgency::Critical,
                _ => {
                    eprintln!("Invalid urgency setting '{}'", urgency);
                    Profile::default_urgency()
                }
            },
            None => Profile::default_urgency(),
        };

        Profile {
            name: name,
            body: body,
            icon: icon,
            summary: summary,
            timeout: timeout,
            urgency: urgency,
        }
    }
}

impl Config {
    fn default_config() -> Config {
        Config {
            profile: Profile::default_profile(),
        }
    }

    fn merge_toml(&mut self, target_profile: &str, other: &TomlConfig) {
        if other.profile.is_none() {
            // config file contained no profiles, we can safely return
            return;
        }

        for profile in other.profile.as_ref().unwrap() {
            if profile.name == target_profile {
                self.profile = Profile::from_toml(profile);
            }
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

pub fn get_config(profile_name: &str) -> Config {
    let mut config = Config::default_config();
    let config_path = get_config_path();

    match read_config_file(config_path.as_path()) {
        Some(toml_config) => config.merge_toml(profile_name, &toml_config),
        None => (),
    }

    return config;
}

#[cfg(test)]
mod tests {
    use super::{Profile, TomlProfile};

    #[test]
    fn toml_profile_defaults() {
        let tp = TomlProfile {
            name: "test".to_string(),
            body: None,
            icon: None,
            summary: None,
            timeout: None,
            urgency: None,
        };

        let p = Profile::from_toml(&tp);

        assert_eq!(p.body, Profile::default_body());
        assert_eq!(p.icon, Profile::default_icon());
        assert_eq!(p.summary, Profile::default_summary());
        assert_eq!(p.timeout, Profile::default_timeout());
        assert_eq!(p.urgency, Profile::default_urgency());
    }
}
