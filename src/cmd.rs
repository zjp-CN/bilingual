use crate::config::{Config, DirFile, API};
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

    /// md 文件的输入路径。
    #[argh(option, short = 'm', long = "input-dirs")]
    input_files: Vec<PathBuf>,

    /// 输入目录。
    #[argh(option, short = 'd', long = "input-files")]
    input_dirs: Vec<PathBuf>,

    /// md 文件的输出路径。
    #[argh(option, short = 'M', long = "output-dirs")]
    output_files: Vec<PathBuf>,

    /// 输出目录。
    #[argh(option, short = 'D', long = "output-files")]
    output_dirs: Vec<PathBuf>,

    /// 如果输出文件已存在，是否替换。默认不替换。
    #[argh(switch, short = 'r', long = "replace-file")]
    replace_file: bool,

    /// 在输出文件夹时不存在时，禁止创建输出文件夹。默认总是创建新文件夹。
    #[argh(switch, long = "forbid-dir-creation")]
    forbid_dir_creation: bool,

    /// 配置文件 bilingual.toml 的路径。默认是当前目录下。
    #[argh(option, default = "\"bilingual.toml\".into()")]
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
            API::Niutrans => niutrans(self.key, &mut cf)?,
            _ => unimplemented!(),
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

fn new_filename(f: &PathBuf, to: &str) -> PathBuf {
    let mut stem = f.file_stem().unwrap().to_os_string();
    stem.reserve(6);
    stem.push("-");
    stem.push(to);
    stem.push(".");
    stem.push(f.extension().unwrap());
    f.with_file_name(stem)
}

fn new_dir(f: &PathBuf, to: &str) -> PathBuf {
    let mut stem = f.file_stem().unwrap().to_os_string();
    dbg!(&stem);
    stem.reserve(3);
    stem.push("-");
    stem.push(to);
    f.with_file_name(stem)
}

fn niutrans(key: String, cf: &mut Config) -> Result<()> {
    if cf.niutrans.is_none() {
        cf.niutrans.replace(translation_api_cn::niutrans::User::default());
    }
    if !key.is_empty() {
        cf.niutrans.as_mut().ok_or(anyhow!("覆盖腾讯云 API.key 时出错"))?.key = key;
    }
    Ok(())
}

fn tencent(id: String, key: String, cf: &mut Config) -> Result<()> {
    if cf.tencent.is_none() {
        cf.tencent.replace(translation_api_cn::tencent::User::default());
    }
    if !id.is_empty() {
        cf.tencent.as_mut().ok_or(anyhow!("覆盖腾讯云 API.id 时出错"))?.id = id;
    }
    if !key.is_empty() {
        cf.tencent.as_mut().ok_or(anyhow!("覆盖腾讯云 API.key 时出错"))?.key = key;
    }
    Ok(())
}

fn baidu(id: String, key: String, cf: &mut Config) -> Result<()> {
    if cf.baidu.is_none() {
        cf.baidu.replace(translation_api_cn::baidu::User::default());
    }
    if !id.is_empty() {
        cf.baidu.as_mut().ok_or(anyhow!("覆盖百度翻译 API.id 时出错"))?.appid = id;
    }
    if !key.is_empty() {
        cf.baidu.as_mut().ok_or(anyhow!("覆盖百度翻译 API.key 时出错"))?.key = key;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_filename_dir() {
        assert_eq!(PathBuf::from("/root/test-zh.md"), new_filename(&"/root/test.md".into(), "zh"));
        assert_eq!(PathBuf::from("/root/test-zh/"), new_dir(&"/root/test/".into(), "zh"));
        assert_eq!(PathBuf::from("/root/test-zh"), new_dir(&"/root/test".into(), "zh"));
    }
}
