mod config;
mod event_handler;
mod gestures;
mod ipc;
mod ipc_client;
mod utils;

#[cfg(test)]
mod tests;

use std::{
    path::PathBuf,
    sync::{Arc, RwLock},
    thread::{self, JoinHandle},
};

use clap::{Parser, Subcommand};
use env_logger::Builder;
use log::LevelFilter;
use miette::Result;

use crate::config::*;

fn main() -> Result<()> {
    let app = App::parse();

    {
        let mut l = Builder::from_default_env();

        if app.verbose > 0 {
            l.filter_level(match app.verbose {
                1 => LevelFilter::Info,
                2 => LevelFilter::Debug,
                _ => LevelFilter::max(),
            });
        }

        if app.debug {
            l.filter_level(LevelFilter::Debug);
        }

        l.init();
    }

    let c = if let Some(p) = app.conf {
        Config::read_from_file(&p)?
    } else {
        config::Config::read_default_config().unwrap_or_else(|_| {
            log::error!("Could not read configuration file, using empty config!");
            Config::default()
        })
    };
    log::debug!("{:#?}", &c);

    match app.command {
        c @ Commands::Reload => {
            ipc_client::handle_command(c);
        }
        Commands::Start => run_eh(Arc::new(RwLock::new(c)))?,
    }

    Ok(())
}

fn run_eh(config: Arc<RwLock<Config>>) -> Result<()> {
    let eh_thread: JoinHandle<Result<()>>;

    {
        let config = config.clone();
        eh_thread = thread::spawn(|| -> Result<()> {
            log::debug!("Starting event handler in new thread");
            let mut eh = event_handler::EventHandler::new(config);
            let mut interface = input::Libinput::new_with_udev(event_handler::Interface);
            eh.init(&mut interface)?;
            eh.main_loop(&mut interface);
            Ok(())
        });
    }

    ipc::create_socket(config);

    eh_thread.join().unwrap()?;
    Ok(())
}

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct App {
    /// Verbosity, can be repeated
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,
    /// Debug mode
    #[arg(short, long)]
    debug: bool,
    /// Path to config file
    #[arg(short, long, value_name = "FILE")]
    conf: Option<PathBuf>,
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Reload the configuration
    Reload,
    /// Start the program
    Start,
}
