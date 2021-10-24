use dialoguer::Input;
use crate::client::{Client, LoginInfo};
use anyhow::Result;
use dialoguer::theme::ColorfulTheme;

pub mod client;
pub mod error;
pub mod upos;
pub mod video;

pub async fn login_by_password(client: Client) -> Result<LoginInfo> {
    let username : String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("请输入账号")
        .interact_text()?;
    let password : String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("请输入密码")
        .interact_text()?;
    let res = client.login_by_password(&username, &password).await?;
    Ok(res)
}

pub async fn login_by_sms(client: Client) -> Result<LoginInfo> {
    let country_code : u32 = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("请输入手机国家代码")
        .default(86)
        .interact_text()?;
    let phone : u64 = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("请输入手机号")
        .interact_text()?;
    let res = client.send_sms(phone, country_code).await?;
    let input : u32 = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("请输入验证码")
        .interact_text()?;
    // println!("{}", payload);
    let ret = client.login_by_sms(input, res).await?;
    Ok(ret)
}


#[cfg(test)]
mod tests {
    use crate::client::Client;
    use crate::video::{BiliBili, Studio, Video};
    use anyhow::{Result};
    #[tokio::test]
    async fn it_works() -> Result<()> {

        println!("yes");
        Ok(())
    }
}
