use crate::error::{Kind, Result};
use crate::uploader::credential::LoginInfo;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::fmt::{Display, Formatter};
use std::num::ParseIntError;
use std::str::FromStr;
use std::time::Duration;
use tracing::info;
use typed_builder::TypedBuilder;

#[derive(Serialize, Deserialize, Debug, TypedBuilder)]
#[builder(field_defaults(default))]
#[derive(clap::Args)]
pub struct Studio {
    /// 是否转载, 1-自制 2-转载
    #[clap(long, default_value = "1")]
    #[builder(default = 1)]
    pub copyright: u8,

    /// 转载来源
    #[clap(long, default_value_t)]
    pub source: String,

    /// 投稿分区
    #[clap(long, default_value = "171")]
    #[builder(default = 171)]
    pub tid: u16,

    /// 视频封面
    #[clap(long, default_value_t)]
    #[clap(long)]
    pub cover: String,

    /// 视频标题
    #[clap(long, default_value_t)]
    #[builder(!default, setter(into))]
    pub title: String,
    #[clap(skip)]
    pub desc_format_id: u32,
    /// 视频简介
    #[clap(long, default_value_t)]
    pub desc: String,
    /// 空间动态
    #[clap(long, default_value_t)]
    pub dynamic: String,
    #[clap(skip)]
    #[serde(default)]
    #[builder(default, setter(skip))]
    pub subtitle: Subtitle,
    /// 视频标签，逗号分隔多个tag
    #[clap(long, default_value_t)]
    pub tag: String,
    #[serde(default)]
    #[builder(!default)]
    #[clap(skip)]
    pub videos: Vec<Video>,

    /// 延时发布时间，距离提交大于4小时，格式为10位时间戳
    #[clap(long)]
    pub dtime: Option<u32>,
    #[clap(skip)]
    #[serde(default)]
    pub open_subtitle: bool,

    #[clap(long, default_value = "0")]
    #[serde(default)]
    pub interactive: u8,

    #[clap(long)]
    pub mission_id: Option<u32>,

    // #[clap(long, default_value = "0")]
    // pub act_reserve_create: u8,
    /// 是否开启杜比音效, 0-关闭 1-开启
    #[clap(long, default_value = "0")]
    #[serde(default)]
    pub dolby: u8,

    /// 是否开启 Hi-Res, 0-关闭 1-开启
    #[clap(long="hires", default_value = "0")]
    #[serde(default)]
    pub lossless_music: u8,

    /// 0-允许转载，1-禁止转载
    #[clap(long)]
    pub no_reprint: Option<u8>,

    /// aid 要追加视频的 avid
    #[clap(skip)]
    pub aid: Option<u64>,

    #[clap(long)]
    #[serde(default)]
    pub up_selection_reply: bool,

    #[clap(long)]
    #[serde(default)]
    pub up_close_reply: bool,

    #[clap(long)]
    #[serde(default)]
    pub up_close_danmu: bool,

