use crate::Limit;
use std::borrow::Cow;

use serde::{Deserialize, Serialize};

pub const URL: &str = "https://api.niutrans.com/NiuTransServer/translation";

/// 翻译前的必要信息
///
/// 参考：https://niutrans.com/documents/contents/trans_text
#[derive(Debug, Serialize)]
pub struct Query<'q> {
    /// 请求翻译 query，必须为 UTF-8 编码。
    ///
    /// TODO: 在传入之前应该把文字控制在 6000 字节以内（汉字约为 2000 个字符），
    ///       超过 6000 字节要分段请求。
    pub q:    &'q str,
    /// 翻译源语言，不可设置为 auto
    ///
    /// TODO：变成 Option + enum 类型，None 表示 auto
    pub from: &'q str,
    /// 翻译目标语言，不可设置为 auto
    ///
    /// TODO：和 `from` 共用 enum 类型，但无需是 Option 类型
    pub to:   &'q str,
}

impl<'q> Query<'q> {
    /// 实例化
    #[rustfmt::skip]
    pub fn new(q: &'q str, from: &'q str, to: &'q str) -> Self { Self { q, from, to } }

    pub fn form(&self, user: &'q User) -> Form { Form::new(user, self) }
}

/// 账户信息
#[derive(Debug, Deserialize)]
#[serde(rename = "niutrans")] // for config or cmd
pub struct User {
    /// 用户申请得到的密钥
    pub key:    String,
    /// 默认为 50
    #[serde(default = "default_qps")]
    pub qps:    u8,
    /// 每秒并发请求的限制，默认为 Char(5000)。
    #[serde(default = "default_limit")]
    // #[serde(skip_deserializing)]
    pub limit:  Limit,
    /// 术语词典子库 ID
    #[serde(default)]
    pub dict:   String,
    /// 翻译记忆子库 ID
    #[serde(default)]
    pub memory: String,
}

fn default_qps() -> u8 { 50 }
fn default_limit() -> Limit { Limit::Char(5000) }

impl Default for User {
    fn default() -> Self {
        Self { key:    String::new(),
               qps:    default_qps(),
               limit:  default_limit(),
               dict:   String::new(),
               memory: String::new(), }
    }
}

/// 以表单方式提交的数据
#[derive(Debug, Serialize)]
pub struct Form<'f> {
    pub src_text: &'f str,
    pub from:     &'f str,
    pub to:       &'f str,
    pub apikey:   &'f str,
    #[serde(rename = "dictNo")]
    pub dict:     &'f str,
    #[serde(rename = "memoryNo")]
    pub memory:   &'f str,
}

impl<'f> super::Sender for Form<'f> {
    type Form = Self;
    type Header = ();
    type Json = ();
    type Query = &'f Query<'f>;
    type User = &'f User;

    const URL: &'static str = URL;

    fn from_user_query2(user: Self::User, query: Self::Query) -> Self {
        Self { src_text: query.q,
               from:     query.from,
               to:       query.to,
               apikey:   &user.key,
               dict:     &user.dict,
               memory:   &user.dict, }
    }

    fn form2(&self) -> &Self::Form { self }

    fn header2(&self) -> std::collections::HashMap<&str, &str> { std::collections::HashMap::new() }

    fn json2(&self) -> &Self::Json { &() }
}

impl<'f> Form<'f> {
    pub fn new(user: &'f User, query: &'f Query) -> Self {
        Self { src_text: query.q,
               from:     query.from,
               to:       query.to,
               apikey:   &user.key,
               dict:     &user.dict,
               memory:   &user.dict, }
    }
}

/// 响应的信息。要么返回翻译结果，要么返回错误信息。
#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum Response<'r> {
    Ok {
        from:     &'r str,
        to:       &'r str,
        /// 需要手动进行 `\n` 分隔。（注意：末尾似乎会多出一个 \n）
        #[serde(borrow)]
        tgt_text: Cow<'r, str>,
        // tgt_text: String,
    },
    Err {
        from:   &'r str,
        to:     &'r str,
        // /// 需要手动进行 `\n` 分隔。
        // #[serde(borrow)]
        // src_text: Cow<'r, str>,
        apikey: &'r str,
        #[serde(flatten)]
        error:  Error,
    },
}

