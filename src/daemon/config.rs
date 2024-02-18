use serde::{self, Deserialize};

pub fn default_config() -> &'static str {
    r#"max_items=100"#
}

#[derive(Clone, Debug, Deserialize)]
pub struct Config {
    pub max_items: usize,
}

impl oxilib::Config<ConfigOptional> for Config {
    fn create_from_optional(optional: ConfigOptional) -> Self {
        let max_items = if let Some(items) = optional.max_items {
            items
        } else {
            100
        };
        Self { max_items }
    }
}

#[derive(Debug, Deserialize)]
pub struct ConfigOptional {
    max_items: Option<usize>,
}

impl oxilib::ConfigOptional for ConfigOptional {}
