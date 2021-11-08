use anyhow::{anyhow, Result as AnyResult};
use reqwest::blocking::{self, Client};
use translation_api_cn::tencent::{Header, Query, Response, User};

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
    // let query = Query { from:      "zh",
    //                     to:        "en",
    //                     projectid: 0,
    //                     q:         &["你好", "世界"], };
    let mut header = Header::new(&user, &query);
    let bytes = send(&mut header)?.bytes()?;
    let json: Response = serde_json::from_slice(&bytes)?;
    dbg!(&json, json.dst()?, json.is_borrowed());
    dbg!(json.dst_owned()?);
    Ok(())
}

pub fn send(header: &mut Header) -> AnyResult<blocking::Response> {
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
    // dbg!(&map);
    // Ok(Client::new().post(Header::URL).headers(map).json(header.query).send()?.text()?)
    let res = Client::new().post(Header::URL).headers(map).json(header.query).send()?;
    Ok(res)
}
