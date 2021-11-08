use hmac::{
    crypto_mac::{InvalidKeyLength, Output as HmacOutput},
    Hmac, Mac, NewMac,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use time::OffsetDateTime;

mod region;
pub use region::Region;
pub mod ser_json;

mod hash;
pub use hash::*;

// HMAC-SHA256 算法
pub type HmacSha256 = Hmac<Sha256>;
pub type Output = HmacOutput<HmacSha256>;
pub type Result<T> = std::result::Result<T, Error>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("序列化时出错")]
    Ser(#[from] serde_json::Error),
    #[error("计算 HMAC-SHA256 时出错")]
    Hash(#[from] InvalidKeyLength),
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
    pub fn to_hashed(&self) -> Result<String> { Ok(hash256(&serde_json::to_vec(self)?)) }

    pub fn to_json_string(&self) -> Result<String> {
        serde_json::to_string(self).map_err(|e| e.into())
    }

    // 由于 [`reqwest::RequestBuilder::json`] 调用了 [`serde_json::to_vec`]，
    // 直接调用 [`reqwest::RequestBuilder::json`] 会触发
    // `{"Error":{"Code":"AuthFailure.SignatureFailure","Message":"The provided credentials
    // could not be validated. Please check your signature is correct."}"}}`
    // 。
    pub fn to_hashed2(&self) -> Result<String> { Ok(hash256(&ser_json::to_vec(self)?)) }

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
    #[serde(default)]
    pub projectid: u8,
    /// 每秒并发请求
    #[serde(default)]
    #[serde(skip_deserializing)]
    pub qps:       u8,
    #[serde(default)]
    #[serde(skip_deserializing)]
    pub action:    String,
    #[serde(default)]
    #[serde(skip_deserializing)]
    pub service:   String,
    #[serde(default)]
    #[serde(skip_deserializing)]
    pub version:   String,
}

impl Default for User {
    fn default() -> Self {
        Self { id:        String::new(),
               key:       String::new(),
               region:    Region::default(),
               projectid: 0,
               qps:       5,
               action:    "TextTranslateBatch".into(),
               service:   "tmt".into(),
               version:   "2018-03-21".into(), }
    }
}

/// 生成请求结构
pub struct Header<'u, 'q> {
    pub datetime:         OffsetDateTime,
    pub timestamp:        String,
    pub credential_scope: String,
    pub authorization:    String,
    pub user:             &'u User,
    pub query:            &'q Query<'q>,
}

impl<'u, 'q> Header<'u, 'q> {
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
    const SIGNEDHEADERS: &'static str = "content-type;host";
    pub const URL: &'static str = "https://tmt.tencentcloudapi.com";

    #[rustfmt::skip]
    pub fn new(user: &'u User, query: &'q Query) -> Self {
        let datetime = time::OffsetDateTime::now_utc();
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
        self.credential_scope = format!("{}/{}/{}", date, self.user.service, Self::CREDENTIALSCOPE);
        let stringtosign = format!("{}\n{}\n{}\n{}",
                                   Self::ALGORITHM,
                                   self.timestamp,
                                   self.credential_scope,
                                   hash256(canonical_request.as_bytes()));
        let secret_date =
            hash_2u8(format!("TC3{}", self.user.key).as_bytes(), format!("{}", date).as_bytes())?;
        let secret_service = hash_hash_u8(secret_date, self.user.service.as_bytes())?;
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
        let mut map = HashMap::new();
        map.insert("authorization", self.authorization.as_str()).unwrap_or_default();
        map.insert("content-type", Self::CONTENTTYPE).unwrap_or_default();
        map.insert("host", Self::HOST).unwrap_or_default();
        map.insert("x-tc-action", &self.user.action).unwrap_or_default();
        map.insert("x-tc-version", &self.user.version).unwrap_or_default();
        map.insert("x-tc-region", self.user.region.as_str()).unwrap_or_default();
        map.insert("x-tc-timestamp", &self.timestamp).unwrap_or_default();
        map
    }
}

#[derive(Debug, Deserialize)]
pub struct Response<'r> {
    #[serde(borrow)]
    #[serde(rename = "Response")]
    pub res: ResponseInner<'r>,
}

/// 响应的信息。要么返回翻译结果，要么返回错误信息。
#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum ResponseInner<'r> {
    #[serde(borrow)]
    Ok(Success<'r>),
    #[serde(borrow)]
    Err(ResponseErr<'r>),
}

/// 返回的数据
#[derive(Debug, Deserialize)]
pub struct Success<'r> {
    #[serde(rename = "RequestId")]
    pub id:   &'r str,
    #[serde(rename = "Source")]
    pub from: &'r str,
    #[serde(rename = "Target")]
    pub to:   &'r str,
    #[serde(borrow)]
    #[serde(rename = "TargetTextList")]
    pub res:  Vec<std::borrow::Cow<'r, str>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ResponseErr<'r> {
    #[serde(rename = "RequestId")]
    pub id:    &'r str,
    #[serde(rename = "Error")]
    pub error: ResponseError,
}

/// 错误处理 / 错误码
///
/// see:
/// - https://cloud.tencent.com/document/product/551/30637
/// - https://cloud.tencent.com/api/error-center?group=PLATFORM&page=1
/// - https://cloud.tencent.com/document/product/551/40566
#[derive(Debug, Clone, Deserialize)]
pub struct ResponseError {
    #[serde(rename = "Code")]
    pub code: String,
    #[serde(rename = "Message")]
    pub msg:  String,
}

#[test]
fn response_test() {
    let success = r#"{"Response":{"RequestId":"7895050c-b0bd-45f2-ba88-c95c509020f2","Source":"en","Target":"zh","TargetTextList":["嗨","那里"]}}"#;
    assert_eq!(format!("{:?}", serde_json::from_str::<Response>(success).unwrap()),
               "Response { res: Ok(Success { id: \"7895050c-b0bd-45f2-ba88-c95c509020f2\", from: \
                \"en\", to: \"zh\", res: [\"嗨\", \"那里\"] }) }");
    let error = r#"{"Response":{"Error":{"Code":"AuthFailure.SignatureFailure","Message":"The provided credentials could not be validated. Please check your signature is correct."},"RequestId":"47546ee3-767c-4671-8f90-2c02c7484a42"}}"#;
    #[rustfmt::skip]
    assert_eq!(
               format!("{:?}", serde_json::from_str::<Response>(error).unwrap()),
               "Response { res: Err(ResponseErr { id: \"47546ee3-767c-4671-8f90-2c02c7484a42\", \
                error: ResponseError { code: \"AuthFailure.SignatureFailure\", \
                msg: \"The provided credentials could not be validated. \
                Please check your signature is correct.\" } }) }"
    );
}
