use crate::error::Result;
use crate::uploader::{Uploader, VideoFile, VideoStream};
use futures::{Stream, TryStreamExt};
use reqwest::{Body, RequestBuilder};

use serde::{Deserialize, Serialize};
use serde_json::json;
use std::ffi::OsStr;

use crate::client::StatelessClient;
use crate::error::Kind::Custom;
use crate::uploader::bilibili::{BiliBili, Video};
use crate::uploader::line::upos::Upos;
use std::time::Instant;
use tracing::info;

pub mod upos;

pub struct Parcel {
    // line: &'a Line,
    line: Bucket,
    video_file: VideoFile,
}

impl Parcel {
    pub async fn upload<F, S, B>(
        self,
        client: StatelessClient,
        limit: usize,
        progress: F,
    ) -> Result<Video>
    where
        F: FnOnce(VideoStream) -> S,
        S: Stream<Item = Result<(B, usize)>>,
        B: Into<Body> + Clone,
    {
        let mut video = match self.line {
            Bucket::Upos(bucket) => {
                // let bucket: crate::uploader::upos::Bucket = self.pre_upload(client).await?;
                let chunk_size = bucket.chunk_size;
                let upos = Upos::from(client, bucket).await?;
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

        if video.title.is_none() {
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
    pub async fn probe(client: &reqwest::Client) -> Result<Line> {
        let res: Self = client
            .get("https://member.bilibili.com/preupload?r=probe")
            .send()
            .await?
            .json()
            .await?;
        // let client = res.ping(client);
        let mut choice_line: Line = Default::default();
        for mut line in res.lines {
            let instant = Instant::now();
            if Probe::ping(&res.probe, &format!("https:{}", line.probe_url), client)
                .send()
                .await?
                .status()
                .is_success()
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

    fn ping(probe: &serde_json::Value, url: &str, client: &reqwest::Client) -> RequestBuilder {
        if !probe["get"].is_null() {
            client.get(url)
        } else {
            client
                .post(url)
                .body(vec![0; (1024. * 0.1 * 1024.) as usize])
        }
    }
}

enum Bucket {
    Upos(upos::Bucket),
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

        let mut json_response: serde_json::Value = response.json().await?;

        if let Uploader::Upos = self.os {
            let upcdn = self.query.split('&')
                            .find_map(|s| {
                                let mut split = s.splitn(2, '=');
                                match (split.next(), split.next()) {
                                    (Some("upcdn"), Some(value)) => Some(value),
                                    _ => None,
                                }
                            })
                            .expect("upcdn parameter is missing");
            match upcdn  {
                "ws" => json_response["endpoint"] = serde_json::to_value("//upos-cs-upcdnws.bilivideo.com").unwrap(),
                "qn" => json_response["endpoint"] = serde_json::to_value("//upos-cs-upcdnqn.bilivideo.com").unwrap(),
                "bldsa" => json_response["endpoint"] = serde_json::to_value("//upos-cs-upcdnbldsa.bilivideo.com").unwrap(),
                "tx" => json_response["endpoint"] = serde_json::to_value("//upos-cs-upcdntx.bilivideo.com").unwrap(),
                "txa" => json_response["endpoint"] = serde_json::to_value("//upos-cs-upcdntxa.bilivideo.com").unwrap(),
                "bda" => json_response["endpoint"] = serde_json::to_value("//upos-cs-upcdnbda.bilivideo.com").unwrap(),
                _ => (),  // No modification for other cases
            }
        }

        match self.os {
            Uploader::Upos => Ok(Parcel {
                line: Bucket::Upos(serde_json::from_value::<upos::Bucket>(json_response)?),
                video_file,
            })
        }
    }
}

impl Default for Line {
    fn default() -> Self {
        Line {
            os: Uploader::Upos,
            probe_url: "//upos-cs-upcdnbda2.bilivideo.com/OK".to_string(),
            query: "probe_version=20221109&upcdn=bda2&zone=cs".to_string(),
            cost: u128::MAX,
        }
    }
}

pub fn bda2() -> Line {
    Line {
        os: Uploader::Upos,
        query: "probe_version=20221109&upcdn=bda2&zone=cs".into(),
        probe_url: "//upos-cs-upcdnbda2.bilivideo.com/OK".into(),
        cost: 0,
    }
}

pub fn ws() -> Line {
    Line {
        os: Uploader::Upos,
        query: "probe_version=20221109&upcdn=ws&zone=cs".into(),
        probe_url: "//upos-cs-upcdnws.bilivideo.com/OK".into(),
        cost: 0,
    }
}

pub fn qn() -> Line {
    Line {
        os: Uploader::Upos,
        query: "probe_version=20221109&upcdn=qn&zone=cs".into(),
        probe_url: "//upos-cs-upcdnqn.bilivideo.com/OK".into(),
        cost: 0,
    }
}

pub fn bldsa() -> Line {
    Line {
        os: Uploader::Upos,
        query: "zone=cs&upcdn=bldsa&probe_version=20221109".into(),
        probe_url: "//upos-cs-upcdnbldsa.bilivideo.com/OK".into(),
        cost: 0,
    }
}

pub fn tx() -> Line {
    Line {
        os: Uploader::Upos,
        query: "zone=cs&upcdn=tx&probe_version=20221109".into(),
        probe_url: "//upos-cs-upcdntx.bilivideo.com/OK".into(),
        cost: 0,
    }
}

pub fn txa() -> Line {
    Line {
        os: Uploader::Upos,
        query: "zone=cs&upcdn=txa&probe_version=20221109".into(),
        probe_url: "//upos-cs-upcdntxa.bilivideo.com/OK".into(),
        cost: 0,
    }
}

pub fn bda() -> Line {
    Line {
        os: Uploader::Upos,
        query: "probe_version=20221109&upcdn=bda&zone=cs".into(),
        probe_url: "//upos-cs-upcdnbda.bilivideo.com/OK".into(),
        cost: 0,
    }
}
