use std::{
    fs::{File, OpenOptions},
    os::unix::prelude::{AsRawFd, FromRawFd, IntoRawFd, OpenOptionsExt, RawFd},
    path::Path,
    process::Command,
    rc::Rc,
};

use input::{
    event::{
        gesture::{
            GestureEndEvent, GestureEventCoordinates, GestureEventTrait, GestureHoldEvent,
            GesturePinchEvent, GesturePinchEventTrait, GestureSwipeEvent,
        },
        Event, EventTrait, GestureEvent,
    },
    DeviceCapability, Libinput, LibinputInterface,
};
use libc::{O_RDWR, O_WRONLY};
use nix::poll::{poll, PollFd, PollFlags};
use serde::{Deserialize, Serialize};

use crate::config::Config;

// Tiny little macro to keep from having to write if statements everywhere
#[macro_export]
macro_rules! if_debug {
    ($d:expr, $($item:expr),*) => {
        if $d {
            dbg!($($item,)*);
            eprintln!();
        }
    }
}

/// Direction of swipe gestures
///
/// NW  N  NE
/// W   C   E
/// SW  S  SE
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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

impl Direction {
    // This code is sort of a mess
    pub fn dir(x: f64, y: f64) -> Direction {
        // ratio at which a gesture is called diagonal
        if x.abs() == 0.0 && y.abs() == 0.0 {
            return Direction::C;
        }
        let oblique_ratio = 0.414;
        if x.abs() > y.abs() {
            let sd = if x < 0.0 { Direction::W } else { Direction::E };
            if y.abs() / x.abs() > oblique_ratio {
                if sd == Direction::W {
                    if y < 0.0 {
                        Direction::NW
                    } else {
                        Direction::SW
                    }
                } else if sd == Direction::E {
                    if y < 0.0 {
                        Direction::NE
                    } else {
                        Direction::SE
                    }
                } else {
                    Direction::C
                }
            } else {
                sd
            }
        } else {
            // Don't ask me why, but for libinput the coordinates increase downward. This does
            // hold out the same as screen coordinates, but it starts in the center instead of
            // the upper left
            let sd = if y < 0.0 { Direction::N } else { Direction::S };
            if x.abs() / y.abs() > oblique_ratio {
                if sd == Direction::N {
                    if x < 0.0 {
                        Direction::NW
                    } else {
                        Direction::NE
                    }
                } else if sd == Direction::S {
                    if x < 0.0 {
                        Direction::SW
                    } else {
                        Direction::SE
                    }
                } else {
                    Direction::C
                }
            } else {
                sd
            }
        }
    }
}

