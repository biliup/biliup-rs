use crate::video::{Studio, Video};
use anyhow::{anyhow, Result};
use async_stream::try_stream;
use bytes::Buf;
use bytes::{BufMut, Bytes, BytesMut};
use futures::{AsyncReadExt, Stream, TryStream};
use reqwest::Body;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Read};
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
    buf: BytesMut,
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
            buf: BytesMut::with_capacity(capacity),
            buffer: vec![0u8; capacity],
            file,
        }
    }

    pub fn read(&mut self) -> Result<Option<(Bytes, usize)>> {
        // println!("cap {}", self.buf.capacity());
        let n = self.file.read(&mut self.buffer)?;
        // println!("cur size: {n}");
        if n == 0 {
            return Ok(None);
        }
        self.buf.put_slice(&self.buffer[..n]);
        // println!("cap2 {}", self.buf.capacity());
        Ok(Some((self.buf.split().freeze(), n)))
    }
}

impl Stream for VideoStream {
    type Item = Result<(Bytes, usize)>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match self.read()? {
            None => Poll::Ready(None),
            Some(b) => Poll::Ready(Some(Ok(b))),
        }
    }
}

pub struct VideoFile {
    // capacity: usize,
    // buf: BytesMut,
    // buffer: Vec<u8>,
    // video_stream: Option<VideoStream>,
    // file info
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
            // video_stream: None,
            file,
            // capacity: 10485760,
            // buf: Default::default(),
            // buffer: Default::default(),
            total_size,
            file_name: file_name.into(),
            filepath: filepath.into(),
        })
    }

    // pub fn set_capacity(&mut self, capacity: usize) {
    //     self.capacity = capacity;
    //     self.buffer = vec![0u8; capacity];
    //     self.buf = BytesMut::with_capacity(capacity);
    // }
    //
    // pub fn read(&mut self) -> Result<Option<Bytes>> {
    //     // println!("cap {}", self.buf.capacity());
    //     let n = self.file.read(&mut self.buffer)?;
    //     // println!("cur size: {n}");
    //     if n == 0 {
    //         return Ok(None);
    //     }
    //     self.buf.put_slice(&self.buffer[..n]);
    //     // println!("cap2 {}", self.buf.capacity());
    //     Ok(Some(self.buf.split().freeze()))
    // }

    pub fn get_stream(&self, capacity: usize) -> Result<VideoStream> {
        Ok(VideoStream::with_capacity(self.file.try_clone()?, capacity))
    }
}

// impl Stream for VideoFile {
//     type Item = Result<Bytes>;
//
//     fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
//         // let mut buffer = vec![0u8; self.capacity];
//         // self.file.read(buffer.as_mut());
//         // let n = self.file.read(buffer.as_mut())?;
//         // self.buf.put_slice(&buffer[..n]);
//         // // println!("{:?}", buf);
//         // // self.test(1);
//         //
//         // if n == 0 {
//         //     return Poll::Ready(None);
//         // }
//         // Poll::Ready(Some(Ok(self.buf.split().freeze())))
//         match self.read()? {
//             None => {Poll::Ready(None)}
//             Some(b) => {Poll::Ready(Some(Ok(b)))}
//         }
//     }
// }

pub fn read_chunk(mut file: std::fs::File, len: usize) -> impl Stream<Item = Result<Bytes>> {
    let mut buffer = vec![0u8; len];
    // let mut buffer = [0; len];

    let mut buf = BytesMut::with_capacity(len);
    try_stream! {
        loop {
            // let n = file.read(&mut buffer).await?;
            let n = file.read(&mut buffer)?;
            buf.put_slice(&buffer[..n]);
        // println!("{:?}", buf);
            if n == 0 {
                return;
            }
            // if !process(n) {
            //     // bail!("移除视频");
            //     Err(anyhow!("移除视频"))?;
            // }
            yield buf.split().freeze();
        }
        // loop {
        //     // let n = file.read(&mut buffer).await?;
        //     let n = file.read(&mut buffer)?;
        //     // buf.put_slice(&buffer[..n]);
        // // println!("{:?}", buf);
        //     if n == 0 {
        //         return;
        //     }
        //     // if !process(n) {
        //     //     // bail!("移除视频");
        //     //     Err(anyhow!("移除视频"))?;
        //     // }
        //     yield (&buffer[..]).copy_to_bytes(n);
        // }
    }
    // loop {
    //     // let n = file.read(&mut buffer).await?;
    //     let n = file.read(&mut buffer)?;
    //     // buf.put_slice(&buffer[..n]);
    //     // println!("{:?}", buf);
    //     if n == 0 {
    //         // return;
    //     }
    //     // if !process(n) {
    //     //     // bail!("移除视频");
    //     //     Err(anyhow!("移除视频"))?;
    //     // }
    //     Buf::copy_to_bytes();
    //     (&buffer[..]).copy_to_bytes(n)
    //     // yield buf.split().freeze();
    // }
}

#[cfg(test)]
mod tests {
    use crate::client::Client;
    use crate::video::{BiliBili, Studio, Video};
    use anyhow::Result;
    use bytes::{Buf, Bytes};

    #[tokio::test]
    async fn it_works() -> Result<()> {
        let mut bytes = (&b"hello world"[..]).copy_to_bytes(11);
        let mut bytesc = bytes.clone();
        println!("{:?}", bytes);
        // let mut bytes = Bytes::;

        let bytes1 = bytes.copy_to_bytes(6);
        // bytes =
        // bytes1.remaining()
        // let bytes1 = bytes.copy_to_bytes(5);
        println!("{:?}", bytes1);
        // let bytes2 = bytes.take(6).copy_to_bytes(6);
        // bytes.advance(1);
        let bytes2 = bytes.copy_to_bytes(5);
        println!("{:?}", bytes2);
        // assert_eq!(&bytes[..], &b"hello"[..]);
        // assert_eq!(&bytes[..], &b"hello"[..]);
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
