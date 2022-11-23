mod config;
mod gestures;
mod utils;

use std::{path::PathBuf, rc::Rc};

use clap::{Parser, Subcommand};

fn main() {
    let app = App::parse();

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
