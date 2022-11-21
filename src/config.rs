use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use serde_lexpr::{from_str, to_string};

#[derive(Serialize, Deserialize, PartialEq)]
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

#[derive(Serialize, Deserialize, PartialEq)]
pub struct Gesture {
    direction: Direction,
    fingers: u8,
    action: String,
}

#[derive(Serialize, Deserialize, PartialEq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}
