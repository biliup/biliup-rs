mod cli;

use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    cli::parse().await?;
    Ok(())
}
