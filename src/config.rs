use crate::md::Md;
use anyhow::{Context, Error, Result};
use reqwest::blocking::{self, Client};
use serde_json::from_slice;
use std::path::{Path, PathBuf};
use translation_api_cn::{
    baidu::User as Baidu, niutrans::User as Niutrans, tencent::User as Tencent, Limit,
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

#[allow(clippy::upper_case_acronyms)]
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
    type Err = Error;

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
    pub from:         String,
    /// 目标语言
    pub to:           String,
    /// 来自输入的命令行参数
    pub query:        String,
    /// 如果输出文件已存在，是否替换。默认不替换。
    pub dir_file:     DirFile,
    /// 未校验 md 后缀的文件
    pub input_files:  Vec<PathBuf>,
    /// 会校验 md 后缀的文件
    pub input_dirs:   Vec<PathBuf>,
    /// 未校验 md 后缀的文件
    pub output_files: Vec<PathBuf>,
    /// 会校验 md 后缀的文件
    pub output_dirs:  Vec<PathBuf>,
}

#[derive(Debug, Default)]
pub struct DirFile {
    dir:          Option<std::fs::DirBuilder>,
    replace_file: bool,
}

impl DirFile {
    pub fn new(replace_file: bool, forbid_dir_creation: bool) -> Self {
        Self { dir: if forbid_dir_creation {
                   None
               } else {
                   let mut d = std::fs::DirBuilder::new();
                   d.recursive(true);
                   Some(d)
               },
               replace_file }
    }

    fn create_dir(&self, d: impl AsRef<Path>) -> Option<()> {
        self.dir
            .as_ref()
            .and_then(|db| db.create(&d).map_err(print_err).ok())
            .or_else(|| {
                if !d.as_ref().exists() {
                    error!("{:?} 文件夹不存在，且不被允许创建。", d.as_ref());
                    None
                } else {
                    Some(())
                }
            })
    }

    #[rustfmt::skip]
    fn create_parent(&self, f: &PathBuf) -> Option<()> {
        self.create_dir(f.parent().or_else(|| { error!("{:?} 无父目录", f); None })?)
    }

    fn read_file(&self, from: PathBuf, into: PathBuf) -> Option<TextItem> {
        Some(if into.exists() && !self.replace_file {
                 // 输出文件已存在，当不被允许覆盖，因此跳过
                 TextItem::Skip { from, into }
             } else {
                 let text = std::fs::read_to_string(&from).map_err(print_err).ok()?;
                 TextItem::Normal { text, from, into }
             })
    }
}

#[rustfmt::skip]
fn filter_md_files(d: impl AsRef<Path>) -> Option<impl Iterator<Item = PathBuf>> {
    Some(std::fs::read_dir(d).ok()?
            .filter_map(|e| e.ok()).map(|f| f.path())
            .filter(|p| p.extension().map(|f| f == "md").unwrap_or(false)))
}

impl Iterator for Src {
    type Item = TextItem;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(from) = self.input_files.pop() {
            let into = self.output_files.pop()?;
            self.dir_file.create_parent(&into)?;
            self.dir_file.read_file(from, into)
        } else if let Some(d) = self.input_dirs.pop() {
            self.input_files = filter_md_files(d)?.collect();
            let d = self.output_dirs.pop()?;
            self.dir_file.create_dir(&d)?;
            self.output_files = self.input_files
                                    .iter()
                                    .map(|f| {
                                        if let Some(fname) = f.file_name() {
                                            Some(d.join(fname))
                                        } else {
                                            error!("路径 {:?} 无法获取文件名", f);
                                            None
                                        }
                                    })
                                    .collect::<Option<_>>()?;
            let from = self.input_files.pop()?;
            let into = self.output_files.pop()?;
            self.dir_file.read_file(from, into)
        } else if !self.query.is_empty() {
            Some(TextItem::Stdout(std::mem::take(&mut self.query)))
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let n = if self.query.is_empty() { 0 } else { 1 };
        (n,
         Some(n
              + self.input_files.len()
              + self.input_dirs.iter().filter_map(filter_md_files).count()))
    }
}

#[derive(Debug)]
pub enum TextItem {
    Normal {
        text: String,
        from: PathBuf,
        into: PathBuf,
    },
    Skip {
        from: PathBuf,
        into: PathBuf,
    },
    Stdout(String),
}

impl std::fmt::Display for TextItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            TextItem::Normal { text, .. } => text,
            TextItem::Skip { .. } => "",
            TextItem::Stdout(s) => s,
        })
    }
}

impl Config {
    pub fn init(path: impl AsRef<std::path::Path>) -> Result<Self> {
        let path = path.as_ref();
        if let Ok(ref f) = std::fs::read_to_string(path) {
            toml::from_str(f).with_context(|| "请检查 `bilingual.toml` 配置文件的内容")
        } else {
            debug!("{path:?} 配置文件不存在");
            Ok(Self::default())
        }
    }

