use crate::downloader::httpflv::Connection;
use flv_parser::header;
use nom::Err;
use reqwest::header::{
    HeaderMap, HeaderName, HeaderValue, ACCEPT, ACCEPT_ENCODING, ACCEPT_LANGUAGE, USER_AGENT,
};
use std::collections::HashMap;

use crate::downloader::util::Segmentable;
use crate::uploader::retryable::retry;
use reqwest::Response;
use std::io::Read;
use std::str::FromStr;
use std::thread::sleep;
use std::time::Duration;
use tokio::io::AsyncReadExt;
use util::Segment;

pub mod error;
pub mod extractor;
pub mod flv_parser;
pub mod flv_writer;
mod hls;
pub mod httpflv;
pub mod util;

#[tokio::main]
pub async fn download(
    url: &str,
    headers: HeaderMap,
    file_name: &str,
    segment: Segmentable,
) -> anyhow::Result<()> {
    let mut response = get_response(url, &headers).await?;
    let mut connection = Connection::new(response);
    // let buf = &mut [0u8; 9];
    let bytes = connection.read_frame(9).await?;
    // response.read_exact(buf)?;
    // let out = File::create(format!("{}.flv", file_name)).expect("Unable to create file.");
    // let mut writer = BufWriter::new(out);
    // let mut buf = [0u8; 8 * 1024];
    // response.copy_to(&mut writer)?;
    // io::copy(&mut resp, &mut out).expect("Unable to copy the content.");
    match header(&bytes) {
        Ok((_i, header)) => {
            println!("header: {header:#?}");
            println!("Downloading {}...", url);
            httpflv::download(connection, file_name, segment).await;
        }
        Err(Err::Incomplete(needed)) => {
            println!("needed: {needed:?}")
        }
        Err(e) => {
            println!("{e}");
            hls::download(url, &headers, file_name, segment).await?;
        }
    }
    Ok(())
}

pub fn construct_headers(hash_map: HashMap<String, String>) -> HeaderMap {
    let mut headers = HeaderMap::new();
    for (key, value) in hash_map.iter() {
        headers.insert(
            HeaderName::from_str(key).unwrap(),
            HeaderValue::from_str(value).unwrap(),
        );
    }
    headers
}

pub async fn get_response(url: &str, headers: &HeaderMap) -> reqwest::Result<Response> {
    let resp = retry(|| {
        reqwest::Client::new()
            .get(url)
            .header(ACCEPT, "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8")
            .header(ACCEPT_ENCODING, "gzip, deflate")
            .header(ACCEPT_LANGUAGE, "zh-CN,zh;q=0.8,en-US;q=0.5,en;q=0.3")
            .header(USER_AGENT, "Mozilla/5.0 (X11; Linux x86_64; rv:38.0) Gecko/20100101 Firefox/38.0 Iceweasel/38.2.1")
            .headers(headers.clone())
            .send()
    }).await?;
    resp.error_for_status_ref()?;
    Ok(resp)
}

// fn retry<O, E: std::fmt::Display>(mut f: impl FnMut() -> Result<O, E>) -> Result<O, E> {
//     let mut retries = 0;
//     let mut wait = 1;
//     loop {
//         match f() {
//             Err(e) if retries < 3 => {
//                 retries += 1;
//                 println!(
//                     "Retry attempt #{}. Sleeping {wait}s before the next attempt. {e}",
//                     retries,
//                 );
//                 sleep(Duration::from_secs(wait));
//                 wait *= 2;
//             }
//             res => break res,
//         }
//     }
// }

#[cfg(test)]
mod tests {
    use crate::downloader::download;
    use crate::downloader::util::{Segment, Segmentable};
    use anyhow::Result;
    use reqwest::header::{HeaderMap, HeaderValue, REFERER};

    #[test]
    #[ignore]
    fn it_works() -> Result<()> {
        tracing_subscriber::fmt::init();

        let mut headers = HeaderMap::new();
        headers.insert(
            REFERER,
            HeaderValue::from_static("https://live.bilibili.com"),
        );
        download(
            "",
            headers,
            "testdouyu%Y-%m-%dT%H_%M_%S",
            // Segment::Size(20 * 1024 * 1024, 0),
            Segmentable::new(std::time::Duration::from_secs(6000), u64::MAX),
        )?;
        Ok(())
    }
}
