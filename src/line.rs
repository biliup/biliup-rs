use std::io::Read;
use std::ops::Deref;
use crate::client::Client;
use crate::uploader::kodo::Kodo;
use crate::uploader::upos::Upos;
use crate::uploader::{cos, Uploader};
use crate::{read_chunk, Video, VideoFile, VideoStream};
use anyhow::{bail, Context, Result};
use futures::{Stream, TryStream, TryStreamExt};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::Arc;
use std::time::Instant;
use async_stream::{stream, try_stream};
use bytes::{Buf, Bytes, BytesMut};
use reqwest::Body;
use crate::uploader::cos::Cos;
use futures::StreamExt;

pub struct Parcel<'a> {
    line: &'a Line,
    video_file: VideoFile,
    params: serde_json::Value,
}

impl<'a> Parcel<'a> {
    fn new(line: &'a Line, video_file: VideoFile) -> Parcel<'a> {
        let total_size = video_file.total_size;
        let file_name = video_file.file_name.clone();
        let profile = if let Uploader::Upos = line.os {
            "ugcupos/bup"
        } else {
            "ugcupos/bupfetch"
        };
        let params = json!({
            "r": line.os,
            "profile": profile,
            "ssl": 0,
            "version": "2.11.0",
            "build": 2110000,
            "name": file_name,
            "size": total_size,
        });
        println!("pre_upload: {}", params);
        Self {
            line,
            params,
            video_file
        }
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

    pub async fn upload<F, S, B>(
        &self,
        client: &Client,
        limit: usize,
        progress: F
    ) -> Result<Video>
    where
        F: FnOnce(VideoStream) -> S,
        S: Stream<Item=Result<(B, usize)>>,
        B: Into<Body> + Clone,
    {
        // let file = std::fs::File::open(&self.filepath)?;
        // let total_size = file.metadata()?.len();
        // let stream = |chunk_size| {
        //     read_chunk(file, chunk_size)
        // };

        // let file_name = filepath.file_name().ok_or("No filename")?.to_str();
        match self.line.os {
            Uploader::Upos => {
                let bucket: crate::uploader::upos::Bucket = self.pre_upload(client).await?;
                let chunk_size = bucket.chunk_size;
                let upos = Upos::from(bucket).await?;
                let mut parts = Vec::new();
                let stream = upos.upload_stream(progress(self.video_file.get_stream(chunk_size)?), self.video_file.total_size, limit).await?;
                tokio::pin!(stream);
                while let Some((part, size)) = stream.try_next().await? {
                    parts.push(part);
                    // if !process(size) {
                    //     bail!("移除视频");
                    // }
                }
                upos.get_ret_video_info(&parts, &self.video_file.filepath).await
            }
            Uploader::Kodo => {
                let bucket = self.pre_upload(client).await?;
                let chunk_size = 4194304;
                Kodo::from(bucket)
                    .await?
                    .upload_stream(progress(self.video_file.get_stream(chunk_size)?), self.video_file.total_size, &self.video_file.filepath, limit)
                    .await
            }
            Uploader::Bos => {
                panic!()
            }
            Uploader::Gcs => {
                panic!()
            }
            Uploader::Cos => {
                let bucket = self.pre_upload(client).await?;
                let cos_client = Cos::form_post(bucket).await?;
                // let tp = process;
                // let stream =  |chunk_size| {
                //     stream(chunk_size).map(|chunk| {
                //         let async_stream = stream! {
                //             let mut  content_bytes = chunk?.clone();
                //             loop {
                //                 let n = content_bytes.remaining();
                //                 if n == 0 {
                //                     return;
                //                 } else if n < 4194304 {
                //                     tp(n);
                //                     // process(n);
                //                     yield Ok::<_, anyhow::Error>(content_bytes.copy_to_bytes(n))
                //                 } else {
                //                     yield Ok::<_, anyhow::Error>(content_bytes.copy_to_bytes(4194304))
                //                 }
                //             }
                //         };
                //         Body::wrap_stream(async_stream)
                //     })
                // };
                let chunk_size = 10485760;
                let enable_internal = self.line.probe_url == "internal";
                let parts = cos_client.upload_stream(progress(self.video_file.get_stream(chunk_size)?), self.video_file.total_size, limit, enable_internal).await?;
                let video = cos_client.merge_files(parts).await?;
                Ok(video)
            }
        }
    }
}

pub struct ProgressBody<F>
    where F: FnMut(usize) -> bool {
    content: Bytes,
    progress: F
}

impl <F> ProgressBody<F> where F: FnMut(usize) -> bool {
    fn new(content: Bytes, progress: F) -> Self {
        ProgressBody{
            content,
            progress
        }
    }
}

impl <F> From<ProgressBody<F>> for Body
    where F: FnMut(usize) -> bool {
    fn from(content: ProgressBody<F>) -> Self {
        let mut  content_bytes = content.content.clone();
        let async_stream = stream! {
            loop {
                let n = content_bytes.remaining();
                if n == 0 {
                    return;
                } else if n < 4194304 {
                    yield Ok::<_, anyhow::Error>(content_bytes.copy_to_bytes(n))
                } else {
                    yield Ok::<_, anyhow::Error>(content_bytes.copy_to_bytes(4194304))
                }
            }
        };
        Body::wrap_stream(async_stream)
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
    pub fn to_uploader(&self, filepath: VideoFile) -> Parcel<'_> {
        Parcel::new(self, filepath)
    }
}

impl Default for Line {
    fn default() -> Self {
        Line {
            os: Uploader::Upos,
            probe_url: "//upos-sz-upcdnbda2.bilivideo.com/OK".to_string(),
            query: "upcdn=bda2&probe_version=20211012".to_string(),
            cost: u128::MAX,
        }
    }
}

pub fn kodo() -> Line {
    Line {
        os: Uploader::Kodo,
        query: "bucket=bvcupcdnkodobm&probe_version=20211012".into(),
        probe_url: "//up-na0.qbox.me/crossdomain.xml".into(),
        cost: 0,
    }
}

pub fn bda2() -> Line {
    Line {
        os: Uploader::Upos,
        query: "upcdn=bda2&probe_version=20211012".into(),
        probe_url: "//upos-sz-upcdnbda2.bilivideo.com/OK".into(),
        cost: 0,
    }
}

pub fn ws() -> Line {
    Line {
        os: Uploader::Upos,
        query: "upcdn=ws&probe_version=20211012".into(),
        probe_url: "//upos-sz-upcdnws.bilivideo.com/OK".into(),
        cost: 0,
    }
}

pub fn qn() -> Line {
    Line {
        os: Uploader::Upos,
        query: "upcdn=qn&probe_version=20211012".into(),
        probe_url: "//upos-sz-upcdnqn.bilivideo.com/OK".into(),
        cost: 0,
    }
}

pub fn cos() -> Line {
    Line {
        os: Uploader::Cos,
        query: "&probe_version=20211012&r=cos&profile=ugcupos%2Fbupfetch&ssl=0&version=2.10.4.0&build=2100400&webVersion=2.0.0".into(),
        probe_url: "".into(),
        cost: 0,
    }
}

pub fn cos_internal() -> Line {
    Line {
        os: Uploader::Cos,
        query: "".into(),
        probe_url: "internal".into(),
        cost: 0,
    }
}
