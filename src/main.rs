mod config;
mod gestures;
mod utils;

use std::env;

fn main() {
    if args.debug || args.raw || args.list {
        args.verbose = true;
    }

    if args.verbose {
        let xsession = env::var("XDG_SESSION_DESKTOP").unwrap_or_else(|_| {
            env::var("DESKTOP_SESSION").unwrap_or_else(|_| "default".to_string())
        });
        let xtype = env::var("XDG_SESSION_TYPE").unwrap_or_else(|_| "unknown".to_string());
        let xstr = format!("session {}+{}", xsession, xtype);
        println!("gestures: {} on {}", xstr, env::consts::OS);
    }
}
