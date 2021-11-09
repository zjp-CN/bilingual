const HELLO_WORLD: &str = r#"{"from":"en","to":"en","trans_result":[{"src":"hello","dst":"hello"},{"src":"world","dst":"world"}]}"#;
const HELLO_WORLD_ZH: &str = r#"{"from":"en","to":"zh","trans_result":[{"src":"hello","dst":"\u4f60\u597d"},{"src":"world","dst":"\u4e16\u754c"}]}"#;

use anyhow::{Context, Result};
use insta::assert_debug_snapshot;

#[test]
fn hello_world_chinese() -> Result<()> {
    let dst: owned::Response = serde_json::from_str(HELLO_WORLD_ZH).with_context(|| "JSON 失败")?;
    assert_debug_snapshot!(dst.result().with_context(||"解析返回数据时失败")?, @r###"
    [
        "你好",
        "世界",
    ]
    "###);

    let dst: borrowed::Response =
        serde_json::from_str(HELLO_WORLD_ZH).with_context(|| "JSON 失败")?;
    assert_debug_snapshot!(dst, @r###"
    Ok(
        Success {
            from: "en",
            to: "zh",
            res: [
                SrcDst {
                    dst: "你好",
                },
                SrcDst {
                    dst: "世界",
                },
            ],
        },
    )
    "###);
    assert_debug_snapshot!(dst.is_borrowed(), @r###"
    Some(
        false,
    )
    "###);
    assert_debug_snapshot!(dst.result().with_context(||"解析返回数据时失败")?, @r###"
    [
        "你好",
        "世界",
    ]
    "###);

    let dst: serde_json::Value = serde_json::from_str(HELLO_WORLD_ZH).with_context(|| "JSON 失败")?;
    assert_debug_snapshot!(dst, @r###"
    Object({
        "from": String(
            "en",
        ),
        "to": String(
            "zh",
        ),
        "trans_result": Array([
            Object({
                "dst": String(
                    "你好",
                ),
                "src": String(
                    "hello",
                ),
            }),
            Object({
                "dst": String(
                    "世界",
                ),
                "src": String(
                    "world",
                ),
            }),
        ]),
    })
    "###);

    Ok(())
}

#[test]
fn hello_world() -> Result<()> {
    let dst: owned::Response = serde_json::from_str(HELLO_WORLD).with_context(|| "JSON 失败")?;
    assert_debug_snapshot!(dst.result().with_context(||"解析返回数据时失败")?, @r###"
    [
        "hello",
        "world",
    ]
    "###);

    let dst: borrowed::Response = serde_json::from_str(HELLO_WORLD).with_context(|| "JSON 失败")?;
    assert_debug_snapshot!(dst, @r###"
    Ok(
        Success {
            from: "en",
            to: "en",
            res: [
                SrcDst {
                    dst: "hello",
                },
                SrcDst {
                    dst: "world",
                },
            ],
        },
    )
    "###);
    assert_debug_snapshot!(dst.is_borrowed(), @r###"
    Some(
        true,
    )
    "###);
    assert_debug_snapshot!(dst.result().with_context(||"解析返回数据时失败")?, @r###"
    [
        "hello",
        "world",
    ]
    "###);

    let dst: serde_json::Value = serde_json::from_str(HELLO_WORLD).with_context(|| "JSON 失败")?;
    assert_debug_snapshot!(dst, @r###"
    Object({
        "from": String(
            "en",
        ),
        "to": String(
            "en",
        ),
        "trans_result": Array([
            Object({
                "dst": String(
                    "hello",
                ),
                "src": String(
                    "hello",
                ),
            }),
            Object({
                "dst": String(
                    "world",
                ),
                "src": String(
                    "world",
                ),
            }),
        ]),
    })
    "###);

    Ok(())
}

mod owned {
    use serde::Deserialize;

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
}

mod borrowed {
    use serde::Deserialize;
    use std::borrow::Cow;

    /// 响应的信息。要么返回翻译结果，要么返回错误信息。
    #[derive(Debug, Deserialize)]
    #[serde(untagged)]
    pub enum Response<'r> {
        #[serde(borrow)]
        Ok(Success<'r>),
        #[serde(borrow)]
        Err(BaiduError<'r>),
    }

    impl<'r> Response<'r> {
        /// 提取翻译内容
        pub fn result(&self) -> Result<Vec<&str>, BaiduErrorOwned> {
            match self {
                Response::Ok(s) => Ok(s.res.iter().map(|x| x.dst.as_ref()).collect()),
                Response::Err(e) => Err(BaiduErrorOwned::from_baidu_error(e)),
            }
        }

        /// 翻译内容是否为 `Cow::Borrowed` 类型。
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
                    if !res.is_empty() {
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
    #[derive(Debug, Clone, Deserialize)]
    pub struct Success<'r> {
        pub from: &'r str,
        pub to:   &'r str,
        /// 原文中被 `\n` 分隔的多条翻译文本。
        #[serde(rename = "trans_result")]
        #[serde(borrow)]
        pub res:  Vec<SrcDst<'r>>,
    }

    /// 单条翻译文本
    #[derive(Debug, Clone, Deserialize)]
    pub struct SrcDst<'r> {
        // pub src: &'r str,
        #[serde(borrow)]
        pub dst: Cow<'r, str>,
    }

    /// 错误处理 / 错误码
    #[derive(Debug, Deserialize)]
    pub struct BaiduError<'r> {
        pub error_code: &'r str,
        pub error_msg:  &'r str,
    }

    impl<'r> std::fmt::Display for BaiduError<'r> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f,
                   "错误码：`{}`\n错误信息：`{}`\n错误含义：{}\n以上内容由百度翻译 API 返回",
                   self.error_code,
                   self.error_msg,
                   self.solution())
        }
    }

    impl<'r> std::error::Error for BaiduError<'r> {}

    impl<'r> BaiduError<'r> {
        /// 参考：[错误码列表](https://fanyi-api.baidu.com/doc/21)
        pub fn solution(&self) -> &str {
            match self.error_code {
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

    /// 错误处理 / 错误码。
    #[derive(Debug, Deserialize)]
    pub struct BaiduErrorOwned {
        pub error_code: String,
        pub error_msg:  String,
    }

    impl<'a> From<BaiduError<'a>> for BaiduErrorOwned {
        fn from(e: BaiduError<'a>) -> Self { Self::from_baidu_error(&e) }
    }

    impl BaiduErrorOwned {
        pub fn to_baidu_error(&self) -> BaiduError {
            BaiduError { error_code: &self.error_code,
                         error_msg:  &self.error_msg, }
        }

        pub fn from_baidu_error(e: &BaiduError) -> Self {
            Self { error_code: e.error_code.into(),
                   error_msg:  e.error_msg.into(), }
        }
    }

    impl std::fmt::Display for BaiduErrorOwned {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            self.to_baidu_error().fmt(f)
        }
    }
    impl std::error::Error for BaiduErrorOwned {}
}
