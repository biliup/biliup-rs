mod cli;

use std::ffi::OsStr;
use std::future::Future;
use serde_json::json;
use biliup::client::Client;
use dialoguer::{Input, Select};
use anyhow::Result;
use dialoguer::theme::ColorfulTheme;
use std::path::PathBuf;
use biliup::video::{BiliBili, Studio};


#[tokio::main]
async fn main() -> Result<()> {
    cli::parse().await?;
    Ok(())
}
