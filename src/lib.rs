use crate::uploader::upos::Bucket;
use crate::video::Video;
use anyhow::Result;
use async_std::fs::File;
use time::Instant;

pub mod client;
pub mod error;
pub mod uploader;
pub mod video;

#[cfg(test)]
mod tests {
    use crate::client::Client;
    use crate::video::{BiliBili, Studio, Video};
    use anyhow::Result;
    #[tokio::test]
    async fn it_works() -> Result<()> {
        println!("yes");
        Ok(())
    }
}
