use crate::error::{CustomError, Result};
use base64::encode;
use cookie::Cookie;
use md5::{Digest, Md5};
use rand::rngs::OsRng;
use reqwest::header;
use reqwest_cookie_store::CookieStoreMutex;
use rsa::{pkcs8::FromPublicKey, PaddingScheme, PublicKey, RsaPublicKey};
use serde::ser::Error;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use std::fmt::{Display, Formatter};

use reqwest::header::USER_AGENT;
use std::io::Seek;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tracing::info;
use url::Url;

// const APP_KEY: &str = "ae57252b0c09105d";
// const APPSEC: &str = "c75875c596a69eb55bd119e74b07cfe3";
// const APP_KEY: &str = "783bbb7264451d82";
// const APPSEC: &str = "2653583c8873dea268ab9386918b1d65";
// const APP_KEY: &str = "4409e2ce8ffd12b8";
// const APPSEC: &str = "59b43e04ad6965f34319062b478f83dd";
// const APP_KEY: &str = "37207f2beaebf8d7";
// const APPSEC: &str = "e988e794d4d4b6dd43bc0e89d6e90c43";
// const APP_KEY: &str = "bca7e84c2d947ac6";
// const APPSEC: &str = "60698ba2f68e01ce44738920a0ffe768";
// const APP_KEY: &str = "bb3101000e232e27";
// const APPSEC: &str = "36efcfed79309338ced0380abd824ac1";
pub(crate) enum AppKeyStore {
    BiliTV,
    Android,
}

impl AppKeyStore {
    fn app_key(&self) -> &'static str {
        match self {
            AppKeyStore::BiliTV => "4409e2ce8ffd12b8",
            AppKeyStore::Android => "783bbb7264451d82",
        }
    }

    fn appsec(&self) -> &'static str {
        match self {
            AppKeyStore::BiliTV => "59b43e04ad6965f34319062b478f83dd",
            AppKeyStore::Android => "2653583c8873dea268ab9386918b1d65",
        }
    }
}

#[derive(Debug)]
pub struct Client {
    pub client: reqwest::Client,
    cookie_store: Arc<CookieStoreMutex>,
}

impl Client {
    pub fn new() -> Self {
        let mut headers = header::HeaderMap::new();
        headers.insert(
            "Referer",
            header::HeaderValue::from_static("https://www.bilibili.com/"),
        );
        headers.insert("Connection", header::HeaderValue::from_static("keep-alive"));
        let cookie_store = cookie_store::CookieStore::default();
        let cookie_store = reqwest_cookie_store::CookieStoreMutex::new(cookie_store);
        let cookie_store = std::sync::Arc::new(cookie_store);
        Client {
            client: reqwest::Client::builder()
                .cookie_provider(std::sync::Arc::clone(&cookie_store))
                .user_agent(
                    "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 Chrome/63.0.3239.108",
                )
                .default_headers(headers)
                .timeout(Duration::new(60, 0))
                .build()
                .unwrap(),
            cookie_store,
        }
    }

    pub async fn login_by_cookies(&self, mut file: std::fs::File) -> Result<LoginInfo> {
        let login_info: LoginInfo = serde_json::from_reader(std::io::BufReader::new(&file))?;
        self.set_cookie(&login_info.cookie_info);
        info!("通过cookie登录");
        let response = self.validate_tokens(&login_info).await?;
        // if response.code != 0 {
        //     return Err(CustomError::Custom(response.to_string()));
        // }
        match response {
            ResponseData {
                data: ResponseValue::OAuth(OAuthInfo { refresh: true, .. }),
                ..
            } => {
                let new_info = self.renew_tokens(login_info).await?;
                file.rewind()?;
                file.set_len(0)?;
                serde_json::to_writer_pretty(std::io::BufWriter::new(&file), &new_info)?;
                Ok(new_info)
            }
            ResponseData {
                data: ResponseValue::OAuth(OAuthInfo { refresh: false, .. }),
                ..
            } => {
                info!("无需更新cookie");
                Ok(login_info)
            }
            _ => Err(CustomError::Custom(response.to_string())),
        }
    }

