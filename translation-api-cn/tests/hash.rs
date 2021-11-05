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
    assert_display_snapshot!(datetime, @"2021-11-05 11:27:25.0 +00:00:00");
    let mut user = User::default();
    user.id = "0".into();
    user.key = "0".into();
    let query = Query { from:      "en",
                        to:        "zh",
                        projectid: 0,
                        q:         &["hi", "there"], };
    assert_display_snapshot!(query.to_json().unwrap(), @r###"{"Source":"en","Target":"zh","projectid":0,"SourceTextList":["hi","there"]}"###);
    assert_display_snapshot!(query.to_hashed().unwrap(), @"c9a232d18031eb1583ca657256a3ff53f450aa689951d1b202e6a2dae9417782");
    let header = HeaderJson { datetime,
                              authorization: "".into(),
                              user: &user,
                              query };
    assert_display_snapshot!(header.signature().unwrap(), @"64eb4a2101c5a0defe8973074f7f7aa96d303bef88f28a3372da878b85f675ad");
}

#[test]
fn compare_test() {
    use hmac::{Mac, NewMac};
    use translation_api_cn::tencent::{self, hash_u8_to_string};
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

    let mac = tencent::HmacSha256::new_from_slice(b"a").unwrap();
    let x = format!("{:x}", mac.finalize().into_bytes());
    assert_display_snapshot!(x, 
        @"3acdaa86b3d73e8d18b7019d3f520000531a23db3b6dda7a94ad28db61a9008c");
}

#[test]
fn sha256_test() {
    use hmac::{Hmac, Mac, NewMac};
    use sha2::{Digest, Sha256};

    let data = b"Nobody inspects the spammish repetition";
    // let data = b"a";
    let mut hasher = Sha256::new();
    hasher.update(data);

    // python: [hashlib.sha256](https://docs.python.org/3/library/hashlib.html)
    // 031edd7d41651593c5fe5c006fa5752b37fddff7bc4e843aa6af0c950f4b9406
    assert_display_snapshot!(format!("{:x}", hasher.finalize()),
        @"031edd7d41651593c5fe5c006fa5752b37fddff7bc4e843aa6af0c950f4b9406");

    // Create alias for HMAC-SHA256
    type HmacSha256 = Hmac<Sha256>;
    let mac = HmacSha256::new_from_slice(data).expect("HMAC can take key of any size");
    assert_display_snapshot!(format!("{:x}", mac.finalize().into_bytes()),
        @"f5d6645a2c7bc9a9f1ffffe0931ae24c4dd37904cdab576bc4147ef1b0441a9f");
    use std as std2;
    use std::borrow::Cow;
    use std2::borrow::Cow as C2;
}
