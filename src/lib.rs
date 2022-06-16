use crate::video::{Studio, Video};
use anyhow::{anyhow, Result};
use bytes::Bytes;
use futures::Stream;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::Read;
use std::path::Path;
use std::pin::Pin;
use std::task::{Context, Poll};

pub mod client;
pub mod error;
pub mod line;
pub mod video;

pub mod uploader {
    use serde::{Deserialize, Serialize};
    pub mod cos;
    pub mod kodo;
    pub mod retryable;
    pub mod upos;

    #[derive(Deserialize, Serialize, Debug)]
    #[serde(rename_all = "lowercase")]
    pub enum Uploader {
        Upos,
        Kodo,
        Bos,
        Gcs,
        Cos,
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct User {
    pub account: Account,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Account {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub user: Option<User>,
    pub line: Option<String>,
    #[serde(default = "default_limit")]
    pub limit: usize,
    pub streamers: HashMap<String, Studio>,
}

fn default_limit() -> usize {
    3
}

pub fn load_config(config: &Path) -> Result<Config> {
    let file = std::fs::File::open(config)?;
    let config: Config = serde_yaml::from_reader(file)?;
    // println!("body = {:?}", client);
    Ok(config)
}

pub struct VideoStream {
    pub capacity: usize,
    buffer: Vec<u8>,
    pub file: std::fs::File,
}

impl VideoStream {
    pub fn with_capacity(file: std::fs::File, capacity: usize) -> Self {
        // self.capacity = capacity;
        // self.buffer = vec![0u8; capacity];
        // self.buf = BytesMut::with_capacity(capacity);
        VideoStream {
            capacity,
            buffer: vec![0u8; capacity],
            file,
        }
    }

    pub fn read(&mut self) -> Result<Option<Bytes>> {
        // println!("cap {}", self.buf.capacity());
        let n = self.file.read(&mut self.buffer)?;
        // println!("cur size: {n}");
        if n == 0 {
            return Ok(None);
        }
        // self.buf.put_slice(&self.buffer[..n]);
        // println!("cap2 {}", self.buf.capacity());
        Ok(Some(Bytes::copy_from_slice(&self.buffer[..n])))
    }
}

impl Stream for VideoStream {
    type Item = Result<Bytes>;

    fn poll_next(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match self.read()? {
            None => Poll::Ready(None),
            Some(b) => Poll::Ready(Some(Ok(b))),
        }
    }
}

pub struct VideoFile {
    pub total_size: u64,
    pub file_name: String,
    pub filepath: std::path::PathBuf,
    pub file: std::fs::File,
}

impl VideoFile {
    pub fn new(filepath: &std::path::Path) -> Result<Self> {
        let file = std::fs::File::open(&filepath)?;
        let total_size = file.metadata()?.len();
        let file_name = filepath
            .file_name()
            .and_then(|file_name| file_name.to_str())
            .ok_or(anyhow!("No filename"))?;
        Ok(Self {
            file,
            // capacity: 10485760,
            total_size,
            file_name: file_name.into(),
            filepath: filepath.into(),
        })
    }

    pub fn get_stream(&self, capacity: usize) -> Result<VideoStream> {
        Ok(VideoStream::with_capacity(self.file.try_clone()?, capacity))
    }
}

#[cfg(test)]
mod tests {
    use crate::video::Vid;
    use anyhow::Result;
    use bytes::Buf;
    use std::num::IntErrorKind::InvalidDigit;
    use std::num::{IntErrorKind, ParseIntError};
    use std::str::FromStr;

    #[tokio::test]
    async fn it_works() -> Result<()> {
        assert_eq!(Ok(Vid::Aid(971158452)), Vid::from_str("971158452"));
        assert_eq!(Ok(Vid::Aid(971158452)), Vid::from_str("av971158452"));
        assert_eq!(
            Ok(Vid::Bvid("BV1ip4y1x7Gi".into())),
            Vid::from_str("BV1ip4y1x7Gi")
        );
        Ok(())
    }

    #[test]
    fn try_clone_stream() {
        let chunks: Vec<Result<_, ::std::io::Error>> = vec![Ok("hello"), Ok(" "), Ok("world")];
        let stream = futures::stream::iter(chunks);
        let client = reqwest::Client::new();
        retry(|| {
            let builder = client
                .get("http://httpbin.org/get")
                .body(reqwest::Body::wrap_stream(stream));
            let clone = builder.try_clone();
            assert!(clone.is_none());
        })
    }

    fn retry(f: impl FnOnce()) {
        f()
    }
}
