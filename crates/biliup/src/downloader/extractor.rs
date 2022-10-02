use crate::downloader;
use crate::downloader::httpflv::Connection;
use crate::downloader::util::Segmentable;
use crate::downloader::{hls, httpflv};
use async_trait::async_trait;
use reqwest::header::{ACCEPT_ENCODING, HeaderMap, HeaderValue};
use std::fmt::{Display, Formatter};
use std::path::Path;
use reqwest::{Request, RequestBuilder};
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
        // client: &StatelessClient
    ) -> downloader::error::Result<()> {
        let file_name = file_name.replace("{title}", &self.title);
        // let mut header_map = HeaderMap::new();
        // file_name.canonicalize()
        self.client.headers.append(ACCEPT_ENCODING, HeaderValue::from_static("gzip, deflate"));
        // self.header_map
        // for (k, v) in self.header_map.into_iter() {
        //     header_map.append(k, v);
        //     // header_map.insert()
        // }
        // header_map.extend(self.header_map.clone());
        // .header(ACCEPT, "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8")
        //     .header(ACCEPT_ENCODING, "gzip, deflate")
            // .header(ACCEPT_LANGUAGE, "zh-CN,zh;q=0.8,en-US;q=0.5,en;q=0.3")
            // .headers(headers.clone())
        println!("Save to {}", Path::new(&file_name).display());
        // let client = StatelessClient::new(header_map);
        // let req = self.req_builder.cl.build()?;
        println!("{}", self);
        match self.extension {
            Extension::Flv => {
                let response = self.client.retryable(&self.direct_url).await?;
                // response.bytes_stream()
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
    for extractor in EXTRACTORS {
        if extractor.can_handle_url(url) {
            return Some(extractor);
        }
    }
    None
}
