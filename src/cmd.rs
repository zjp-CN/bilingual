use crate::config::{Config, DirFile, API};
use anyhow::{anyhow, Result};
use argh::FromArgs;
use std::{
    env::var,
    path::{Path, PathBuf},
};

const VERSION: &str = include_str!(concat!(env!("OUT_DIR"), "/VERSION"));

#[derive(FromArgs, Debug)]
#[argh(description = r#"
【bilingual】 作者：苦瓜小仔

针对 markdown 文件的命令行翻译。使用 `bilingual --help` 查看此帮助说明。

例子：
* `bilingual -a baidu multi queries -q single-query`
* `bilingual -a tencent -m xx.md`
* `bilingual -a niutrans -d ./dir-path`
* `bilingual -a tencent \#\ 标题 正文：模拟\ markdown\ 文件的内容。 -f zh -t en`
* `bilingual -a tencent -m xx.md -M xx-中文.md -d path -D path-中文`
* `bilingual -a tencent -m a.md -M a.md -m b.md -M b.md -d assets -D assets -d test -D test -r`
  这个命令表明：将 a.md、b.md 文件以及 assets 目录、test 目录下的 md 文件的翻译结果直接替换掉源文件。 

注意：本程序使用翻译云服务，因此需要自行申请翻译 API。命令行提供的 id 和 key 会覆盖掉配置文件的信息。
      支持从环境变量或者配置文件 `bilingual.toml` 中获取信息，见 https://github.com/zjp-CN/bilingual/issues/27
"#)]
pub struct Bilingual {
    /// 翻译 API。必选参数。目前支持：baidu | tencent | niutrans。
    #[argh(option, short = 'a', default = "API::default()")]
    api: API,

    /// 翻译 API 账户的 id。
    #[argh(option, short = 'i', default = "String::new()")]
    id: String,

    /// 翻译 API 账户的 key。
    #[argh(option, short = 'k', default = "String::new()")]
    key: String,

    /// 原语言。默认为 en。
    #[argh(option, short = 'f', default = "String::from(\"en\")")]
    from: String,

    /// 目标语言。默认为 zh。
    #[argh(option, short = 't', default = "String::from(\"zh\")")]
    to: String,

    /// 单行翻译文本：翻译文本内特殊符号以 `\` 转义。翻译的顺序位于所有多行翻译文本之后。
    #[argh(option, short = 'q', default = "String::new()")]
    singlequery: String,

    /// md 文件的输入路径。此工具把读取到的文件内容只当作 md 文件进行处理。且不修改 API
    /// 返回的任何内容。可多次指定。
    #[argh(option, short = 'm', long = "input-files")]
    input_files: Vec<PathBuf>,

    /// 输入目录。此工具只识别和读取目录下以 `.md` 结尾的文件。可多次指定。
    #[argh(option, short = 'd', long = "input-dirs")]
    input_dirs: Vec<PathBuf>,

    /// md 文件的输出路径。默认在输入的文件路径下，但是翻译后的文件名会增加 `--to`
    /// 标识。可多次指定。
    #[argh(option, short = 'M', long = "output-files")]
    output_files: Vec<PathBuf>,

    /// 输出目录。默认在输入的目录旁，但是翻译后的目录会增加 `--to` 标识。可多次指定。
    #[argh(option, short = 'D', long = "output-dirs")]
    output_dirs: Vec<PathBuf>,

    /// 如果输出文件已存在，是否替换。默认不替换。
    #[argh(switch, short = 'r', long = "replace-files")]
    replace_file: bool,

    /// 在输出文件夹时不存在时，禁止创建输出文件夹。默认总是创建新文件夹。
    #[argh(switch, long = "forbid-dir-creation")]
    forbid_dir_creation: bool,

    /// 版本号。
    #[argh(switch, short = 'v')]
    version: bool,

    /// 配置文件 bilingual.toml 的路径。默认为 `./bilingual.toml` 或者
    /// `~/.config/bilingual.toml`。
    #[argh(option, default = "default_toml()")]
    toml: PathBuf,

    /// 多行翻译文本：每行翻译文本以空格分隔。按照输入的顺序进行翻译。特殊符号需以 `\` 转义。
    #[argh(positional)]
    multiquery: Vec<String>,
}

fn default_toml() -> PathBuf {
    const PWD_BILINGUAL_TOML: &str = "bilingual.toml";
    if let Ok(s) = var("BILINGUAL_TOML") {
        s.into()
    } else if std::path::Path::new(PWD_BILINGUAL_TOML).exists() {
        PWD_BILINGUAL_TOML.into()
    } else if let Some(mut config_dir) = dirs::config_dir() {
        debug!("配置目录为 {config_dir:?}");
        config_dir.push(PWD_BILINGUAL_TOML); // GLOBAL_BILINGUAL_TOML
        config_dir
    } else {
        PathBuf::new()
    }
}

