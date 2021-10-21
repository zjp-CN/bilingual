//! 同步请求。qps = 1。默认英译中。
//! ```console
//! DEBUG=1 cargo run --example baidu -- -a xx -k xx -q hello\ world hi China
//! ```
//!
//! 正常返回：
//! ```text
//! Cmd {
//!     appid: "xx",
//!     key: "xx",
//!     from: "en",
//!     to: "zh",
//!     query: "hello world",
//!     multiquery: [
//!         "hi",
//!         "China",
//!     ],
//! }
//! Query {
//!     q: "hi\nChina\nhello world",
//!     from: "en",
//!     to: "zh",
//!     sign: "xx",
//! }
//! 翻译结果：[
//!     "你好",
//!     "中国",
//!     "你好，世界",
//! ]
//! ```
//!
//! 错误返回（以 54001 错误码为例）：
//! ```text
//! Cmd {
//!     appid: "xx",
//!     key: "xx",
//!     from: "en",
//!     to: "zh",
//!     query: "hello world",
//!     multiquery: [
//!         "hi",
//!         "China",
//!     ],
//! }
//! Error: 解析返回数据时失败
//!
//! Caused by:
//!     错误码：`54001`
//!     错误信息：`Invalid Sign`
//!     错误含义：签名错误。
//!     解决方法：请检查您的签名生成方法。
//!     以上内容由百度翻译 API 返回
//! ```

use anyhow::{Context, Result};
use reqwest::blocking;
use translation_api_cn::baidu::{Form, Query, Response, User};

macro_rules! log {
    (display: $v:expr) => {
        log!("{}", $v)
    };
    ($v:expr) => {
        log!("{:#?}", $v)
    };
    ($fmt:expr, $($arg:tt)*)=>{
        match std::env::var("DEBUG") {
            Ok(x) if x == "true" || x == "1" => println!($fmt, $($arg)*),
            _ => (),
        }
    }
}

fn main() -> Result<()> {
    let mut cmd: Cmd = argh::from_env();
    let user = cmd.to_user();
    let mut query = cmd.to_query();

    let form = query.sign(&user);
    let text = translate(&form)?;

    let response: Response =
        serde_json::from_str(&text).with_context(|| format!("JSON 格式化失败：{}", text))?;
    let dst = response.dst().with_context(|| "解析返回数据时失败")?;

    #[rustfmt::skip]
    log!("{:#?}\ntext: {}\ndst is borrowed: {:?}\nresponse: {:#?}",
         query, text, response.is_borrowed(), response);

    // 从响应数据取翻译结果（3 种方式）：
    println!("翻译结果：{:#?}", dst);
    log!("翻译结果(the same)：{:#?}", response.dst_owned()?);
    log!("翻译结果(the same)：{:#?}", serde_json::from_str::<Response>(&text)?.dst_owned()?);

    Ok(())
}

fn send<T: serde::Serialize + ?Sized>(form: &T) -> Result<blocking::Response> {
    let response =
        blocking::Client::new().post("https://fanyi-api.baidu.com/api/trans/vip/translate")
                               .form(form)
                               .send()
                               .with_context(|| "发送数据失败")?;
    assert!(response.error_for_status_ref().is_ok());
    Ok(response)
}

fn translate<'a>(form: &'a Form<'a>) -> Result<String> {
    send(form).with_context(|| "接收数据失败")?
              .text()
              .with_context(|| "解析文本数据失败")
}

use argh::FromArgs;

/// 百度翻译的简单命令行 demo
#[derive(FromArgs, Debug)]
struct Cmd {
    /// appid
    #[argh(option, short = 'a')]
    appid: String,

    /// key
    #[argh(option, short = 'k')]
    key: String,

    /// 原语言。
    #[argh(option, short = 'f', default = "String::from(\"en\")")]
    from: String,

    /// 目标语言。
    #[argh(option, short = 't', default = "String::from(\"zh\")")]
    to: String,

    /// 单行翻译文本：翻译文本内的空格以 `\ ` 转义。
    #[argh(option, short = 'q', default = "String::new()")]
    query: String,

    /// 多行翻译文本：每行翻译文本以空格分隔。
    #[argh(positional)]
    multiquery: Vec<String>,
}

impl Cmd {
    fn to_query(&mut self) -> Query {
        log!(self);
        let mut query = self.multiquery.join("\n");
        if self.query.len() != 0 {
            if query.len() != 0 {
                query.push_str("\n")
            };
            query.push_str(&self.query);
        };
        self.query = query;
        Query::new(&self.query, &self.from, &self.to)
    }

    /// 注意：salt 应为随机的字母或数字，此处为了简化取 0。
    ///       这会导致查询字符串和身份验证信息（appid 和 key）不变时，计算的 MD5 （sign）不变。
    fn to_user(&self) -> User {
        User { appid: self.appid.clone(),
               key:   self.key.clone(),
               qps:   Some(1),
               salt:  "0".into(), }
    }
}
