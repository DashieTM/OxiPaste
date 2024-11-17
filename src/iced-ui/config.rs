use std::{
    fs::{self, OpenOptions},
    path::PathBuf,
};

use serde::Deserialize;

use crate::{into_general_error, OxiPasteError};

#[derive(Deserialize, Clone, Default, Debug)]
#[allow(non_snake_case)]
pub struct Config {
    pub keepOpen: bool,
    pub PlainTextContextActions: Vec<Vec<String>>,
    pub AddressContextActions: Vec<Vec<String>>,
    pub ImageContextActions: Vec<Vec<String>>,
}

#[derive(Deserialize, Default)]
#[allow(non_snake_case)]
struct ConfigOptional {
    keepOpen: Option<bool>,
    PlainTextContextActions: Option<Vec<Vec<String>>>,
    AddressContextActions: Option<Vec<Vec<String>>>,
    ImageContextActions: Option<Vec<Vec<String>>>,
}

pub fn default_config() -> Config {
    Config {
        keepOpen: false,
        PlainTextContextActions: vec![vec!["notify".into(), "notify-send".into()]],
        AddressContextActions: vec![
            vec!["open".into(), "xdg-open".into()],
            vec!["notify".into(), "notify-send".into()],
        ],
        ImageContextActions: vec![vec![
            "satty".into(),
            "sh".into(),
            "-c".into(),
            "wl-paste | satty -f -".into(),
        ]],
    }
}

pub fn parse_config(path: &PathBuf) -> Result<Config, Option<OxiPasteError>> {
    let contents = fs::read_to_string(path);
    if contents.is_err() {
        return Err(into_general_error(contents.err()));
    }
    let parsed_conf: ConfigOptional = match toml::from_str(&contents.unwrap()) {
        Ok(d) => d,
        Err(error) => {
            return Err(into_general_error(Some(error)));
        }
    };
    let default_conf = default_config();
    Ok(Config {
        keepOpen: parsed_conf.keepOpen.unwrap_or(default_conf.keepOpen),
        PlainTextContextActions: parsed_conf
            .PlainTextContextActions
            .unwrap_or(default_conf.PlainTextContextActions),
        AddressContextActions: parsed_conf
            .AddressContextActions
            .unwrap_or(default_conf.AddressContextActions),
        ImageContextActions: parsed_conf
            .ImageContextActions
            .unwrap_or(default_conf.ImageContextActions),
    })
}

pub fn create_config_dir() -> Result<PathBuf, Option<OxiPasteError>> {
    let base_dir = xdg::BaseDirectories::new();
    if let Err(error) = base_dir {
        return Err(into_general_error(Some(error)));
    }
    let base_dir = base_dir.unwrap().get_config_home();
    let project_dir = base_dir.join("oxipaste");
    let res = fs::create_dir_all(&project_dir);
    if let Err(error) = res {
        return Err(into_general_error(Some(error)));
    }
    Ok(project_dir)
}

pub fn create_config() -> Result<PathBuf, Option<OxiPasteError>> {
    let config_dir = create_config_dir()?;
    let config_file = config_dir.join("config.toml");
    if !config_file.is_file() {
        let res = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&config_file);
        if let Err(error) = res {
            return Err(into_general_error(Some(error)));
        }
    }
    if let Err(error) = config_file.metadata() {
        return Err(into_general_error(Some(error)));
    }
    Ok(config_file)
}
