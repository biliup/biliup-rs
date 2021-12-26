mod cli;

use anyhow::Result;
use biliup::client::Client;
use biliup::video::{BiliBili, Studio};
use dialoguer::theme::ColorfulTheme;
use dialoguer::{Input, Select};
use serde_json::json;
use std::ffi::OsStr;
use std::future::Future;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<()> {
    cli::parse().await?;
    Ok(())
}
