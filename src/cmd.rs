use crate::config::{Config, API};
use anyhow::Result;
use argh::FromArgs;
use std::path::PathBuf;

/// bilingual
/// 针对 markdown 文件的命令行翻译。
#[derive(FromArgs, Debug)]
pub struct Bilingual {
    /// 翻译 API
    #[argh(option, short = 'a')]
    api: API,

    /// 翻译 API 账户的 id。
    /// 命令行提供的 id 和 key 会覆盖掉配置文件的信息。
    /// 换言之，未提供命令行的 appid 和 key，则使用配置文件的信息。
    #[argh(option, short = 'i', default = "String::new()")]
    id: String,

    /// 翻译 API 账户的 key。
    /// 命令行提供的 id 和 key 会覆盖掉配置文件的信息。
    /// 换言之，未提供命令行的 appid 和 key，则使用配置文件的信息。
    #[argh(option, short = 'k', default = "String::new()")]
    key: String,

    /// 原语言。
    #[argh(option, short = 'f', default = "String::from(\"en\")")]
    from: String,

    /// 目标语言。
    #[argh(option, short = 't', default = "String::from(\"zh\")")]
    to: String,

    /// 单行翻译文本：翻译文本内的空格以 `\ ` 转义。翻译的顺序位于所有多行翻译文本之后。
    #[argh(option, short = 'q', default = "String::new()")]
    singlequery: String,

    /// md 文件路径。
    #[argh(option, short = 'm')]
    files: Vec<PathBuf>,

    /// 目录。
    #[argh(option, short = 'd')]
    dirs: Vec<PathBuf>,

    /// 配置文件 bilingual.toml 的路径。默认是当前目录下。
    #[argh(option, short = 'l', default = "\"bilingual.toml\".into()")]
    toml: std::path::PathBuf,

    /// 多行翻译文本：每行翻译文本以空格分隔。按照输入的顺序进行翻译。
    #[argh(positional)]
    multiquery: Vec<String>,
}

impl Bilingual {
    pub fn run(mut self) -> Result<Config> {
        #[cfg(debug_assertions)]
        dbg!(&self);
        let mut cf = Config::init(self.toml)?;
        if !self.id.is_empty() {
            if cf.tencent.is_none() {
                cf.tencent = Some(translation_api_cn::tencent::User::default());
            }
            cf.tencent.as_mut().unwrap().id = self.id; // TODO
        }
        if !self.key.is_empty() {
            cf.tencent.as_mut().unwrap().key = self.key; // TODO
        }
        cf.api = self.api;
        cf.src.from = self.from;
        cf.src.to = self.to;
        cf.src.files = self.files;
        cf.src.dirs = self.dirs;
        if !self.singlequery.is_empty() {
            self.multiquery.push(self.singlequery);
        }
        cf.src.query = self.multiquery.join("\n\n");
        Ok(cf)
    }
}
