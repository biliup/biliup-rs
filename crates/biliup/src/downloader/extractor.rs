use crate::downloader;
use crate::downloader::httpflv::Connection;
use crate::downloader::util::Segmentable;
use crate::downloader::{hls, httpflv};
use async_trait::async_trait;
use reqwest::header::{HeaderValue, ACCEPT_ENCODING};
use std::fmt::{Display, Formatter};
use std::path::Path;

use crate::client::StatelessClient;

mod bilibili;
mod huya;

const EXTRACTORS: [&dyn SiteDefinition; 2] = [&bilibili::BiliLive {}, &huya::HuyaLive {}];

#[async_trait]
pub trait SiteDefinition {
    // true, if this site can handle <url>.
    fn can_handle_url(&self, url: &str) -> bool;

    async fn get_site(&self, url: &str, client: StatelessClient) -> super::error::Result<Site>;
}

pub struct Site {
    pub name: &'static str,
    pub title: String,
    pub direct_url: String,
    extension: Extension,
    client: StatelessClient,
}

impl Display for Site {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Name: {}", self.name)?;
        writeln!(f, "Title: {}", self.title)?;
        write!(f, "Direct url: {}", self.direct_url)
    }
}

enum Extension {
    Flv,
    Ts,
}

impl Site {
    pub async fn download(
        &mut self,
        file_name: &str,
        segment: Segmentable,
    ) -> downloader::error::Result<()> {
        let file_name = file_name.replace("{title}", &self.title);
        self.client
            .headers
            .append(ACCEPT_ENCODING, HeaderValue::from_static("gzip, deflate"));
        println!("Save to {}", Path::new(&file_name).display());
        println!("{}", self);
        match self.extension {
            Extension::Flv => {
                let response = self.client.retryable(&self.direct_url).await?;
                let mut connection = Connection::new(response);
                connection.read_frame(9).await?;
                httpflv::parse_flv(connection, &file_name, segment).await?
            }
            Extension::Ts => {
                hls::download(&self.direct_url, &self.client, &file_name, segment).await?
            }
        }
        Ok(())
    }
}

pub fn find_extractor(url: &str) -> Option<&dyn SiteDefinition> {
    EXTRACTORS
        .into_iter()
        .find(|&extractor| extractor.can_handle_url(url))
}
