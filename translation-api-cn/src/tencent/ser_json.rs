//! 自定义序列化 json Formatter
//!
//! [`serde_json`] 主要有两种 [`Formatter`][`serde_json::ser::Formatter`]：
//! 1. [`CompactFormatter`][`serde_json::ser::CompactFormatter`]，不含空格或换行的格式：
//!
//!   ```json
//!   {"Source":"en","Target":"zh","ProjectId":0,"SourceTextList":["hi","there"]}`
//!   ```
//! 2. [`PrettyFormatter`][`serde_json::ser::PrettyFormatter`]，美观的空格+换行的格式：
//!
//!   ```JSON
//!   {
//!     "Source": "en",
//!     "Target": "zh",
//!     "ProjectId": 0,
//!     "SourceTextList": [
//!       "hi",
//!       "there"
//!     ]
//!   }
//!   ```
//!
//! 而腾讯云需要的 json 格式不属于上面两种（参考 python `json.dumps` 的样式）。
//! 因此需要实现新的样式：
//! ```json
//! {"Source": "en", "Target": "zh", "ProjectId": 0, "SourceTextList": ["hi", "there"]}
//! ```
use serde::Serialize;
use serde_json::ser::{Formatter, Serializer};
use std::io;

#[derive(Debug, Clone)]
pub struct SimpleFormatter;

#[inline]
fn format_begin<W: ?Sized + io::Write>(writer: &mut W, first: bool) -> io::Result<()> {
    if first {
        Ok(())
    } else {
        writer.write_all(b", ")
    }
}

impl Formatter for SimpleFormatter {
    #[inline]
    fn begin_array_value<W>(&mut self, writer: &mut W, first: bool) -> io::Result<()>
        where W: ?Sized + io::Write {
        format_begin(writer, first)
    }

    #[inline]
    fn begin_object_key<W>(&mut self, writer: &mut W, first: bool) -> io::Result<()>
        where W: ?Sized + io::Write {
        format_begin(writer, first)
    }

    #[inline]
    fn begin_object_value<W>(&mut self, writer: &mut W) -> io::Result<()>
        where W: ?Sized + io::Write {
        writer.write_all(b": ")
    }
}

#[inline]
pub fn to_writer<W, T>(writer: W, value: &T) -> serde_json::Result<()>
    where W: io::Write,
          T: ?Sized + Serialize
{
    let mut ser = Serializer::with_formatter(writer, SimpleFormatter);
    value.serialize(&mut ser)?;
    Ok(())
}

#[inline]
pub fn to_vec<T>(value: &T) -> serde_json::Result<Vec<u8>>
    where T: ?Sized + Serialize {
    let mut writer = Vec::with_capacity(128);
    to_writer(&mut writer, value)?;
    Ok(writer)
}

#[inline]
pub fn to_string<T: ?Sized + serde::Serialize>(value: &T) -> serde_json::Result<String> {
    // serde_json does not emit invalid UTF-8.
    Ok(unsafe { String::from_utf8_unchecked(to_vec(value)?) })
}

#[test]
fn signature_to_string_test() -> super::Result<()> {
    use super::*;
    // sample starts
    let datetime = OffsetDateTime::from_unix_timestamp(1636111645)?;
    let timestamp = datetime.unix_timestamp().to_string();
    let mut user = User::default();
    user.id = "0".into();
    user.key = "0".into();
    let query = Query { from:      "en",
                        to:        "zh",
                        projectid: 0,
                        q:         &["hi", "there"], };
    // sample ends
    let canonical_request = format!("{}\n{}\n{}\n{}\n{}\n{}",
                                    Header::HTTPREQUESTMETHOD,
                                    Header::CANONICALURI,
                                    Header::CANONICALQUERYSTRING,
                                    Header::CANONICALHEADERS,
                                    Header::SIGNEDHEADERS,
                                    query.to_hashed2()?);
    #[rustfmt::skip]
    assert_eq!(canonical_request,
               "POST\n/\n\ncontent-type:application/json\n\
                host:tmt.tencentcloudapi.com\n\ncontent-type;host\n\
                132203170c4d03f4b351cacc51a7ceeed78ca571be42688945f74bb0796bb739");
    let mut header = Header { datetime,
                              timestamp,
                              credential_scope: "".into(),
                              authorization: "".into(),
                              user: &user,
                              query: &query };
    let date = datetime.date();
    header.credential_scope =
        format!("{}/{}/{}", date, header.user.service, Header::CREDENTIALSCOPE);
    assert_eq!(header.credential_scope, "2021-11-05/tmt/tc3_request");
    let stringtosign = format!("{}\n{}\n{}\n{}",
                               Header::ALGORITHM,
                               header.timestamp,
                               header.credential_scope,
                               hash256(canonical_request.as_bytes()));
    #[rustfmt::skip]
    assert_eq!(stringtosign, "TC3-HMAC-SHA256\n1636111645\n2021-11-05/tmt/tc3_request\n\
                              ef9234630cfbd7baf254265506ed5d0193d278468d367a9c8a809d6300173df1");
    let secret_date =
        hash_2u8(format!("TC3{}", header.user.key).as_bytes(), format!("{}", date).as_bytes())?;
    let secret_service = hash_hash_u8(secret_date, header.user.service.as_bytes())?;
    let secret_signing = hash_hash_u8(secret_service, Header::CREDENTIALSCOPE.as_bytes())?;
    let hex = hmac_sha256_string(hash_hash_u8(secret_signing, stringtosign.as_bytes())?);
    assert_eq!(hex, "5a4474831e97a0b0e37730abf8de690234fb750be49bf5033469f2b626752eb5");
    Ok(())
}
