mod config;
mod gestures;
mod utils;

use std::{path::PathBuf, rc::Rc};

use clap::{Parser, Subcommand};
use serde_lexpr::to_string;

fn main() {
    let app = App::parse();

    let c = config::Config::new(
        None,
        vec![
            gestures::Gesture {
                gesture_type: gestures::GesType::Swipe,
                direction: gestures::Direction::N,
                fingers: 4,
                action: "killall rofi".to_string(),
            },
            gestures::Gesture {
                gesture_type: gestures::GesType::Swipe,
                direction: gestures::Direction::S,
                fingers: 4,
                action: "/home/riley/.config/rofi/scripts/launcher_custom".to_string(),
            },
        ],
    );
    let debug = app.debug || app.verbose > 0;
    if debug {
        dbg!(&c);
    }
    let mut eh = gestures::EventHandler::new(Rc::new(c), debug);
    let mut interface = input::Libinput::new_with_udev(gestures::Interface);
    eh.init(&mut interface)
        .expect("could not initialize libinput");
    eh.main_loop(&mut interface);
}

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct App {
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,
    #[arg(short, long)]
    debug: bool,
    #[arg(short, long, value_name = "FILE")]
    conf: Option<PathBuf>,
}
