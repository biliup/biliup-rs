use anyhow::Result;
use async_std::fs::File;
use biliup::client::{Client, LoginInfo};
use biliup::line::{Line, Probe};
use biliup::video::{Studio, Video};
use biliup::{line, load_config};
use clap::{IntoApp, Parser, Subcommand};
use dialoguer::theme::ColorfulTheme;
use dialoguer::Input;
use dialoguer::Select;
use image::Luma;
use indicatif::{ProgressBar, ProgressStyle};
use qrcode::render::unicode;
use qrcode::QrCode;
use std::ffi::OsStr;
use std::path::PathBuf;
use std::time::Instant;

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
    },
}

pub async fn parse() -> Result<()> {
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
            login(client).await?;
        }
        Commands::Upload {
            video_path,
            config: None,
        } if video_path.len() > 0 => {
            let login_info = client
                .login_by_cookies(std::fs::File::open("cookies.json")?)
                .await?;
            let studio: Studio = Studio::builder()
                .title(
                    video_path[0]
                        .file_stem()
                        .and_then(OsStr::to_str)
                        .map(|s| s.to_string())
                        .unwrap(),
                )
                .videos(upload(video_path, &client).await?)
                .build();
            studio.submit(&login_info).await?;
        }
        Commands::Upload {
            video_path,
            config: Some(config),
        } => {
            let login_info = client
                .login_by_cookies(std::fs::File::open("cookies.json")?)
                .await?;
            for (prefix_filename, mut studio) in load_config(config)?.streamers {
                let mut paths = Vec::new();
                for entry in glob::glob(&format!("./{prefix_filename}*"))?.filter_map(Result::ok) {
                    paths.push(entry);
                }
                studio.videos = upload(&paths, &client).await?;
                studio.submit(&login_info).await?;
            }
        }
        _ => {
            println!("参数不正确请参阅帮助");
            Cli::into_app().print_help()?
        }
    };
    Ok(())
}

async fn login(client: Client) -> Result<()> {
    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("选择一种登录方式")
        .default(1)
        .item("账号密码")
        .item("短信登录")
        .item("扫码登录")
        .interact()?;
    match selection {
        0 => login_by_password(client).await?,
        1 => login_by_sms(client).await?,
        2 => login_by_qrcode(client).await?,
        _ => panic!(),
    };
    Ok(())
}

pub async fn upload(video_path: &[PathBuf], client: &Client) -> Result<Vec<Video>> {
    let mut videos = Vec::new();
    let line = Probe::probe().await.unwrap_or_default();
    // let line = line::kodo();
    for video_path in video_path {
        println!("{line:?}");
        let uploader = line.to_uploader(video_path).await?;
        //Progress bar
        let total_size = uploader.total_size;
        let pb = ProgressBar::new(total_size);
        pb.set_style(ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})"));
        pb.enable_steady_tick(1000);
        let mut uploaded = 0;

        let instant = Instant::now();
        let video = uploader
            .upload(&client, |len| {
                uploaded += len;
                pb.set_position(uploaded as u64);
                true
            })
            .await?;
        pb.finish_and_clear();
        println!(
            "Upload completed: {:.2} MB/s.",
            total_size as f64 / 1000. / instant.elapsed().as_millis() as f64
        );
        videos.push(video);
    }
    Ok(videos)
}

pub async fn login_by_password(client: Client) -> Result<LoginInfo> {
    let username: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("请输入账号")
        .interact()?;
    let password: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("请输入密码")
        .interact()?;
    let res = client.login_by_password(&username, &password).await?;
    Ok(res)
}

pub async fn login_by_sms(client: Client) -> Result<LoginInfo> {
    let country_code: u32 = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("请输入手机国家代码")
        .default(86)
        .interact_text()?;
    let phone: u64 = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("请输入手机号")
        .interact_text()?;
    let res = client.send_sms(phone, country_code).await?;
    let input: u32 = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("请输入验证码")
        .interact_text()?;
    // println!("{}", payload);
    let ret = client.login_by_sms(input, res).await?;
    Ok(ret)
}

pub async fn login_by_qrcode(client: Client) -> Result<LoginInfo> {
    let value = client.get_qrcode().await?;
    let code = QrCode::new(value["data"]["url"].as_str().unwrap()).unwrap();
    let image = code
        .render::<unicode::Dense1x2>()
        .dark_color(unicode::Dense1x2::Light)
        .light_color(unicode::Dense1x2::Dark)
        .build();
    println!("{}", image);
    // Render the bits into an image.
    let image = code.render::<Luma<u8>>().build();
    println!("在Windows下建议使用Windows Terminal(支持utf8，可完整显示二维码)\n否则可能无法正常显示，此时请打开./qrcode.png扫码");
    // Save the image.
    image.save("qrcode.png").unwrap();
    client.login_by_qrcode(value).await
}
