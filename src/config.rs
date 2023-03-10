use std::{fs, path::PathBuf};

use anyhow::Result;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct ConfigFile {
    pub radius: Option<String>,
    pub padding_x: Option<String>,
    pub padding_y: Option<String>,
    pub blur_x: Option<String>,
    pub blur_y: Option<String>,
    pub shadow_color: Option<String>,
    pub offset_x: Option<String>,
    pub offset_y: Option<String>,
    pub input: Option<String>,
    pub output: Option<String>,
}

impl ConfigFile {
    pub fn read(path: PathBuf) -> Result<Self> {
        let content = fs::read_to_string(path)?;

        let config: ConfigFile = toml::from_str(&content)?;

        Ok(config)
    }
}
