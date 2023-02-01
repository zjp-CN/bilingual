use insta::{assert_debug_snapshot, assert_display_snapshot};
use translation_api_cn::tencent::*;

static PAYLOAD: &str =
    r#"{"Source": "en", "Target": "zh", "ProjectId": 0, "SourceTextList": ["hi", "there"]}"#;

#[test]
fn usage_test() -> Result<()> {
    // sample starts
    use time::OffsetDateTime;
    let datetime = OffsetDateTime::from_unix_timestamp(1636111645)?;
    // let timestamp = datetime.unix_timestamp().to_string();
    assert_display_snapshot!(datetime, @"2021-11-05 11:27:25.0 +00:00:00");
    let mut user = User::default();
    user.id = "0".into();
    user.key = "0".into();
    let _query = Query { from:      "en",
                         to:        "zh",
                         projectid: 0,
                         q:         &["hi", "there"], };
    // sample ends

    // assert_eq!(query.to_hashed2()?,
    //            "132203170c4d03f4b351cacc51a7ceeed78ca571be42688945f74bb0796bb739");
    // 需要增加 在 .signature 部分替换成 SimpleFormatter
    // let mut header = Header { datetime,
    //                           timestamp,
    //                           credential_scope: "".into(),
    //                           authorization: "".into(),
    //                           user: &user,
    //                           query: &query };
    // assert_eq!(header.signature()?,
    //            "5a4474831e97a0b0e37730abf8de690234fb750be49bf5033469f2b626752eb5");
    // assert_eq!(header.authorization()?,
    //            "TC3-HMAC-SHA256 Credential=0/2021-11-05/tmt/tc3_request, \
    //             SignedHeaders=content-type;host, \
    //             Signature=5a4474831e97a0b0e37730abf8de690234fb750be49bf5033469f2b626752eb5");
    Ok(())
}

/// 基于 1636111645 timestamp（2021-11-05）的例子，已完全对照腾讯云 SDK_PYTHON_3.0.519
#[test]
fn detaild_hexhash_in_signature_and_authorization() -> Result<()> {
    assert_eq!(hash256(PAYLOAD.as_bytes()),
               "132203170c4d03f4b351cacc51a7ceeed78ca571be42688945f74bb0796bb739");

    #[rustfmt::skip]
    assert_eq!(hash256(b"POST\n/\n\ncontent-type:application/json\n\
                host:tmt.tencentcloudapi.com\n\ncontent-type;host\n\
                132203170c4d03f4b351cacc51a7ceeed78ca571be42688945f74bb0796bb739"),
                "ef9234630cfbd7baf254265506ed5d0193d278468d367a9c8a809d6300173df1");

    let stringtosign = b"TC3-HMAC-SHA256\n1636111645\n2021-11-05/tmt/tc3_request\
                       \nef9234630cfbd7baf254265506ed5d0193d278468d367a9c8a809d6300173df1";
    let secret_date = hash_2u8(format!("TC3{}", "0").as_bytes(), b"2021-11-05")?;
    assert_eq!(hmac_sha256_string(secret_date.clone()),
               "a907a3013f010c8b934249c4400a8d634f0731d34475ea9875cc791587eadc47");
    let secret_service = hash_hash_u8(secret_date, b"tmt")?;
    assert_eq!(hmac_sha256_string(secret_service.clone()),
               "59632d54ee396568ca49b593a96a61bf6c9c7342ff2eb1853b4c47128faa784a");
    let secret_signing = hash_hash_u8(secret_service, b"tc3_request")?;
    assert_eq!(hmac_sha256_string(secret_signing.clone()),
               "e67867cc6f99a088d74085ca77a6b4c1df2d4d888320836de1b00f7d7fbeb7b5");
    let auth = hmac_sha256_string(hash_hash_u8(secret_signing, stringtosign)?);
    assert_eq!(auth, "5a4474831e97a0b0e37730abf8de690234fb750be49bf5033469f2b626752eb5");

    Ok(())
}

