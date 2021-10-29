use anyhow::{Context, Result};
use reqwest::blocking;
use std::path::PathBuf;
use translation_api_cn::baidu::{User as Baidu, API as BAIDU_API};

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
}

impl Default for API {
    fn default() -> Self { Self::All }
}

#[derive(Debug, Default)]
pub struct Src {
    pub from:  String,
    pub to:    String,
    pub query: String,
    pub files: Vec<PathBuf>,
    pub dirs:  Vec<PathBuf>,
}

impl Config {
    #[rustfmt::skip]
    pub fn init(path: impl AsRef<std::path::Path>) -> Result<Self> {
        Ok(
            toml::from_slice(&std::fs::read(path)
                             .with_context(|| "未找到 `bilingual.toml` 配置文件")?)
                 .with_context(|| "请检查`bilingual.toml` 配置文件的内容")?
          )
    }

    pub fn user(&self) -> &Baidu {
        match self.api {
            API::Baidu => &self.baidu,
            _ => &self.baidu,
        }
    }

    pub fn do_single_file(&mut self) -> Option<String> {
        let file = self.src.files.pop()?;
        let md = std::fs::read_to_string(file).ok()?;
        translate(&md, &self.src.from, &self.src.to, &self.user()).ok()
    }

    pub fn do_query(&mut self) -> Option<String> {
        translate(&self.src.query, &self.src.from, &self.src.to, &self.user()).ok()
    }
}

pub fn send<T: serde::Serialize + ?Sized>(form: &T) -> Result<blocking::Response> {
    let response = blocking::Client::new().post(BAIDU_API).form(form).send()?;
    debug_assert!(response.error_for_status_ref().is_ok());
    Ok(response)
}

pub fn translate(md: &str, from: &str, to: &str, user: &Baidu) -> Result<String> {
    use translation_api_cn::baidu::{Query, Response};
    let md = crate::md::Md::new(md);
    let buf = md.extract();
    let mut query = Query::new(buf.trim(), from, to);
    let output = md.done(serde_json::from_slice::<Response>(&send(&dbg!(query.sign(user)))?
                            .bytes()?)?.dst()?.into_iter());
    Ok(output)
}
