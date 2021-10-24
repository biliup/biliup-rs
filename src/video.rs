use anyhow::{bail, Result};
use async_std::fs::File;

use bytes::{BufMut, Bytes, BytesMut};

use futures::{AsyncReadExt, Stream};

use serde::{Deserialize, Serialize};

use serde_json::{json, Value};

use std::time::Instant;

use crate::client::{Client, LoginInfo};

use async_stream::try_stream;

use crate::upos::Upos;
use typed_builder::TypedBuilder;

#[derive(Serialize, Debug, TypedBuilder)]
#[builder(field_defaults(default))]
pub struct Studio {
    #[builder(default = 1)]
    copyright: i8,
    source: String,
    #[builder(default = 171)]
    tid: i16,
    cover: String,
    #[builder(!default, setter(into))]
    title: String,
    desc_format_id: i8,
    desc: String,
    dynamic: String,
    #[builder(default, setter(skip))]
    subtitle: Subtitle,
    #[builder(default="biliup".into())]
    tag: String,
    #[builder(!default)]
    videos: Vec<Video>,
    dtime: Option<i32>,
    open_subtitle: bool,
}

#[derive(Serialize, Debug, Default)]
pub struct Subtitle {
    open: i8,
    lan: String,
}

#[derive(Serialize, Debug)]
pub struct Video {
    pub title: Option<String>,
    pub filename: String,
    pub desc: &'static str,
}

impl Video {
    pub fn new(filename: &str) -> Video {
        Video {
            title: None,
            filename: filename.into(),
            desc: "",
        }
    }
}

pub struct BiliBili {
    client: reqwest::Client,
    login_info: LoginInfo,
}
impl BiliBili {
    pub fn new((login_info, login): (LoginInfo, Client)) -> BiliBili {
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

    pub async fn upload_file(
        &self,
        filepath: impl std::convert::AsRef<async_std::path::Path>,
        callback: impl FnMut(Instant, u64, usize) -> bool,
    ) -> Result<Video> {
        let file = File::open(&filepath).await?;
        let line = Probe::probe().await?;
        let file_name = filepath
            .as_ref()
            .file_name()
            .ok_or("No filename")
            .unwrap()
            .to_str();
        let params = json!({
            "r": line.os,
            "profile": "ugcupos/bup",
            "ssl": 0,
            "version": "2.8.12",
            "build": 2081200,
            "name": file_name,
            "size": file.metadata().await?.len(),
        });
        println!("{}", params);
        let res: serde_json::Value = self
            .client
            .get(format!(
                "https://member.bilibili.com/preupload?{}",
                line.query
            ))
            .query(&params)
            .send()
            .await?
            .json()
            .await?;
        Upos::upload(file, filepath.as_ref(), res, callback).await
    }

    pub async fn submit(&self, studio: Studio) -> Result<serde_json::Value> {
        // studio.videos =
        let ret: serde_json::Value = self
            .client
            .post(format!(
                "http://member.bilibili.com/x/vu/client/add?access_key={}",
                &self.login_info.token_info.access_token
            ))
            .json(&studio)
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

#[derive(Deserialize, Serialize, Debug)]
pub struct Probe {
    #[serde(rename = "OK")]
    ok: u8,
    lines: Vec<Line>,
    probe: serde_json::Value,
}
#[derive(Deserialize, Serialize, Debug)]
pub struct Line {
    os: String,
    probe_url: String,
    query: String,
    #[serde(skip)]
    cost: u128,
}

impl Probe {
    pub async fn probe() -> Result<Line> {
        let res: Self = reqwest::get("https://member.bilibili.com/preupload?r=probe")
            .await?
            .json()
            .await?;
        let client = if !res.probe["get"].is_null() {
            |url| reqwest::Client::new().get(url)
        } else {
            |url| {
                reqwest::Client::new()
                    .post(url)
                    .body(vec![0; (1024. * 0.1 * 1024.) as usize])
            }
        };
        let choice_line: Line = Default::default();
        for mut line in res.lines {
            let instant = Instant::now();
            if client(format!("https:{}", line.probe_url))
                .send()
                .await?
                .status()
                == 200
            {
                line.cost = instant.elapsed().as_millis();
                println!("{}: {}", line.query, line.cost);
                // if choice_line.cost > line.cost {
                //     choice_line = line
                // }
            };
        }
        Ok(choice_line)
    }
}

impl Default for Line {
    fn default() -> Self {
        Line {
            os: "upos".to_string(),
            probe_url: "//upos-sz-upcdnbda2.bilivideo.com/OK".to_string(),
            query: "upcdn=bda2&probe_version=20200810".to_string(),
            cost: u128::MAX,
        }
    }
}

pub(crate) fn read_chunk(mut file: File, len: usize) -> impl Stream<Item = Result<Bytes>> {
    let mut buffer = vec![0u8; len];

    let mut buf = BytesMut::with_capacity(len);
    try_stream! {
        loop {
            let n = file.read(&mut buffer).await?;
            buf.put_slice(&buffer[..n]);
        // println!("{:?}", buf);
            if n == 0 {
                return;
            }
            yield buf.split().freeze();
        }
    }
}
