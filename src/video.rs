use std::path::Path;
use crate::client::{Client, LoginInfo, ResponseData, ResponseValue};
use anyhow::{anyhow, bail, Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::time::Duration;
use typed_builder::TypedBuilder;
use crate::error::CustomError;

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
    #[serde(skip_deserializing)]
    #[builder(!default)]
    pub videos: Vec<Video>,
    pub dtime: Option<i32>,
    pub open_subtitle: bool,
}

impl Studio {
    pub async fn submit(&mut self, login_info: &LoginInfo) -> Result<serde_json::Value> {
        if self.tag.is_empty() {
            self.tag = "biliup".into();
        } else {
            self.tag +=  ",biliup";
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

impl BiliBili<'_, '_>{
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
        let csrf = self.login_info.cookie_info.get("cookies")
            .and_then(|c| c.as_array()).ok_or(CustomError::Custom("cover_up cookie error".into()))?
            .iter()
            .filter_map(|c| c.as_object())
            .find(|c| c["name"] == "bili_jct").ok_or(CustomError::Custom("cover_up jct error".into()))?;
        let response: ResponseData = self.client.post("https://member.bilibili.com/x/vu/web/cover/up")
            .form(&json!({
                "cover":  format!("data:image/jpeg;base64,{}", base64::encode(input)),
                "csrf": csrf["value"]
            })).send().await?.json().await?;
        match &response {
            ResponseData { code: _ , data: ResponseValue::Value(value), .. } if value.is_null() => bail!("{response}"),
            ResponseData { code: _ , data: ResponseValue::Value(value), .. } => Ok(value["url"].as_str().ok_or(anyhow!("cover_up error"))?.into()),
            _ => { unreachable!()}
        }
    }
}
