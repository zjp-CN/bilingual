use anyhow::{Context, Result};
use reqwest::blocking;
use std::path::{Path, PathBuf};
use translation_api_cn::baidu::User as Baidu;

#[derive(Debug, serde::Deserialize)]
pub struct Config {
    #[serde(skip_deserializing)]
    pub src:   Src,
    #[serde(skip_deserializing)]
    pub api:   API,
    pub baidu: Baidu,
}

#[derive(Debug)]
pub enum API {
    All,
    Baidu,
    Tencent,
}

impl Default for API {
    fn default() -> Self { Self::All }
}

#[derive(Debug, Default)]
pub struct Src {
    /// 原语言
    pub from:  String,
    /// 目标语言
    pub to:    String,
    /// 来自输入的命令行参数
    pub query: String,
    /// 未校验 md 后缀的文件
    pub files: Vec<PathBuf>,
    /// 会校验 md 后缀的文件
    pub dirs:  Vec<PathBuf>,
}

#[rustfmt::skip]
fn filter_md_files(d: impl AsRef<Path>) -> Option<impl Iterator<Item = PathBuf>> {
    Some(std::fs::read_dir(d).ok()?
            .filter_map(|e| e.ok()).map(|f| f.path())
            .filter(|p| p.extension().map(|f| f == "md").unwrap_or(false)))
}

impl Iterator for Src {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(p) = self.files.pop() {
            std::fs::read_to_string(p).ok()
        } else if let Some(d) = self.dirs.pop() {
            self.files = filter_md_files(d)?.collect();
            std::fs::read_to_string(self.files.pop()?).ok()
        } else if !self.query.is_empty() {
            Some(std::mem::take(&mut self.query))
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let n = if self.query.is_empty() { 0 } else { 1 };
        (n, Some(n + self.files.len() + self.dirs.iter().map(filter_md_files).count()))
    }
}

impl Config {
    #[rustfmt::skip]
    pub fn init(path: impl AsRef<std::path::Path>) -> Result<Self> {
        Ok(
            toml::from_slice(&std::fs::read(path)
                             .with_context(|| "未找到 `bilingual.toml` 配置文件")?)
                 .with_context(|| "请检查 `bilingual.toml` 配置文件的内容")?
          )
    }

    /// 按照 [`files`][`Src::file`] / [`dirs`][`Src::dirs`] / [`query`][`Src::query`]
    /// 顺序查询。
    pub fn do_single_query(&mut self) -> Option<String> {
        let md = self.src.next()?;
        match self.api {
            API::Baidu => translate_via_baidu(&md, &self.src.from, &self.src.to, &self.baidu).ok(),
            _ => unimplemented!(),
        }
    }
}

pub fn translate_via_baidu(md: &str, from: &str, to: &str, user: &Baidu) -> Result<String> {
    use translation_api_cn::baidu::{Query, Response, API};
    pub fn send<T: serde::Serialize + ?Sized>(form: &T) -> Result<blocking::Response> {
        let response = blocking::Client::new().post(API).form(form).send()?;
        debug_assert!(response.error_for_status_ref().is_ok());
        Ok(response)
    }

    let md = crate::md::Md::new(md);
    let buf = md.extract();
    let mut query = Query::new(buf.trim(), from, to);
    let output = md.done(
                    serde_json::from_slice::<Response>(&send(&dbg!(query.sign(user)))?.bytes()?)?
                        .dst()?.into_iter()
                );
    Ok(output)
}
