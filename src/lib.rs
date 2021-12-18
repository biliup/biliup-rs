use async_std::fs::File;
use time::Instant;
use anyhow::Result;
use crate::uploader::upos::Bucket;
use crate::video::Video;

pub mod client;
pub mod error;
pub mod video;
pub mod uploader;


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