impl<'r> Response<'r> {
    /// 提取翻译内容。
    pub fn dst(&self) -> Result<impl Iterator<Item = &str>, Error> {
        match self {
            Response::Ok { tgt_text, .. } => Ok(tgt_text.trim_end().split("\n")),
            Response::Err { error, .. } => Err(error.clone()),
        }
    }

    /// 提取翻译内容。
    pub fn dst_owned(self) -> Result<Vec<String>, Error> {
        match self {
            Response::Ok { tgt_text, .. } => {
                Ok(tgt_text.trim_end().split("\n").map(|s| s.into()).collect())
            }
            Response::Err { error, .. } => Err(error),
        }
    }

    /// 返回的翻译内容是否为 `&str` 类型。
    /// ## 注意
    /// 目前发现
    /// - 有翻译内容时，且含 `\n` 之类的转义符号时，返回 `Some(false)`；
    /// - 有翻译内容时，且不含转义符号时，返回 `Some(true)`；
    /// - 无翻译内容时，返回 `None`。
    pub fn is_borrowed(&self) -> Option<bool> {
        match self {
            Response::Ok { tgt_text, .. } => Some(matches!(tgt_text, Cow::Borrowed(_))),
            _ => None,
        }
    }
}

/// response error
/// 错误处理 / 错误码
#[derive(Debug, Clone, Deserialize)]
pub struct Error {
    #[serde(rename = "error_code")]
    pub code: String,
    #[serde(rename = "error_msg")]
    pub msg:  String,
}

impl std::error::Error for Error {}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,
               "错误码：`{}`\n错误信息：`{}`\n错误含义：{}\n以上内容由小牛翻译 API 返回",
               self.code,
               self.msg,
               self.solution())
    }
}

impl Error {
    /// 参考：[错误码列表](https://niutrans.com/documents/contents/trans_text)
    pub fn solution(&self) -> &str {
        match self.code.as_bytes() {
            b"10000" => "输入为空",
            b"10001" => "请求频繁，超出QPS限制",
            b"10003" => "请求字符串长度超过限制",
            b"10005" => "源语编码有问题，非UTF-8",
            b"13001" => "字符流量不足或者没有访问权限",
            b"13002" => "apikey 参数不可以是空",
            b"13003" => "内容过滤异常",
            b"13007" => "语言不支持",
            b"13008" => "请求处理超时",
            b"14001" => "分句异常",
            b"14002" => "分词异常",
            b"14003" => "后处理异常",
            b"14004" => "对齐失败，不能够返回正确的对应关系",
            b"000000" => "请求参数有误，请检查参数",
            b"000001" => "Content-Type不支持【multipart/form-data】",
            _ => "未知错误。",
        }
    }
}

#[test]
fn response_test() {
    let success = "{\"tgt_text\":\"嗨\\n那里\\n\",\"to\":\"zh\",\"from\":\"en\"}";
    let res: Response = serde_json::from_str(success).unwrap();
    assert_eq!(res.is_borrowed(), Some(false));

    let success = "{\"tgt_text\":\"嗨 那里\",\"to\":\"zh\",\"from\":\"en\"}";
    let res: Response = serde_json::from_str(success).unwrap();
    assert_eq!(res.is_borrowed(), Some(true));

    #[rustfmt::skip]
    let error = "{\"to\":\"zh\",\"error_code\":\"13001\",\"from\":\"en\",\
                   \"error_msg\":\"apikey error OR apikey unauthorized OR service package \
                   running out\",\"src_text\":\"hi\\nthere\",\"apikey\":\"xx\"}";
    let res: Response = serde_json::from_str(error).unwrap();
    assert!(res.dst().is_err());
}