impl Bilingual {
    pub fn run(mut self) -> Result<Config> {
        debug!("{:#?}", self);
        if self.version {
            println!("{VERSION}");
            std::process::exit(0);
        }
        let mut cf = Config::init(self.toml)?;
        match self.api {
            API::Baidu => baidu(self.id, self.key, &mut cf)?,
            API::Tencent => tencent(self.id, self.key, &mut cf)?,
            API::Niutrans => niutrans(self.key, &mut cf)?,
            _ => anyhow::bail!("请输入 `-a` 参数来指定 baidu | tencent | niutrans 中的一个"),
        }

        if self.output_files.is_empty() {
            cf.src.output_files =
                self.input_files.iter().map(|f| new_filename(f, &self.to)).collect();
        } else if self.input_files.len() == self.output_files.len() {
            cf.src.output_files = self.output_files;
        } else {
            anyhow::bail!("-m 与 -M 的数量必须相等")
        }
        cf.src.input_files = self.input_files;

        if self.output_dirs.is_empty() {
            cf.src.output_dirs = self.input_dirs.iter().map(|f| new_dir(f, &self.to)).collect();
        } else if self.input_dirs.len() == self.output_dirs.len() {
            cf.src.output_dirs = self.output_dirs;
        } else {
            anyhow::bail!("-d 与 -D 的数量必须相等")
        }
        cf.src.input_dirs = self.input_dirs;

        if !self.singlequery.is_empty() {
            self.multiquery.push(self.singlequery);
        }
        cf.src.query = self.multiquery.join("\n\n");

        cf.api = self.api;
        cf.src.from = self.from;
        cf.src.to = self.to;
        cf.src.dir_file = DirFile::new(self.replace_file, self.forbid_dir_creation);
        Ok(cf)
    }
}

fn new_filename(f: &Path, to: &str) -> PathBuf {
    let mut stem = f.file_stem().unwrap().to_os_string();
    stem.reserve(6);
    stem.push("-");
    stem.push(to);
    stem.push(".");
    stem.push(f.extension().unwrap());
    f.with_file_name(stem)
}

fn new_dir(f: &Path, to: &str) -> PathBuf {
    let mut stem = f.file_stem().unwrap().to_os_string();
    stem.reserve(3);
    stem.push("-");
    stem.push(to);
    f.with_file_name(stem)
}

macro_rules! id_key {
    (
        $cf:ident, $api:ident, $name:literal, $key:ident = $keyl:literal $(, $id:ident = $idl:literal)?
    ) => {{
        let cf = $cf;
        if cf.$api.is_none() {
            cf.$api.replace(translation_api_cn::$api::User::default());
            debug!("由于未找到配置文件，先使用默认配置（无 id 和 key）");
        }
        let c = cf.$api.as_mut().ok_or(anyhow!("覆盖 {} API 时出错", $name))?;
        $(
            if !$id.is_empty() {
                c.$id = $id;
                debug!("id 被命令行参数覆盖");
            } else if let Ok(s) = var($idl) {
                c.$id = s;
                debug!("id 被 {} 环境变量覆盖", $idl);
            }
            ::anyhow::ensure!(!c.$id.is_empty(), "id 不应该为空");
        )?
        if !$key.is_empty() {
            c.key = $key;
            debug!("key 被命令行参数覆盖");
        } else if let Ok(s) = var($keyl) {
            c.key = s;
            debug!("id 被 {} 环境变量覆盖", $keyl);
        }
        ::anyhow::ensure!(!c.key.is_empty(), "key 不应该为空");
        Ok(())
    }};
}

fn niutrans(key: String, cf: &mut Config) -> Result<()> {
    id_key! {
        cf, niutrans, "小牛翻译",
        key = "BILINGUAL_NIUTRANS_KEY"
    }
}

fn tencent(id: String, key: String, cf: &mut Config) -> Result<()> {
    id_key! {
        cf, tencent, "腾讯云",
        key = "BILINGUAL_TENCENT_KEY",
        id = "BILINGUAL_TENCENT_ID"
    }
}

fn baidu(appid: String, key: String, cf: &mut Config) -> Result<()> {
    id_key! {
        cf, baidu, "百度翻译",
        key = "BILINGUAL_BAIDU_KEY",
        appid = "BILINGUAL_BAIDU_ID"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_test() {
        insta::assert_display_snapshot!(VERSION, @r###"
        v0.1.1
        git ref: v0.1.1-6-g754d353
        "###);
    }

    #[test]
    fn test_new_filename_dir() {
        assert_eq!(PathBuf::from("/root/test-zh.md"), new_filename("/root/test.md".as_ref(), "zh"));
        assert_eq!(PathBuf::from("/root/test-zh/"), new_dir("/root/test/".as_ref(), "zh"));
        assert_eq!(PathBuf::from("/root/test-zh"), new_dir("/root/test".as_ref(), "zh"));
    }
}