    async fn validate_tokens(&self, login_info: &LoginInfo) -> Result<ResponseData> {
        let payload = {
            let mut payload = json!({
                "access_key": login_info.token_info.access_token,
                "actionKey": "appkey",
                "appkey": AppKeyStore::Android.app_key(),
                "ts": SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            });

            let urlencoded = serde_urlencoded::to_string(&payload)?;
            let sign = Client::sign(&urlencoded, AppKeyStore::Android.appsec());
            payload["sign"] = Value::from(sign);
            payload
        };

        let response: ResponseData = self
            .client
            .get("https://passport.bilibili.com/x/passport-login/oauth2/info")
            .query(&payload)
            .send()
            .await?
            .json()
            .await?;
        info!("验证cookie");
        Ok(response)
    }

    pub async fn renew_tokens(&self, login_info: LoginInfo) -> Result<LoginInfo> {
        let keypair = match login_info.platform.as_deref() {
            Some("BiliTV") => AppKeyStore::BiliTV,
            Some("Android") => AppKeyStore::Android,
            Some(_) => return Err("未知平台".into()),
            None => return Ok(login_info),
        };
        let payload = {
            let mut payload = json!({
                "access_key": login_info.token_info.access_token,
                "actionKey": "appkey",
                "appkey": keypair.app_key(),
                "refresh_token": login_info.token_info.refresh_token,
                "ts": SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            });

            let urlencoded = serde_urlencoded::to_string(&payload)?;
            let sign = Client::sign(&urlencoded, keypair.appsec());
            payload["sign"] = Value::from(sign);
            payload
        };
        let response: ResponseData = self
            .client
            .post("https://passport.bilibili.com/x/passport-login/oauth2/refresh_token")
            .form(&payload)
            .send()
            .await?
            .json()
            .await?;
        info!("更新cookie");
        match response.data {
            ResponseValue::Login(info) if !info.cookie_info.is_null() => {
                self.set_cookie(&info.cookie_info);
                Ok(LoginInfo {
                    platform: login_info.platform,
                    ..info
                })
            }
            _ => Err(CustomError::Custom(response.to_string())),
        }
    }

    pub async fn login_by_password(&self, username: &str, password: &str) -> Result<LoginInfo> {
        // The type of `payload` is `serde_json::Value`
        let (key_hash, pub_key) = self.get_key().await?;
        let pub_key = RsaPublicKey::from_public_key_pem(&pub_key).unwrap();
        let padding = PaddingScheme::new_pkcs1v15_encrypt();
        let enc_data = pub_key
            .encrypt(
                &mut OsRng,
                padding,
                format!("{}{}", key_hash, password).as_bytes(),
            )
            .expect("failed to encrypt");
        let encrypt_password = encode(enc_data);
        let mut payload = json!({
            "actionKey": "appkey",
            "appkey": AppKeyStore::Android.app_key(),
            "build": 6270200,
            "captcha": "",
            "challenge": "",
            "channel": "bili",
            "device": "phone",
            "mobi_app": "android",
            "password": encrypt_password,
            "permission": "ALL",
            "platform": "android",
            "seccode": "",
            "subid": 1,
            "ts": SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            "username": username,
            "validate": "",
        });
        let urlencoded = serde_urlencoded::to_string(&payload)?;
        let sign = Client::sign(&urlencoded, AppKeyStore::Android.appsec());
        payload["sign"] = Value::from(sign);
        let response: ResponseData = self
            .client
            .post("https://passport.bilibili.com/x/passport-login/oauth2/login")
            .form(&payload)
            .send()
            .await?
            .json()
            .await?;
        info!("通过密码登录");
        match response.data {
            ResponseValue::Login(info) if !info.cookie_info.is_null() => {
                self.set_cookie(&info.cookie_info);
                Ok(LoginInfo {
                    platform: Some("Android".to_string()),
                    ..info
                })
            }
            _ => Err(CustomError::Custom(response.to_string())),
        }
    }

    pub async fn login_by_sms(
        &self,
        code: u32,
        mut payload: serde_json::Value,
    ) -> Result<LoginInfo> {
        payload["code"] = Value::from(code);
        let urlencoded = serde_urlencoded::to_string(&payload)?;
        let sign = Client::sign(&urlencoded, AppKeyStore::Android.appsec());
        payload["sign"] = Value::from(sign);
        let res: ResponseData = self
            .client
            .post("https://passport.bilibili.com/x/passport-login/login/sms")
            .form(&payload)
            .send()
            .await?
            .json()
            .await?;
        match res.data {
            ResponseValue::Login(info) => Ok(LoginInfo {
                platform: Some("Android".to_string()),
                ..info
            }),
            _ => Err(CustomError::Custom(res.to_string())),
        }
    }

