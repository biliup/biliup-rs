
use anyhow::{Result};
use biliup::client::Client;
use biliup::{client,};

pub async fn login_by_cookies()->Result<client::LoginInfo>{
    let login_info= Client::new().login_by_cookies(std::fs::File::open("cookies.json")?).await?;
    Ok(login_info)
}
pub async fn send_sms(country_code: u32, phone: u64) -> Result<serde_json::Value> {
    let ret = Client::new().send_sms(phone, country_code).await?;
    Ok(ret)
}
pub async fn login_by_sms(code: u32, res: serde_json::Value) -> Result<bool> {
    let info = Client::new().login_by_sms(code, res).await?;
    let file = std::fs::File::create("cookies.json")?;
    serde_json::to_writer_pretty(&file, &info)?;
    Ok(true)
}
pub async fn get_qrcode() -> Result<serde_json::Value> {
    let qrcode = Client::new().get_qrcode().await?;
    Ok(qrcode)
}
pub async fn login_by_qrcode(res: serde_json::Value) -> Result<bool> {
    let info = Client::new().login_by_qrcode(res).await?;
    let file = std::fs::File::create("cookies.json")?;
    serde_json::to_writer_pretty(&file, &info)?;
    Ok(true)
}





