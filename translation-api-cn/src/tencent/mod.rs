use hmac::{
    crypto_mac::{InvalidKeyLength, Output as HmacOutput},
    Hmac, Mac, NewMac,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::result::Result as StdResult;
use time::OffsetDateTime;

mod region;
pub use region::Region;
pub mod ser_json;

mod hash;
pub use hash::*;

// HMAC-SHA256 算法
pub type HmacSha256 = Hmac<Sha256>;
pub type Output = HmacOutput<HmacSha256>;
pub type Result<T> = StdResult<T, Error>;

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

impl<'r> Response<'r> {
    /// 提取翻译内容。无翻译内容时，返回错误。
    ///
    /// TODO: [`BaiduError`] 会经过两次内存分配，这种设计的原因是
    ///       `anyhow` crate 要求错误的类型必须是 `'static`。
    ///       [`BaiduError`] 一次分配的例子见 `tests/baidu.rs`。
    pub fn dst(&self) -> StdResult<Vec<&str>, ResponseError> {
        match &self.res {
            ResponseInner::Ok(s) => Ok(s.res.iter().map(|x| x.as_ref()).collect()),
            ResponseInner::Err(e) => Err(e.error.clone()),
        }
    }

    /// 提取翻译内容。无翻译内容时，返回错误。
    pub fn dst_owned(self) -> StdResult<Vec<String>, ResponseError> {
        match self.res {
            ResponseInner::Ok(s) => Ok(s.res.into_iter().map(|x| x.into()).collect()),
            ResponseInner::Err(e) => Err(e.error),
        }
    }

    /// 翻译内容（即 [`SrcDst`] 的 `dst`字段）是否为 `Cow::Borrowed` 类型。
    /// 比如英译中时，中文为代码点：
    /// ```text
    /// {
    ///   "from": "en",
    ///   "to":   "zh",
    ///   "trans_result":[
    ///     {"src": "hello", "dst": "\u4f60\u597d"},
    ///     {"src": "world", "dst": "\u4e16\u754c"}
    ///   ]
    /// }
    /// ```
    /// 必须使用 `String` 或者 `Cow::Owned` 类型。
    ///
    /// 而 dst 为英文时，使用 `&str` 或者 `Cow::Borrowed` 类型可以减少分配。
    ///
    /// ## 注意
    /// 无翻译内容时，返回 `None`。
    pub fn is_borrowed(&self) -> Option<bool> {
        match &self.res {
            ResponseInner::Ok(Success { res, .. }) => {
                if res.len() != 0 {
                    Some(true)
                    // Some(matches!(res[0], std::borrow::Cow::Borrowed(_)))
                } else {
                    None
                }
            }
            &ResponseInner::Err(_) => None,
        }
    }
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
    #[serde(rename = "TargetTextList")]
    pub res:  Vec<&'r str>,
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

impl std::error::Error for ResponseError {}
impl std::fmt::Display for ResponseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,
               "错误码：`{}`\n错误信息：`{}`\n错误含义：{}\n以上内容由腾讯云 API 返回",
               self.code,
               self.msg,
               self.solution())
    }
}

