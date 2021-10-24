use thiserror::Error;

#[derive(Error, Debug)]
pub enum CustomError {
    #[error("{0}")]
    Custom(String),

    #[error(transparent)]
    IO(#[from] reqwest::Error),

    #[error(transparent)]
    Other(#[from] anyhow::Error), // source and Display delegate to anyhow::Error
}
