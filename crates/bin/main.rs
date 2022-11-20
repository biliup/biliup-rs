mod cli;
mod downloader;
mod server;
mod uploader;

use anyhow::Result;

use crate::cli::{Cli, Commands};
use crate::downloader::{download, generate_json};
use crate::uploader::{append, login, renew, show, upload_by_command, upload_by_config};

use clap::Parser;
use tracing::Level;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> Result<()> {
    // a builder for `FmtSubscriber`.
    // let subscriber = FmtSubscriber::builder()
    //     // all spans/events with a level higher than TRACE (e.g, debug, info, warn, etc.)
    //     // will be written to stdout.
    //     .with_max_level(Level::INFO)
    //     // completes the builder.
    //     .finish();

    // tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
    let cli = Cli::parse();

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(&cli.rust_log))
        .with(tracing_subscriber::fmt::layer())
        .init();

    match cli.command {
        Commands::Login => login(cli.user_cookie).await?,
        Commands::Renew => {
            renew(cli.user_cookie).await?;
        }
        Commands::Upload {
            video_path,
            config: None,
            line,
            limit,
            studio,
        } => upload_by_command(studio, cli.user_cookie, video_path, line, limit).await?,
        Commands::Upload {
            video_path: _,
            config: Some(config),
            ..
        } => upload_by_config(config, cli.user_cookie).await?,
        Commands::Append {
            video_path,
            vid,
            line,
            limit,
            studio: _,
        } => append(cli.user_cookie, vid, video_path, line, limit).await?,
        Commands::Show { vid } => show(cli.user_cookie, vid).await?,
        Commands::DumpFlv { file_name } => generate_json(file_name)?,
        Commands::Download {
            url,
            output,
            split_size,
            split_time,
        } => download(&url, output, split_size, split_time).await?,
        Commands::Server { bind, port } => server::run((&bind, port)).await?,
    };
    Ok(())
}
