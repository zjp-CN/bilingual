use arrayvec::ArrayVec;
use pulldown_cmark::{
    Event::{self, *},
    Options,
    Tag::*,
};
use pulldown_cmark_to_cmark::Options as OutOptions;
use std::mem::{replace, take};

#[derive(Debug)]
pub struct Md<'e> {
    /// 解析 md 文件的事件
    events: Vec<Event<'e>>,
    /// 内部缓冲。有两个用途：
    /// 1. 提取的原文段落；
    /// 2. 原文填充翻译内容之后的 md 文本。
    ///
    /// 为了减少分配，小于 1024B 的文本以 1024B 字节长度初始化；
    /// 大于 1024B 的文本以原文 2 倍字节长度初始化。
    buffer: String,
    /// 提取的原文段落的 bytes 分布
    bytes:  Vec<usize>,
    /// 提取的原文段落的 chars 分布
    chars:  Vec<usize>,
    /// 用于段落分批
    limit:  Limit,
}

impl<'e> Md<'e> {
    /// 构造函数。
    pub fn new(md: &'e str) -> Self {
        Self { events: pulldown_cmark::Parser::new_ext(md, cmark_opt()).collect(),
               buffer: {
                   const MINIMUM_CAPACITY: usize = 1 << 10;
                   let capacity = md.len();
                   let capacity =
                       if capacity < MINIMUM_CAPACITY { MINIMUM_CAPACITY } else { capacity * 2 };
                   String::with_capacity(capacity)
               },
               bytes:  Vec::with_capacity(128), // 预先分配 128 个段落
               chars:  Vec::with_capacity(128), // 预先分配 128 个段落
               limit:  Limit::default(), }
    }

    /// 提取原文的段落文本，并以字符为单位记录段落分布。
    ///
    /// TODO: 尽可能保存原样式/结构
    fn extract_with_chars(&mut self) {
        fn inner(m: &mut Md) {
            let not_codeblock = &mut true;
            let table = &mut false;
            let buf = &mut m.buffer;
            let len = &mut 0;
            let cnt = &mut 0;
            let bytes = &mut m.bytes;
            let chars = &mut m.chars;
            #[rustfmt::skip]
            m.events.iter().for_each(|e| { extract_with_chars(e, not_codeblock, table, buf, len, bytes, cnt, chars) });
        }
        if self.buffer.is_empty() {
            inner(self);
        } else if self.chars.is_empty() {
            if !self.bytes.is_empty() {
                self.bytes.clear();
            }
            self.buffer.clear();
            inner(self);
        }
    }

    /// 以字符数量分割段落批次。
    /// 分批策略如下：
    /// - 当返回 `Some` 时，意味着返回完整的段落，且至少返回一个段落；
    /// - 当返回 `None` 时，意味着已经返回所有段落。
    /// - 每次返回的段落有两种情况：
    ///
    ///   1. 不多于 limit 字符大小的完整段落（至少一个完整段落）；
    ///   2. 字符大小超过 limit 的**一个**段落。
    ///
    /// ## 注意
    /// - 本方法比 [`bytes_paragraph`] 多做了一件事：计算和记录每个段落的字符长度。
    /// - 需要每个段落的字符或字节长度，请再调用： [`chars`][`Md::chars`] 或
    ///   [`bytes`][`Md::bytes`]。
    /// - 此方法可以多次调用：这在需要不同 limit 的分批时很有用。但是注意：
    ///   - 调用 [`extract`] 之后再调用此方法会重复提取段落；
    ///   - 调用 [`bytes_paragraph`] 之后再调用此方法会重复提取段落；
    ///   - 多次调用此方法不会重复提取段落；
    ///
    /// [`bytes_paragraph`]: `Md::bytes_paragraph`
    /// [`extract`]: `Md::extract`
    pub fn chars_paragraph(&mut self, limit: usize) -> impl Iterator<Item = &str> {
        self.extract_with_chars();
        self.limit = Limit::new(limit);
        let limit = &mut self.limit;
        let f = |(c, l): (&usize, &usize)| {
            if let Some(i) = limit.chars(*c, *l) {
                self.buffer.get(i)
            } else {
                None
            }
        };
        let iter = std::iter::once(&usize::MAX);
        self.chars
            .iter()
            .chain(iter.clone())
            .zip(self.bytes.iter().chain(iter))
            .filter_map(f)
    }

