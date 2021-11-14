#[cfg(feature = "baidu")]
pub mod baidu;

#[cfg(feature = "tencent")]
pub mod tencent;

#[cfg(feature = "niutrans")]
pub mod niutrans;

/// 单次调用各 API 时，被限制的“字符”单位
///
/// 对于百度翻译，为 Byte；对于腾讯云和小牛翻译，为 Char。
#[derive(Debug, serde::Deserialize)]
pub enum Limit {
    #[serde(rename = "bytes")]
    Byte(usize),
    #[serde(rename = "chars")]
    Char(usize),
}

impl Limit {
    pub fn limit(&self) -> usize {
        let (&Limit::Byte(l) | &Limit::Char(l)) = self;
        l
    }
}
