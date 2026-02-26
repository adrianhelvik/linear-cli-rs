use anyhow::{Result, bail};
use std::io::IsTerminal;

pub fn is_tty() -> bool {
    std::io::stdin().is_terminal()
}

pub fn require_text(flag_name: &str, prompt: &str, value: Option<String>) -> Result<String> {
    if let Some(v) = value {
        return Ok(v);
    }
    if !is_tty() {
        bail!("Missing required flag --{flag_name} (non-interactive mode)");
    }
    let result = inquire::Text::new(prompt).prompt()?;
    Ok(result)
}

pub fn select<T: std::fmt::Display>(prompt: &str, options: Vec<T>, flag: &str) -> Result<T> {
    if !is_tty() {
        bail!("Missing required --{flag} (non-interactive mode)");
    }
    if options.is_empty() {
        bail!("No options available.");
    }
    let result = inquire::Select::new(prompt, options).prompt()?;
    Ok(result)
}
