use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Config {
    pub api_key: Option<String>,
}

fn config_dir() -> Result<PathBuf> {
    let dir = dirs::config_dir()
        .context("Could not determine config directory")?
        .join("linear-cli");
    Ok(dir)
}

fn config_path() -> Result<PathBuf> {
    Ok(config_dir()?.join("config.toml"))
}

pub fn load() -> Result<Config> {
    let path = config_path()?;
    if !path.exists() {
        return Ok(Config::default());
    }
    let content = std::fs::read_to_string(&path)
        .with_context(|| format!("Failed to read {}", path.display()))?;
    let config: Config =
        toml::from_str(&content).with_context(|| format!("Failed to parse {}", path.display()))?;
    Ok(config)
}

pub fn save(config: &Config) -> Result<()> {
    let dir = config_dir()?;
    std::fs::create_dir_all(&dir).with_context(|| format!("Failed to create {}", dir.display()))?;
    secure_dir_permissions(&dir)?;
    let path = config_path()?;
    let content = toml::to_string_pretty(config)?;
    write_secure_file(&path, &content)?;
    Ok(())
}

pub fn api_key() -> Result<String> {
    if let Ok(key) = std::env::var("LINEAR_API_KEY") {
        if !key.is_empty() {
            return Ok(key);
        }
    }
    let config = load()?;
    match config.api_key {
        Some(key) if !key.is_empty() => Ok(key),
        _ => bail!("Not authenticated. Run `linear auth` or set LINEAR_API_KEY."),
    }
}

fn secure_dir_permissions(path: &Path) -> Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o700))
            .with_context(|| format!("Failed to secure {}", path.display()))?;
    }
    Ok(())
}

fn write_secure_file(path: &Path, content: &str) -> Result<()> {
    #[cfg(unix)]
    {
        use std::io::Write;
        use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};

        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .mode(0o600)
            .open(path)
            .with_context(|| format!("Failed to write {}", path.display()))?;
        file.write_all(content.as_bytes())
            .with_context(|| format!("Failed to write {}", path.display()))?;
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600))
            .with_context(|| format!("Failed to secure {}", path.display()))?;
    }
    #[cfg(not(unix))]
    {
        std::fs::write(path, content)
            .with_context(|| format!("Failed to write {}", path.display()))?;
    }
    Ok(())
}
