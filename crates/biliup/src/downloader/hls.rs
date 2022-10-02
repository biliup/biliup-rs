use crate::downloader::error::Result;
use crate::downloader::util::{format_filename, Segmentable};
use m3u8_rs::Playlist;

use std::fs::File;
use std::io::{BufWriter, Write};
use std::time::Duration;
use tracing::{debug, error, info, warn};
use url::Url;

use crate::client::StatelessClient;

pub async fn download(
    url: &str,
    client: &StatelessClient,
    file_name: &str,
    mut splitting: Segmentable,
) -> Result<()> {
    println!("Downloading {}...", url);
    let resp = client.retryable(url).await?;
    println!("{}", resp.status());
    // let mut resp = resp.bytes_stream();
    let bytes = resp.bytes().await?;
    let mut ts_file = TsFile::new(file_name);

    let mut media_url = Url::parse(url)?;
    let mut pl = match m3u8_rs::parse_playlist(&bytes) {
        Ok((_i, Playlist::MasterPlaylist(pl))) => {
            println!("Master playlist:\n{:#?}", pl);
            media_url = media_url.join(&pl.variants[0].uri)?;
            println!("media url: {media_url}");
            let resp = client.retryable(media_url.as_str()).await?;
            let bs = resp.bytes().await?;
            // println!("{:?}", bs);
            if let Ok((_, pl)) = m3u8_rs::parse_media_playlist(&bs) {
                pl
            } else {
                let mut file = File::create("test.fmp4")?;
                file.write_all(&bs)?;
                panic!("Unable to parse the content.")
            }
        }
        Ok((_i, Playlist::MediaPlaylist(pl))) => {
            println!("Media playlist:\n{:#?}", pl);
            println!("index {}", pl.media_sequence);
            pl
        }
        Err(e) => panic!("Parsing error: \n{}", e),
    };
    let mut previous_last_segment = 0;
    loop {
        if pl.segments.is_empty() {
            println!("Segments array is empty - stream finished");
            break;
        }
        let mut seq = pl.media_sequence;
        for segment in &pl.segments {
            if seq > previous_last_segment {
                if (previous_last_segment > 0) && (seq > (previous_last_segment + 1)) {
                    warn!("SEGMENT INFO SKIPPED");
                }
                debug!("Yield segment");
                if segment.discontinuity {
                    warn!("#EXT-X-DISCONTINUITY");
                    ts_file = TsFile::new(file_name);
                    // splitting = Segment::from_seg(splitting);
                    splitting.reset();
                }
                let length = download_to_file(
                    media_url.join(&segment.uri)?,
                    client,
                    &mut ts_file.buf_writer,
                )
                .await?;
                splitting.increase_size(length);
                splitting.increase_time(Duration::from_secs(segment.duration as u64));
                if splitting.needed() {
                    ts_file = TsFile::new(file_name);
                    info!("{} splitting.{splitting:?}", ts_file.name);
                    splitting.reset();
                }
                previous_last_segment = seq;
            }
            seq += 1;
        }
        let resp = client.retryable(media_url.as_str()).await?;
        let bs = resp.bytes().await?;
        if let Ok((_, playlist)) = m3u8_rs::parse_media_playlist(&bs) {
            pl = playlist;
        }
    }
    println!("Done...");
    Ok(())
}

async fn download_to_file(url: Url, client: &StatelessClient, out: &mut impl Write) -> Result<u64> {
    debug!("url: {url}");
    let mut response = client.retryable(url.as_str()).await?;
    let mut length: u64 = 0;
    while let Some(chunk) = response.chunk().await? {
        length += chunk.len() as u64;
        out.write_all(&chunk)?;
    }
    // let mut out = File::options()
    //     .append(true)
    //     .open(format!("{file_name}.ts"))?;
    // let length = response.copy_to(out)?;
    Ok(length)
}

pub struct TsFile {
    pub buf_writer: BufWriter<File>,
    pub name: String,
}

impl TsFile {
    pub fn new(file_name: &str) -> Self {
        let file_name = format_filename(file_name);
        let out = File::create(format!("{file_name}.ts.part")).expect("Unable to create ts file.");
        let buf_writer = BufWriter::new(out);
        Self {
            buf_writer,
            name: file_name,
        }
    }
}

impl Drop for TsFile {
    fn drop(&mut self) {
        std::fs::rename(
            format!("{}.ts.part", self.name),
            format!("{}.ts", self.name),
        )
        .unwrap_or_else(|e| error!("{e}"))
    }
}

#[cfg(test)]
mod tests {

    use anyhow::Result;
    use reqwest::Url;

    #[test]
    fn test_url() -> Result<()> {
        let url = Url::parse("h://host.path/to/remote/resource.m3u8")?;
        let scheme = url.scheme();
        let new_url = url.join("http://path.host/remote/resource.ts")?;
        println!("{url}, {scheme}");
        println!("{new_url}, {scheme}");
        Ok(())
    }

    #[test]
    fn it_works() -> Result<()> {
        // download(
        //     "test.ts")?;
        Ok(())
    }
}
