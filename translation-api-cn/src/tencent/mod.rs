use crate::Limit;
use hmac::{
    digest::{CtOutput as HmacOutput, InvalidLength},
    Hmac, Mac,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use time::OffsetDateTime;

mod region;
pub use region::Region;

// 预计会被删除的模块
#[cfg(test)]
pub mod ser_json;

mod hash;
pub use hash::*;

mod response;
pub use response::{Response, ResponseError, ResponseInner};

pub const URL: &str = "https://tmt.tencentcloudapi.com";

/// HMAC-SHA256 算法
pub type HmacSha256 = Hmac<Sha256>;
pub type Output = HmacOutput<HmacSha256>;
pub type Result<T> = std::result::Result<T, Error>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("序列化时出错")]
    Ser(#[from] serde_json::Error),
    #[error("计算 HMAC-SHA256 时出错")]
    Hash(#[from] InvalidLength),
    #[error("计算 unix timestamp 时出错")]
    UnixTimeStamp(#[from] time::error::ComponentRange),
}

/// 翻译前的必要信息
///
/// https://cloud.tencent.com/document/product/551/40566
#[derive(Debug, Serialize)]
pub struct Query<'q> {
    /// 翻译源语言，可设置为 auto
    ///
    /// TODO：变成 enum 类型
    #[serde(rename = "Source")]
    pub from:      &'q str,
    /// 翻译目标语言，不可设置为 auto
    ///
    /// TODO：和 `from` 共用 enum 类型，但是并非任意两个语言之间可以互译。
    /// 比如 `ar（阿拉伯语）：en（英语）` 表明阿拉伯语只能从英语中翻译过去。
    /// 请求翻译 query，必须为 UTF-8 编码。
    ///
    /// TODO: 在传入之前应该把文字控制在 2000 以内,
    /// 一个汉字、一个字母、一个标点都计为一个字符，
    /// 超过 2000 字节要分段请求。
    #[serde(rename = "Target")]
    pub to:        &'q str,
    #[serde(rename = "ProjectId")]
    pub projectid: u8,
    #[serde(rename = "SourceTextList")]
    pub q:         &'q [&'q str],
}

impl<'q> Query<'q> {
    #[rustfmt::skip]
    pub fn new(q: &'q [&'q str], from: &'q str, to: &'q str, projectid: u8) -> Self {
        Self { q, from, to, projectid }
    }

    pub fn to_hashed(&self) -> Result<String> { Ok(hash256(&serde_json::to_vec(self)?)) }

    pub fn to_json_string(&self) -> Result<String> {
        serde_json::to_string(self).map_err(|e| e.into())
    }

    // 由于 [`reqwest::RequestBuilder::json`] 调用了 [`serde_json::to_vec`]，
    // 直接调用 [`reqwest::RequestBuilder::json`] 会触发
    // `{"Error":{"Code":"AuthFailure.SignatureFailure","Message":"The provided credentials
    // could not be validated. Please check your signature is correct."}"}}`
    // 。
    #[cfg(test)]
    pub fn to_hashed2(&self) -> Result<String> { Ok(hash256(&ser_json::to_vec(self)?)) }

    #[cfg(test)]
    pub fn to_json_string2(&self) -> Result<String> {
        ser_json::to_string(self).map_err(|e| e.into())
    }
}

/// 账户信息以及一些不变的信息
/// 需要：机器翻译（TMT）全读写访问权限
#[derive(Debug, Deserialize)]
#[serde(rename = "tencent")] // for config or cmd
pub struct User {
    /// SecretId
    pub id:        String,
    /// SecretKey
    pub key:       String,
    /// 地域列表，默认为北京。
    #[serde(default)]
    pub region:    Region,
    /// 项目ID，可以根据控制台-账号中心-项目管理中的配置填写，如无配置请填写默认项目ID:0
    #[serde(default = "default_projectid")]
    pub projectid: u8,
    /// 每秒并发请求，默认为 5。
    #[serde(default = "default_qps")]
    // #[serde(skip_deserializing)]
    pub qps: u8,
    /// 每秒并发请求的限制，默认为 Char(2000)。
    #[serde(default = "default_limit")]
    // #[serde(skip_deserializing)]
    pub limit: Limit,
}

fn default_qps() -> u8 { 5 }
fn default_limit() -> Limit { Limit::Char(2000) }
fn default_projectid() -> u8 { 0 }

