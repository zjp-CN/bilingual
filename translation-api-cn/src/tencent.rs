use hmac::{
    crypto_mac::{InvalidKeyLength, Output as HmacOutput},
    Hmac, Mac, NewMac,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use time::OffsetDateTime;

// Create alias for HMAC-SHA256
pub type HmacSha256 = Hmac<Sha256>;
pub type Output = HmacOutput<HmacSha256>;
pub type HashResult<T> = Result<T, InvalidKeyLength>;
pub type MultiErrResult<T> = Result<T, Box<dyn std::error::Error>>;

pub fn hash256(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    format!("{:x}", hasher.finalize())
}
pub fn hash_string_from_u8(v: &[u8]) -> HashResult<String> {
    Ok(format!("{:x}", HmacSha256::new_from_slice(v)?.finalize().into_bytes()))
}
pub fn hash_string_from_hash(v: Output) -> HashResult<String> {
    Ok(format!("{:x}",
               HmacSha256::new_from_slice(v.into_bytes().as_slice())?.finalize().into_bytes()))
}
pub fn hash_u8_hash(key: &[u8], msg: Output) -> HashResult<Output> {
    let mut mac = HmacSha256::new_from_slice(key)?;
    mac.update(msg.into_bytes().as_slice());
    Ok(mac.finalize())
}
pub fn hash_hash_u8(key: Output, msg: &[u8]) -> HashResult<Output> {
    let mut mac = HmacSha256::new_from_slice(key.into_bytes().as_slice())?;
    mac.update(msg);
    Ok(mac.finalize())
}
pub fn hash_2u8(key: &[u8], msg: &[u8]) -> HashResult<Output> {
    let mut mac = HmacSha256::new_from_slice(key)?;
    mac.update(msg);
    Ok(mac.finalize())
}
pub fn hash_2hash(key: Output, msg: Output) -> HashResult<Output> {
    let mut mac = HmacSha256::new_from_slice(key.into_bytes().as_slice())?;
    mac.update(msg.into_bytes().as_slice());
    Ok(mac.finalize())
}

// /// 使用 HMAC-SHA256 算法，对 `&[u8]` 或者具有 `.as_bytes()` 方法的数据计算 Hash 十六进制值
// macro_rules! hash {
//     // (@b $v:expr) => { hash($v.as_bytes()) };
//     (@@b $v:expr) => { // 输入 Output<HmacSha256>> 生成 Output<HmacSha256>>
//             $v.as_bytes()
//     };
//     (@@h $v:expr) => {{ // 生成 Hash 字符串
//             $v.into_bytes().as_slice()
//     }};
//     // (@b $key:expr, $($msg:expr),+) => { hash!($key.as_bytes(), $($msg.as_bytes()),+) };
//     ($v:expr) => {
//         Ok::<_, InvalidKeyLength>(
//             format!("{:x}", HmacSha256::new_from_slice($v)?.finalize().into_bytes())
//         )
//     };
//     (@h $key:expr, $($msg:expr),+) => {{
//         let mut mac = HmacSha256::new_from_slice( hash!(@@h $key))?;
//         $(mac.update(hash!(@@h $msg));)+
//         // Ok::<_, InvalidKeyLength>(format!("{:x}", mac.finalize().into_bytes()))
//         Ok::<_, InvalidKeyLength>(mac.finalize())
//     }};
//     (@b $key:expr, $($msg:expr),+) => {{
//         let mut mac = HmacSha256::new_from_slice( hash!(@@b $key))?;
//         $(mac.update(hash!(@@b $msg));)+
//         // Ok::<_, InvalidKeyLength>(format!("{:x}", mac.finalize().into_bytes()))
//         Ok::<_, InvalidKeyLength>(mac.finalize())
//     }};
// }

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
    pub projectid: u8,
    #[serde(rename = "SourceTextList")]
    pub q:         &'q [&'q str],
}

impl<'q> Query<'q> {
    pub fn to_hashed(&self) -> serde_json::Result<String> {
        Ok(hash256(&serde_json::to_vec(self)?))
    }

    pub fn to_json(&self) -> serde_json::Result<String> { serde_json::to_string(self) }
}

/// 账户信息以及一些不变的信息
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
pub struct HeaderJson<'u, 'q> {
    pub datetime:         OffsetDateTime,
    pub timestamp:        String,
    pub credential_scope: String,
    pub authorization:    String,
    pub user:             &'u User,
    pub query:            &'q Query<'q>,
}

impl<'u, 'q> HeaderJson<'u, 'q> {
    const ALGORITHM: &'static str = "TC3-HMAC-SHA256";
    const CANONICALHEADERS: &'static str =
        "content-type:application/json; charset=utf-8\nhost:cvm.tencentcloudapi.com\n";
    const CANONICALQUERYSTRING: &'static str = "";
    const CANONICALURI: &'static str = "POST";
    const CONTENTTYPE: &'static str = "application/json; charset=utf-8";
    const CREDENTIALSCOPE: &'static str = "tc3_request";
    const HOST: &'static str = "tmt.tencentcloudapi.com";
    const HTTPREQUESTMETHOD: &'static str = "/";
    const SIGNEDHEADERS: &'static str = "content-type;host";
    pub const URL: &'static str = "https://tmt.tencentcloudapi.com";

