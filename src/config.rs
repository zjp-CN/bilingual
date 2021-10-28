use anyhow::{Context, Result};
use reqwest::blocking;
use translation_api_cn::baidu::{User as Baidu, API as BAIDU_API};

#[derive(Debug, serde::Deserialize)]
pub struct Config {
    baidu: Baidu,
}

impl Config {
    #[rustfmt::skip]
    pub fn init() -> Result<Self> {
        Ok(
            toml::from_slice(
                    &std::fs::read("bilingual.toml").with_context(
                        || "未找到 `bilingual.toml` 配置文件")?)
                 .with_context(|| "请检查`bilingual.toml` 配置文件的内容")?
          )
    }

    pub fn baidu(&self) -> &Baidu { &self.baidu }
}

pub fn send<T: serde::Serialize + ?Sized>(form: &T) -> Result<blocking::Response> {
    let response = blocking::Client::new().post(BAIDU_API).form(form).send()?;
    debug_assert!(response.error_for_status_ref().is_ok());
    Ok(response)
}

pub fn baidu_en_zh(md: &str, user: &Baidu) -> Result<String> {
    use translation_api_cn::baidu::{Query, Response};
    let md = crate::md::Md::new(md);
    let buf = md.extract();
    let mut query = Query::new(buf.trim(), "en", "zh");
    let output = md.done(serde_json::from_slice::<Response>(&send(&dbg!(query.sign(user)))?
                            .bytes()?)?.dst()?.into_iter());
    Ok(output)
}