/// Direction of pinch gestures
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InOut {
    In,
    Out,
    None,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Repeat {
    Oneshot,
    Continuous,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Gesture {
    Swipe(Swipe),
    Pinch(Pinch),
    Hold(Hold),
    Rotate(Rotate),
    None,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Swipe {
    pub direction: Direction,
    pub fingers: i32,
    pub repeat: Repeat,
    pub action: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pinch {
    pub scale: f64,
    pub fingers: i32,
    pub direction: InOut,
    pub repeat: Repeat,
    pub action: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Hold {
    pub fingers: i32,
    pub action: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Rotate {
    pub scale: f64,
    pub fingers: i32,
    pub delta_angle: f64,
    pub repeat: Repeat,
    pub action: String,
}

#[derive(Debug)]
pub struct EventHandler {
    config: Rc<Config>,
    event: Gesture,
    debug: bool,
}

// This looks almost too much like a 'god object'
impl EventHandler {
    pub fn new(config: Rc<Config>, debug: bool) -> Self {
        Self {
            config,
            event: Gesture::None,
            debug,
        }
    }

    pub fn init(&mut self, input: &mut Libinput) -> Result<(), String> {
        if_debug!(self.debug, &self, &input);
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
        for event in input.clone() {
            if let Event::Device(e) = event {
                if_debug!(self.debug, &e);
                found = e.device().has_capability(DeviceCapability::Gesture);
                if_debug!(self.debug, found);
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
                    GestureEvent::Pinch(e) => self.handle_pinch_event(e),
                    GestureEvent::Swipe(e) => self.handle_swipe_event(e),
                    GestureEvent::Hold(e) => self.handle_hold_event(e),
                    _ => (),
                }
            }
            input.dispatch().unwrap();
        }
    }

    fn handle_hold_event(&mut self, event: GestureHoldEvent) {
        match event {
            GestureHoldEvent::Begin(e) => {
                self.event = Gesture::Hold(Hold {
                    fingers: e.finger_count(),
                    action: "".to_string(),
                })
            }
            GestureHoldEvent::End(_e) => {
                if let Gesture::Hold(s) = &self.event {
                    if_debug!(self.debug, "Hold", &s.fingers);
                    for i in &self.config.clone().gestures {
                        if let Gesture::Hold(j) = i {
                            if j.fingers == s.fingers {
                                exec_command_from_string(&j.action);
                            }
                        }
                    }
                }
            }
            _ => (),
        }
    }

    fn handle_pinch_event(&mut self, event: GesturePinchEvent) {
        match event {
            GesturePinchEvent::Begin(e) => {
                self.event = Gesture::Pinch(Pinch {
                    fingers: e.finger_count(),
                    scale: 0.0,
                    direction: InOut::None,
                    repeat: Repeat::Oneshot,
                    action: "".to_string(),
                })
            }
            GesturePinchEvent::Update(e) => {
                let scale = e.scale();
                if let Gesture::Pinch(s) = &self.event {
                    let dir = if scale > 1.0 { InOut::Out } else { InOut::In };
                    if_debug!(self.debug, &scale, &dir, &s.fingers);
                    for i in &self.config.clone().gestures {
                        if let Gesture::Pinch(j) = i {
                            if j.direction == dir
                                && j.fingers == s.fingers
                                && j.repeat == Repeat::Continuous
                            {
                                if_debug!(self.debug, "continuous pinch gesture");
                                exec_command_from_string(&j.action);
                            }
                        }
                    }
                    self.event = Gesture::Pinch(Pinch {
                        fingers: s.fingers,
                        scale: 0.0,
                        direction: dir,
                        repeat: Repeat::Oneshot,
                        action: "".to_string(),
                    })
                }
            }
            GesturePinchEvent::End(_e) => {
                if let Gesture::Pinch(s) = &self.event {
                    for i in &self.config.clone().gestures {
                        if let Gesture::Pinch(j) = i {
                            if j.direction == s.direction && j.fingers == s.fingers {
                                if_debug!(self.debug, "oneshot pinch gesture");
                                exec_command_from_string(&j.action);
                            }
                        }
                    }
                }
            }
            _ => (),
        }
    }

    fn handle_swipe_event(&mut self, event: GestureSwipeEvent) {
        match event {
            GestureSwipeEvent::Begin(e) => {
                self.event = Gesture::Swipe(Swipe {
                    direction: Direction::C,
                    fingers: e.finger_count(),
                    repeat: Repeat::Oneshot,
                    action: "".to_string(),
                });
            }
            GestureSwipeEvent::Update(e) => {
                let (x, y) = (e.dx(), e.dy());
                let swipe_dir = Direction::dir(x, y);

                if let Gesture::Swipe(s) = &self.event {
                    if_debug!(self.debug, &swipe_dir, &s.fingers);
                    for i in &self.config.clone().gestures {
                        if let Gesture::Swipe(j) = i {
                            if j.fingers == s.fingers
                                && j.direction == swipe_dir
                                && j.repeat == Repeat::Continuous
                            {
                                exec_command_from_string(&j.action);
                            }
                        }
                    }
                    self.event = Gesture::Swipe(Swipe {
                        direction: swipe_dir,
                        fingers: s.fingers,
                        repeat: Repeat::Oneshot,
                        action: "".to_string(),
                    })
                }
            }
            GestureSwipeEvent::End(e) => {
                if let Gesture::Swipe(s) = &self.event {
                    if !e.cancelled() {
                        for i in &self.config.clone().gestures {
                            if let Gesture::Swipe(j) = i {
                                if j.fingers == s.fingers && j.direction == s.direction {
                                    exec_command_from_string(&j.action);
                                }
                            }
                        }
                    }
                }
            }
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
    // let args = s.split(' ');
    // dbg!(&args);
    Command::new("sh")
        .arg("-c")
        .arg(s)
        .spawn()
        .expect("Could not execute external command");
}
