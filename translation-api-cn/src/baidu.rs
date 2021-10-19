// use nanorand::{Rng, WyRand};
use serde::{Deserialize, Serialize};

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
    // pub fn en_zh(q: &'q str) -> Self {
    //     Self { q,
    //            from: "en",
    //            to: "zh",
    //            sign: "".into() }
    // }
    //
    // pub fn zh_en(q: &'q str) -> Self {
    //     Self { q,
    //            from: "zh",
    //            to: "en",
    //            sign: "".into() }
    // }
    //
    // pub fn q(mut self, q: &'q str) -> Self {
    //     self.q = q;
    //     self
    // }
    //
    // pub fn from(mut self, from: &'q str) -> Self {
    //     self.from = from;
    //     self
    // }
    //
    // pub fn to(mut self, to: &'q str) -> Self {
    //     self.to = to;
    //     self
    // }

    /// 实例化
    pub fn new(q: &'q str, from: &'q str, to: &'q str) -> Self {
        Self { q,
               from,
               to,
               sign: "".into() }
    }

    /// 计算 MD5 值，返回以表单方式提交的数据，用于身份验证/登录。
    /// 当以下内容至少一项发生变动时，必须调用此方法：
    /// - User：appid、salt、key
    /// - Query：q
    pub fn sign<'f>(&'f mut self, user: &'f User) -> Form<'f> {
        let data = format!("{}{}{}{}", &user.appid, self.q, &user.salt, &user.key);
        self.sign = format!("{:x}", md5::compute(data));
        Form::from_user_query(user, self)
    }

    // /// 未计算 MD5 值，返回以表单方式提交的数据。
    // pub fn form<'f>(&'f self, user: &'f User) -> Form<'f> { Form::from_user_query(user, self) }
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
pub enum Response {
    Ok(Success),
    Err(BaiduError),
}

impl Response {
    /// 合并翻译内容
    /// TODO: 测试 slice::join 和 iter::fold 在拼接 str 上的速度
    pub fn result(self) -> Result<Vec<String>, BaiduError> {
        match self {
            Response::Ok(res) => Ok(res.dst()),
            Response::Err(e) => Err(e),
        }
    }
}

/// 返回的数据
#[derive(Debug, Clone, Deserialize)]
pub struct Success {
    pub from: String,
    pub to:   String,
    /// 原文中被 `\n` 分隔的多条翻译文本。
    #[serde(rename = "trans_result")]
    pub res:  Vec<SrcDst>,
}

/// 单条翻译文本
#[derive(Debug, Clone, Deserialize)]
pub struct SrcDst {
    // src: String,
    pub dst: String,
}

impl Success {
    /// 取出翻译内容
    /// TODO: 测试 slice::join 和 iter::fold 在拼接 str 上的速度
    pub fn dst(self) -> Vec<String> {
        self.res.into_iter().map(|x| x.dst).collect()
        // self.res.join("\n").to_string() // error: Vec<T> join => T
    }
}

/// 错误处理 / 错误码
#[derive(Debug, Deserialize)]
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
        match self.error_code.as_str() {
            "52000" => "成功。",
            "52001" => "请求超时。\n解决方法：请重试。",
            "52002" => "系统错误。\n解决方法：请重试。",
            "52003" => "未授权用户。\n解决方法：请检查appid是否正确或者服务是否开通。",
            "54000" => "必填参数为空。\n解决方法：请检查是否少传参数。",
            "54001" => "签名错误。\n解决方法：请检查您的签名生成方法。",
            "54003" => {
                "访问频率受限。\n解决方法：请降低您的调用频率，或进行身份认证后切换为高级版/\
                 尊享版。"
            }
            "54004" => "账户余额不足。\n解决方法：请前往管理控制台为账户充值。",
            "54005" => "长 query 请求频繁。\n解决方法：请降低长 query 的发送频率，3s后再试。",
            "58000" => {
                "客户端 IP 非法。\n解决方法：检查个人资料里填写的 IP \
                 地址是否正确，可前往开发者信息-基本信息修改。"
            }
            "58001" => "译文语言方向不支持。\n解决方法：检查译文语言是否在语言列表里。",
            "58002" => "服务当前已关闭。\n解决方法：请前往管理控制台开启服务。",
            "90107" => "认证未通过或未生效。\n解决方法：请前往我的认证查看认证进度。",
            _ => "未知错误。",
        }
    }
}
