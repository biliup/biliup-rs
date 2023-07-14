use anyhow::{Context, Result};
use biliup::client::StatelessClient;
use biliup::error::Kind;
use biliup::uploader::bilibili::{Credit, ResponseData, Studio};
use biliup::uploader::credential::login_by_cookies;
use biliup::uploader::line::Probe;
use biliup::uploader::{line, VideoFile};
use futures::StreamExt;
use pyo3::prelude::*;
use pyo3::pyclass;

use std::path::PathBuf;
use std::time::Instant;
use tracing::info;

#[pyclass]
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum UploadLine {
    Bda2,
    Ws,
    Qn,
    Kodo,
    Cos,
    CosInternal,
    Bldsa,
}

#[derive(FromPyObject)]
pub struct PyCredit {
    #[pyo3(item("type"))]
    type_id: i8,
    #[pyo3(item("raw_text"))]
    raw_text: String,
    #[pyo3(item("biz_id"))]
    biz_id: Option<String>,
}

pub async fn upload(
    video_path: Vec<PathBuf>,
    cookie_file: PathBuf,
    line: Option<UploadLine>,
    limit: usize,
    title: String,
    tid: u16,
    tag: String,
    copyright: u8,
    source: String,
    desc: String,
    dynamic: String,
    cover: String,
    dtime: Option<u32>,
    dolby: u8,
    lossless_music: u8,
    no_reprint: u8,
    open_elec: u8,
    desc_v2_credit: Vec<PyCredit>,
) -> Result<ResponseData> {
    // let file = std::fs::File::options()
    //     .read(true)
    //     .write(true)
    //     .open(&cookie_file);
    let bilibili = login_by_cookies(&cookie_file).await;
    let bilibili = if let Err(Kind::IO(_)) = bilibili {
        bilibili
            .with_context(|| String::from("open cookies file: ") + &cookie_file.to_string_lossy())?
    } else {
        bilibili?
    };
    let client = StatelessClient::default();
    let mut videos = Vec::new();
    let line = match line {
        Some(UploadLine::Kodo) => line::kodo(),
        Some(UploadLine::Bda2) => line::bda2(),
        Some(UploadLine::Ws) => line::ws(),
        Some(UploadLine::Qn) => line::qn(),
        Some(UploadLine::Cos) => line::cos(),
        Some(UploadLine::CosInternal) => line::cos_internal(),
        Some(UploadLine::Bldsa) => line::bldsa(),
        None => Probe::probe(&client.client).await.unwrap_or_default(),
    };
    // let line = line::kodo();
    for video_path in video_path {
        println!("{:?}", video_path.canonicalize()?.to_str());
        info!("{line:?}");
        let video_file = VideoFile::new(&video_path)?;
        let total_size = video_file.total_size;
        let file_name = video_file.file_name.clone();
        let uploader = line.pre_upload(&bilibili, video_file).await?;

        let instant = Instant::now();

        let video = uploader
            .upload(client.clone(), limit, |vs| {
                vs.map(|vs| {
                    let chunk = vs?;
                    let len = chunk.len();
                    Ok((chunk, len))
                })
            })
            .await?;
        let t = instant.elapsed().as_millis();
        info!(
            "Upload completed: {file_name} => cost {:.2}s, {:.2} MB/s.",
            t as f64 / 1000.,
            total_size as f64 / 1000. / t as f64
        );
        videos.push(video);
    }
    let mut desc_v2 = Vec::new();
    for credit in desc_v2_credit {
        desc_v2.push(Credit {
            type_id: credit.type_id,
            raw_text: credit.raw_text,
            biz_id: credit.biz_id,
        });
    }
    let mut studio: Studio = Studio::builder()
        .desc(desc)
        .dtime(dtime)
        .copyright(copyright)
        .cover(cover)
        .dynamic(dynamic)
        .source(source)
        .tag(tag)
        .tid(tid)
        .title(title)
        .videos(videos)
        .dolby(dolby)
        .lossless_music(lossless_music)
        .no_reprint(no_reprint)
        .open_elec(open_elec)
        .desc_v2(Some(desc_v2))
        .build();
    if !studio.cover.is_empty() {
        let url = bilibili
            .cover_up(
                &std::fs::read(&studio.cover)
                    .with_context(|| format!("cover: {}", studio.cover))?,
            )
            .await?;
        println!("{url}");
        studio.cover = url;
    }
    Ok(bilibili.submit(&studio).await?)
    // Ok(videos)
}
