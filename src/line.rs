use crate::client::Client;
use crate::uploader::kodo::Kodo;
use crate::uploader::upos::Upos;
use crate::uploader::Uploader;
use crate::Video;
use anyhow::{bail, Context, Result};
use async_std::fs::File;
use futures::TryStreamExt;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::path::{Path, PathBuf};
use std::time::Instant;

pub struct Parcel<'a> {
    line: &'a Line,
    filepath: PathBuf,
    params: serde_json::Value,
    pub total_size: u64,
}

impl<'a> Parcel<'a> {
    async fn new(line: &'a Line, filepath: &Path) -> Result<Parcel<'a>> {
        let file = File::open(&filepath).await?;
        let total_size = file.metadata().await?.len();
        let file_name = filepath.file_name().ok_or("No filename").unwrap().to_str();
        let profile = if let Uploader::Upos = line.os {
            "ugcupos/bup"
        } else {
            "ugcupos/bupfetch"
        };
        let params = json!({
            "r": line.os,
            "profile": profile,
            "ssl": 0,
            "version": "2.8.12",
            "build": 2081200,
            "name": file_name,
            "size": total_size,
        });
        println!("pre_upload: {}", params);
        Ok(Self {
            line,
            filepath: filepath.into(),
            params,
            total_size,
        })
    }

    pub async fn pre_upload<T: DeserializeOwned>(&self, login: &Client) -> Result<T> {
        Ok(login
            .client
            .get(format!(
                "https://member.bilibili.com/preupload?{}",
                self.line.query
            ))
            .query(&self.params)
            .send()
            .await?
            .json()
            .await
            .with_context(|| "Failed to pre_upload from".to_string())?)
    }

    pub async fn upload(
        &self,
        client: &Client,
        limit: usize,
        mut process: impl FnMut(usize) -> bool,
    ) -> Result<Video> {
        let file = File::open(&self.filepath).await?;
        // let total_size = file.metadata().await?.len();
        // let file_name = filepath.file_name().ok_or("No filename")?.to_str();
        match self.line.os {
            Uploader::Upos => {
                let bucket = self.pre_upload(client).await?;
                let upos = Upos::from(bucket).await?;
                let mut parts = Vec::new();
                let stream = upos.upload_stream(file, limit).await?;
                tokio::pin!(stream);
                while let Some((part, size)) = stream.try_next().await? {
                    parts.push(part);
                    if !process(size) {
                        bail!("移除视频");
                    }
                }
                upos.get_ret_video_info(&parts, &self.filepath).await
            }
            Uploader::Kodo => {
                let bucket = self.pre_upload(client).await?;
                Kodo::from(bucket)
                    .await?
                    .upload_stream(file, &self.filepath, limit, process)
                    .await
            }
            Uploader::Bos => {
                panic!()
            }
            Uploader::Gcs => {
                panic!()
            }
            Uploader::Cos => {
                panic!()
            }
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
        let mut choice_line: Line = Default::default();
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
                if choice_line.cost > line.cost {
                    choice_line = line
                }
            };
        }
        Ok(choice_line)
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Line {
    os: Uploader,
    probe_url: String,
    query: String,
    #[serde(skip)]
    cost: u128,
}

impl Line {
    pub async fn to_uploader(&self, filepath: &Path) -> Result<Parcel<'_>> {
        Parcel::new(self, filepath).await
    }
}

impl Default for Line {
    fn default() -> Self {
        Line {
            os: Uploader::Upos,
            probe_url: "//upos-sz-upcdnbda2.bilivideo.com/OK".to_string(),
            query: "upcdn=bda2&probe_version=20200810".to_string(),
            cost: u128::MAX,
        }
    }
}

pub fn kodo() -> Line {
    Line {
        os: Uploader::Kodo,
        query: "bucket=bvcupcdnkodobm&probe_version=20200810".into(),
        probe_url: "//up-na0.qbox.me/crossdomain.xml".into(),
        cost: 0,
    }
}

pub fn bda2() -> Line {
    Line {
        os: Uploader::Upos,
        query: "upcdn=bda2&probe_version=20200810".into(),
        probe_url: "//upos-sz-upcdnbda2.bilivideo.com/OK".into(),
        cost: 0,
    }
}

pub fn ws() -> Line {
    Line {
        os: Uploader::Upos,
        query: "upcdn=ws&probe_version=20200810".into(),
        probe_url: "//upos-sz-upcdnws.bilivideo.com/OK".into(),
        cost: 0,
    }
}

pub fn qn() -> Line {
    Line {
        os: Uploader::Upos,
        query: "upcdn=qn&probe_version=20200810".into(),
        probe_url: "//upos-sz-upcdnqn.bilivideo.com/OK".into(),
        cost: 0,
    }
}
