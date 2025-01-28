mod cli;
mod downloader;
#[cfg(feature = "server")]
mod server;
mod uploader;

use anyhow::Result;
use time::macros::format_description;
use tracing::info;

use crate::cli::{Cli, Commands};
use crate::downloader::{download, generate_json};
use crate::uploader::{append, list, login, renew, show, upload_by_command, upload_by_config};

use clap::Parser;

use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

// 判断代理是否有效
async fn check_proxy(proxy: &str) -> Result<()> {
    let client = reqwest::Client::builder()
        .proxy(reqwest::Proxy::all(proxy)?)
        .build()?;
    let resp = client.get("https://www.bilibili.com/").send().await?;
    if resp.status().is_success() {
        Ok(())
    } else {
        Err(anyhow::anyhow!("代理无效"))
    }
}




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
    let mut cli = Cli::parse(); //下面需要修改cli

    unsafe {
        time::util::local_offset::set_soundness(time::util::local_offset::Soundness::Unsound);
    }

    let timer = tracing_subscriber::fmt::time::LocalTime::new(format_description!(
        "[year]-[month]-[day] [hour]:[minute]:[second]"
    ));
    
    let proxy = cli.proxy.as_deref();
    // 判断proxy是否为空 如果为空则尝试从user_cookie的文件名-proxy.json中读取
    if proxy.is_none() {
        let file_name = cli.user_cookie.file_name().unwrap().to_str().unwrap();
        // 去除文件名后缀
        let file_name = file_name.split('.').collect::<Vec<&str>>()[0];
        let proxy_file = format!("{}-proxy.json", file_name);
        let proxy_file = std::path::Path::new(&proxy_file);
        if proxy_file.exists() {
            // 如果文件存在则读取文件
            let proxy_str = std::fs::read_to_string(proxy_file)?;
            let proxy_str = proxy_str.trim();
            // 去除双引号
            let proxy_str = proxy_str.trim_matches(|c| c == '\"');
            // 去除中文引号
            let proxy_str = proxy_str.trim_matches(|c| c == '“' || c == '”');
            if !proxy_str.is_empty() {
                info!("读取到代理配置: {}", proxy_str);
                // 检查代理是否有效
                let is_available = check_proxy(proxy_str).await;
                if is_available.is_err() {
                    return Err(anyhow::anyhow!("代理无效"));
                }
                cli.proxy = Some(proxy_str.to_string());
            }
        }
    }else {
        // 检查代理是否有效
        check_proxy(proxy.unwrap()).await?;
    }
    // 重新使cli不可变
    let cli = cli;
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(&cli.rust_log))
        .with(tracing_subscriber::fmt::layer().with_timer(timer))
        .init();

    match cli.command {
        Commands::Login => login(cli.user_cookie, cli.proxy.as_deref()).await?,
        Commands::Renew => {
            renew(cli.user_cookie, cli.proxy.as_deref()).await?;
        }
        Commands::Upload {
            video_path,
            config: None,
            line,
            limit,
            studio,
            submit,
        } => {
            upload_by_command(
                studio,
                cli.user_cookie,
                video_path,
                line,
                limit,
                submit,
                cli.proxy.as_deref(),
            )
            .await?
        }
        Commands::Upload {
            video_path: _,
            config: Some(config),
            ..
        } => upload_by_config(config, cli.user_cookie, cli.proxy.as_deref()).await?,
        Commands::Append {
            video_path,
            vid,
            line,
            limit,
            studio: _,
        } => {
            append(
                cli.user_cookie,
                vid,
                video_path,
                line,
                limit,
                cli.proxy.as_deref(),
            )
            .await?
        }
        Commands::Show { vid } => show(cli.user_cookie, vid, cli.proxy.as_deref()).await?,
        Commands::DumpFlv { file_name } => generate_json(file_name)?,
        Commands::Download {
            url,
            output,
            split_size,
            split_time,
        } => download(&url, output, split_size, split_time).await?,
        #[cfg(feature = "server")]
        Commands::Server { bind, port } => server::run((&bind, port)).await?,
        Commands::List {
            is_pubing,
            pubed,
            not_pubed,
        } => {
            list(
                cli.user_cookie,
                is_pubing,
                pubed,
                not_pubed,
                cli.proxy.as_deref(),
            )
            .await?
        },
        Commands::Bind  => {
            // 判断proxy是否为空
            if cli.proxy.is_none() {
                return Err(anyhow::anyhow!("请指定代理"));
            }
            // 判断user_cookie是否存在
            if cli.user_cookie.exists() {
                return Err(anyhow::anyhow!("指定的user_cookie文件不存在"));
            }
            let proxy = cli.proxy.as_deref().unwrap();
             // 判断代理是否有效
             let is_available = check_proxy(proxy).await;
             if is_available.is_err() {
                 return Err(anyhow::anyhow!("代理无效"));
             }
            let user = cli.user_cookie.file_name().unwrap().to_str().unwrap().split('.').collect::<Vec<&str>>()[0];
            let file_name = format!("{}-proxy.json", user);
            std::fs::write(file_name, proxy)?;
        }
    };
    Ok(())
}
