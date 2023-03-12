use std::{
    fs::{File, OpenOptions},
    os::unix::prelude::{AsRawFd, FromRawFd, IntoRawFd, OpenOptionsExt, RawFd},
    path::Path,
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
use miette::{miette, Result};
use nix::poll::{poll, PollFd, PollFlags};
// use serde::{Deserialize, Serialize};
use knuffel::{Decode, DecodeScalar};

use crate::config::Config;
use crate::utils::exec_command_from_string;

/// Direction of swipe gestures
///
/// NW  N  NE
/// W   C   E
/// SW  S  SE
#[derive(DecodeScalar, Debug, Clone, PartialEq, Eq)]
pub enum SwipeDir {
    Any,
    N,
    S,
    E,
    W,
    NE,
    NW,
    SE,
    SW,
}

impl SwipeDir {
    // This code is sort of a mess
    pub fn dir(x: f64, y: f64) -> SwipeDir {
        if x.abs() == 0.0 && y.abs() == 0.0 {
            return SwipeDir::Any;
        }
        let oblique_ratio = 0.414;
        if x.abs() > y.abs() {
            let sd = if x < 0.0 { SwipeDir::W } else { SwipeDir::E };
            if y.abs() / x.abs() > oblique_ratio {
                if sd == SwipeDir::W {
                    if y < 0.0 {
                        SwipeDir::NW
                    } else {
                        SwipeDir::SW
                    }
                } else if sd == SwipeDir::E {
                    if y < 0.0 {
                        SwipeDir::NE
                    } else {
                        SwipeDir::SE
                    }
                } else {
                    SwipeDir::Any
                }
            } else {
                sd
            }
        } else {
            let sd = if y < 0.0 { SwipeDir::N } else { SwipeDir::S };
            if x.abs() / y.abs() > oblique_ratio {
                if sd == SwipeDir::N {
                    if x < 0.0 {
                        SwipeDir::NW
                    } else {
                        SwipeDir::NE
                    }
                } else if sd == SwipeDir::S {
                    if x < 0.0 {
                        SwipeDir::SW
                    } else {
                        SwipeDir::SE
                    }
                } else {
                    SwipeDir::Any
                }
            } else {
                sd
            }
        }
    }
}

/// Direction of pinch gestures
#[derive(DecodeScalar, Debug, Clone, PartialEq, Eq)]
pub enum PinchDir {
    In,
    Out,
    Clockwise,
    CounterClockwise,
    Any,
}

impl PinchDir {
    pub fn dir(scale: f64, delta_angle: f64) -> Self {
        // If the scale is low enough, see if there is any rotation
        // These values seem to work fairly well overall
        if (scale > 0.92) && (scale < 1.08) {
            if delta_angle > 0.0 {
                Self::Clockwise
            } else {
                Self::CounterClockwise
            }
        // Otherwise we have a normal pinch
        } else {
            if scale > 1.0 {
                Self::Out
            } else {
                Self::In
            }
        }
    }
}


#[derive(Decode, Debug, Clone, PartialEq)]
pub enum Gesture {
    Swipe(Swipe),
    Pinch(Pinch),
    Hold(Hold),
    // Rotate(Rotate),
    None,
}

#[derive(Decode, Debug, Clone, PartialEq, Eq)]
pub struct Swipe {
    #[knuffel(property)]
    pub direction: SwipeDir,
    #[knuffel(property)]
    pub fingers: i32,
    #[knuffel(property)]
    pub update: Option<String>,
    #[knuffel(property)]
    pub start: Option<String>,
    #[knuffel(property)]
    pub end: Option<String>,
}

#[derive(Decode, Debug, Clone, PartialEq, Eq)]
pub struct Pinch {
    #[knuffel(property)]
    pub fingers: i32,
    #[knuffel(property)]
    pub direction: PinchDir,
    #[knuffel(property)]
    pub update: Option<String>,
    #[knuffel(property)]
    pub start: Option<String>,
    #[knuffel(property)]
    pub end: Option<String>,
}

#[derive(Decode, Debug, Clone, PartialEq, Eq)]
pub struct Hold {
    #[knuffel(property)]
    pub fingers: i32,
    #[knuffel(property)]
    pub action: Option<String>,
}

#[derive(Debug)]
pub struct EventHandler {
    config: Rc<Config>,
    event: Gesture,
}

impl EventHandler {
    pub fn new(config: Rc<Config>) -> Self {
        Self {
            config,
            event: Gesture::None,
        }
    }

    pub fn init(&mut self, input: &mut Libinput) -> Result<()> {
        log::debug!("{:?}  {:?}", &self, &input);
        self.init_ctx(input).expect("Could not initialize libinput");
        if self.has_gesture_device(input) {
            Ok(())
        } else {
            Err(miette!("Could not find gesture device"))
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
                log::debug!("{:?}", &e);
                found = e.device().has_capability(DeviceCapability::Gesture);
                log::debug!("{:?}", found);
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
            self.handle_event(input)
                .expect("An Error occurred while handling an event");
        }
    }

    pub fn handle_event(&mut self, input: &mut Libinput) -> Result<()> {
        input.dispatch().unwrap();
        for event in input.clone() {
            if let Event::Gesture(e) = event {
                match e {
                    GestureEvent::Pinch(e) => self.handle_pinch_event(e)?,
                    GestureEvent::Swipe(e) => self.handle_swipe_event(e)?,
                    GestureEvent::Hold(e) => self.handle_hold_event(e)?,
                    _ => (),
                }
            }
            input.dispatch().unwrap();
        }
        Ok(())
    }

    fn handle_hold_event(&mut self, event: GestureHoldEvent) -> Result<()> {
        match event {
            GestureHoldEvent::Begin(e) => {
                self.event = Gesture::Hold(Hold {
                    fingers: e.finger_count(),
                    action: None,
                })
            }
            GestureHoldEvent::End(_e) => {
                if let Gesture::Hold(s) = &self.event {
                    log::debug!("Hold  {:?}", &s.fingers);
                    for i in &self.config.clone().gestures {
                        if let Gesture::Hold(j) = i {
                            if j.fingers == s.fingers {
                                exec_command_from_string(
                                    &j.action.clone().unwrap_or_default(),
                                    0.0,
                                    0.0,
                                    0.0,
                                    0.0,
                                )?;
                            }
                        }
                    }
                }
            }
            _ => (),
        }
        Ok(())
    }

    fn handle_pinch_event(&mut self, event: GesturePinchEvent) -> Result<()> {
        match event {
            GesturePinchEvent::Begin(e) => {
                self.event = Gesture::Pinch(Pinch {
                    fingers: e.finger_count(),
                    direction: PinchDir::Any,
                    update: None,
                    start: None,
                    end: None,
                });
                if let Gesture::Pinch(s) = &self.event {
                    for i in &self.config.clone().gestures {
                        if let Gesture::Pinch(j) = i {
                            if (j.direction == s.direction || j.direction == PinchDir::Any)
                                && j.fingers == s.fingers
                            {
                                log::debug!("pinch gesture");
                                exec_command_from_string(
                                    &j.start.clone().unwrap_or_default(),
                                    0.0,
                                    0.0,
                                    0.0,
                                    0.0,
                                )?;
                            }
                        }
                    }
                }
            }
            GesturePinchEvent::Update(e) => {
                let scale = e.scale();
                let delta_angle = e.angle_delta();
                if let Gesture::Pinch(s) = &self.event {
                    let dir = PinchDir::dir(scale, delta_angle);
                    log::debug!("{:?}  {:?}  {:?}", &scale, &dir, &s.fingers);
                    for i in &self.config.clone().gestures {
                        if let Gesture::Pinch(j) = i {
                            if (j.direction == dir || j.direction == PinchDir::Any)
                                && j.fingers == s.fingers
                            {
                                exec_command_from_string(
                                    &j.update.clone().unwrap_or_default(),
                                    0.0,
                                    0.0,
                                    delta_angle,
                                    scale,
                                )?;
                            }
                        }
                    }
                    self.event = Gesture::Pinch(Pinch {
                        fingers: s.fingers,
                        direction: dir,
                        update: None,
                        start: None,
                        end: None,
                    })
                }
            }
            GesturePinchEvent::End(_e) => {
                if let Gesture::Pinch(s) = &self.event {
                    for i in &self.config.clone().gestures {
                        if let Gesture::Pinch(j) = i {
                            if (j.direction == s.direction || j.direction == PinchDir::Any)
                                && j.fingers == s.fingers
                            {
                                exec_command_from_string(
                                    &j.end.clone().unwrap_or_default(),
                                    0.0,
                                    0.0,
                                    0.0,
                                    0.0,
                                )?;
                            }
                        }
                    }
                }
            }
            _ => (),
        }
        Ok(())
    }

    fn handle_swipe_event(&mut self, event: GestureSwipeEvent) -> Result<()> {
        match event {
            GestureSwipeEvent::Begin(e) => {
                self.event = Gesture::Swipe(Swipe {
                    direction: SwipeDir::Any,
                    fingers: e.finger_count(),
                    update: None,
                    start: None,
                    end: None,
                });
                if let Gesture::Swipe(s) = &self.event {
                    for i in &self.config.clone().gestures {
                        if let Gesture::Swipe(j) = i {
                            if j.fingers == s.fingers
                                && (j.direction == s.direction || j.direction == SwipeDir::Any)
                            {
                                exec_command_from_string(
                                    &j.start.clone().unwrap_or_default(),
                                    0.0,
                                    0.0,
                                    0.0,
                                    0.0,
                                )?;
                            }
                        }
                    }
                }
            }
            GestureSwipeEvent::Update(e) => {
                let (x, y) = (e.dx(), e.dy());
                let swipe_dir = SwipeDir::dir(x, y);

                if let Gesture::Swipe(s) = &self.event {
                    log::debug!("{:?}  {:?}", &swipe_dir, &s.fingers);
                    for i in &self.config.clone().gestures {
                        if let Gesture::Swipe(j) = i {
                            if j.fingers == s.fingers
                                && (j.direction == swipe_dir || j.direction == SwipeDir::Any)
                            {
                                exec_command_from_string(
                                    &j.update.clone().unwrap_or_default(),
                                    x,
                                    y,
                                    0.0,
                                    0.0,
                                )?;
                            }
                        }
                    }
                    self.event = Gesture::Swipe(Swipe {
                        direction: swipe_dir,
                        fingers: s.fingers,
                        update: None,
                        start: None,
                        end: None,
                    })
                }
            }
            GestureSwipeEvent::End(e) => {
                if let Gesture::Swipe(s) = &self.event {
                    if !e.cancelled() {
                        for i in &self.config.clone().gestures {
                            if let Gesture::Swipe(j) = i {
                                if j.fingers == s.fingers
                                    && (j.direction == s.direction || j.direction == SwipeDir::Any)
                                {
                                    exec_command_from_string(
                                        &j.end.clone().unwrap_or_default(),
                                        0.0,
                                        0.0,
                                        0.0,
                                        0.0,
                                    )?;
                                }
                            }
                        }
                    }
                }
            }
            _ => (),
        }
        Ok(())
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