    /// 是否开启充电, 0-关闭 1-开启
    #[clap(long)]
    pub open_elec: Option<u8>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Subtitle {
    open: i8,
    lan: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Video {
    pub title: Option<String>,
    pub filename: String,
    pub desc: String,
}

impl Video {
    pub fn new(filename: &str) -> Video {
        Video {
            title: None,
            filename: filename.into(),
            desc: "".into(),
        }
    }
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum Vid {
    Aid(u64),
    Bvid(String),
}

impl FromStr for Vid {
    type Err = ParseIntError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let s = s.trim();
        match &s[..2] {
            "BV" => Ok(Vid::Bvid(s.to_string())),
            "av" => Ok(Vid::Aid(s[2..].parse()?)),
            _ => Ok(Vid::Aid(s.parse()?)),
        }
    }
}

impl Display for Vid {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Vid::Aid(aid) => write!(f, "aid={}", aid),
            Vid::Bvid(bvid) => write!(f, "bvid={}", bvid),
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ResResult {
    pub code: i32,
    pub data: Option<Value>,
    message: String,
    ttl: u8,
}

pub struct BiliBili {
    pub client: reqwest::Client,
    pub login_info: LoginInfo,
}

impl BiliBili {
    pub async fn submit(&self, studio: &Studio) -> Result<serde_json::Value> {
        let ret: serde_json::Value = reqwest::Client::builder()
            .user_agent("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 Chrome/63.0.3239.108")
            .timeout(Duration::new(60, 0))
            .build()?
            .post(format!(
                "http://member.bilibili.com/x/vu/client/add?access_key={}",
                self.login_info.token_info.access_token
            ))
            .json(studio)
            .send()
            .await?
            .json()
            .await?;
        info!("{}", ret);
        if ret["code"] == 0 {
            info!("投稿成功");
            Ok(ret)
        } else {
            Err(Kind::Custom(ret.to_string()))
        }
    }

    pub async fn edit(&self, studio: &Studio) -> Result<serde_json::Value> {
        let ret: serde_json::Value = reqwest::Client::builder()
            .user_agent("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 Chrome/63.0.3239.108")
            .timeout(Duration::new(60, 0))
            .build()?
            .post(format!(
                "http://member.bilibili.com/x/vu/client/edit?access_key={}",
                self.login_info.token_info.access_token
            ))
            .json(studio)
            .send()
            .await?
            .json()
            .await?;
        info!("{}", ret);
        if ret["code"] == 0 {
            info!("稿件修改成功");
            Ok(ret)
        } else {
            Err(Kind::Custom(ret.to_string()))
        }
    }

    /// 查询视频的 json 信息
    pub async fn video_data(&self, vid: Vid) -> Result<Value> {
        let res: ResResult = reqwest::Client::builder()
            .user_agent("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 Chrome/63.0.3239.108")
            .timeout(Duration::new(60, 0))
            .build()?
            .get(format!(
                "http://member.bilibili.com/x/client/archive/view?access_key={}&{vid}",
                self.login_info.token_info.access_token
            ))
            .send()
            .await?
            .json()
            .await?;
        match res {
            res @ ResResult {
                code: _,
                data: None,
                ..
            } => Err(Kind::Custom(format!("{res:?}"))),
            ResResult {
                code: _,
                data: Some(v),
                ..
            } => Ok(v),
        }
    }

    pub async fn studio_data(&self, vid: Vid) -> Result<Studio> {
        let mut video_info = self.video_data(vid).await?;

        let mut studio: Studio = serde_json::from_value(video_info["archive"].take())?;
        let videos: Vec<Video> = serde_json::from_value(video_info["videos"].take())?;

        studio.videos = videos;
        Ok(studio)
    }

    pub async fn my_info(&self) -> Result<Value> {
        Ok(self
            .client
            .get("https://api.bilibili.com/x/space/myinfo")
            .send()
            .await?
            .json()
            .await?)
    }

    pub async fn archive_pre(&self) -> Result<Value> {
        Ok(self
            .client
            .get("https://member.bilibili.com/x/vupre/web/archive/pre")
            .send()
            .await?
            .json()
            .await?)
    }

    pub async fn cover_up(&self, input: &[u8]) -> Result<String> {
        let csrf = self
            .login_info
            .cookie_info
            .get("cookies")
            .and_then(|c| c.as_array())
            .ok_or("cover_up cookie error")?
            .iter()
            .filter_map(|c| c.as_object())
            .find(|c| c["name"] == "bili_jct")
            .ok_or("cover_up jct error")?;
        let response = self
            .client
            .post("https://member.bilibili.com/x/vu/web/cover/up")
            .form(&json!({
                "cover":  format!("data:image/jpeg;base64,{}", base64::encode(input)),
                "csrf": csrf["value"]
            }))
            .send()
            .await?;
        let res: ResResult = if !response.status().is_success() {
            return Err(Kind::Custom(response.text().await?));
        } else {
            response.json().await?
        };

        if let ResResult {
            code: _,
            data: Some(value),
            ..
        } = res
        {
            Ok(value["url"].as_str().ok_or("cover_up error")?.into())
        } else {
            Err(Kind::Custom(format!("{res:?}")))
        }
    }
}
