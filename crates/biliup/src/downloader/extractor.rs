use crate::downloader;
use crate::downloader::httpflv::Connection;
use crate::downloader::util::{Segment, Segmentable};
use crate::downloader::{get_response, hls, httpflv};
use async_trait::async_trait;
use reqwest::header::{HeaderMap, HeaderValue};
use std::fmt::{Display, Formatter};
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

mod bilibili;
mod huya;

const EXTRACTORS: [&dyn SiteDefinition; 2] = [&bilibili::BiliLive {}, &huya::HuyaLive {}];

#[async_trait]
pub trait SiteDefinition {
    // true, if this site can handle <url>.
    fn can_handle_url(&self, url: &str) -> bool;

    async fn get_site(&self, url: &str) -> super::error::Result<Site>;
}

pub struct Site {
    pub name: &'static str,
    pub title: String,
    pub direct_url: String,
    extension: Extension,
    header_map: HeaderMap,
}

impl Display for Site {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Name: {}\n", self.name)?;
        write!(f, "Title: {}\n", self.title)?;
        write!(f, "Direct url: {}", self.direct_url)
    }
}

enum Extension {
    Flv,
    Ts,
}

impl Site {
    pub async fn download(
        &self,
        file_name: &str,
        segment: Segmentable,
    ) -> downloader::error::Result<()> {
        println!("{}", self);
        match self.extension {
            Extension::Flv => {
                let response = get_response(&self.direct_url, &self.header_map).await?;
                // response.bytes_stream()
                let mut connection = Connection::new(response);
                connection.read_frame(9).await?;
                httpflv::parse_flv(connection, file_name, segment).await?
            }
            Extension::Ts => {
                hls::download(&self.direct_url, &self.header_map, file_name, segment).await?
            }
        }
        Ok(())
    }
}

pub fn find_extractor(url: &str) -> Option<&dyn SiteDefinition> {
    for extractor in EXTRACTORS {
        if extractor.can_handle_url(url) {
            return Some(extractor);
        }
    }
    return None;
}
