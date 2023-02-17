use std::{env, fs, path::Path};

use miette::{bail, IntoDiagnostic, Result};
use serde::{Deserialize, Serialize};
use serde_lexpr::from_str;

use crate::gestures::Gesture;

#[derive(Serialize, Deserialize, PartialEq, Debug, Default)]
pub struct Config {
    pub device: Option<String>,
    pub gestures: Vec<Gesture>,
}

impl Config {
    pub fn read_from_file(file: &Path) -> Result<Self> {
        log::debug!("{:?}", &file);
        match fs::read_to_string(file) {
            Ok(s) => Ok(from_str(&s).into_diagnostic()?),
            _ => bail!("Could not read config file"),
        }
    }

    pub fn read_default_config() -> Result<Self> {
        let home = env::var("HOME").into_diagnostic()?;

        log::debug!("{:?}", &home);

        let path = &format!("{home}/.config/gestures.conf");
        if let Ok(s) = Self::read_from_file(Path::new(path)) {
            return Ok(s);
        }

        let path = &format!("{home}/.config/gestures/gestures.conf");
        if let Ok(s) = Self::read_from_file(Path::new(path)) {
            return Ok(s);
        }

        let path = &format!("{home}/.gestures.conf");
        if let Ok(s) = Self::read_from_file(Path::new(path)) {
            return Ok(s);
        }

        bail!("could not find config file")
    }
}
