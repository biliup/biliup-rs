use futures::Stream;

use rand::distributions::uniform::{UniformFloat, UniformSampler};
use std::future::Future;
use std::time::Duration;
use tokio::time::sleep;
use tracing::info;

pub mod client;
pub mod downloader;
pub mod error;
#[cfg(feature = "server")]
pub mod server;
pub mod uploader;

pub use uploader::bilibili;
pub use uploader::credential;

pub async fn retry<F, Fut, O, E: std::fmt::Display>(mut f: F) -> Result<O, E>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<O, E>>,
{
    let mut retries = 3;
    let mut wait = 1;
    let mut jittered_wait_for;
    loop {
        match f().await {
            Err(e) if retries > 0 => {
                retries -= 1;
                let jitter_factor =
                    UniformFloat::<f64>::sample_single(0., 1., &mut rand::thread_rng());
                wait *= 2;

                jittered_wait_for = f64::min(jitter_factor + (wait as f64), 64.);
                info!(
                    "Retry attempt #{}. Sleeping {:?} before the next attempt. {e}",
                    3 - retries,
                    jittered_wait_for
                );
                sleep(Duration::from_secs_f64(jittered_wait_for)).await;
            }
            res => break res,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::uploader::bilibili::Vid;
    use std::str::FromStr;
    use tracing::Level;
    use tracing_subscriber::FmtSubscriber;
    use url::Url;

    #[test]
    fn it_works() {
        assert_eq!(Ok(Vid::Aid(971158452)), Vid::from_str("971158452"));
        assert_eq!(Ok(Vid::Aid(971158452)), Vid::from_str("av971158452"));
        assert_eq!(
            Ok(Vid::Bvid("BV1ip4y1x7Gi".into())),
            Vid::from_str("BV1ip4y1x7Gi")
        );
    }

    #[tokio::test]
    async fn try_clone_stream() {
        let subscriber = FmtSubscriber::builder()
            .with_max_level(Level::INFO)
            .finish();
        tracing::subscriber::set_global_default(subscriber)
            .expect("setting default subscriber failed");
        Url::parse("https://bilibili.com/").unwrap();
        let chunks: Vec<Result<_, ::std::io::Error>> = vec![Ok("hello"), Ok(" "), Ok("world")];
        let _stream = futures::stream::iter(chunks);
    }
}
