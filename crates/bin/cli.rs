use biliup::video::{Studio, Vid};
use clap::{ArgEnum, Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[clap(author, version, about)]
pub struct Cli {
    /// Turn debugging information on
    // #[clap(short, long, parse(from_occurrences))]
    // debug: usize,

    #[clap(subcommand)]
    pub command: Commands,

    /// 登录信息文件
    #[clap(short, long, default_value = "cookies.json")]
    pub user_cookie: PathBuf,
}

#[derive(Subcommand)]
pub enum Commands {
    /// 登录B站并保存登录信息
    Login,
    /// 手动验证并刷新登录信息
    Renew,
    /// 上传视频
    Upload {
        // Optional name to operate on
        // name: Option<String>,
        /// 需要上传的视频路径,若指定配置文件投稿不需要此参数
        #[clap(parse(from_os_str))]
        video_path: Vec<PathBuf>,

        /// Sets a custom config file
        #[clap(short, long, parse(from_os_str), value_name = "FILE")]
        config: Option<PathBuf>,

        /// 选择上传线路
        #[clap(short, long, arg_enum)]
        line: Option<UploadLine>,

        /// 单视频文件最大并发数
        #[clap(long, default_value = "3")]
        limit: usize,

        #[clap(flatten)]
        studio: Studio,
    },
    /// 是否要对某稿件追加视频
    Append {
        // Optional name to operate on
        // name: Option<String>,
        /// vid为稿件 av 或 bv 号
        #[clap(short, long)]
        vid: Vid,
        /// 需要上传的视频路径,若指定配置文件投稿不需要此参数
        #[clap(parse(from_os_str))]
        video_path: Vec<PathBuf>,

        /// 选择上传线路
        #[clap(short, long, arg_enum)]
        line: Option<UploadLine>,

        /// 单视频文件最大并发数
        #[clap(long, default_value = "3")]
        limit: usize,

        #[clap(flatten)]
        studio: Studio,
    },
    /// 打印视频详情
    Show {
        /// vid为稿件 av 或 bv 号
        // #[clap()]
        vid: Vid,
    },
    /// 输出flv元数据
    DumpFlv {
        #[clap(parse(from_os_str))]
        file_name: PathBuf,
    },
    /// 下载视频
    Download { url: String },
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, ArgEnum)]
pub enum UploadLine {
    Bda2,
    Ws,
    Qn,
    Kodo,
    Cos,
    CosInternal,
}

// pub async fn parse() -> Result<()> {
//     let cli = Cli::parse();
//
//     // You can check the value provided by positional arguments, or option arguments
//     // if let Some(name) = cli.name.as_deref() {
//     //     println!("Value for name: {}", name);
//     // }
//     //
//     // if let Some(config_path) = cli.config.as_deref() {
//     //     println!("Value for config: {}", config_path.display());
//     // }
//
//     // You can see how many times a particular flag or argument occurred
//     // Note, only flags can have multiple occurrences
//     // match cli.debug {
//     //     0 => println!("Debug mode is off"),
//     //     1 => println!("Debug mode is kind of on"),
//     //     2 => println!("Debug mode is on"),
//     //     _ => println!("Don't be crazy"),
//     // }
//
//     // You can check for the existence of subcommands, and if found use their
//     // matches just as you would the top level app
//     let client: Client = Default::default();
//     match cli.command {
//         Commands::Login => {
//             login(client, cli.user_cookie).await?;
//         }
//         Commands::Renew => {
//             renew(client, cli.user_cookie).await?;
//         }
//         Commands::Upload {
//             video_path,
//             config: None,
//             line,
//             limit,
//             mut studio,
//         } if !video_path.is_empty() => {
//             println!("number of concurrent futures: {limit}");
//             let login_info = client.login_by_cookies(fopen_rw(cli.user_cookie)?).await?;
//             if studio.title.is_empty() {
//                 studio.title = video_path[0]
//                     .file_stem()
//                     .and_then(OsStr::to_str)
//                     .map(|s| s.to_string())
//                     .unwrap();
//             }
//             cover_up(&mut studio, &login_info, &client).await?;
//             studio.videos = upload(&video_path, &client, line, limit).await?;
//             studio.submit(&login_info).await?;
//         }
//         Commands::Upload {
//             video_path: _,
//             config: Some(config),
//             ..
//         } => {
//             let login_info = client.login_by_cookies(fopen_rw(cli.user_cookie)?).await?;
//             let config = load_config(&config)?;
//             println!("number of concurrent futures: {}", config.limit);
//             for (filename_patterns, mut studio) in config.streamers {
//                 let mut paths = Vec::new();
//                 for entry in glob::glob(&filename_patterns)?.filter_map(Result::ok) {
//                     paths.push(entry);
//                 }
//                 if paths.is_empty() {
//                     println!("未搜索到匹配的视频文件：{filename_patterns}");
//                     continue;
//                 }
//                 cover_up(&mut studio, &login_info, &client).await?;
//
//                 studio.videos = upload(
//                     &paths,
//                     &client,
//                     config
//                         .line
//                         .as_ref()
//                         .and_then(|l| UploadLine::from_str(l, true).ok()),
//                     config.limit,
//                 )
//                 .await?;
//                 studio.submit(&login_info).await?;
//             }
//         }
//         Commands::Append {
//             video_path,
//             vid,
//             line,
//             limit,
//             studio: _,
//         } if !video_path.is_empty() => {
//             println!("number of concurrent futures: {limit}");
//             let login_info = client.login_by_cookies(fopen_rw(cli.user_cookie)?).await?;
//             let mut uploaded_videos = upload(&video_path, &client, line, limit).await?;
//             let mut studio = BiliBili::new(&login_info, &client).studio_data(vid).await?;
//             studio.videos.append(&mut uploaded_videos);
//             studio.edit(&login_info).await?;
//         }
//         Commands::Show { vid } => {
//             let login_info = client.login_by_cookies(fopen_rw(cli.user_cookie)?).await?;
//             let video_info = BiliBili::new(&login_info, &client).video_data(vid).await?;
//             println!("{}", serde_json::to_string_pretty(&video_info)?)
//         }
//         _ => {
//             println!("参数不正确请参阅帮助");
//             Cli::command().print_help()?
//         }
//     };
//     Ok(())
// }