impl ResponseError {
    /// 参考：[错误码列表](https://cloud.tencent.com/document/product/551/30637)
    pub fn solution(&self) -> &str {
        match self.code.as_bytes() {
            b"ActionOffline" => "接口已下线。",
            b"AuthFailure.InvalidAuthorization" => "请求头部的 Authorization 不符合腾讯云标准。",
            b"AuthFailure.InvalidSecretId" => "密钥非法（不是云 API 密钥类型）。",
            b"AuthFailure.MFAFailure" => {
                "[MFA](https://cloud.tencent.com/document/product/378/12036) 错误。"
            }
            b"AuthFailure.SecretIdNotFound" => {
                "密钥不存在。请在控制台检查密钥是否已被删除或者禁用，如状态正常，\
                 请检查密钥是否填写正确，注意前后不得有空格。"
            }
            b"AuthFailure.SignatureExpire" => {
                "签名过期。Timestamp \
                 和服务器时间相差不得超过五分钟，请检查本地时间是否和标准时间同步。"
            }
            b"AuthFailure.SignatureFailure" => {
                "签名错误。签名计算错误，请对照调用方式中的签名方法文档检查签名计算过程。"
            }
            b"AuthFailure.TokenFailure" => "token 错误。",
            b"AuthFailure.UnauthorizedOperation" => "请求未授权。请参考",
            b"DryRunOperation" => "DryRun 操作，代表请求将会是成功的，只是多传了 DryRun 参数。",
            b"FailedOperation" => "操作失败。",
            b"InternalError" => "内部错误。",
            b"InvalidAction" => "接口不存在。",
            b"InvalidParameter" => "参数错误（包括参数格式、类型等错误）。",
            b"InvalidParameterValue" => "参数取值错误。",
            b"InvalidRequest" => "请求 body 的 multipart 格式错误。",
            b"IpInBlacklist" => "IP地址在黑名单中。",
            b"IpNotInWhitelist" => "IP地址不在白名单中。",
            b"LimitExceeded" => "超过配额限制。",
            b"MissingParameter" => "缺少参数。",
            b"NoSuchProduct" => "产品不存在",
            b"NoSuchVersion" => "接口版本不存在。",
            b"RequestLimitExceeded" => "请求的次数超过了频率限制。",
            b"RequestLimitExceeded.GlobalRegionUinLimitExceeded" => "主账号超过频率限制。",
            b"RequestLimitExceeded.IPLimitExceeded" => "IP限频。",
            b"RequestLimitExceeded.UinLimitExceeded" => "主账号限频。",
            b"RequestSizeLimitExceeded" => "请求包超过限制大小。",
            b"ResourceInUse" => "资源被占用。",
            b"ResourceInsufficient" => "资源不足。",
            b"ResourceNotFound" => "资源不存在。",
            b"ResourceUnavailable" => "资源不可用。",
            b"ResponseSizeLimitExceeded" => "返回包超过限制大小。",
            b"ServiceUnavailable" => "当前服务暂时不可用。",
            b"UnauthorizedOperation" => "未授权操作。",
            b"UnknownParameter" => "未知参数错误，用户多传未定义的参数会导致错误。",
            b"UnsupportedOperation" => "操作不支持。",
            b"UnsupportedProtocol" => "http(s) 请求协议错误，只支持 GET 和 POST 请求。",
            b"UnsupportedRegion" => "接口不支持所传地域。",
            b"FailedOperation.NoFreeAmount" => {
                "本月免费额度已用完，如需继续使用您可以在机器翻译控制台升级为付费使用。"
            }
            b"FailedOperation.ServiceIsolate" => "账号因为欠费停止服务，请在腾讯云账户充值。",
            b"FailedOperation.UserNotRegistered" => {
                "服务未开通，请在腾讯云官网机器翻译控制台开通服务。"
            }
            b"InternalError.BackendTimeout" => "后台服务超时，请稍后重试。",
            b"InternalError.ErrorUnknown" => "未知错误。",
            b"InternalError.RequestFailed" => "请求失败。",
            b"InvalidParameter.DuplicatedSessionIdAndSeq" => "重复的SessionUuid和Seq组合。",
            b"InvalidParameter.MissingParameter" => "参数错误。",
            b"InvalidParameter.SeqIntervalTooLarge" => "Seq之间的间隙请不要大于2000。",
            b"LimitExceeded.LimitedAccessFrequency" => "超出请求频率。",
            b"UnauthorizedOperation.ActionNotFound" => "请填写正确的Action字段名称。",
            b"UnsupportedOperation.AudioDurationExceed" => {
                "音频分片长度超过限制，请保证分片长度小于8s。"
            }
            b"UnsupportedOperation.TextTooLong" => {
                "单次请求text超过长度限制，请保证单次请求长度低于2000。"
            }
            b"UnsupportedOperation.UnSupportedTargetLanguage" => {
                "不支持的目标语言，请参照语言列表。"
            }
            b"UnsupportedOperation.UnsupportedLanguage" => "不支持的语言，请参照语言列表。",
            b"UnsupportedOperation.UnsupportedSourceLanguage" => "不支持的源语言，请参照语言列表。",
            _ => "未知错误。",
        }
    }
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
