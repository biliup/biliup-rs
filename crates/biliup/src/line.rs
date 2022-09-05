use crate::error::Result;
use crate::uploader::cos::Cos;
use crate::uploader::kodo::Kodo;
use crate::uploader::upos::Upos;
use crate::uploader::Uploader;
use crate::{Video, VideoFile, VideoStream};
use futures::{Stream, TryStreamExt};
use reqwest::Body;

use serde::{Deserialize, Serialize};
use serde_json::json;
use std::ffi::OsStr;

use crate::error::CustomError::Custom;
use crate::video::BiliBili;
use std::time::Instant;
use tracing::info;

pub struct Parcel {
    // line: &'a Line,
    line: Bucket,
    video_file: VideoFile,
}

impl Parcel {
    // fn new(line: &'a Line, video_file: VideoFile) -> Parcel<'a> {
    //     let total_size = video_file.total_size;
    //     let file_name = video_file.file_name.clone();
    //     Self {
    //         line,
    //         video_file,
    //     }
    // }

    pub async fn upload<F, S, B>(self, limit: usize, progress: F) -> Result<Video>
    where
        F: FnOnce(VideoStream) -> S,
        S: Stream<Item = Result<(B, usize)>>,
        B: Into<Body> + Clone,
    {
        let mut video = match self.line {
            Bucket::Cos(bucket, enable_internal) => {
                // let bucket = self.pre_upload(client).await?;
                let cos_client = Cos::form_post(bucket).await?;
                let chunk_size = 10485760;
                let parts = cos_client
                    .upload_stream(
                        progress(self.video_file.get_stream(chunk_size)?),
                        self.video_file.total_size,
                        limit,
                        enable_internal,
                    )
                    .await?;
                cos_client.merge_files(parts).await?
            }
            Bucket::Kodo(bucket) => {
                // let bucket = self.pre_upload(client).await?;
                let chunk_size = 4194304;
                Kodo::from(bucket)
                    .await?
                    .upload_stream(
                        progress(self.video_file.get_stream(chunk_size)?),
                        self.video_file.total_size,
                        limit,
                    )
                    .await?
            }
            Bucket::Upos(bucket) => {
                // let bucket: crate::uploader::upos::Bucket = self.pre_upload(client).await?;
                let chunk_size = bucket.chunk_size;
                let upos = Upos::from(bucket).await?;
                let mut parts = Vec::new();
                let stream = upos
                    .upload_stream(
                        progress(self.video_file.get_stream(chunk_size)?),
                        self.video_file.total_size,
                        limit,
                    )
                    .await?;
                tokio::pin!(stream);
                while let Some((part, _size)) = stream.try_next().await? {
                    parts.push(part);
                }
                upos.get_ret_video_info(&parts, &self.video_file.filepath)
                    .await?
            }
        };
        // let mut video = match self.line.os {
        //     Uploader::Upos => {
        //         let bucket: crate::uploader::upos::Bucket = self.pre_upload(client).await?;
        //         let chunk_size = bucket.chunk_size;
        //         let upos = Upos::from(bucket).await?;
        //         let mut parts = Vec::new();
        //         let stream = upos
        //             .upload_stream(
        //                 progress(self.video_file.get_stream(chunk_size)?),
        //                 self.video_file.total_size,
        //                 limit,
        //             )
        //             .await?;
        //         tokio::pin!(stream);
        //         while let Some((part, _size)) = stream.try_next().await? {
        //             parts.push(part);
        //         }
        //         upos.get_ret_video_info(&parts, &self.video_file.filepath)
        //             .await?
        //     }
        //     Uploader::Kodo => {
        //         let bucket = self.pre_upload(client).await?;
        //         let chunk_size = 4194304;
        //         Kodo::from(bucket)
        //             .await?
        //             .upload_stream(
        //                 progress(self.video_file.get_stream(chunk_size)?),
        //                 self.video_file.total_size,
        //                 limit,
        //             )
        //             .await?
        //     }
        //     Uploader::Bos => {
        //         panic!()
        //     }
        //     Uploader::Gcs => {
        //         panic!()
        //     }
        //     Uploader::Cos => {
        //         let bucket = self.pre_upload(client).await?;
        //         let cos_client = Cos::form_post(bucket).await?;
        //         let chunk_size = 10485760;
        //         let enable_internal = self.line.probe_url == "internal";
        //         let parts = cos_client
        //             .upload_stream(
        //                 progress(self.video_file.get_stream(chunk_size)?),
        //                 self.video_file.total_size,
        //                 limit,
        //                 enable_internal,
        //             )
        //             .await?;
        //         cos_client.merge_files(parts).await?
        //     }
        // };
        if video.title == None {
            video.title = self
                .video_file
                .filepath
                .file_stem()
                .and_then(OsStr::to_str)
                .map(|s| s.to_string())
        };
        Ok(video)
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
                info!("{}: {}", line.query, line.cost);
                if choice_line.cost > line.cost {
                    choice_line = line
                }
            };
        }
        Ok(choice_line)
    }
}

