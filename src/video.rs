use crate::client::{Client, LoginInfo};
use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::time::Duration;
use typed_builder::TypedBuilder;

#[derive(Serialize, Deserialize, Debug, TypedBuilder)]
#[builder(field_defaults(default))]
pub struct Studio {
    #[builder(default = 1)]
    pub copyright: i8,
    pub source: String,
    #[builder(default = 171)]
    pub tid: i16,
    pub cover: String,
    #[builder(!default, setter(into))]
    pub title: String,
    pub desc_format_id: i8,
    pub desc: String,
    pub dynamic: String,
    #[builder(default, setter(skip))]
    pub subtitle: Subtitle,
    #[builder(default="biliup".into())]
    pub tag: String,
    #[builder(!default)]
    pub videos: Vec<Video>,
    pub dtime: Option<i32>,
    pub open_subtitle: bool,
}

impl Studio {
    pub async fn submit(&self, login_info: &LoginInfo) -> Result<serde_json::Value> {
        // studio.videos =
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

pub struct BiliBili {
    client: reqwest::Client,
    login_info: LoginInfo,
}

impl BiliBili {
    pub async fn new(login_info: LoginInfo, login: Client) -> BiliBili {
        BiliBili {
            client: login.client,
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
}
