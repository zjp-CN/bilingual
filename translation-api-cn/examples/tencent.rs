use anyhow::{anyhow, Result as AnyResult};
use translation_api_cn::tencent::{HeaderJson, Query, User};

fn main() -> AnyResult<()> {
    let mut user = User::default();
    {
        let mut arg = std::env::args().skip(1);
        user.id = arg.next().ok_or(anyhow!("请输出 ID"))?;
        user.key = arg.next().ok_or(anyhow!("请输出 Key"))?;
    }
    let query = Query { from:      "en",
                        to:        "zh",
                        projectid: 0,
                        q:         &["hi", "there"], };
    let mut header = HeaderJson::new(&user, &query);
    header.authorization()?;
    println!("{}", send(&header, &query)?);
    Ok(())
}

pub fn send(header: &HeaderJson, query: &Query) -> AnyResult<String> {
    use reqwest::blocking::Client;
    let map = {
        use reqwest::header::{HeaderName, HeaderValue};
        use std::str::FromStr;
        header.header()
              .into_iter()
              .map(|(k, v)| (HeaderName::from_str(k).unwrap(), HeaderValue::from_str(v).unwrap()))
              .collect()
    };
    Ok(Client::new().post(HeaderJson::URL).headers(map).json(&query).send()?.text()?)
}
