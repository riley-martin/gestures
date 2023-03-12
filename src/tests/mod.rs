use crate::gestures::SwipeDir;

use super::*;

#[test]
fn test_config_default() {
    let c = Config::default();
    assert_eq!(
        c,
        Config {
            // device: None,
            gestures: vec![],
        }
    );
}

#[test]
fn test_direction_center() {
    assert_eq!(SwipeDir::Any, SwipeDir::dir(0.0, 0.0));
}

#[test]
fn test_direction_n() {
    assert_eq!(SwipeDir::N, SwipeDir::dir(0.0, -1.0));
}

#[test]
fn test_direction_s() {
    assert_eq!(SwipeDir::S, SwipeDir::dir(0.0, 1.0));
}

#[test]
fn test_direction_e() {
    assert_eq!(SwipeDir::E, SwipeDir::dir(1.0, 0.0));
}

#[test]
fn test_direction_w() {
    assert_eq!(SwipeDir::W, SwipeDir::dir(-1.0, 0.0));
}
