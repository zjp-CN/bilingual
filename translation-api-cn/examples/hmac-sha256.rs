use hmac::{Hmac, Mac, NewMac};
use sha2::Sha256;

/// 对比 python 的 hmac-sha256 结果
/// ```python
/// import hashlib, hmac
/// hmac.new(b"0", b"1", hashlib.sha256).hexdigest()
/// hmac.new(hmac.new(b"0", b"1", hashlib.sha256).digest(), None, hashlib.sha256).hexdigest()
/// hmac.new(hmac.new(b"0", b"1", hashlib.sha256).digest(), b"2", hashlib.sha256).hexdigest()
/// ```
fn main() {
    type HmacSha256 = Hmac<Sha256>;

    let mut mac = HmacSha256::new_from_slice(b"0").unwrap();
    mac.update(b"1");

    let result = mac.finalize().into_bytes();
    assert_eq!("0b0065830a5c8d8f2c4997f5468610d6abc5533e49eac939426cf8158035ec3f",
               format!("{:x}", result));

    let result2 = HmacSha256::new_from_slice(result.as_slice()).unwrap().finalize().into_bytes();
    assert_eq!("922394236a962ebc90466942033cf117e347be148f899255de62b1ff4eab21b2",
               format!("{:x}", result2));

    let mut mac = HmacSha256::new_from_slice(result.as_slice()).unwrap();
    mac.update(b"2");
    assert_eq!("905dc60bced243ad461a220a2876ee6dc04c26e93b74a33c6a09c2212399fc0e",
               format!("{:x}", mac.finalize().into_bytes()));
}
