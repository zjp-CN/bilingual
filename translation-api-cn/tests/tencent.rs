// use anyhow::{Context, Result};
use insta::assert_display_snapshot;
use translation_api_cn::tencent;

#[test]
fn hash_test() {
    let q = tencent::Query { q:         &["a"],
                             from:      "en",
                             to:        "zh",
                             projectid: 0, };
    assert_display_snapshot!(
        q.to_hashed().unwrap(),
        @"025ed8bc22c2e13814d3caaa8515405adb50b6acbc532b25735ea96cbd87d0e0"
    );
}
