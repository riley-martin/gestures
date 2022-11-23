use std::{
    env, fs,
    path::{Path, PathBuf},
    str::FromStr,
};

use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use serde_lexpr::{from_str, to_string};

use crate::gestures::{Direction, Gesture};

#[derive(Serialize, Deserialize, PartialEq, Debug, Default)]
pub struct Config {
    pub device: Option<String>,
    pub gestures: Vec<Gesture>,
}

impl Config {
    pub fn new(device: Option<String>, gestures: Vec<Gesture>) -> Self {
        Self { device, gestures }
    }

    pub fn read_from_file(file: &PathBuf) -> Self {
        todo!();
    }

    pub fn read_default_config() -> Result<Self> {
        let home = env::var("HOME")?;
        let path = &format!("{}/.config/gestures.conf", home);
        if let Ok(s) = fs::read_to_string(Path::new(path)) {
            return Ok(from_str(&s)?);
        }
        let path = &format!("{}/.config/gestures/gestures.conf", home);
        if let Ok(s) = fs::read_to_string(Path::new(path)) {
            return Ok(from_str(&s)?);
        }
        let path = &format!("{}/.gestures.conf", home);
        if let Ok(s) = fs::read_to_string(Path::new(path)) {
            return Ok(from_str(&s)?);
        }
        bail!("could not find config file")
    }
}
