use std::{fs, path::PathBuf};

use common::{AppResult, error::AppError};

pub(crate) fn init_paths() -> AppResult<()> {
    fs::create_dir_all(&default_dir()?)?;
    fs::create_dir_all(&storage_path()?)?;
    Ok(())
}

pub(crate) fn default_dir() -> AppResult<PathBuf> {
    let dir = dirs::home_dir()
        .ok_or(AppError::InvalidPath)?
        .join(".flipcoin");
    Ok(dir)
}

pub(crate) fn keystore_path() -> AppResult<PathBuf> {
    Ok(default_dir()?.join("secret.json"))
}

pub(crate) fn storage_path() -> AppResult<PathBuf> {
    Ok(default_dir()?.join("data"))
}

pub(crate) fn config_path() -> AppResult<PathBuf> {
    default_dir().map(|path| path.join("config.toml"))
}
