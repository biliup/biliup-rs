pub mod download_actor;
pub mod live_streamers;
pub mod upload_actor;
pub mod util;

/// Status of the live stream
pub enum LiveStatus {
    /// Stream is online.
    Online,
    /// Stream is offline.
    Offline,
    /// The status of the stream could not be determined.
    Unknown,
}

/// Status of the live stream
#[derive(Clone, Copy, Debug)]
pub enum StreamStatus {
    /// Stream is online.
    Downloading,
    /// Stream is offline.
    Uploading,
    /// The status of the stream could not be determined.
    Pending,
    Idle,
}
