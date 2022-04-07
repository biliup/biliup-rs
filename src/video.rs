use crate::client::{Client, LoginInfo, ResponseData, ResponseValue};
use crate::error::CustomError;
use anyhow::{anyhow, bail, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::time::Duration;
use typed_builder::TypedBuilder;

#[derive(Serialize, Deserialize, Debug, TypedBuilder)]
#[builder(field_defaults(default))]
#[derive(clap::Args)]
pub struct Studio {
    /// 是否转载, 1-自制 2-转载
    #[clap(long, default_value = "1")]
    #[builder(default = 1)]
    pub copyright: i8,

    /// 转载来源
    #[clap(long, default_value_t)]
    pub source: String,

    /// 投稿分区
    #[clap(long, default_value = "171")]
    #[builder(default = 171)]
    pub tid: i16,

    /// 视频封面
    #[clap(long, default_value_t)]
    #[clap(long)]
    pub cover: String,

    /// 视频标题
    #[clap(long, default_value_t)]
    #[builder(!default, setter(into))]
    pub title: String,
    #[clap(skip)]
    pub desc_format_id: i8,
    /// 视频简介
    #[clap(long, default_value_t)]
    pub desc: String,
    /// 空间动态
    #[clap(long, default_value_t)]
    pub dynamic: String,
    #[clap(skip)]
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
    pub dtime: Option<i32>,
    #[clap(skip)]
    pub open_subtitle: bool,

    #[clap(long, default_value = "0")]
    #[serde(default)]
    pub interactive: u8,

    #[clap(long)]
    pub mission_id: Option<usize>,

    // #[clap(long, default_value = "0")]
    // pub act_reserve_create: u8,

    /// 是否开启杜比音效, 0-关闭 1-开启
    #[clap(long, default_value = "0")]
    #[serde(default)]
    pub dolby: u8,

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
    pub open_elec: Option<u8>
}

impl Studio {
    pub async fn submit(&mut self, login_info: &LoginInfo) -> Result<serde_json::Value> {
        if self.tag.is_empty() {
            self.tag = "biliup".into();
        } else {
            self.tag += ",biliup";
        };
        let ret: serde_json::Value = reqwest::Client::builder()
            .user_agent("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 Chrome/63.0.3239.108")
            .timeout(Duration::new(60, 0))
            .build()?
            .post(format!(
                "http://member.bilibili.com/x/vu/client/add?access_key={}",
                login_info.token_info.access_token
            ))
            .json(self)
            .send()
            .await?
            .json()
            .await?;
        println!("{}", ret);
        if ret["code"] == 0 {
            println!("投稿成功");
            Ok(ret)
        } else {
            bail!("{}", ret)
        }
    }
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

pub struct BiliBili<'a, 'b> {
    client: &'a reqwest::Client,
    login_info: &'b LoginInfo,
}

impl BiliBili<'_, '_> {
    pub fn new<'a, 'b>(login_info: &'b LoginInfo, login: &'a Client) -> BiliBili<'a, 'b> {
        BiliBili {
            client: &login.client,
            login_info,
        }
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
            .ok_or(CustomError::Custom("cover_up cookie error".into()))?
            .iter()
            .filter_map(|c| c.as_object())
            .find(|c| c["name"] == "bili_jct")
            .ok_or(CustomError::Custom("cover_up jct error".into()))?;
        let response: ResponseData = self
            .client
            .post("https://member.bilibili.com/x/vu/web/cover/up")
            .form(&json!({
                "cover":  format!("data:image/jpeg;base64,{}", base64::encode(input)),
                "csrf": csrf["value"]
            }))
            .send()
            .await?
            .json()
            .await?;
        match &response {
            ResponseData {
                code: _,
                data: ResponseValue::Value(value),
                ..
            } if value.is_null() => bail!("{response}"),
            ResponseData {
                code: _,
                data: ResponseValue::Value(value),
                ..
            } => Ok(value["url"]
                .as_str()
                .ok_or(anyhow!("cover_up error"))?
                .into()),
            _ => {
                unreachable!()
            }
        }
    }

    /// 查询视频的 json 信息
    pub async fn video_data(&self, aid: u64) -> Result<serde_json::Value> {
        let res: ResponseData = self
            .client
            .get(format!("https://member.bilibili.com/x/vupre/web/archive/view?aid={}", aid))
            .send()
            .await?
            .json()
            .await?;
        let json: serde_json::Value = match res {
            ResponseData {
                code: _,
                data: ResponseValue::Value(value),
                ..
            } if value.is_null() => bail!("video query failed..."),
            ResponseData {
                code: _,
                data: ResponseValue::Value(value),
                ..
            } => value,
            _ => {
                unreachable!()
            }
        };
        Ok(json)
    }

    pub async fn edit(&mut self, studio: &Studio) -> Result<serde_json::Value> {
        let csrf = self
            .login_info
            .cookie_info
            .get("cookies")
            .and_then(|c| c.as_array())
            .ok_or(CustomError::Custom("video_edit cookie error".into()))?
            .iter()
            .filter_map(|c| c.as_object())
            .find(|c| c["name"] == "bili_jct")
            .ok_or(CustomError::Custom("video_edit jct error".into()))?;
        let csrf_str = csrf["value"].as_str().unwrap().to_string();
        let url = format!(
            "https://member.bilibili.com/x/vu/web/edit?csrf={}",
            csrf_str
        );
        println!("{}", url);
        let ret: serde_json::Value = self
            .client
            .post(url)
            .json(studio)
            .send()
            .await?
            .json()
            .await?;
        println!("{}", ret);
        if ret["code"] == 0 {
            println!("稿件修改成功");
            Ok(ret)
        } else {
            bail!("{}", ret)
        }
    }
}
