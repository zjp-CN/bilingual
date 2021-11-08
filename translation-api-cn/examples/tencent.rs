use anyhow::{anyhow, Result as AnyResult};
use translation_api_cn::tencent::{Header, Query, User};

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
    let mut header = Header::new(&user, &query);
    header.authorization()?;
    println!("{}", send(&header)?);
    Ok(())
}

pub fn send(header: &Header) -> AnyResult<String> {
    use reqwest::blocking::Client;
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
    dbg!(&map);
    Ok(Client::new().post(Header::URL).headers(map).json(header.query).send()?.text()?)
}
