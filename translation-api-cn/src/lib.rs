pub mod baidu;
// trait Translate {
//     /// 登录验证身份
//     fn auth(&mut self);
//     fn doit(&mut self)-> Result<reqwest::Response>;
// }
//

// use serde::{Deserialize, Serialize};
//
// #[derive(Debug, Deserialize)]
// pub struct Config {
//     baidu:    baidu::User,
//     #[serde(skip_serializing_if = "Option::is_none")]
//     pub dir:  Vec<Dir>, // 共享相同路径的文件
//     pub file: Vec<File>, // 路径不同的文件
// }
