use libc::{O_RDONLY, O_RDWR, O_WRONLY};
use std::{
    fs::{File, OpenOptions},
    os::unix::prelude::{FromRawFd, IntoRawFd, OpenOptionsExt, RawFd},
    path::Path,
    rc::Rc,
};

use crate::config::Config;
use input::{
    event::{Event, EventTrait, GestureEvent},
    DeviceCapability, Libinput, LibinputInterface,
};
use serde::{Deserialize, Serialize};
/// Direction of gestures
///
/// NW  N  NE
/// W   C   E
/// SW  S  SE
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Direction {
    /// No swipe
    C,
    N,
    S,
    E,
    W,
    NE,
    NW,
    SE,
    SW,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Gesture {
    direction: Direction,
    fingers: u8,
    action: String,
}

#[derive(Debug)]
pub struct EventHandler {
    config: Rc<Config>,
    event: Option<GestureEvent>,
    input: Option<Libinput>,
}

impl EventHandler {
    pub fn new(config: Rc<Config>) -> Self {
        Self {
            config,
            event: None,
            input: None,
        }
    }

    pub fn init(&mut self) -> Result<(), String> {
        self.init_ctx().expect("Could not initialize libinput");
        if self.has_gesture_device() {
            Ok(())
        } else {
            Err("Could not find gesture device".to_string())
        }
    }

    fn init_ctx(&mut self) -> Result<(), ()> {
        self.input = Some(Libinput::new_with_udev(Self::new(Rc::new(
            Config::default(),
        ))));
        self.input.unwrap().udev_assign_seat("seat0")?;
        Ok(())
    }

    fn has_gesture_device(&mut self) -> bool {
        let mut found = false;
        for event in self.input.unwrap() {
            if let Event::Device(e) = event {
                found = e.device().has_capability(DeviceCapability::Gesture);
            } else {
                continue;
            }
            self.input.unwrap().dispatch().unwrap();
            drop(event);
        }
        found
    }
}

impl LibinputInterface for EventHandler {
    fn open_restricted(&mut self, path: &Path, flags: i32) -> Result<RawFd, i32> {
        OpenOptions::new()
            .custom_flags(flags)
            .read((flags & O_RDONLY != 0) | (flags & O_RDWR != 0))
            .write((flags & O_WRONLY != 0) | (flags & O_RDWR != 0))
            .open(path)
            .map(|file| file.into_raw_fd())
            .map_err(|err| err.raw_os_error().unwrap())
    }
    fn close_restricted(&mut self, fd: RawFd) {
        unsafe {
            File::from_raw_fd(fd);
        }
    }
}
