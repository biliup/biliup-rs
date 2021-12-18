use async_std::fs::File;
use std::time::{Duration, Instant};
use crate::video::Video;
use anyhow::Result;
use crate::uploader::upos::Bucket;
use serde::{Deserialize, Serialize};
pub mod upos;

pub enum UploadStatus {
    Processing(usize),
    Completed(Video)
}

// pub struct Upload {
//     pub uploader: Uploader,
//     pub file: File,
//     pub path: async_std::path::PathBuf,
//     pub ret: Bucket,
// }
//
// impl Upload {
//     pub fn new(
//         uploader: Uploader,
//         file: File,
//         path: async_std::path::PathBuf,
//         ret: Bucket,
//     ) -> Self {
//         Upload {
//             uploader,
//             file,
//             path,
//             ret,
//         }
//     }
//
//     pub async fn upload(&mut self) ->Result<Video> {
//         Ok(self.uploader.upload(&mut self).await?)
//     }
//
//     pub async fn total_size(&self) -> Result<u64> {
//         Ok(self.file.metadata().await?.len())
//     }
// }
//
#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum Uploader {
    Upos,
    Kodo,
    Bos,
    Gcs,
    Cos
}

// impl Uploader {
//     pub async fn upload(
//         self,
//         upload: &mut Upload,
//     ) -> Result<Video> {
//         match self {
//             Uploader::Upos => {upos::Upos::new(bucket).upload(upload)}
//             Uploader::Kodo => {panic!()}
//             Uploader::Bos => {panic!()}
//             Uploader::Gcs => {panic!()}
//             Uploader::Cos => {panic!()}
//         }
//     }
// }
//
// impl From<String> for Uploader {
//     fn from(os: String) -> Self {
//         match &os[..] {
//             "upos" => Self::Upos(),
//             "kodo" => Self::Kodo,
//             "gcs" => Self::Gcs,
//             "bos" => Self::Bos,
//             "cos" => Self::Cos,
//             unknown @ _ => panic!("{}", unknown)
//         }
//     }
// }
