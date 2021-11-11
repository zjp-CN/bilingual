pub const URL: &str = "https://api.niutrans.com/NiuTransServer/translation";

/// 翻译前的必要信息
#[derive(Debug)]
pub struct Query<'q> {
    /// 请求翻译 query，必须为 UTF-8 编码。
    ///
    /// TODO: 在传入之前应该把文字控制在 6000 字节以内（汉字约为 2000 个字符），
    ///       超过 6000 字节要分段请求。
    pub q:    &'q str,
    /// 翻译源语言，可设置为 auto
    ///
    /// TODO：变成 Option + enum 类型，None 表示 auto
    pub from: &'q str,
    /// 翻译目标语言，不可设置为 auto
    ///
    /// TODO：和 `from` 共用 enum 类型，但无需是 Option 类型
    pub to:   &'q str,
}