    /// 按照 [`files`][`Src::file`] -> [`dirs`][`Src::dirs`] -> [`query`][`Src::query`] 的
    /// 顺序查询。
    pub fn do_single_query(&mut self) -> Option<TextItem> {
        use TextItem::*;
        let text_item = self.src.next()?;
        let doit = |text: &str| {
            let md = Md::new(text);
            match self.api {
                API::Baidu => self.do_single_query_baidu(md),
                API::Tencent => self.do_single_query_tencent(md),
                API::Niutrans => self.do_single_query_niutrans(md),
                _ => unimplemented!(),
            }
        };
        Some(match text_item {
            Normal { ref text, from, into } => Normal { text: doit(text)?, from, into },
            Stdout(ref s) => Stdout(doit(s)?),
            x => x,
        })
    }

    pub fn do_single_query_write(&mut self) -> Option<String> {
        match self.do_single_query()? {
            TextItem::Normal { text, from, into } => {
                std::fs::write(&into, text.as_bytes()).map_err(print_err).ok()?;
                info!("翻译成功：{:?} => {:?}", from, into);
                Some(text)
            }
            TextItem::Stdout(text) => {
                info!("命令行翻译内容：\n{:?}", text);
                println!("{text}");
                Some(text)
            }
            TextItem::Skip { from, into } => {
                error!("翻译未开始：\n * {:?} 被跳过，因为 {:?} \
                        已存在，而且不被允许覆盖。\n请指明 `-r` 参数或者手动删除已存在的文件",
                       from, into);
                None
            }
        }
    }

    pub fn do_single_query_baidu(&self, md: Md) -> Option<String> {
        self.baidu
            .as_ref()
            .or_else(|| {
                error!("请设置百度翻译 API 帐号的 id 和 key");
                None
            })
            .and_then(|b| {
                if b.limit.limit() == 0 {
                    via_baidu(md, &self.src.from, &self.src.to, b)
                } else {
                    via_baidu_batch(md, &self.src.from, &self.src.to, b)
                }.map_err(|e| {
                     print_err(e);
                 })
                 .ok()
            })
    }

    pub fn do_single_query_tencent(&self, md: Md) -> Option<String> {
        self.tencent
            .as_ref()
            .or_else(|| {
                error!("请设置腾讯云 API 帐号的 id 和 key");
                None
            })
            .and_then(|t| {
                if t.limit.limit() == 0 {
                    via_tencent(md, &self.src.from, &self.src.to, t)
                } else {
                    via_tencent_batch(md, &self.src.from, &self.src.to, t)
                }.map_err(|e| {
                     print_err(e);
                 })
                 .ok()
            })
    }

    pub fn do_single_query_niutrans(&self, md: Md) -> Option<String> {
        self.niutrans
            .as_ref()
            .or_else(|| {
                error!("请设置小牛翻译 API 帐号的 key");
                None
            })
            .and_then(|n| {
                if n.limit.limit() == 0 {
                    via_niutrans(md, &self.src.from, &self.src.to, n)
                } else {
                    via_niutrans_batch(md, &self.src.from, &self.src.to, n)
                }.map_err(|e| {
                     print_err(e);
                 })
                 .ok()
            })
    }
}

fn print_err<E: Into<Error> + std::fmt::Display>(e: E) { error!("{}", e) }

/// 以 post + 表单方式发送
fn send<T: serde::Serialize + ?Sized>(url: &str, form: &T) -> Result<blocking::Response> {
    let response = Client::new().post(url).form(form).send()?;
    debug_assert!(response.error_for_status_ref().is_ok());
    Ok(response)
}

trait Batch {
    fn limit_field(&self) -> &Limit;
    fn limit<'t>(&self, m: &'t mut Md) -> Box<dyn Iterator<Item = &'t str> + 't> {
        match *self.limit_field() {
            Limit::Byte(l) => Box::new(m.bytes_paragraph(l)),
            Limit::Char(l) => Box::new(m.chars_paragraph(l)),
        }
    }
}

macro_rules! batch {
    ($($i:ident),+) => {
        $(impl Batch for $i { fn limit_field(&self) -> &Limit { &self.limit } })+
    };
}

batch!(Baidu, Tencent, Niutrans);

pub fn via_baidu_batch(mut md: Md, from: &str, to: &str, user: &Baidu) -> Result<String> {
    use translation_api_cn::baidu::{Query, Response, URL};
    let mut res = Vec::new();
    let f = |q: &str| {
        let mut query = Query::new(q.trim(), from, to);
        let bytes = send(URL, &{
                        let sign = query.sign(user);
                        debug!("sign = {:#?}", sign);
                        sign
                    })?.bytes()?;
        debug!("\nq = {:?}\nquery = {:#?}\nbytes = {:?}", q, query, bytes);
        res.push(bytes);
        Ok::<(), Error>(())
    };
    user.limit(&mut md).try_for_each(f)?;
    let iter: Vec<Response> = res.iter().map(|bytes| from_slice(bytes)).collect::<Result<_, _>>()?;
    debug!("iter = {:#?}", iter);
    let output = md.done(iter.iter().flat_map(|r| r.dst().map_err(print_err).unwrap()));
    Ok(output)
}

