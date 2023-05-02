use knuffel::{Decode, DecodeScalar};

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
        // But maybe could be improved by checking here for large rotation
        if (scale > 0.92) && (scale < 1.08) {
            if delta_angle > 0.0 {
                Self::Clockwise
            } else {
                Self::CounterClockwise
            }
        // Otherwise we have a normal pinch
        } else if scale > 1.0 {
            Self::Out
        } else {
            Self::In
        }
    }
}