enum Bucket {
    Cos(crate::uploader::cos::Bucket, bool),
    Kodo(crate::uploader::kodo::Bucket),
    Upos(crate::uploader::upos::Bucket),
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
    // pub fn to_uploader(&self, filepath: VideoFile) -> Parcel<'_> {
    //     Parcel::new(self, filepath)
    // }

    pub async fn pre_upload(&self, bili: &BiliBili, video_file: VideoFile) -> Result<Parcel> {
        let total_size = video_file.total_size;
        let file_name = video_file.file_name.clone();
        let profile = if let Uploader::Upos = self.os {
            "ugcupos/bup"
        } else {
            "ugcupos/bupfetch"
        };
        let params = json!({
            "r": self.os,
            "profile": profile,
            "ssl": 0,
            "version": "2.11.0",
            "build": 2110000,
            "name": file_name,
            "size": total_size,
        });
        info!("pre_upload: {}", params);
        let response = bili
            .client
            .get(format!(
                "https://member.bilibili.com/preupload?{}",
                self.query
            ))
            .query(&params)
            .send()
            .await?;
        if !response.status().is_success() {
            return Err(Custom(format!(
                "Failed to pre_upload from {}",
                response.text().await?
            )));
        }
        match self.os {
            Uploader::Upos => Ok(Parcel {
                line: Bucket::Upos(response.json().await?),
                video_file,
            }),
            Uploader::Kodo => Ok(Parcel {
                line: Bucket::Kodo(response.json().await?),
                video_file,
            }),
            Uploader::Bos | Uploader::Gcs => {
                panic!("unsupported")
            }
            Uploader::Cos => Ok(Parcel {
                line: Bucket::Cos(response.json().await?, self.probe_url == "internal"),
                video_file,
            }),
        }
    }

    // pub async fn pre_upload<T: DeserializeOwned>(&self, client: &reqwest::Client, video_file: VideoFile) -> Result<T> {
    //     let total_size = video_file.total_size;
    //     let file_name = video_file.file_name.clone();
    //     let profile = if let Uploader::Upos = line.os {
    //         "ugcupos/bup"
    //     } else {
    //         "ugcupos/bupfetch"
    //     };
    //     let params = json!({
    //         "r": self.os,
    //         "profile": profile,
    //         "ssl": 0,
    //         "version": "2.11.0",
    //         "build": 2110000,
    //         "name": file_name,
    //         "size": total_size,
    //     });
    //     info!("pre_upload: {}", params);
    //     let response = client
    //         .get(format!(
    //             "https://member.bilibili.com/preupload?{}",
    //             self.query
    //         ))
    //         .query(&self.params)
    //         .send()
    //         .await?;
    //     if !response.status().is_success() {
    //         return Err(Custom(format!(
    //             "Failed to pre_upload from {}",
    //             response.text().await?
    //         )));
    //     }
    //     Ok(response.json().await?)
    // }
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
