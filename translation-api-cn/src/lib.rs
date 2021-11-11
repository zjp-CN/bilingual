#[cfg(feature = "baidu")]
pub mod baidu;

#[cfg(feature = "tencent")]
pub mod tencent;

#[cfg(feature = "niutrans")]
pub mod niutrans;

/// 单次调用各 API 时，被限制的“字符”单位
///
/// 对于百度翻译，为 Byte；对于腾讯云和小牛翻译，为 Char。
#[derive(Debug)]
pub enum Limit {
    Byte(u16),
    Char(u16),
}
