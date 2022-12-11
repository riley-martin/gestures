use anyhow::Result;
use std::process::Command;

pub fn exec_command_from_string(s: &str) -> Result<()> {
    // let args = s.split(' ');
    // dbg!(&args);
    Command::new("sh").arg("-c").arg(s).spawn()?;
    Ok(())
}
