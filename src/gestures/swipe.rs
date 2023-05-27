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
    pub fn dir(x: f64, y: f64) -> Self {
        if x.abs() == 0.0 && y.abs() == 0.0 {
            return SwipeDir::Any;
        };

        let oblique_ratio = 0.414;

        let primary_direction = if y.abs() > x.abs() {
            if y < 0.0 {
                Self::N
            } else {
                Self::S
            }
        } else if x < 0.0 {
            Self::W
        } else {
            Self::E
        };

        let (ratio, secondary_direction) = match primary_direction {
            Self::N | Self::S => (x.abs() / y.abs(), if x < 0.0 { Self::W } else { Self::E }),
            Self::E | Self::W => (y.abs() / x.abs(), if y < 0.0 { Self::N } else { Self::S }),
            _ => (0.0, Self::Any),
        };

        // If we're going more diagonal than vertical or horizontal:
        if ratio > oblique_ratio {
            match (primary_direction, secondary_direction) {
                (Self::N, Self::E) | (Self::E, Self::N) => Self::NE,
                (Self::N, Self::W) | (Self::W, Self::N) => Self::NW,
                (Self::S, Self::E) | (Self::E, Self::S) => Self::SE,
                (Self::S, Self::W) | (Self::W, Self::S) => Self::SW,
                _ => Self::Any,
            }
        } else {
            primary_direction
        }
    }
}
