mod login;
mod uploader;

use pyo3::prelude::*;

use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;

use crate::uploader::UploadLine;
use biliup::downloader::construct_headers;
use biliup::downloader::util::Segmentable;
use tracing_subscriber::layer::SubscriberExt;

#[derive(FromPyObject)]
pub enum PySegment {
    Time {
        #[pyo3(attribute("time"))]
        time: u64,
    },
    Size {
        #[pyo3(attribute("size"))]
        size: u64,
    },
}

#[pyfunction]
fn download(
    py: Python<'_>,
    url: &str,
    header_map: HashMap<String, String>,
    file_name: &str,
    segment: PySegment,
) -> PyResult<()> {
    py.allow_threads(|| {
        let map = construct_headers(header_map);
        // 输出到控制台中
        let formatting_layer = tracing_subscriber::FmtSubscriber::builder()
            // will be written to stdout.
            // builds the subscriber.
            .finish();
        let file_appender = tracing_appender::rolling::never("", "download.log");
        let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
        let file_layer = tracing_subscriber::fmt::layer()
            .with_ansi(false)
            .with_writer(non_blocking);

        let collector = formatting_layer.with(file_layer);
        let segment = match segment {
            PySegment::Time { time } => Segmentable::new(Some(Duration::from_secs(time)), None),
            PySegment::Size { size } => Segmentable::new(None, Some(size)),
        };
        tracing::subscriber::with_default(collector, || -> PyResult<()> {
            match biliup::downloader::download(url, map, file_name, segment) {
                Ok(res) => Ok(res),
                Err(err) => Err(pyo3::exceptions::PyRuntimeError::new_err(format!(
                    "{}, {}",
                    err.root_cause(),
                    err
                ))),
            }
        })
    })
}
#[pyfunction]
fn login_by_cookies() -> PyResult<bool> {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let result = rt.block_on(async { login::login_by_cookies().await });
    match result {
        Ok(_) => Ok(true),
        Err(err) => Err(pyo3::exceptions::PyRuntimeError::new_err(format!(
            "{}, {}",
            err.root_cause(),
            err
        ))),
    }
}
#[pyfunction]
fn send_sms(country_code: u32, phone: u64) -> PyResult<String> {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let result = rt.block_on(async { login::send_sms(country_code, phone).await });
    match result {
        Ok(res) => Ok(res.to_string()),
        Err(err) => Err(pyo3::exceptions::PyRuntimeError::new_err(format!(
            "{}",
            err
        ))),
    }
}
#[pyfunction]
fn login_by_sms(code: u32, ret: String) -> PyResult<bool> {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let result =
        rt.block_on(async { login::login_by_sms(code, serde_json::from_str(&ret).unwrap()).await });
    match result {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}
#[pyfunction]
fn get_qrcode() -> PyResult<String> {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let result = rt.block_on(async { login::get_qrcode().await });
    match result {
        Ok(res) => Ok(res.to_string()),
        Err(err) => Err(pyo3::exceptions::PyRuntimeError::new_err(format!(
            "{}",
            err
        ))),
    }
}
#[pyfunction]
fn login_by_qrcode(ret: String) -> PyResult<bool> {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let result =
        rt.block_on(async { login::login_by_qrcode(serde_json::from_str(&ret).unwrap()).await });
    match result {
        Ok(_) => Ok(true),
        Err(err) => Err(pyo3::exceptions::PyRuntimeError::new_err(format!(
            "{}",
            err
        ))),
    }
}

#[pyfunction]
fn upload(
    py: Python<'_>,
    video_path: Vec<PathBuf>,
    cookie_file: PathBuf,
    title: String,
    tid: u16,
    tag: String,
    copyright: u8,
    source: String,
    desc: String,
    dynamic: String,
    cover: String,
    dtime: Option<u32>,
    line: Option<UploadLine>,
    limit: usize,
) -> PyResult<()> {
    py.allow_threads(|| {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()?;
        // 输出到控制台中
        let formatting_layer = tracing_subscriber::FmtSubscriber::builder()
            // will be written to stdout.
            // builds the subscriber.
            .finish();
        let file_appender = tracing_appender::rolling::never("", "upload.log");
        let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
        let file_layer = tracing_subscriber::fmt::layer()
            .with_ansi(false)
            .with_writer(non_blocking);

        let collector = formatting_layer.with(file_layer);

        tracing::subscriber::with_default(collector, || -> PyResult<()> {
            match rt.block_on(uploader::upload(
                video_path,
                cookie_file,
                line,
                limit,
                title,
                tid,
                tag,
                copyright,
                source,
                desc,
                dynamic,
                cover,
                dtime,
            )) {
                Ok(_res) => Ok(()),
                // Ok(_) => {  },
                Err(err) => Err(pyo3::exceptions::PyRuntimeError::new_err(format!(
                    "{}, {}",
                    err.root_cause(),
                    err
                ))),
            }
        })
    })
}

/// A Python module implemented in Rust.
#[pymodule]
fn stream_gears(_py: Python, m: &PyModule) -> PyResult<()> {
    // let file_appender = tracing_appender::rolling::daily("", "upload.log");
    // let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
    // tracing_subscriber::fmt()
    //     .with_writer(non_blocking)
    //     .init();
    m.add_function(wrap_pyfunction!(upload, m)?)?;
    m.add_function(wrap_pyfunction!(download, m)?)?;
    m.add_function(wrap_pyfunction!(login_by_cookies, m)?)?;
    m.add_function(wrap_pyfunction!(send_sms, m)?)?;
    m.add_function(wrap_pyfunction!(login_by_qrcode, m)?)?;
    m.add_function(wrap_pyfunction!(get_qrcode, m)?)?;
    // m.add_function(wrap_pyfunction(login_by_sms, m)?)?;
    m.add_class::<UploadLine>()?;
    Ok(())
}
