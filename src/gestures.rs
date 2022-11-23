use libc::{O_RDONLY, O_RDWR, O_WRONLY};
use nix::poll::{poll, PollFd, PollFlags};
use std::{
    fs::{File, OpenOptions},
    os::unix::prelude::{AsRawFd, FromRawFd, IntoRawFd, OpenOptionsExt, RawFd},
    path::Path,
    process::Command,
    rc::Rc,
};

use crate::config::Config;
use input::{
    event::{
        gesture::{
            GestureEventCoordinates, GestureEventTrait, GestureHoldEvent, GestureSwipeEvent,
        },
        Event, EventTrait, GestureEvent,
    },
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
pub enum GesType {
    Swipe,
    Hold,
    Pinch,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Gesture {
    pub direction: Direction,
    pub fingers: i32,
    pub action: String,
    pub gesture_type: GesType,
}

#[derive(Debug)]
pub struct EventHandler {
    config: Rc<Config>,
    event: Gesture,
    debug: bool,
}

impl EventHandler {
    pub fn new(config: Rc<Config>, debug: bool) -> Self {
        Self {
            config,
            event: Gesture {
                direction: Direction::C,
                fingers: 0,
                action: "".to_string(),
                gesture_type: GesType::Swipe,
            },
            debug,
        }
    }

    pub fn init(&mut self, input: &mut Libinput) -> Result<(), String> {
        self.init_ctx(input).expect("Could not initialize libinput");
        if self.has_gesture_device(input) {
            Ok(())
        } else {
            Err("Could not find gesture device".to_string())
        }
    }

    fn init_ctx(&mut self, input: &mut Libinput) -> Result<(), ()> {
        input.udev_assign_seat("seat0")?;
        Ok(())
    }

    fn has_gesture_device(&mut self, input: &mut Libinput) -> bool {
        let mut found = false;
        input.dispatch().unwrap();
        if self.debug {
            dbg!(&input);
        };
        for event in input.clone() {
            if let Event::Device(e) = event {
                if self.debug {
                    dbg!(&e);
                }
                found = e.device().has_capability(DeviceCapability::Gesture);
                if self.debug {
                    dbg!(found);
                }
                if found {
                    return found;
                }
            } else {
                continue;
            }
            input.dispatch().unwrap();
        }
        found
    }

    pub fn main_loop(&mut self, input: &mut Libinput) {
        let fds = PollFd::new(input.as_raw_fd(), PollFlags::POLLIN);
        while poll(&mut [fds], -1).is_ok() {
            self.handle_event(input);
        }
    }

    pub fn handle_event(&mut self, input: &mut Libinput) {
        input.dispatch().unwrap();
        for event in input.clone() {
            if let Event::Gesture(e) = event {
                match e {
                    GestureEvent::Pinch(_) => (),
                    GestureEvent::Swipe(e) => self.handle_swipe_event(e),
                    GestureEvent::Hold(_) => (),
                    _ => (),
                }
            }
            input.dispatch().unwrap();
        }
    }

    fn handle_swipe_event(&mut self, event: GestureSwipeEvent) {
        match event {
            GestureSwipeEvent::Begin(e) => {
                self.event.fingers = e.finger_count();
            }
            GestureSwipeEvent::Update(e) => {
                let (x, y) = (e.dx(), e.dy());
                let oblique_ratio = 0.414;
                let mut swipe_dir = Direction::C;

                // Needs refactored
                if x.abs() > y.abs() {
                    swipe_dir = if x < 0.0 { Direction::W } else { Direction::E };
                    if y.abs() / x.abs() > oblique_ratio {
                        if swipe_dir == Direction::W {
                            swipe_dir = if y < 0.0 {
                                Direction::NW
                            } else {
                                Direction::SW
                            };
                        } else if swipe_dir == Direction::E {
                            swipe_dir = if y < 0.0 {
                                Direction::NE
                            } else {
                                Direction::SE
                            };
                        }
                    }
                } else {
                    swipe_dir = if y < 0.0 { Direction::N } else { Direction::S };
                    if x.abs() / y.abs() > oblique_ratio {
                        if swipe_dir == Direction::N {
                            swipe_dir = if x < 0.0 {
                                Direction::NW
                            } else {
                                Direction::NE
                            };
                        } else if swipe_dir == Direction::S {
                            swipe_dir = if x < 0.0 {
                                Direction::SW
                            } else {
                                Direction::SE
                            };
                        }
                    }
                }
                if self.debug {
                    dbg!(GesType::Swipe, &swipe_dir, self.event.fingers);
                }
                for i in &self.config.clone().gestures {
                    if i.gesture_type == GesType::Swipe
                        && i.direction == swipe_dir
                        && i.fingers == self.event.fingers
                    {
                        exec_command_from_string(&i.action)
                    }
                }
            }
            GestureSwipeEvent::End(_) => (),
            _ => (),
        }
    }
}

pub struct Interface;

impl LibinputInterface for Interface {
    fn open_restricted(&mut self, path: &Path, flags: i32) -> Result<RawFd, i32> {
        OpenOptions::new()
            .custom_flags(flags)
            .read((false) | (flags & O_RDWR != 0))
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

pub fn exec_command_from_string(s: &str) {
    let args = s.split(" ");
    Command::new("sh")
        .arg("-c")
        .args(args)
        .spawn()
        .expect("Could not execute external command");
}
