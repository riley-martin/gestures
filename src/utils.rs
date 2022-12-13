use anyhow::Result;
use regex::Regex;
use std::process::Command;

pub fn exec_command_from_string(args: &str, dx: f64, dy: f64) -> Result<()> {
    let rx = Regex::new(r"[^\\]\$delta_x")?;
    let ry = Regex::new(r"[^\\]\$delta_y")?;
    let args = ry.replace_all(args, format!(" {} ", dy));
    let args = rx.replace_all(&args, format!(" {} ", dx));
    crate::if_debug!(true, &args);
    Command::new("sh").arg("-c").arg(&*args).spawn()?;
    Ok(())
}
