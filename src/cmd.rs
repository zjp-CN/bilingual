use crate::config::{Config, API};
use anyhow::{anyhow, Result};
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

/// 命令行目前只输入基本的 id 和 key，指定其他请求选项需指定 toml 文件。
/// 比如指定：
/// - 百度：qps、salt
/// - 腾讯：projectid
impl Bilingual {
    pub fn run(mut self) -> Result<Config> {
        #[cfg(debug_assertions)]
        dbg!(&self);
        let mut cf = Config::init(self.toml)?;
        match self.api {
            API::Baidu => baidu(self.id, self.key, &mut cf)?,
            API::Tencent => tencent(self.id, self.key, &mut cf)?,
            _ => (),
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

fn tencent(id: String, key: String, cf: &mut Config) -> Result<()> {
    cf.tencent.replace(translation_api_cn::tencent::User::default());
    if !id.is_empty() {
        cf.tencent.as_mut().ok_or(anyhow!("覆盖腾讯云 API.id 时出错"))?.id = id;
    }
    if !key.is_empty() {
        cf.tencent.as_mut().ok_or(anyhow!("覆盖腾讯云 API.key 时出错"))?.key = key;
    }
    Ok(())
}

fn baidu(id: String, key: String, cf: &mut Config) -> Result<()> {
    cf.baidu.replace(translation_api_cn::baidu::User::default());
    if !id.is_empty() {
        cf.baidu.as_mut().ok_or(anyhow!("覆盖百度翻译 API.id 时出错"))?.appid = id;
    }
    if !key.is_empty() {
        cf.baidu.as_mut().ok_or(anyhow!("覆盖百度翻译 API.key 时出错"))?.key = key;
    }
    Ok(())
}
