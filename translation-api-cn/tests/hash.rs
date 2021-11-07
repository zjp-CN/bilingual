use insta::{assert_debug_snapshot, assert_display_snapshot};

#[test]
fn hmac_sha256() {
    use hmac::{Hmac, Mac, NewMac};
    use sha2::Sha256;

    // Create alias for HMAC-SHA256
    type HmacSha256 = Hmac<Sha256>;

    // Create HMAC-SHA256 instance which implements `Mac` trait
    let mut mac = HmacSha256::new_from_slice(b"my secret and secure key").expect("HMAC can take \
                                                                                  key of any size");
    mac.update(b"input message");
    mac.update(b" input message");

    // `result` has type `Output` which is a thin wrapper around array of
    // bytes for providing constant time equality check
    let result = mac.finalize();
    // To get underlying array use `into_bytes` method, but be careful, since
    // incorrect use of the code value may permit timing attacks which defeat
    // the security provided by the `Output`
    let code_bytes = result.into_bytes();

    let mut mac = HmacSha256::new_from_slice(b"my secret and secure key").expect("HMAC can take \
                                                                                  key of any size");
    mac.update(b"input message input message");

    // `verify` will return `Ok(())` if code is correct, `Err(MacError)` otherwise
    mac.verify(&code_bytes).unwrap();
}

