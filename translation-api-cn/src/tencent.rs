use hmac::{Hmac, Mac, NewMac};
use serde::{Deserialize, Serialize};
use sha2::Sha256;

// Create alias for HMAC-SHA256
type HmacSha256 = Hmac<Sha256>;

/// 翻译前的必要信息
///
/// https://cloud.tencent.com/document/product/551/40566
#[derive(Debug)]
pub struct Query<'q> {
    /// 请求翻译 query，必须为 UTF-8 编码。
    ///
    /// TODO: 在传入之前应该把文字控制在 2000 以内,
    /// 一个汉字、一个字母、一个标点都计为一个字符，
    /// 超过 2000 字节要分段请求。
    pub q:    &'q str,
    /// 翻译源语言，可设置为 auto
    ///
    /// TODO：变成 enum 类型
    pub from: &'q str,
    /// 翻译目标语言，不可设置为 auto
    ///
    /// TODO：和 `from` 共用 enum 类型，但是并非任意两个语言之间可以互译。
    /// 比如 `ar（阿拉伯语）：en（英语）` 表明阿拉伯语只能从英语中翻译过去。
    pub to:   &'q str,
}

/// 账户信息以及一些不变的信息
#[derive(Debug, Deserialize)]
#[serde(rename = "baidu")] // for config or cmd
pub struct User {
    /// SecretId
    pub id:        String,
    /// SecretKey
    pub key:       String,
    /// 地域列表，默认为上海。
    #[serde(default)]
    pub region:    Region,
    /// 项目ID，可以根据控制台-账号中心-项目管理中的配置填写，如无配置请填写默认项目ID:0
    #[serde(default)]
    pub projectid: u8,
    /// 每秒并发请求
    #[serde(default = "default_qps")]
    #[serde(skip_deserializing)]
    pub qps:       u8,
    #[serde(default = "default_action")]
    #[serde(skip_deserializing)]
    pub action:    String,
    #[serde(default = "default_service")]
    #[serde(skip_deserializing)]
    pub service:   String,
    #[serde(default = "default_version")]
    #[serde(skip_deserializing)]
    pub version:   String,
}

fn default_qps() -> u8 { 5 }
fn default_action() -> String { "TextTranslateBatch".into() }
fn default_service() -> String { "tmt".into() }
fn default_version() -> String { "2018-03-21".into() }

/// 生成请求结构
pub struct HeaderJson<'u, 'q> {
    pub timestamp:     i64,
    pub authorization: String,
    pub user:          &'u User,
    pub query:         &'q Query<'q>,
}

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
    fn default() -> Self { Self::Shanghai }
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
