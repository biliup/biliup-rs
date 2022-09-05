use nom::Needed;
use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    IOError(#[from] io::Error),

    #[error(transparent)]
    ReqwestError(#[from] reqwest::Error),

    #[error(transparent)]
    UrlParseError(#[from] url::ParseError),

    #[error("Parsing {0} requires {1:?} bytes/chars.")]
    NomIncomplete(String, Needed),
}

pub type Result<T> = core::result::Result<T, Error>;
