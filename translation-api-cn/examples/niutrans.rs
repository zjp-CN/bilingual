use anyhow::{anyhow, Result as AnyResult};
use reqwest::blocking::{self, Client};
use translation_api_cn::niutrans::*;

fn main() -> AnyResult<()> {
    let user = {
        let mut arg = std::env::args().skip(1);
        User { key: arg.next().ok_or_else(|| anyhow!("请输出 Key"))?,
               ..User::default() }
    };
    let query = Query { from: "en",
                        to:   "zh",
                        q:    "hi there", };
    // let query = Query { from: "en",
    //                     to:   "zh",
    //                     q:    "hi\nthere", };
    // let query = Query { from: "zh",
    //                     to:   "en",
    //                     q:    "你好，世界！", };
    // let query = Query { from: "zh",
    //                     to:   "en",
    //                     q:    "你好，\n世界！", };
    let mut form = Form::new(&user, &query);
    let bytes = send(&mut form)?.bytes()?;
    let json: Response = serde_json::from_slice(&bytes)?;
    dbg!(&json, json.dst()?.collect::<Vec<_>>(), json.is_borrowed());
    dbg!(json.dst_owned()?);
    Ok(())
}

#[rustfmt::skip]
pub fn send(form: &Form) -> AnyResult<blocking::Response> {
    Client::new().post(URL).form(form).send().map_err(|e| e.into())
}
