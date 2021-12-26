use crate::uploader::upos::{Bucket, Upos};
use crate::video::Video;
use anyhow::Result;
use async_std::fs::File;
use futures::Stream;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::{Duration, Instant};

pub mod upos;

pub enum UploadStatus {
    Processing(usize),
    Completed(Video),
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum Uploader {
    Upos,
    Kodo,
    Bos,
    Gcs,
    Cos,
}

impl Uploader {
    pub async fn upload(&self, bucket: Bucket, file: File, filepath: &PathBuf) -> Result<Video> {
        match self {
            Uploader::Upos => Upos::form(bucket).await?.upload(file, filepath).await,
            Uploader::Kodo => {
                panic!()
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

    pub async fn upload_stream<'a>(
        &self,
        bucket: Bucket,
        file: File,
        filepath: &'a PathBuf,
    ) -> Result<impl Stream<Item = Result<UploadStatus>> + 'a> {
        match self {
            Uploader::Upos => {
                Upos::form(bucket)
                    .await?
                    .upload_stream(file, filepath)
                    .await
            }
            Uploader::Kodo => {
                panic!()
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
