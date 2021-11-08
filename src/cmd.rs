use crate::config::{Config, API};
use anyhow::Result;
use argh::FromArgs;
use std::path::PathBuf;

/// bilingual
/// 针对 markdown 文件的命令行翻译。
#[derive(FromArgs, PartialEq, Debug)]
pub struct Bilingual {
    #[argh(subcommand)]
    sub: SubCommand,

    /// 可选。打印 TopLevel（及子命令） 结构体。比如 `rustdx -p day`。
    #[argh(switch, short = 'p', long = "print-struct")]
    print_struct: bool,

    /// 配置文件 bilingual.toml 的路径。默认是当前目录下。
    #[argh(option, short = 't', default = "\"bilingual.toml\".into()")]
    toml: std::path::PathBuf,
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand)]
enum SubCommand {
    Baidu(Baidu),
}

impl Bilingual {
    pub fn run(self) -> Result<Config> {
        use SubCommand::*;
        if self.print_struct {
            println!("{:#?}", self);
        }
        let mut cf = Config::init(self.toml)?;
        match self.sub {
            Baidu(cmd) => cmd.run(&mut cf),
        }
        Ok(cf)
    }
}

/// bilingual via Baidu API
#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "baidu")]
struct Baidu {
    /// 百度翻译 API 提供的 appid。注意：
    /// 命令行提供的 appid 和 key 会覆盖掉配置文件的信息。
    /// 换言之，未提供命令行的 appid 和 key，则使用配置文件的信息。
    #[argh(option, short = 'a', default = "String::new()")]
    appid: String,

    /// 百度翻译 API 提供的 key。
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

    /// 多行翻译文本：每行翻译文本以空格分隔。按照输入的顺序进行翻译。
    #[argh(positional)]
    multiquery: Vec<String>,
}

impl Baidu {
    fn run(mut self, cf: &mut Config) {
        if self.appid.len() != 0 {
            cf.baidu.appid = self.appid;
        }
        if self.key.len() != 0 {
            cf.baidu.key = self.key;
        }
        cf.api = API::Baidu;
        cf.src.from = self.from;
        cf.src.to = self.to;
        cf.src.files = self.files;
        cf.src.dirs = self.dirs;
        if self.singlequery.len() != 0 {
            self.multiquery.push(self.singlequery);
        }
        cf.src.query = self.multiquery.join("\n\n");
    }
}