    /// 提取原文的段落文本，并以字节为单位记录段落分布。
    /// # 注意
    /// - 本方法比 [`extract`][`Md::extract`] 多做了一件事：计算和记录每个段落的字节长度。
    /// - 需要每个段落的字节长度，请再调用：[`bytes`][`Md::bytes`]。
    ///
    /// TODO: 尽可能保存原样式/结构
    fn extract_with_bytes(&mut self) {
        fn inner(m: &mut Md) {
            let not_codeblock = &mut true;
            let table = &mut false;
            let buf = &mut m.buffer;
            let len = &mut 0;
            let bytes = &mut m.bytes;
            m.events
             .iter()
             .for_each(|event| extract_with_bytes(event, not_codeblock, table, buf, len, bytes));
        }
        if self.buffer.is_empty() {
            inner(self);
        } else if self.bytes.is_empty() {
            self.buffer.clear();
            inner(self);
        }
    }

    /// 提取原文的段落文本，并返回以字节数量分割的段落批次。
    /// 分批策略如下：
    /// - 当返回 `Some` 时，意味着返回完整的段落，且至少返回一个段落；
    /// - 当返回 `None` 时，意味着已经返回所有段落。
    /// - 每次返回的段落有两种情况：
    ///
    ///   1. 不多于 limit 字节大小的完整段落（至少一个完整段落）；
    ///   2. 字节大小超过 limit 的**一个**段落。
    ///
    /// ## 注意
    /// - 本方法比 [`extract`] 多做了一件事：计算和记录每个段落的字节长度。
    /// - 需要每个段落的字节长度，请调用：[`bytes`][`Md::bytes`]。
    /// - 此方法可以多次调用：这在需要不同 limit 的分批时很有用。但是注意：
    ///   - 调用 [`extract`] 之后再调用此方法会重复提取段落；
    ///   - 调用 [`chars_paragraph`] 之后再调用此方法不会重复提取段落；
    ///   - 多次调用此方法不会重复提取段落；
    ///
    /// [`chars_paragraph`]: `Md::chars_paragraph`
    /// [`extract`]: `Md::extract`
    pub fn bytes_paragraph(&mut self, limit: usize) -> impl Iterator<Item = &str> {
        self.extract_with_bytes();
        self.limit = Limit::new(limit);
        let limit = &mut self.limit;
        let f = |l: &usize| if let Some(i) = limit.bytes(*l) { self.buffer.get(i) } else { None };
        self.bytes.iter().chain(std::iter::once(&usize::MAX)).filter_map(f)
    }

    /// 提取原文的段落文本。
    ///
    /// ## 注意
    /// - 此方法可以多次调用：
    ///   - 调用 [`bytes_paragraph`] 之后再调用此方法不会重复提取段落；
    ///   - 调用 [`chars_paragraph`] 之后再调用此方法不会重复提取段落；
    ///   - 多次调用此方法不会重复提取段落；
    ///
    /// TODO: 尽可能保存原样式/结构
    ///
    /// [`bytes_paragraph`]: `Md::bytes_paragraph`
    /// [`chars_paragraph`]: `Md::chars_paragraph`
    pub fn extract(&mut self) -> &str {
        if self.buffer.is_empty() {
            let not_codeblock = &mut true;
            let table = &mut false;
            let buf = &mut self.buffer;
            self.events.iter().for_each(|event| extract(event, not_codeblock, table, buf));
        }
        &self.buffer
    }

    /// 浏览提取后的原文段落文本。
    pub fn paragraphs(&self) -> &str { &self.buffer }

    /// 提取的每个原文段落的字节数。
    pub fn bytes(&self) -> impl Iterator<Item = usize> + '_ { self.bytes.iter().copied() }

    /// 提取的每个原文段落的字符数。
    pub fn chars(&self) -> impl Iterator<Item = usize> + '_ { self.chars.iter().copied() }

