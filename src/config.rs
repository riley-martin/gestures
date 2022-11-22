use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use serde_lexpr::{from_str, to_string};

use crate::gestures::{Direction, Gesture};

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Config {
    device: Option<String>,
    gestures: Vec<Gesture>,
}

impl Config {
    pub fn new(device: Option<String>, gestures: Vec<Gesture>) -> Self {
        Self { device, gestures }
    }

    pub fn new_from_file(file: PathBuf) -> Self {
        todo!();
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            device: None,
            gestures: vec![],
        }
    }
}
