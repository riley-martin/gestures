use knuffel::{Decode, DecodeScalar};

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
