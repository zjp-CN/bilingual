use super::*;
pub fn hash256(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    format!("{:x}", hasher.finalize())
}
pub fn hash256_string(v: &[u8]) -> Result<String> {
    Ok(format!("{:x}", HmacSha256::new_from_slice(v)?.finalize().into_bytes()))
}
pub fn hmac_sha256_string(v: Output) -> String { format!("{:x}", v.into_bytes()) }
pub fn hash_u8_hash(key: &[u8], msg: Output) -> Result<Output> {
    let mut mac = HmacSha256::new_from_slice(key)?;
    mac.update(msg.into_bytes().as_slice());
    Ok(mac.finalize())
}
pub fn hash_hash_u8(key: Output, msg: &[u8]) -> Result<Output> {
    let mut mac = HmacSha256::new_from_slice(key.into_bytes().as_slice())?;
    mac.update(msg);
    Ok(mac.finalize())
}
pub fn hash_2u8(key: &[u8], msg: &[u8]) -> Result<Output> {
    let mut mac = HmacSha256::new_from_slice(key)?;
    mac.update(msg);
    Ok(mac.finalize())
}
pub fn hash_2hash(key: Output, msg: Output) -> Result<Output> {
    let mut mac = HmacSha256::new_from_slice(key.into_bytes().as_slice())?;
    mac.update(msg.into_bytes().as_slice());
    Ok(mac.finalize())
}

// 使用 HMAC-SHA256 算法，对 `&[u8]` 或者具有 `.as_bytes()` 方法的数据计算 Hash 十六进制值
// macro_rules! hash {
//     // (@b $v:expr) => { hash($v.as_bytes()) };
//     (@@b $v:expr) => { // 输入 Output<HmacSha256>> 生成 Output<HmacSha256>>
//             $v.as_bytes()
//     };
//     (@@h $v:expr) => {{ // 生成 Hash 字符串
//             $v.into_bytes().as_slice()
//     }};
//     // (@b $key:expr, $($msg:expr),+) => { hash!($key.as_bytes(), $($msg.as_bytes()),+) };
//     ($v:expr) => {
//         Ok::<_, InvalidKeyLength>(
//             format!("{:x}", HmacSha256::new_from_slice($v)?.finalize().into_bytes())
//         )
//     };
//     (@h $key:expr, $($msg:expr),+) => {{
//         let mut mac = HmacSha256::new_from_slice( hash!(@@h $key))?;
//         $(mac.update(hash!(@@h $msg));)+
//         // Ok::<_, InvalidKeyLength>(format!("{:x}", mac.finalize().into_bytes()))
//         Ok::<_, InvalidKeyLength>(mac.finalize())
//     }};
//     (@b $key:expr, $($msg:expr),+) => {{
//         let mut mac = HmacSha256::new_from_slice( hash!(@@b $key))?;
//         $(mac.update(hash!(@@b $msg));)+
//         // Ok::<_, InvalidKeyLength>(format!("{:x}", mac.finalize().into_bytes()))
//         Ok::<_, InvalidKeyLength>(mac.finalize())
//     }};
// }