impl Default for User {
    fn default() -> Self {
        Self { id:        String::new(),
               key:       String::new(),
               region:    Region::default(),
               projectid: 0,
               qps:       5,
               limit:     default_limit(), }
    }
}

/// 生成请求结构
#[derive(Debug)]
pub struct Header<'u, 'q> {
    pub datetime:         OffsetDateTime,
    pub timestamp:        String,
    pub credential_scope: String,
    pub authorization:    String,
    pub user:             &'u User,
    pub query:            &'q Query<'q>,
}

impl<'u, 'q> Header<'u, 'q> {
    const ACTION: &'static str = "TextTranslateBatch";
    const ALGORITHM: &'static str = "TC3-HMAC-SHA256";
    const CANONICALHEADERS: &'static str =
        "content-type:application/json\nhost:tmt.tencentcloudapi.com\n";
    // const CANONICALHEADERS: &'static str =
    //     "content-type:application/json; charset=utf-8\nhost:cvm.tencentcloudapi.com\n";
    const CANONICALQUERYSTRING: &'static str = "";
    const CANONICALURI: &'static str = "/";
    const CONTENTTYPE: &'static str = "application/json";
    // const CONTENTTYPE: &'static str = "application/json; charset=utf-8";
    const CREDENTIALSCOPE: &'static str = "tc3_request";
    const HOST: &'static str = "tmt.tencentcloudapi.com";
    const HTTPREQUESTMETHOD: &'static str = "POST";
    const SERVICE: &'static str = "tmt";
    const SIGNEDHEADERS: &'static str = "content-type;host";
    const VERSION: &'static str = "2018-03-21";

    #[rustfmt::skip]
    pub fn new(user: &'u User, query: &'q Query) -> Self {
        let datetime = OffsetDateTime::now_utc();
        let timestamp = datetime.unix_timestamp().to_string();
        Self { datetime, timestamp, credential_scope: String::new(),
               authorization: String::new(), user, query }
    }

    pub fn signature(&mut self) -> Result<String> {
        let canonical_request = format!("{}\n{}\n{}\n{}\n{}\n{}",
                                        Self::HTTPREQUESTMETHOD,
                                        Self::CANONICALURI,
                                        Self::CANONICALQUERYSTRING,
                                        Self::CANONICALHEADERS,
                                        Self::SIGNEDHEADERS,
                                        self.query.to_hashed()?);

        let date = self.datetime.date();
        self.credential_scope = format!("{}/{}/{}", date, Self::SERVICE, Self::CREDENTIALSCOPE);
        let stringtosign = format!("{}\n{}\n{}\n{}",
                                   Self::ALGORITHM,
                                   self.timestamp,
                                   self.credential_scope,
                                   hash256(canonical_request.as_bytes()));
        let secret_date =
            hash_2u8(format!("TC3{}", self.user.key).as_bytes(), format!("{date}").as_bytes())?;
        let secret_service = hash_hash_u8(secret_date, Self::SERVICE.as_bytes())?;
        let secret_signing = hash_hash_u8(secret_service, Self::CREDENTIALSCOPE.as_bytes())?;
        Ok(hmac_sha256_string(hash_hash_u8(secret_signing, stringtosign.as_bytes())?))
    }

    pub fn authorization(&mut self) -> Result<&str> {
        let signature = self.signature()?;
        self.authorization = format!("{} Credential={}/{}, SignedHeaders={}, Signature={}",
                                     Self::ALGORITHM,
                                     self.user.id,
                                     self.credential_scope,
                                     Self::SIGNEDHEADERS,
                                     signature);
        Ok(&self.authorization)
    }

    pub fn header(&self) -> HashMap<&str, &str> {
        let mut map = HashMap::with_capacity(8);
        map.insert("authorization", self.authorization.as_str()).unwrap_or_default();
        map.insert("content-type", Self::CONTENTTYPE).unwrap_or_default();
        map.insert("host", Self::HOST).unwrap_or_default();
        map.insert("x-tc-action", Self::ACTION).unwrap_or_default();
        map.insert("x-tc-version", Self::VERSION).unwrap_or_default();
        map.insert("x-tc-region", self.user.region.as_str()).unwrap_or_default();
        map.insert("x-tc-timestamp", &self.timestamp).unwrap_or_default();
        map
    }
}
