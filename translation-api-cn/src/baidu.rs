// use nanorand::{Rng, WyRand};
use serde::{Deserialize, Serialize};
use std::borrow::Cow;

/// 翻译前的必要信息
#[derive(Debug)]
pub struct Query<'q> {
    /// 请求翻译 query , UTF-8 编码
    /// TODO: 在传入之前应该把文字控制在 6000 字节以内（汉字约为 2000 个字符），然后分段请求
    pub q:    &'q str,
    /// 翻译源语言，可设置为 auto
    /// TODO：变成 Option + enum 类型，None 表示 auto
    pub from: &'q str,
    /// 翻译目标语言，不可设置为 auto
    /// TODO：和 `from` 共用 enum 类型，但无需是 Option 类型
    pub to:   &'q str,
    /// appid+q+salt+密钥的 MD5 值，q 是待查询的原文字符串
    pub sign: String,
}

/// 账户信息
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename = "baidu")] // for config or cmd
#[allow(dead_code)]
pub struct User {
    /// 用户申请得到的 APP ID
    pub appid: String,
    /// 用户申请得到的密钥，这个字段用于生成 MD5 ，不用于直接构造请求内容
    pub key:   String,
    /// TODO: QPS：这涉及并发请求，允许不填，默认为 1
    pub qps:   Option<u8>,
    /// 随机的字母或数字的字符串
    pub salt:  String,
}

impl<'q> Query<'q> {
    /// 实例化
    pub fn new(q: &'q str, from: &'q str, to: &'q str) -> Self {
        Self { q,
               from,
               to,
               sign: "".into() }
    }

    /// 计算 MD5 值，返回以表单方式提交的数据，用于身份验证/登录。
    /// 当以下内容至少一项发生变动时，必须调用此方法：
    /// - User: [appid]、[salt]、[key]
    /// - Query: [q][`Query::q`]
    ///
    /// [appid]: `User::appid`
    /// [salt]: `User::salt`
    /// [key]: `User::key`
    pub fn sign<'f>(&'f mut self, user: &'f User) -> Form<'f> {
        let data = format!("{}{}{}{}", &user.appid, self.q, &user.salt, &user.key);
        self.sign = format!("{:x}", md5::compute(data));
        Form::from_user_query(user, self)
    }
}

/// 以表单方式提交的数据
#[derive(Debug, Serialize)]
pub struct Form<'f> {
    pub q:     &'f str,
    pub from:  &'f str,
    pub to:    &'f str,
    pub appid: &'f str,
    pub salt:  &'f str,
    pub sign:  &'f str,
}

impl<'f> Form<'f> {
    pub fn from_user_query(user: &'f User, query: &'f Query) -> Self {
        Self { q:     query.q,
               from:  query.from,
               to:    query.to,
               appid: &user.appid,
               salt:  &user.salt,
               sign:  &query.sign, }
    }
}

/// 响应的信息。要么返回翻译结果，要么返回错误信息。
#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum Response<'r> {
    #[serde(borrow)]
    Ok(Success<'r>),
    Err(BaiduError),
}

impl<'r> Response<'r> {
    /// 提取翻译内容。无翻译内容时，返回错误。
    ///
    /// TODO: [`BaiduError`] 会经过两次内存分配，这种设计的原因是
    ///       `anyhow` crate 要求错误的类型必须是 `'static`。
    ///       [`BaiduError`] 一次分配的例子见 `tests/baidu.rs`。
    pub fn dst(&self) -> Result<Vec<&str>, BaiduError> {
        match self {
            Response::Ok(s) => Ok(s.res.iter().map(|x| x.dst.as_ref()).collect()),
            Response::Err(e) => Err(e.clone()),
        }
    }

    /// 提取翻译内容。无翻译内容时，返回错误。
    pub fn dst_owned(self) -> Result<Vec<String>, BaiduError> {
        match self {
            Response::Ok(s) => Ok(s.res.into_iter().map(|x| x.dst.into()).collect()),
            Response::Err(e) => Err(e),
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
    pub fn is_borrowed(&self) -> Option<bool> {
        match self {
            Response::Ok(Success { res, .. }) => {
                if res.len() != 0 {
                    Some(matches!(res[0].dst, Cow::Borrowed(_)))
                } else {
                    None
                }
            }
            Response::Err(_) => None,
        }
    }
}

/// 返回的数据
#[derive(Debug, Deserialize)]
pub struct Success<'r> {
    pub from: &'r str,
    pub to:   &'r str,
    /// 原文中被 `\n` 分隔的多条翻译文本。
    #[serde(rename = "trans_result")]
    #[serde(borrow)]
    pub res:  Vec<SrcDst<'r>>,
}

/// 单条翻译文本
#[derive(Debug, Deserialize)]
pub struct SrcDst<'r> {
    // pub src: &'r str,
    #[serde(borrow)]
    pub dst: Cow<'r, str>,
}

/// 错误处理 / 错误码
#[derive(Debug, Clone, Deserialize)]
pub struct BaiduError {
    pub error_code: String,
    pub error_msg:  String,
}

impl std::fmt::Display for BaiduError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,
               "错误码：`{}`\n错误信息：`{}`\n错误含义：{}\n以上内容由百度翻译 API 返回",
               self.error_code,
               self.error_msg,
               self.solution())
    }
}

impl std::error::Error for BaiduError {}

impl BaiduError {
    /// 参考：[错误码列表](https://fanyi-api.baidu.com/doc/21)
    pub fn solution(&self) -> &str {
        match self.error_code.as_bytes() {
            b"52000" => "成功。",
            b"52001" => "请求超时。\n解决方法：请重试。",
            b"52002" => "系统错误。\n解决方法：请重试。",
            b"52003" => "未授权用户。\n解决方法：请检查appid是否正确或者服务是否开通。",
            b"54000" => "必填参数为空。\n解决方法：请检查是否少传参数。",
            b"54001" => "签名错误。\n解决方法：请检查您的签名生成方法。",
            b"54003" => {
                "访问频率受限。\n解决方法：请降低您的调用频率，或进行身份认证后切换为高级版/\
                 尊享版。"
            }
            b"54004" => "账户余额不足。\n解决方法：请前往管理控制台为账户充值。",
            b"54005" => "长 query 请求频繁。\n解决方法：请降低长 query 的发送频率，3s后再试。",
            b"58000" => {
                "客户端 IP 非法。\n解决方法：检查个人资料里填写的 IP \
                 地址是否正确，可前往开发者信息-基本信息修改。"
            }
            b"58001" => "译文语言方向不支持。\n解决方法：检查译文语言是否在语言列表里。",
            b"58002" => "服务当前已关闭。\n解决方法：请前往管理控制台开启服务。",
            b"90107" => "认证未通过或未生效。\n解决方法：请前往我的认证查看认证进度。",
            _ => "未知错误。",
        }
    }
}