    #[rustfmt::skip]
    pub fn new(user: &'u User, query: &'q Query) -> Self {
        let datetime = time::OffsetDateTime::now_utc();
        let timestamp = datetime.unix_timestamp().to_string();
        Self { datetime, timestamp, credential_scope: String::new(),
               authorization: String::new(), user, query }
    }

    pub fn signature(&mut self) -> MultiErrResult<String> {
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
        hash_string_from_hash(hash_hash_u8(secret_signing, stringtosign.as_bytes())?).map_err(InvalidKeyLength::into)
    }

    pub fn authorization(&mut self) -> MultiErrResult<&str> {
        let signature = self.signature()?;
        self.authorization = format!("{} Credential={}/{},SignedHeaders={},Signature={}",
                                     Self::ALGORITHM,
                                     self.user.id,
                                     self.credential_scope,
                                     Self::SIGNEDHEADERS,
                                     signature);
        Ok(&self.authorization)
    }

    pub fn header(&self) -> Option<HashMap<&str, &str>> {
        let mut map = HashMap::new();
        map.insert("authorization", self.authorization.as_str())?;
        map.insert("content-type", Self::CONTENTTYPE)?;
        map.insert("host", Self::HOST)?;
        map.insert("x-tc-action", &self.user.action)?;
        map.insert("x-tc-version", &self.user.version)?;
        map.insert("x-tc-region", self.user.region.as_str())?;
        map.insert("x-tc-timestamp", &self.timestamp)?;
        Some(map)
    }
}
use std::collections::HashMap;

/// | 地域 | 取值 |
/// | --- | --- |
/// | 亚太东南(曼谷) | ap-bangkok |
/// | 华北地区(北京) | ap-beijing |
/// | 西南地区(成都) | ap-chengdu |
/// | 西南地区(重庆) | ap-chongqing |
/// | 华南地区(广州) | ap-guangzhou |
/// | 港澳台地区(中国香港) | ap-hongkong |
/// | 亚太南部(孟买) | ap-mumbai |
/// | 亚太东北(首尔) | ap-seoul |
/// | 华东地区(上海) | ap-shanghai |
/// | 华东地区(上海金融) | ap-shanghai-fsi |
/// | 华南地区(深圳金融) | ap-shenzhen-fsi |
/// | 亚太东南(新加坡) | ap-singapore |
/// | 欧洲地区(法兰克福) | eu-frankfurt |
/// | 美国东部(弗吉尼亚) | na-ashburn |
/// | 美国西部(硅谷) | na-siliconvalley |
/// | 北美地区(多伦多) | na-toronto |
///
/// ## 注意
/// 金融区需要单独申请，而且只为金融客户服务。具体见：
/// https://cloud.tencent.com/document/product/304/2766
#[derive(Debug, Deserialize, Serialize)]
pub enum Region {
    #[serde(rename = "ap-beijing")]
    Beijing,
    #[serde(rename = "ap-shanghai")]
    Shanghai,
    #[serde(rename = "ap-shanghai-fsi")]
    ShanghaiFsi,
    #[serde(rename = "ap-guangzhou")]
    Guangzhou,
    #[serde(rename = "ap-shenzhen-fsi")]
    ShenzhenFsi,
    #[serde(rename = "ap-chengdu")]
    Chengdu,
    #[serde(rename = "ap-chongqing")]
    Chongqing,
    #[serde(rename = "ap-hongkong")]
    Hongkong,
    #[serde(rename = "ap-bangkok")]
    Bangkok,
    #[serde(rename = "ap-mumbai")]
    Mumbai,
    #[serde(rename = "ap-seoul")]
    Seoul,
    #[serde(rename = "ap-singapore")]
    Singapore,
    #[serde(rename = "ap-frankfurt")]
    Frankfurt,
    #[serde(rename = "ap-ashburn")]
    Ashburn,
    #[serde(rename = "ap-siliconvalley")]
    Siliconvalley,
    #[serde(rename = "ap-toronto")]
    Toronto,
}

impl Default for Region {
    fn default() -> Self { Self::Beijing }
}

impl Region {
    #[rustfmt::skip]
    pub fn as_str(&self) -> &str {
        use Region::*;
        match self {
            Beijing       => "ap-beijing",
            Shanghai      => "ap-shanghai",
            ShanghaiFsi   => "ap-shanghai-fsi",
            Guangzhou     => "ap-guangzhou",
            ShenzhenFsi   => "ap-shenzhen-fsi",
            Chengdu       => "ap-chengdu",
            Chongqing     => "ap-chongqing",
            Hongkong      => "ap-hongkong",
            Bangkok       => "ap-bangkok",
            Mumbai        => "ap-mumbai",
            Seoul         => "ap-seoul",
            Singapore     => "ap-singapore",
            Frankfurt     => "ap-frankfurt",
            Ashburn       => "ap-ashburn",
            Siliconvalley => "ap-siliconvalley",
            Toronto       => "ap-toronto",
        }
    }
}

/// 错误处理 / 错误码
///
/// see:
/// - https://cloud.tencent.com/document/product/551/30637
/// - https://cloud.tencent.com/api/error-center?group=PLATFORM&page=1
/// - https://cloud.tencent.com/document/product/551/40566
#[derive(Debug, Clone, Deserialize)]
pub struct Error {
    #[serde(rename = "error_code")]
    pub code: String,
    #[serde(rename = "error_msg")]
    pub msg:  String,
}