pub fn via_baidu(mut md: Md, from: &str, to: &str, user: &Baidu) -> Result<String> {
    use translation_api_cn::baidu::{Query, Response, URL};
    let q = md.extract();
    let mut query = Query::new(q.trim(), from, to);
    let bytes = send(URL, &{
                    let sign = query.sign(user);
                    debug!("sign = {:#?}", sign);
                    sign
                })?.bytes()?;
    let response = from_slice::<Response>(&bytes)?;
    debug!("\nq = {:?}\nquery = {:#?}\nbytes = {:?}\nresponse = {:#?}", q, query, bytes, response);
    let output = md.done(response.dst()?);
    Ok(output)
}

pub fn via_niutrans_batch(mut md: Md, from: &str, to: &str, user: &Niutrans) -> Result<String> {
    use translation_api_cn::niutrans::{Query, Response, URL};
    let mut res = Vec::new();
    let f = |q: &str| {
        let query = Query::new(q.trim(), from, to);
        let bytes = send(URL, &{
                        let form = query.form(user);
                        debug!("form = {:#?}", form);
                        form
                    })?.bytes()?;
        debug!("\nq = {:?}\nquery = {:#?}\nbytes = {:?}", q, query, bytes);
        res.push(bytes);
        Ok::<(), Error>(())
    };
    user.limit(&mut md).try_for_each(f)?;
    let iter: Vec<Response> = res.iter().map(|bytes| from_slice(bytes)).collect::<Result<_, _>>()?;
    debug!("iter = {:#?}", iter);
    let output = md.done(iter.iter().flat_map(|r| r.dst().map_err(print_err).unwrap()));
    Ok(output)
}

pub fn via_niutrans(mut md: Md, from: &str, to: &str, user: &Niutrans) -> Result<String> {
    use translation_api_cn::niutrans::{Query, Response, URL};
    let q = md.extract();
    let query = Query::new(q.trim(), from, to);
    let bytes = send(URL, &{
                    let form = query.form(user);
                    debug!("form = {:#?}", form);
                    form
                })?.bytes()?;
    let response = from_slice::<Response>(&bytes)?;
    debug!("\nq = {:?}\nquery = {:#?}\nbytes = {:?}\nresponse = {:#?}", q, query, bytes, response);
    let output = md.done(response.dst()?);
    Ok(output)
}

#[rustfmt::skip]
fn send2(header: &mut translation_api_cn::tencent::Header) -> Result<blocking::Response> {
    header.authorization()?; // 更改 query 或者 user 时必须重新生成验证信息
    let map = {
        use reqwest::header::{HeaderName, HeaderValue};
        use std::str::FromStr;
        header.header()
              .into_iter()
              .filter_map(|(k, v)| match (HeaderName::from_str(k), HeaderValue::from_str(v)) {
                  (Ok(key), Ok(value)) => Some((key, value)),
                  _ => None,
              }) // 遇到 Err 时，把 Ok 的部分 collect
              .collect()
    };
    Client::new().post(translation_api_cn::tencent::URL).headers(map).json(header.query).send().map_err(|e| e.into())
}

pub fn via_tencent_batch(mut md: Md, from: &str, to: &str, user: &Tencent) -> Result<String> {
    use translation_api_cn::tencent::{Header, Query, Response};
    // debug!("raw md = {:#?}", md.extract());
    let mut res = Vec::new();
    let f = |buf: &str| {
        let q: Vec<&str> = buf.trim().split('\n').collect();
        debug!("\nq = {:?}", q);
        let query = Query::new(&q, from, to, user.projectid);
        let mut header = Header::new(user, &query);
        let bytes = send2(&mut header)?.bytes()?;
        debug!("\nq = {:?}\nquery = {:#?}\nheader = {:#?}\nbytes = {:#?}", q, query, header, bytes);
        res.push(bytes);
        Ok::<(), Error>(())
    };
    user.limit(&mut md).try_for_each(f)?;
    let iter: Vec<Response> = res.iter().map(|bytes| from_slice(bytes)).collect::<Result<_, _>>()?;
    debug!("iter = {:#?}", iter);
    let output = md.done(iter.iter().flat_map(|r| r.dst().map_err(print_err).unwrap()));
    Ok(output)
}

pub fn via_tencent(mut md: Md, from: &str, to: &str, user: &Tencent) -> Result<String> {
    use translation_api_cn::tencent::{Header, Query, Response};

    let buf = md.extract();
    let q: Vec<&str> = buf.trim().split('\n').collect();
    let query = Query::new(&q, from, to, user.projectid);
    let mut header = Header::new(user, &query);
    let bytes = send2(&mut header)?.bytes()?;
    let response = from_slice::<Response>(&bytes)?;
    debug!("\nq = {:?}\nquery = {:#?}\nheader = {:#?}\nbytes = {:?}\nresponse = {:#?}",
           q, query, header, bytes, response);
    let output = md.done(response.dst()?);
    Ok(output)
}
