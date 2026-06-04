use common::AppResult;

use crate::{cli::start_cli, paths::init_paths};

pub mod api;
pub(crate) mod cli;
pub(crate) mod config;
pub(crate) mod keystore;
pub mod model;
pub(crate) mod node;
pub(crate) mod paths;
pub(crate) mod state;

pub async fn run() -> AppResult<()> {
    init_paths()?;
    start_cli().await?;
    Ok(())
}
