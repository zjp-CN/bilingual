use insta::assert_display_snapshot;

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
    assert_display_snapshot!(query.to_hashed().unwrap(), @"7f094fbfcf0acb1b19713860d2124257efa63987d7a4323979c8e622a6449c66");
    let mut header = HeaderJson { datetime,
                                  timestamp,
                                  credential_scope: "".into(),
                                  authorization: "".into(),
                                  user: &user,
                                  query: &query };
    // 23560c1a452d368647768283d3b03db2f59a0a2f2a12c72b1045eefa3b304d3f
    assert_display_snapshot!(header.signature().unwrap(), @"cb70ba36721c49fce7a420c4294191a2dcb6b2684d242e1be4f7d6c664245832");
    // TC3-HMAC-SHA256 Credential=0/2021-11-05/tmt/tc3_request,SignedHeaders=content-type;host,
    // Signature=23560c1a452d368647768283d3b03db2f59a0a2f2a12c72b1045eefa3b304d3f
    assert_display_snapshot!(header.authorization().unwrap(),
    @"TC3-HMAC-SHA256 Credential=0/2021-11-05/tmt/tc3_request,SignedHeaders=content-type;host,Signature=cb70ba36721c49fce7a420c4294191a2dcb6b2684d242e1be4f7d6c664245832");
}

#[test]
fn compare_test() {
    use hmac::{Mac, NewMac};
    use translation_api_cn::tencent::{hash_u8_to_string, HmacSha256};
    let test = r#"{"Limit": 1, "Filters": [{"Name": "instance-name", "Values": ["\u{672a}\u{547d}\u{540d}"]}]}"#;
    assert_display_snapshot!(test, @r###"{"Limit": 1, "Filters": [{"Name": "instance-name", "Values": ["\u{672a}\u{547d}\u{540d}"]}]}"###);
    assert_display_snapshot!(hash_u8_to_string(test.as_bytes()).unwrap(),
        @"f1de55c15235a9d71fcc6859825b76202b6c9018df3f9c6f3d3ba2eca2e6d1b9");

    let test = r#"a"#;
    assert_display_snapshot!(test, @"a");
    assert_display_snapshot!(hash_u8_to_string(test.as_bytes()).unwrap(),
        @"3acdaa86b3d73e8d18b7019d3f520000531a23db3b6dda7a94ad28db61a9008c");
    assert_display_snapshot!(hash_u8_to_string(b"a").unwrap(),
        @"3acdaa86b3d73e8d18b7019d3f520000531a23db3b6dda7a94ad28db61a9008c");
    let mac = HmacSha256::new_from_slice(b"a").unwrap();
    let x = format!("{:x}", mac.finalize().into_bytes());
    assert_display_snapshot!(x, 
        @"3acdaa86b3d73e8d18b7019d3f520000531a23db3b6dda7a94ad28db61a9008c");
}

#[test]
fn sha256_test() {
    use translation_api_cn::tencent::{hash256, hash_u8_to_string};

    let data = b"Nobody inspects the spammish repetition";

    // python: [hashlib.sha256](https://docs.python.org/3/library/hashlib.html)
    // 031edd7d41651593c5fe5c006fa5752b37fddff7bc4e843aa6af0c950f4b9406
    assert_display_snapshot!(hash256(data),
        @"031edd7d41651593c5fe5c006fa5752b37fddff7bc4e843aa6af0c950f4b9406"); // √
    assert_display_snapshot!(hash_u8_to_string(data).unwrap(),
        @"f5d6645a2c7bc9a9f1ffffe0931ae24c4dd37904cdab576bc4147ef1b0441a9f");

    let data =
        r#"{"Source": "en", "Target": "zh", "ProjectId": 0, "SourceTextList": ["hi", "there"]}"#;
    assert_display_snapshot!(hash256(data.as_bytes()), 
        @"132203170c4d03f4b351cacc51a7ceeed78ca571be42688945f74bb0796bb739"); // √
    assert_display_snapshot!(hash_u8_to_string(data.as_bytes()).unwrap(),
        @"3436b7117ac9800bd6c9f8e2834c58ede79dfc7b9dd9064e5b98929af6dd30df");
}
