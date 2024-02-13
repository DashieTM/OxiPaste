use serde::{self, Deserialize};
use std::fs;
use toml;

fn default_config() -> String {
    format!(
        r#"max_items=100
        "#,
    )
}

#[derive(Deserialize, Clone)]
pub struct Config {
    pub max_items: usize,
}

#[derive(Deserialize)]
pub struct ConfigOptional {
    max_items: Option<usize>,
}

pub fn parse_config() -> Config {
    let base = directories_next::BaseDirs::new().unwrap();
    let config_dir = base.config_dir();
    if !config_dir.is_dir() {
        fs::create_dir(config_dir).expect("Could not create config folder");
    }
    let config_file = &config_dir.join("oxipaste/config.toml");
    if !config_file.is_file() {
        fs::File::create(config_file).expect("Could not create config file");
    }
    let contents = match fs::read_to_string(config_file) {
        Ok(c) => c,
        Err(_) => default_config(),
    };
    let parsed_conf: ConfigOptional = match toml::from_str(&contents) {
        Ok(d) => d,
        Err(_) => toml::from_str(&default_config()).unwrap(),
    };
    Config {
        max_items: parsed_conf.max_items.unwrap_or(100),
    }
}