    /// 提取的每个原文段落的字符数、字节数和字节范围。
    pub fn chars_bytes_range(&self) -> impl Iterator<Item = (usize, usize, Range)> + '_ {
        self.chars()
            .zip(self.bytes())
            .scan(0, |state, (c, l)| Some((c, l, replace(state, *state + l)..*state)))
    }

    /// 完成并返回写入翻译内容。参数 `paragraph` 为按段落翻译的**译文**。
    pub fn done(mut self, mut paragraph: impl Iterator<Item = &'e str>) -> String {
        self.buffer.clear();
        let table = &mut false;
        let output = self.events.into_iter().map(|e| prepend(e, table, &mut paragraph)).flatten();
        let opt = cmark_to_cmark_opt();
        pulldown_cmark_to_cmark::cmark_with_options(output, &mut self.buffer, None, opt).unwrap();
        // dbg!(self.output.len(),
        //      self.output.capacity(),
        //      self.raw_len * 2,
        //      self.output.len() <= self.raw_len * 2,
        //      self.output.len() <= MINIMUM_CAPACITY,
        //      self.output.len() <= self.raw_len * 2 || self.output.len() <= MINIMUM_CAPACITY);
        self.buffer
    }
}

type Range = std::ops::Range<usize>;

#[derive(Debug, Default)]
struct Limit {
    limit: usize,
    cnt:   usize,
    len:   usize,
    pos:   usize,
}

impl Limit {
    #[rustfmt::skip]
    fn new(limit: usize) -> Self { Self { limit, cnt: 0, len: 0, pos: 0 } }

    fn bytes(&mut self, len: usize) -> Option<Range> {
        // dbg!(len, &self);
        if let Some(add) = self.len.checked_add(len) {
            if add <= self.limit {
                self.len = add;
                return None;
            } else {
                let p = self.pos;
                let rhs = if self.len == 0 { len } else { replace(&mut self.len, len) };
                // 到达最后一批时：当 bat=0, len=usize::Max 时会出现 Err
                if let Some(add) = self.pos.checked_add(rhs) {
                    self.pos = add;
                    return Some(p..self.pos);
                }
            }
        }
        // 返回最后一批段落
        if self.len == 0 {
            // 当 len=0 时，最后一批段落是空串，因此需要提前结束掉
            None
        } else {
            Some(self.pos..self.pos + self.len)
        }
    }

    fn chars(&mut self, cnt: usize, len: usize) -> Option<Range> {
        // dbg!(cnt, len, &self);
        if let Some(add) = self.len.checked_add(len) {
            if self.cnt + cnt <= self.limit {
                self.len = add;
                self.cnt += cnt;
                return None;
            } else {
                let p = self.pos;
                let rhs = if self.len == 0 {
                    len
                } else {
                    self.cnt = cnt;
                    replace(&mut self.len, len)
                };
                // 到达最后一批时：当 bat=0, len=usize::Max 时会出现 Err
                if let Some(add) = self.pos.checked_add(rhs) {
                    self.pos = add;
                    return Some(p..self.pos);
                }
            }
        }
        // 返回最后一批段落
        if self.len == 0 {
            // 当 len=0 时，最后一批段落是空串，因此需要提前结束掉
            None
        } else {
            Some(self.pos..self.pos + self.len)
        }
    }
}

/// 开启 `pulldown_cmark::Options` 除 `SMART_PUNCTUATION` 之外的所有功能
pub fn cmark_opt() -> Options {
    let mut options = Options::all();
    options.remove(Options::ENABLE_SMART_PUNCTUATION);
    options
}

/// 把 `pulldown_cmark_to_cmark::Options` 的 `code_block_backticks` 设置为 3
pub fn cmark_to_cmark_opt() -> OutOptions {
    OutOptions { code_block_backticks: 3,
                 ..OutOptions::default() }
}

const MAXIMUM_EVENTS: usize = 4;