    pub async fn send_sms(
        &self,
        phone_number: u64,
        country_code: u32,
    ) -> Result<serde_json::Value> {
        let mut payload = json!({
            "actionKey": "appkey",
            "appkey": AppKeyStore::Android.app_key(),
            "build": 6510400,
            "channel": "bili",
            "cid": country_code,
            "device": "phone",
            "mobi_app": "android",
            "platform": "android",
            // "platform": "pc",
            "tel": phone_number,
            "ts": SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
        });

        let urlencoded = serde_urlencoded::to_string(&payload)?;
        let sign = Client::sign(&urlencoded, AppKeyStore::Android.appsec());
        let urlencoded = format!("{}&sign={}", urlencoded, sign);
        // let mut form = payload.clone();
        // form["sign"] = Value::from(sign);
        let res: ResponseData = self
            .client
            .post("https://passport.bilibili.com/x/passport-login/sms/send")
            .header("content-type", "application/x-www-form-urlencoded")
            .body(urlencoded)
            .send()
            .await?
            .json()
            .await?;
        // println!("{}", res);
        match res.data {
            ResponseValue::Value(mut data)
                if !data["captcha_key"]
                    .as_str()
                    .ok_or("send sms error")?
                    .is_empty() =>
            {
                payload["captcha_key"] = data["captcha_key"].take();
                Ok(payload)
            }
            _ => Err(CustomError::Custom(res.to_string())),
        }
    }

    pub async fn login_by_qrcode(&self, value: Value) -> Result<LoginInfo> {
        let mut form = json!({
            "appkey": AppKeyStore::BiliTV.app_key(),
            "local_id": "0",
            "auth_code": value["data"]["auth_code"],
            "ts": SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()
        });
        let urlencoded = serde_urlencoded::to_string(&form)?;
        let sign = Client::sign(&urlencoded, AppKeyStore::BiliTV.appsec());
        form["sign"] = Value::from(sign);
        loop {
            tokio::time::sleep(Duration::from_secs(1)).await;
            let res: ResponseData = self
                .client
                .post("http://passport.bilibili.com/x/passport-tv-login/qrcode/poll")
                .form(&form)
                .send()
                .await?
                .json()
                .await?;
            match res {
                ResponseData {
                    code: 0,
                    data: ResponseValue::Login(info),
                    ..
                } => {
                    break Ok(LoginInfo {
                        platform: Some("BiliTV".to_string()),
                        ..info
                    });
                }
                ResponseData { code: 86039, .. } => {
                    // 二维码尚未确认;
                    // form["ts"] = Value::from(SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs());
                }
                _ => {
                    break Err(CustomError::Custom(format!("{res:#?}")));
                }
            }
        }
    }

    pub async fn get_qrcode(&self) -> Result<Value> {
        let mut form = json!({
            "appkey": AppKeyStore::BiliTV.app_key(),
            "local_id": "0",
            "ts": SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()
        });
        let urlencoded = serde_urlencoded::to_string(&form)?;
        let sign = Client::sign(&urlencoded, AppKeyStore::BiliTV.appsec());
        form["sign"] = Value::from(sign);
        Ok(self
            .client
            .post("http://passport.bilibili.com/x/passport-tv-login/qrcode/auth_code")
            .form(&form)
            .send()
            .await?
            .json()
            .await?)
    }

    pub async fn get_key(&self) -> Result<(String, String)> {
        let payload = json!({
            "appkey": AppKeyStore::Android.app_key(),
            "sign": Client::sign(&format!("appkey={}", AppKeyStore::Android.app_key()), AppKeyStore::Android.appsec()),
        });
        let response: Value = self
            .client
            .get("https://passport.bilibili.com/x/passport-login/web/key")
            .json(&payload)
            .send()
            .await?
            .json()
            .await?;
        let response = response
            .get("data")
            .ok_or_else(|| CustomError::Custom(response.to_string()))?;
        let hash = response
            .get("hash")
            .and_then(Value::as_str)
            .ok_or_else(|| CustomError::Custom(response.to_string()))?;
        let key = response
            .get("key")
            .and_then(Value::as_str)
            .ok_or_else(|| CustomError::Custom(response.to_string()))?;
        Ok((hash.to_string(), key.to_string()))
    }

