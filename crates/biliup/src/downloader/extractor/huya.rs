use crate::downloader::error::Result;
use crate::downloader::extractor::{Extension, Site, SiteDefinition};
use async_trait::async_trait;
use serde_json::Value;
use std::fmt::format;

pub struct HuyaLive {}

#[async_trait]
impl SiteDefinition for HuyaLive {
    fn can_handle_url(&self, url: &str) -> bool {
        regex::Regex::new(r"(?:https?://)?(?:(?:www|m)\.)?huya\.com")
            .unwrap()
            .is_match(url)
    }

    async fn get_site(&self, url: &str) -> Result<Site> {
        let response = reqwest::Client::new().get(url).send().await?;
        // println!("{:?}", response);

        let text = response.text().await?;
        let mut stream: Value = if let Some(captures) = regex::Regex::new(r"stream: (\{.+)\n.*?\};")
            .unwrap()
            .captures(&text)
        {
            serde_json::from_str(&captures[1])?
        } else {
            return Err(crate::downloader::error::Error::Custom(format!(
                "Not online: {text}"
            )));
        };
        let mut game = stream["data"][0].take();
        let mut game_stream_info_list = game["gameStreamInfoList"].take();
        let game_stream_info = game_stream_info_list[0].take();
        let mut v_multi_stream_info = stream["vMultiStreamInfo"].take();
        // vec![1,2].iter().max()
        // println!("{}", v_multi_stream_info);
        let stream_info = v_multi_stream_info
            .as_array()
            .and_then(|v| v.iter().max_by_key(|info| info["iBitRate"].as_i64()));
        // println!("{:?}", stream_info);
        // let ratio = ;
        let direct_url = format!(
            "{}/{}.{}?{}&ratio={}",
            game_stream_info["sFlvUrl"].as_str().unwrap(),
            game_stream_info["sStreamName"].as_str().unwrap(),
            game_stream_info["sFlvUrlSuffix"].as_str().unwrap(),
            game_stream_info["sFlvAntiCode"].as_str().unwrap(),
            v_multi_stream_info[0]["iBitRate"].take()
        );
        // println!("{}", direct_url);
        Ok(Site {
            name: "huya",
            title: game["gameLiveInfo"]["roomName"]
                .as_str()
                .unwrap()
                .to_string(),
            direct_url,
            extension: Extension::Flv,
            header_map: Default::default(),
        })
    }
}
