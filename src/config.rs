use crate::md::Md;
use anyhow::{Context, Result};
use reqwest::blocking::{self, Client};
use serde_json::from_slice;
use std::path::{Path, PathBuf};
use translation_api_cn::{
    baidu::User as Baidu, niutrans::User as Niutrans, tencent::User as Tencent,
};

#[derive(Debug, Default, serde::Deserialize)]
pub struct Config {
    #[serde(skip_deserializing)]
    pub src:      Src,
    #[serde(skip_deserializing)]
    pub api:      API,
    pub baidu:    Option<Baidu>,
    pub tencent:  Option<Tencent>,
    pub niutrans: Option<Niutrans>,
}

#[derive(Debug)]
pub enum API {
    None,
    Baidu,
    Tencent,
    Niutrans,
}

impl Default for API {
    fn default() -> Self { Self::None }
}

impl std::str::FromStr for API {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.as_bytes() {
            b"baidu" => Ok(API::Baidu),
            b"tencent" => Ok(API::Tencent),
            b"niutrans" => Ok(API::Niutrans),
            _ => anyhow::bail!("请输入以下 API 之一: baidu | tencent | niutrans"),
        }
    }
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
    pub fn init(path: impl AsRef<std::path::Path>) -> Result<Self> {
        if let Ok(f) = std::fs::read(path) {
            toml::from_slice(&f).with_context(|| "请检查 `bilingual.toml` 配置文件的内容")
        } else {
            Ok(Self::default())
        }
    }

    /// 按照 [`files`][`Src::file`] -> [`dirs`][`Src::dirs`] -> [`query`][`Src::query`] 的
    /// 顺序查询。
    pub fn do_single_query(&mut self) -> Option<String> {
        let src = self.src.next()?;
        let md = Md::new(&src);
        match self.api {
            API::Baidu => self.do_single_query_baidu(md),
            API::Tencent => self.do_single_query_tencent(md),
            API::Niutrans => self.do_single_query_niutrans(md),
            _ => unimplemented!(),
        }
    }

    pub fn do_single_query_baidu(&self, md: Md) -> Option<String> {
        self.baidu
            .as_ref()
            .or_else(|| {
                eprintln!("请设置百度翻译 API 帐号的 id 和 key");
                None
            })
            .map(|b| via_baidu(md, &self.src.from, &self.src.to, b).or_else(print_err).ok())
            .flatten()
    }

    pub fn do_single_query_tencent(&self, md: Md) -> Option<String> {
        self.tencent
            .as_ref()
            .or_else(|| {
                eprintln!("请设置腾讯云 API 帐号的 id 和 key");
                None
            })
            .map(|b| via_tencent(md, &self.src.from, &self.src.to, b).or_else(print_err).ok())
            .flatten()
    }

    pub fn do_single_query_niutrans(&self, md: Md) -> Option<String> {
        self.niutrans
            .as_ref()
            .or_else(|| {
                eprintln!("请设置小牛翻译 API 帐号的 key");
                None
            })
            .map(|b| via_niutrans(md, &self.src.from, &self.src.to, b).or_else(print_err).ok())
            .flatten()
    }
}

fn print_err(e: anyhow::Error) -> Result<String, ()> { Err(println!("{}", e)) }

/// 以 post + 表单方式发送
fn send<T: serde::Serialize + ?Sized>(url: &str, form: &T) -> Result<blocking::Response> {
    let response = Client::new().post(url).form(form).send()?;
    debug_assert!(response.error_for_status_ref().is_ok());
    Ok(response)
}

pub fn via_baidu(mut md: Md, from: &str, to: &str, user: &Baidu) -> Result<String> {
    use translation_api_cn::baidu::{Query, Response, URL};
    let buf = md.extract();
    let mut query = Query::new(buf.trim(), from, to);
    let bytes = send(URL, &dbg!(query.sign(user)))?.bytes()?;
    let response = from_slice::<Response>(&bytes)?;
    let output = md.done(response.dst()?);
    Ok(output)
}

pub fn via_niutrans(mut md: Md, from: &str, to: &str, user: &Niutrans) -> Result<String> {
    use translation_api_cn::niutrans::{Query, Response, URL};
    let buf = md.extract();
    let query = Query::new(buf.trim(), from, to);
    let bytes = send(URL, &query.form(user))?.bytes()?;
    let response = from_slice::<Response>(&bytes)?;
    let output = md.done(response.dst()?);
    Ok(output)
}

pub fn via_tencent(mut md: Md, from: &str, to: &str, user: &Tencent) -> Result<String> {
    use translation_api_cn::tencent::{Header, Query, Response, URL};
    #[rustfmt::skip]
    fn send2(header: &mut Header) -> Result<blocking::Response> {
        header.authorization()?; // 更改 query 或者 user 时必须重新生成验证信息
        let map = {
            use reqwest::header::{HeaderName, HeaderValue};
            use std::str::FromStr;
            header.header()
                  .into_iter()
                  .map(|(k, v)| match (HeaderName::from_str(k), HeaderValue::from_str(v)) {
                      (Ok(key), Ok(value)) => Some((key, value)),
                      _ => None,
                  })
                  .flatten() // 遇到 Err 时，把 Ok 的部分 collect
                  .collect()
        };
        Client::new().post(URL).headers(map).json(header.query).send().map_err(|e| e.into())
    }

    let buf = md.extract();
    let q: Vec<&str> = buf.trim().split("\n").collect();
    let query = Query::new(&q, from, to, user.projectid);
    let mut header = Header::new(user, &query);
    let bytes = send2(&mut header)?.bytes()?;
    // dbg!(&buf, &query, &header, &bytes);
    let response = from_slice::<Response>(dbg!(&bytes))?;
    let output = md.done(response.dst()?);
    Ok(output)
}