#[test]
fn serde_json_format_test() -> serde_json::Result<()> {
    fn to_string_pretty2<T: ?Sized + serde::Serialize>(value: &T) -> serde_json::Result<String> {
        use serde_json::ser::{PrettyFormatter, Serializer};
        let mut vec = Vec::with_capacity(128);
        // let mut vec = Vec::with_capacity(2 * 1 << 10);
        let mut ser = Serializer::with_formatter(&mut vec, PrettyFormatter::with_indent(b""));
        value.serialize(&mut ser)?;
        dbg!(vec.capacity(), vec.len());
        // serde_json does not emit invalid UTF-8.
        Ok(unsafe { String::from_utf8_unchecked(vec) })
    }
    let query = Query { from:      "en",
                        to:        "zh",
                        projectid: 0,
                        q:         &["hi", "there"], };
    assert_display_snapshot!(serde_json::to_string(&query)?, @r###"{"Source":"en","Target":"zh","ProjectId":0,"SourceTextList":["hi","there"]}"###);
    assert_display_snapshot!(serde_json::to_string_pretty(&query)?, @r###"
    {
      "Source": "en",
      "Target": "zh",
      "ProjectId": 0,
      "SourceTextList": [
        "hi",
        "there"
      ]
    }
    "###);

    // 方法一：删除或替换 `\n`
    let s = to_string_pretty2(&query)?;
    assert_display_snapshot!(s, @r###"
    {
    "Source": "en",
    "Target": "zh",
    "ProjectId": 0,
    "SourceTextList": [
    "hi",
    "there"
    ]
    }
    "###);
    assert_debug_snapshot!(s, @r###""{\n\"Source\": \"en\",\n\"Target\": \"zh\",\n\"ProjectId\": 0,\n\"SourceTextList\": [\n\"hi\",\n\"there\"\n]\n}""###);
    let ss = s.replace(",\n", ", ")
              .replace("[\n", "[")
              .replace("\n]", "]")
              .replace("{\n", "{")
              .replace("\n}", "}");
    assert_eq!(ss, PAYLOAD);
    let ss = s.replace(",\n", ", ").replace('\n', "");
    assert_eq!(ss, PAYLOAD);

    // 方法二：重新实现 `serde_json::ser::Formatter`
    // let s = query.to_json_string2().unwrap();
    // assert_eq!(s, PAYLOAD);
    // assert_display_snapshot!(PAYLOAD, @r###"{"Source": "en", "Target": "zh", "ProjectId": 0,
    // "SourceTextList": ["hi", "there"]}"###);
    //
    Ok(())
}

#[cfg(test)]
mod hash_tests {
    use super::*;
    use hmac::{Mac, NewMac};

    // 测试多条消息
    #[test]
    fn hmac_sha256() {
        let mut mac = HmacSha256::new_from_slice(b"my secret and secure key").unwrap();
        mac.update(b"input message");
        mac.update(b" input message");
        let result = mac.finalize();
        let code_bytes = result.into_bytes();
        let mut mac = HmacSha256::new_from_slice(b"my secret and secure key").unwrap();
        mac.update(b"input message input message");
        mac.verify(&code_bytes).unwrap();
    }

    // 对比 python 的 hmac-sha256 结果
    // ```python
    // import hashlib, hmac
    // hmac.new(b"0", b"1", hashlib.sha256).hexdigest()
    // hmac.new(hmac.new(b"0", b"1", hashlib.sha256).digest(), None, hashlib.sha256).hexdigest()
    // hmac.new(hmac.new(b"0", b"1", hashlib.sha256).digest(), b"2", hashlib.sha256).hexdigest()
    // ```
    #[test]
    fn hmac_sha256_vs_python() {
        let mut mac = HmacSha256::new_from_slice(b"0").unwrap();
        mac.update(b"1");

        let result = mac.finalize().into_bytes();
        assert_eq!("0b0065830a5c8d8f2c4997f5468610d6abc5533e49eac939426cf8158035ec3f",
                   format!("{result:x}"));

        let result2 =
            HmacSha256::new_from_slice(result.as_slice()).unwrap().finalize().into_bytes();
        assert_eq!("922394236a962ebc90466942033cf117e347be148f899255de62b1ff4eab21b2",
                   format!("{result2:x}"));

        let mut mac = HmacSha256::new_from_slice(result.as_slice()).unwrap();
        mac.update(b"2");
        assert_eq!("905dc60bced243ad461a220a2876ee6dc04c26e93b74a33c6a09c2212399fc0e",
                   format!("{:x}", mac.finalize().into_bytes()));
    }

    // python: [hashlib.sha256](https://docs.python.org/3/library/hashlib.html)
    // 031edd7d41651593c5fe5c006fa5752b37fddff7bc4e843aa6af0c950f4b9406
    #[test]
    fn sha256_test() {
        let data = r#"{"Source": "en", "Target": "zh", "ProjectId": 0, "SourceTextList": ["hi", "there"]}"#;
        assert_eq!(hash256(data.as_bytes()),
                   "132203170c4d03f4b351cacc51a7ceeed78ca571be42688945f74bb0796bb739");
        assert_display_snapshot!(hash256_string(data.as_bytes()).unwrap(),
    @"3436b7117ac9800bd6c9f8e2834c58ede79dfc7b9dd9064e5b98929af6dd30df"); // 不需要的
    }
}
