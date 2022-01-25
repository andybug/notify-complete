use std::path::{Path, PathBuf};
use std::vec::Vec;
use std::fs;
use serde_derive::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
pub struct Profile {
  pub name: String,
}

pub struct Config {
  pub profiles: HashMap<String, Profile>,
}

#[derive(Debug, Deserialize)]
struct TomlConfig {
  pub profile: Option<Vec<Profile>>,
}

fn default_config() -> Config {
  let mut profiles = HashMap::new();
  profiles.insert(String::from("default"), Profile { name: String::from("default") });

  Config {
    profiles: profiles,
  }
}

fn get_config_path() -> PathBuf {
  const CONFIG_DIR: &str = "notify-complete";
  const CONFIG_FILE: &str = "config.toml";

  let mut config_path = match dirs::config_dir() {
    Some(dir) => dir,
    None => {
      let default_path = format!("~/.config/{}/{}", CONFIG_DIR, CONFIG_FILE);
      eprintln!("Could not determine user config directory -- using {}", default_path);
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
      eprintln!("Error reading config file '{}': {}", path.to_str().unwrap(), e);
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

pub fn get_config() -> Config {
  let mut config = default_config();
  let config_path = get_config_path();

  match read_config_file(config_path.as_path()) {
    Some(toml_config) => {
      if toml_config.profile.is_some() {
        for profile in toml_config.profile.unwrap() {
          config.profiles.insert(String::from(&profile.name), profile);
        }
      }
    }
    None => (),
  }

  return config;
}