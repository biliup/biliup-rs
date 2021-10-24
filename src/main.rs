use std::ffi::OsStr;
use std::future::Future;
use serde_json::json;
use biliup::client::Client;
use dialoguer::{Input, Select};
use anyhow::Result;
use dialoguer::theme::ColorfulTheme;
use biliup::{login_by_password, login_by_sms};
use std::path::PathBuf;
use clap::{Parser, Subcommand, IntoApp};
use biliup::video::{BiliBili, Studio};

#[derive(Parser)]
#[clap(author, version, about)]
struct Cli {
    
    /// Turn debugging information on
    // #[clap(short, long, parse(from_occurrences))]
    // debug: usize,

    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// 登录B站并保存登录信息在执行目录下
    Login,
    /// 上传视频
    Upload {
        // Optional name to operate on
        // name: Option<String>,
        
        /// 需要上传的视频路径
        #[clap(parse(from_os_str))]
        video_path: Vec<PathBuf>,

        /// Sets a custom config file
        #[clap(short, long, parse(from_os_str), value_name = "FILE")]
        config: Option<PathBuf>,
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // You can check the value provided by positional arguments, or option arguments
    // if let Some(name) = cli.name.as_deref() {
    //     println!("Value for name: {}", name);
    // }
    //
    // if let Some(config_path) = cli.config.as_deref() {
    //     println!("Value for config: {}", config_path.display());
    // }

    // You can see how many times a particular flag or argument occurred
    // Note, only flags can have multiple occurrences
    // match cli.debug {
    //     0 => println!("Debug mode is off"),
    //     1 => println!("Debug mode is kind of on"),
    //     2 => println!("Debug mode is on"),
    //     _ => println!("Don't be crazy"),
    // }

    // You can check for the existence of subcommands, and if found use their
    // matches just as you would the top level app
    let client: Client = Default::default();
    match &cli.command {
        Commands::Login => {
            let selection = Select::with_theme(&ColorfulTheme::default())
                .with_prompt("选择一种登录方式")
                .default(1)
                .item("账号密码")
                .item("短信登录")
                .interact()?;
            match selection {
                0 => login_by_password(client).await?,
                1 => login_by_sms(client).await?,
                _ => panic!()
            };
        }
        Commands::Upload { video_path, config} if video_path.len() > 0 => {
            let bilibili = BiliBili::new((client.login_by_cookies(std::fs::File::open("cookies.json")?).await?, client));
            let mut videos = Vec::new();
            for video in video_path {
                let uploaded = bilibili
                    .upload_file(video, |instant, total, size| false)
                    .await?;
                videos.push(uploaded);
            }

            let result = bilibili
                .submit(Studio::builder().title(video_path[0].file_stem()
                    .and_then(OsStr::to_str)
                    .map(|s| s.to_string()).unwrap()).videos(videos).build())
                .await?;
            println!("{}", result);
        }
        _ => {
            println!("参数不正确请参阅帮助");
            Cli::into_app().print_help()?
        }
    };

    Ok(())
}
