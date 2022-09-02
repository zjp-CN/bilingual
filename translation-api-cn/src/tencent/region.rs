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
#[derive(Debug, serde:: Deserialize, serde:: Serialize)]
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
