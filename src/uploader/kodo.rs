use crate::read_chunk;
use crate::Video;
use anyhow::{anyhow, bail, Context, Result};
use async_std::fs::File;
use async_stream::try_stream;
use base64::URL_SAFE;
use futures::stream::IntoStream;
use futures::{Stream, StreamExt, TryStreamExt};
use reqwest::header;
use reqwest::header::{HeaderMap, HeaderName};
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use reqwest_retry::policies::ExponentialBackoff;
use reqwest_retry::RetryTransientMiddleware;
use serde::{Deserialize, Serialize};
use serde_json::{json, Map};
use std::collections::HashMap;
use std::ffi::OsStr;
use std::path::PathBuf;
use std::pin::Pin;
use std::str::FromStr;
use std::time::{Duration, Instant};

pub struct Kodo {
    client: ClientWithMiddleware,
    bucket: Bucket,
    url: String,
}

impl Kodo {
    pub async fn from(bucket: Bucket) -> Result<Self> {
        let mut headers = header::HeaderMap::new();
        headers.insert(
            "Authorization",
            format!("UpToken {}", bucket.uptoken).parse()?,
        );
        let client = reqwest::Client::builder()
            .user_agent("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 Chrome/63.0.3239.108")
            .default_headers(headers)
            .timeout(Duration::new(60, 0))
            .build()
            .unwrap();
        let retry_policy = ExponentialBackoff::builder().build_with_max_retries(3);
        let client = ClientBuilder::new(client)
            // Retry failed requests.
            .with(RetryTransientMiddleware::new_with_policy(retry_policy))
            .build();
        let url = format!("https:{}/mkblk", bucket.endpoint); // 视频上传路径
        Ok(Kodo {
            client,
            bucket,
            url,
        })
    }

    pub async fn upload_stream(
        self,
        file: File,
        path: &PathBuf,
        mut process: impl FnMut(usize) -> bool,
    ) -> Result<Video> {
        let total_size = file.metadata().await?.len();
        let chunk_size = 4194304;
        let mut parts = Vec::new();
        // let parts_cell = &RefCell::new(parts);
        let chunks_num = (total_size as f64 / chunk_size as f64).ceil() as usize; // 获取分块数量
                                                                                  // let file = tokio::io::BufReader::with_capacity(chunk_size, file);
        let client = &self.client;
        let url = &self.url;

        let mut stream = read_chunk(file, chunk_size)
            // let mut chunks = read_chunk(file, chunk_size)
            .enumerate()
            .map(|(i, chunk)| async move {
                let chunk = chunk?;
                let len = chunk.len();
                // println!("{}", len);
                let url = format!("{url}/{len}");
                let mut ctx: serde_json::Value =
                    client.post(url).body(chunk).send().await?.json().await?;
                Ok::<_, reqwest_middleware::Error>((
                    Ctx {
                        index: i,
                        ctx: ctx["ctx"].as_str().unwrap_or_default().into(),
                    },
                    len,
                ))
            })
            .buffer_unordered(3);
        tokio::pin!(stream);
        while let Some((part, size)) = stream.try_next().await? {
            parts.push(part);
            if !process(size) {
                bail!("移除视频");
            }
        }
        parts.sort_by_key(|x| x.index);
        let key = base64::encode_config(self.bucket.key, URL_SAFE);
        self.client
            .post(format!(
                "https:{}/mkfile/{total_size}/key/{key}",
                self.bucket.endpoint,
            ))
            .body(
                parts
                    .iter()
                    .map(|x| &x.ctx[..])
                    .collect::<Vec<_>>()
                    .join(","),
            )
            .send()
            .await?;
        let mut headers = HeaderMap::new();
        for (key, value) in self.bucket.fetch_headers {
            headers.insert(HeaderName::from_str(&key)?, value.parse()?);
        }
        // reqwest::header::HeaderName::
        let result: serde_json::Value = self
            .client
            .post(format!("https:{}", self.bucket.fetch_url))
            .headers(headers)
            .send()
            .await?
            .json()
            .await?;
        Ok(match result.get("OK") {
            Some(x) if x.as_i64().ok_or(anyhow!("kodo fetch err"))? != 1 => {
                bail!("{result}")
            }
            _ => Video {
                title: path
                    .file_stem()
                    .and_then(OsStr::to_str)
                    .map(|s| s.to_string()),
                filename: self.bucket.bili_filename,
                desc: "".into(),
            },
        })
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct Ctx {
    index: usize,
    ctx: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Bucket {
    bili_filename: String,
    fetch_url: String,
    endpoint: String,
    uptoken: String,
    key: String,
    fetch_headers: HashMap<String, String>,
}