#[test]
fn tencent_test() {
    use time::OffsetDateTime;
    use translation_api_cn::tencent::{HeaderJson, Query, User};
    let datetime = OffsetDateTime::from_unix_timestamp(1636111645).unwrap();
    let timestamp = datetime.unix_timestamp().to_string();
    assert_display_snapshot!(datetime, @"2021-11-05 11:27:25.0 +00:00:00");
    let mut user = User::default();
    user.id = "0".into();
    user.key = "0".into();
    let query = Query { from:      "en",
                        to:        "zh",
                        projectid: 0,
                        q:         &["hi", "there"], };
    assert_display_snapshot!(query.to_json().unwrap(), @r###"{"Source":"en","Target":"zh","projectid":0,"SourceTextList":["hi","there"]}"###);
    assert_display_snapshot!(query.to_json_pretty().unwrap(), @r###"
    {
      "Source": "en",
      "Target": "zh",
      "projectid": 0,
      "SourceTextList": [
        "hi",
        "there"
      ]
    }
    "###);
    assert_debug_snapshot!(to_string_pretty2(&query).unwrap(), @r###""{\n\"Source\": \"en\",\n\"Target\": \"zh\",\n\"projectid\": 0,\n\"SourceTextList\": [\n\"hi\",\n\"there\"\n]\n}""###);
    let s = to_string_pretty2(&query).unwrap()
                                     .replace(",\n", ", ")
                                     .replace("{\n", "{")
                                     .replace("\n}", "}");
    assert_display_snapshot!(s, @r###"
    {"Source": "en", "Target": "zh", "projectid": 0, "SourceTextList": [
    "hi", "there"
    ]}
    "###);
    assert_display_snapshot!(query.to_hashed().unwrap(), @"7f094fbfcf0acb1b19713860d2124257efa63987d7a4323979c8e622a6449c66");
    let mut header = HeaderJson { datetime,
                                  timestamp,
                                  credential_scope: "".into(),
                                  authorization: "".into(),
                                  user: &user,
                                  query: &query };
    // 23560c1a452d368647768283d3b03db2f59a0a2f2a12c72b1045eefa3b304d3f
    assert_display_snapshot!(header.signature().unwrap(), @"196745d1ec3873fdbcb7c05a766b91a464db7c8e645a979df0b81c840aca3444");
    // TC3-HMAC-SHA256 Credential=0/2021-11-05/tmt/tc3_request,SignedHeaders=content-type;host,
    // Signature=23560c1a452d368647768283d3b03db2f59a0a2f2a12c72b1045eefa3b304d3f
    assert_display_snapshot!(header.authorization().unwrap(),
    @"TC3-HMAC-SHA256 Credential=0/2021-11-05/tmt/tc3_request,SignedHeaders=content-type;host,Signature=196745d1ec3873fdbcb7c05a766b91a464db7c8e645a979df0b81c840aca3444");
}

#[test]
fn compare_test() {
    use hmac::{Mac, NewMac};
    use translation_api_cn::tencent::{hash256_string, HmacSha256};
    let test = r#"{"Limit": 1, "Filters": [{"Name": "instance-name", "Values": ["\u{672a}\u{547d}\u{540d}"]}]}"#;
    assert_display_snapshot!(test, @r###"{"Limit": 1, "Filters": [{"Name": "instance-name", "Values": ["\u{672a}\u{547d}\u{540d}"]}]}"###);
    assert_display_snapshot!(hash256_string(test.as_bytes()).unwrap(),
    @"f1de55c15235a9d71fcc6859825b76202b6c9018df3f9c6f3d3ba2eca2e6d1b9");

    let test = r#"a"#;
    assert_display_snapshot!(test, @"a");
    assert_display_snapshot!(hash256_string(test.as_bytes()).unwrap(),
    @"3acdaa86b3d73e8d18b7019d3f520000531a23db3b6dda7a94ad28db61a9008c");
    assert_display_snapshot!(hash256_string(b"a").unwrap(),
    @"3acdaa86b3d73e8d18b7019d3f520000531a23db3b6dda7a94ad28db61a9008c");
    let mac = HmacSha256::new_from_slice(b"a").unwrap();
    let x = format!("{:x}", mac.finalize().into_bytes());
    assert_display_snapshot!(x,
        @"3acdaa86b3d73e8d18b7019d3f520000531a23db3b6dda7a94ad28db61a9008c");
}

#[test]
fn sha256_test() {
    use translation_api_cn::tencent::{hash256, hash256_string};

    let data = b"Nobody inspects the spammish repetition";

    // python: [hashlib.sha256](https://docs.python.org/3/library/hashlib.html)
    // 031edd7d41651593c5fe5c006fa5752b37fddff7bc4e843aa6af0c950f4b9406
    assert_display_snapshot!(hash256(data),
    @"031edd7d41651593c5fe5c006fa5752b37fddff7bc4e843aa6af0c950f4b9406"); // √
    assert_display_snapshot!(hash256_string(data).unwrap(),
    @"f5d6645a2c7bc9a9f1ffffe0931ae24c4dd37904cdab576bc4147ef1b0441a9f");

    let data =
        r#"{"Source": "en", "Target": "zh", "ProjectId": 0, "SourceTextList": ["hi", "there"]}"#;
    assert_display_snapshot!(hash256(data.as_bytes()),
    @"132203170c4d03f4b351cacc51a7ceeed78ca571be42688945f74bb0796bb739"); // √
    assert_display_snapshot!(hash256_string(data.as_bytes()).unwrap(),
    @"3436b7117ac9800bd6c9f8e2834c58ede79dfc7b9dd9064e5b98929af6dd30df");
}

/// 基于 1636111645 timestamp（2021-11-05）的例子，已完全对照腾讯云 SDK_PYTHON_3.0.519
#[test]
fn detaild_hexhash_in_signature_and_authorization(
    )
    -> translation_api_cn::tencent::MultiErrResult<()>
{
    use translation_api_cn::tencent::{hash256, hash_2u8, hash_hash_u8, hmac_sha256_string};
    let payload =
        r#"{"Source": "en", "Target": "zh", "ProjectId": 0, "SourceTextList": ["hi", "there"]}"#;
    assert_eq!(hash256(payload.as_bytes()),
               "132203170c4d03f4b351cacc51a7ceeed78ca571be42688945f74bb0796bb739"); // √

    #[rustfmt::skip]
    assert_eq!(hash256(b"POST\n/\n\ncontent-type:application/json\n\
                host:tmt.tencentcloudapi.com\n\ncontent-type;host\n\
                132203170c4d03f4b351cacc51a7ceeed78ca571be42688945f74bb0796bb739"),
                "ef9234630cfbd7baf254265506ed5d0193d278468d367a9c8a809d6300173df1"); // √

    let stringtosign = b"TC3-HMAC-SHA256\n1636111645\n2021-11-05/tmt/tc3_request\
                       \nef9234630cfbd7baf254265506ed5d0193d278468d367a9c8a809d6300173df1";
    let secret_date = hash_2u8(format!("TC3{}", "0").as_bytes(), b"2021-11-05")?;
    assert_eq!(hmac_sha256_string(secret_date.clone()),
               "a907a3013f010c8b934249c4400a8d634f0731d34475ea9875cc791587eadc47"); // √
    let secret_service = hash_hash_u8(secret_date, b"tmt")?;
    assert_eq!(hmac_sha256_string(secret_service.clone()),
               "59632d54ee396568ca49b593a96a61bf6c9c7342ff2eb1853b4c47128faa784a"); // √
    let secret_signing = hash_hash_u8(secret_service, b"tc3_request")?;
    assert_eq!(hmac_sha256_string(secret_signing.clone()),
               "e67867cc6f99a088d74085ca77a6b4c1df2d4d888320836de1b00f7d7fbeb7b5"); // √
    let auth = hmac_sha256_string(hash_hash_u8(secret_signing, stringtosign)?);
    assert_eq!(auth, "5a4474831e97a0b0e37730abf8de690234fb750be49bf5033469f2b626752eb5");

    Ok(())
}

#[test]
fn serde_json_format_test() {}

use serde_json::ser::{PrettyFormatter, Serializer};
fn to_string_pretty2<T: ?Sized + serde::Serialize>(value: &T) -> serde_json::Result<String> {
    let mut vec = Vec::with_capacity(128);
    let mut ser = Serializer::with_formatter(&mut vec, PrettyFormatter::with_indent(b""));
    value.serialize(&mut ser)?;
    // serde_json does not emit invalid UTF-8.
    Ok(unsafe { String::from_utf8_unchecked(vec) })
}