    pub async fn login_by_web_cookies(&self, sess_data: &str, bili_jct: &str) -> Result<LoginInfo> {
        info!("获取二维码");
        let qrcode = self.get_qrcode().await?;
        let auth_code = qrcode["data"]["auth_code"]
            .as_str()
            .ok_or("Cannot get auth_code")?;
        Self::web_confirm_qrcode(auth_code, sess_data, bili_jct).await?;
        self.login_by_qrcode(qrcode).await
    }

    async fn web_confirm_qrcode(auth_code: &str, sess_data: &str, bili_jct: &str) -> Result<()> {
        let form = json!({
            "auth_code": auth_code,
            "csrf": bili_jct,
            "scanning_type": 3,
        });
        let cookies = format!("SESSDATA={}; bili_jct={}", sess_data, bili_jct);
        info!("自动确认二维码");
        let res: crate::video::Response = reqwest::Client::new()
            .post("https://passport.bilibili.com/x/passport-tv-login/h5/qrcode/confirm")
            .header("Cookie", cookies)
            // .header("native_api_from", "h5")
            .header(USER_AGENT, "Mozilla/5.0 (X11; Linux x86_64; rv:38.0) Gecko/20100101 Firefox/38.0 Iceweasel/38.2.1")
            .form(&form)
            .send()
            .await?
            .json()
            .await?;
        if res.code != 0 {
            return Err(CustomError::Custom(format!("{res:#?}")));
        }
        Ok(())
    }
    pub async fn check_tag(&self,tag: &str) -> Result<bool> {
        if tag.len() == 0{
            return Ok(true)
        }
        let res: crate::video::Response = self.client
            .get("https://member.bilibili.com/x/vupre/web/topic/tag/check")
            .header(USER_AGENT, "Mozilla/5.0 (X11; Linux x86_64; rv:38.0) Gecko/20100101 Firefox/38.0 Iceweasel/38.2.1")
            .query(&[("tag", tag.to_string())])
            .send().await?
            .json().await?;
        if res.code != 0{
            return Ok(false);
        }
        Ok(true)
    }
    pub fn sign(param: &str, app_sec: &str) -> String {
        let mut hasher = Md5::new();
        // process input message
        hasher.update(format!("{}{}", param, app_sec));
        // acquire hash digest in the form of GenericArray,
        // which in this case is equivalent to [u8; 16]
        format!("{:x}", hasher.finalize())
    }

    fn set_cookie(&self, cookie_info: &serde_json::Value) {
        let mut store = self.cookie_store.lock().unwrap();
        for cookie in cookie_info["cookies"].as_array().unwrap() {
            let cookie = Cookie::build(
                cookie["name"].as_str().unwrap(),
                cookie["value"].as_str().unwrap(),
            )
            .domain("bilibili.com")
            .finish();
            store
                .insert_raw(&cookie, &Url::parse("https://bilibili.com/").unwrap())
                .unwrap();
        }
    }
}

impl Default for Client {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ResponseData {
    pub code: i32,
    pub data: ResponseValue,
    message: String,
    ttl: u8,
}

impl Display for ResponseData {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            serde_json::to_string(self).map_err(std::fmt::Error::custom)?
        )
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(untagged)]
pub enum ResponseValue {
    Login(LoginInfo),
    OAuth(OAuthInfo),
    Value(serde_json::Value),
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct LoginInfo {
    pub cookie_info: serde_json::Value,
    // message: String,
    pub sso: Vec<String>,
    // status: u8,
    pub token_info: TokenInfo,
    // url: String,
    pub platform: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct TokenInfo {
    pub access_token: String,
    expires_in: u32,
    mid: u64,
    refresh_token: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct OAuthInfo {
    pub mid: u64,
    pub access_token: String,
    pub expires_in: u32,
    pub refresh: bool,
}
