mod config;
mod gestures;
mod utils;

use std::{path::PathBuf, rc::Rc};

use clap::{Parser, Subcommand};
use serde_lexpr::to_string;

use crate::{config::*, gestures::*};

fn main() {
    let app = App::parse();
    // let c = config::Config::new(
    //     None,
    //     vec![
    //         Gesture::Swipe(Swipe {
    //             direction: Direction::N,
    //             fingers: 4,
    //             action: "/home/riley/.config/rofi/scripts/launcher_custom".to_string(),
    //         }),
    //         Gesture::Swipe(Swipe {
    //             direction: Direction::S,
    //             fingers: 4,
    //             action: "killall rofi".to_string(),
    //         }),
    //         Gesture::Pinch(Pinch {
    //             scale: 1.0,
    //             fingers: 3,
    //             direction: InOut::Out,
    //             action: "/home/riley/.config/rofi/scripts/launcher_custom".to_string(),
    //         }),
    //         Gesture::Pinch(Pinch {
    //             scale: 1.0,
    //             fingers: 3,
    //             direction: InOut::In,
    //             action: "killall rofi".to_string(),
    //         }),
    //     ],
    // );

    let c = config::Config::read_default_config().expect("failed to read config");
    let debug = app.debug || app.verbose > 0;
    if_debug!(debug, &c);
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
