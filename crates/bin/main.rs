mod cli;
mod downloader;
mod uploader;

use anyhow::Result;

use crate::cli::{Cli, Commands};
use crate::downloader::{download, generate_json};
use crate::uploader::{append, login, renew, show, upload_by_command, upload_by_config};

use clap::Parser;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> Result<()> {
    // let args: Vec<String> = std::env::args().collect();
    // a builder for `FmtSubscriber`.
    let subscriber = FmtSubscriber::builder()
        // all spans/events with a level higher than TRACE (e.g, debug, info, warn, etc.)
        // will be written to stdout.
        .with_max_level(Level::INFO)
        // completes the builder.
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
    parse().await?;

    // let args: Vec<String> = env::args().collect();
    // let file_name = &args[1];
    // let flv_file = std::fs::File::open(file_name)?;
    // let buf_reader = BufReader::new(flv_file);
    // let mut connection = Connection::new(buf_reader);
    // connection.read_frame(9)?;
    // download(
    //     connection,
    //     &(file_name.to_owned() + "new%H_%M_%S%.f"),
    //     Segment::Time(Duration::from_secs(60 * 60 * 24), Default::default()),
    // );
    // Ok(result)
    // generate_json()?;
    Ok(())
}

async fn parse() -> Result<()> {
    let cli = Cli::parse();

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
    };
    Ok(())
}