pub fn prepend<'e>(event: Event<'e>, table: &mut bool,
                   paragraph: &mut impl Iterator<Item = &'e str>)
                   -> ArrayVec<Event<'e>, MAXIMUM_EVENTS> {
    let mut arr = ArrayVec::<_, MAXIMUM_EVENTS>::new();
    // dbg!(&event);
    match event {
        End(Paragraph) => {
            arr.push(SoftBreak); // TODO: 是否空行
            arr.extend([SoftBreak, Text(paragraph.next().unwrap().into()), event]);
        }
        End(Heading(n)) => {
            arr.extend([event,
                        Start(Heading(n)),
                        Text(paragraph.next().unwrap().into()),
                        End(Heading(n))]);
        }
        event @ End(Table(_)) => {
            *table = false;
            arr.extend([event])
        }
        event @ Start(Table(_)) => {
            *table = true;
            arr.extend([event])
        }
        event @ Text(_) if *table => {
            arr.extend([event, Text('\t'.into()), Text(paragraph.next().unwrap().into())])
        }
        _ => arr.extend([event]),
    }
    arr
}

/// 取出需要被翻译的内容：按照段落或标题
pub fn extract(event: &Event, not_codeblock: &mut bool, table: &mut bool, buf: &mut String) {
    match event {
        End(Paragraph | Heading(_)) => buf.push('\n'),
        Text(x) if *not_codeblock => {
            buf.push_str(x.as_ref());
            if *table {
                buf.push('\n');
            }
        }
        SoftBreak | HardBreak => buf.push(' '),
        Code(x) => {
            buf.push('`');
            buf.push_str(x.as_ref());
            buf.push('`');
        }
        Start(CodeBlock(_)) => *not_codeblock = false,
        End(CodeBlock(_)) => *not_codeblock = true,
        End(Table(_)) => *table = false,
        Start(Table(_)) => *table = true,
        _ => (),
    }
}

/// 取出需要被翻译的内容：按照段落或标题。
pub fn extract_with_bytes(event: &Event, not_codeblock: &mut bool, table: &mut bool,
                          buf: &mut String, len: &mut usize, vec: &mut Vec<usize>) {
    match event {
        End(Paragraph | Heading(_)) => {
            buf.push('\n');
            vec.push(take(len) + 1);
        }
        Text(x) if *not_codeblock => {
            buf.push_str(x.as_ref());
            if *table {
                buf.push('\n');
                vec.push(take(len) + x.len() + 1);
            } else {
                *len += x.len();
            };
        }
        SoftBreak | HardBreak => {
            buf.push(' ');
            *len += 1;
        }
        Code(x) => {
            buf.push('`');
            buf.push_str(x.as_ref());
            buf.push('`');
            *len += x.len() + 2;
        }
        Start(CodeBlock(_)) => *not_codeblock = false,
        End(CodeBlock(_)) => *not_codeblock = true,
        End(Table(_)) => *table = false,
        Start(Table(_)) => *table = true,
        _ => (),
    }
}

/// 取出需要被翻译的内容：按照段落或标题
pub fn extract_with_chars(event: &Event, not_codeblock: &mut bool, table: &mut bool,
                          buf: &mut String, len: &mut usize, bytes: &mut Vec<usize>,
                          cnt: &mut usize, chars: &mut Vec<usize>) {
    match event {
        End(Paragraph | Heading(_)) => {
            buf.push('\n');
            bytes.push(take(len) + 1);
            chars.push(take(cnt) + 1);
        }
        Text(x) if *not_codeblock => {
            buf.push_str(x.as_ref());
            if *table {
                buf.push('\n');
                bytes.push(take(len) + x.len() + 1);
                chars.push(take(cnt) + x.chars().count() + 1);
            } else {
                *len += x.len();
                *cnt += x.chars().count();
            }
        }
        SoftBreak | HardBreak => {
            buf.push(' ');
            *len += 1;
            *cnt += 1;
        }
        Code(x) => {
            buf.push('`');
            buf.push_str(x.as_ref());
            buf.push('`');
            *len += x.len() + 2;
            *cnt += x.chars().count() + 2;
        }
        Start(CodeBlock(_)) => *not_codeblock = false,
        End(CodeBlock(_)) => *not_codeblock = true,
        End(Table(_)) => *table = false,
        Start(Table(_)) => *table = true,
        _ => (),
    }
}
