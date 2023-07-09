use crate::error::{Kind, Result};
use crate::uploader::credential::LoginInfo;
use serde::ser::Error;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::fmt::{Display, Formatter};
use std::num::ParseIntError;
use std::str::FromStr;
use std::time::Duration;
use tracing::info;
use typed_builder::TypedBuilder;

#[derive(clap::Args, Serialize, Deserialize, Debug, TypedBuilder)]
#[builder(field_defaults(default))]
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

    /// 视频简介v2
    #[serde(default)]
    #[builder(!default)]
    #[clap(skip)]
    pub desc_v2: Option<Vec<Credit>>,

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
    #[builder(default = 0)]
    pub dolby: u8,

    /// 是否开启 Hi-Res, 0-关闭 1-开启
    #[clap(long = "hires", default_value = "0")]
    #[builder(default = 0)]
    pub lossless_music: u8,

    /// 0-允许转载，1-禁止转载
    #[clap(long, default_value = "0")]
    #[builder(default = 0)]
    pub no_reprint: u8,

    /// 是否开启充电, 0-关闭 1-开启
    #[clap(long, default_value = "0")]
    #[builder(default = 0)]
    pub open_elec: u8,

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
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Subtitle {
    open: i8,
    lan: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Credit {
    #[serde(rename(deserialize = "type_id", serialize = "type"))]
    pub type_id: i8,
    pub raw_text: String,
    pub biz_id: Option<String>,
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

pub struct BiliBili {
    pub client: reqwest::Client,
    pub login_info: LoginInfo,
}

impl BiliBili {
    pub async fn submit(&self, studio: &Studio) -> Result<ResponseData> {
        let ret: ResponseData = reqwest::Client::builder()
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
        info!("{:?}", ret);
        if ret.code == 0 {
            info!("投稿成功");
            Ok(ret)
        } else {
            Err(Kind::Custom(format!("{:?}", ret)))
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
    pub async fn video_data(&self, vid: &Vid) -> Result<Value> {
        let res: ResponseData = reqwest::Client::builder()
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
            res @ ResponseData {
                code: _,
                data: None,
                ..
            } => Err(Kind::Custom(format!("{res:?}"))),
            ResponseData {
                code: _,
                data: Some(v),
                ..
            } => Ok(v),
        }
    }

    pub async fn studio_data(&self, vid: &Vid) -> Result<Studio> {
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

    pub async fn recommend_tag(&self, subtype_id: u16, title: &str, key: &str) -> Result<Value> {
        let result: ResponseData = self
            .client
            .get(format!("https://member.bilibili.com/x/vupre/web/tag/recommend?upload_id=&subtype_id={subtype_id}&title={title}&filename={key}&description=&cover_url=&t="))
            .send()
            .await?
            .json()
            .await?;
        if result.code == 0 {
            return Ok(result.data.unwrap_or_default());
        }
        Err(Kind::Custom(result.message))
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
        let res: ResponseData = if !response.status().is_success() {
            return Err(Kind::Custom(response.text().await?));
        } else {
            response.json().await?
        };

        if let ResponseData {
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

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ResponseData<T = Value> {
    pub code: i32,
    pub data: Option<T>,
    message: String,
    ttl: Option<u8>,
}

impl<T: Serialize> Display for ResponseData<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            serde_json::to_string(self).map_err(std::fmt::Error::custom)?
        )
    }
}
