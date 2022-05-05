use rand::distributions::uniform::{UniformFloat, UniformSampler};

use std::future::Future;
use std::time::Duration;
use tokio::time::sleep;

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
                println!(
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
