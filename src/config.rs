use serde::Deserialize;
use std::{collections::HashMap, fs};

#[derive(Deserialize)]
pub struct Config {
    known_languages: HashMap<String, KnownLanguage>,
}

impl Config {
    pub fn new() -> Self {
        Config {
            known_languages: HashMap::new(),
        }
    }

    pub fn get(&self, name: &str) -> Option<&KnownLanguage> {
        self.known_languages.get(name)
    }

    pub fn find_struct_by_key(&self, extension: &str) -> Option<&KnownLanguage> {
        self.known_languages
            .values()
            .find(|s| s.extension == extension)
    }
}

#[derive(Deserialize)]
pub struct KnownLanguage {
    pub path: String,
    pub language: String,
    pub extension: String,
    pub comment_types: Vec<String>
}

impl KnownLanguage {
    pub fn new() -> Self {
        KnownLanguage {
            path: String::new(),
            language: String::new(),
            extension: String::new(),
            comment_types: Vec::new(),
        }
    }
}

pub fn load_config(path: &str) -> Result<Config, anyhow::Error> {
    let config_str = fs::read_to_string(path)?;
    let config: Config = serde_json::from_str(&config_str)?;
    Ok(config)
}
