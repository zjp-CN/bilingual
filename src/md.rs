use arrayvec::ArrayVec;
use pulldown_cmark::{
    Event::{self, *},
    Options,
    Tag::*,
};
use pulldown_cmark_to_cmark::Options as OutOptions;
use std::mem::replace;

#[derive(Debug)]
pub struct Md<'e> {
    /// 解析 md 文件的事件
    events: Box<[Event<'e>]>,
    /// 内部缓冲。有两个用途：
    /// 1. 提取的原文段落；
    /// 2. 原文填充翻译内容之后的 md 文本。
    ///
    /// 为了减少分配，小于 1024B 的文本以 1024B 字节长度初始化；
    /// 大于 1024B 的文本以原文 2 倍字节长度初始化。
    buffer: String,
    /// output 的 bytes 分布
    bytes:  Vec<usize>,
    chars:  Vec<usize>,
    limit:  Limit,
}

impl<'e> Md<'e> {
    pub fn new(md: &'e str) -> Self {
        let capacity = md.len();
        Self { events: pulldown_cmark::Parser::new_ext(md, cmark_opt()).collect(),
               buffer: {
                   const MINIMUM_CAPACITY: usize = 1 << 10;
                   let capacity =
                       if capacity < MINIMUM_CAPACITY { MINIMUM_CAPACITY } else { capacity * 2 };
                   String::with_capacity(capacity)
               },
               bytes:  Vec::with_capacity(128), // 预先分配 128 个段落
               chars:  Vec::with_capacity(128), // 预先分配 128 个段落
               limit:  Limit::default(), }
    }

    /// 提取原文的段落文本。
    ///
    /// TODO: 尽可能保存原样式/结构
    pub fn extract(&mut self) -> &str {
        self.buffer_clear();
        let mut select = true;
        let buf = &mut self.buffer;
        self.events.iter().for_each(|event| extract(event, &mut select, buf));
        &self.buffer
    }

    /// 提取原文的段落文本，并以字节为单位记录段落分布。
    /// # 注意
    /// - 本方法返回提取后的所有文本，而且本方法一般只需要调用一次。
    /// - 本方法比 [`extract`][`Md::extract`] 多做了一件事：计算和记录每个段落的字节长度。
    /// - 需要每个段落的字节长度，请调用：[`bytes`][`Md::bytes`]。
    /// - 需要对段落按字节上限分批，请调用：[`bytes_paragraph`][`Md::bytes_paragraph`]。
    ///
    /// TODO: 尽可能保存原样式/结构
    pub fn extract_with_bytes(&mut self) -> &str {
        self.buffer_clear();
        self.para_clear();
        let mut select = true;
        let buf = &mut self.buffer;
        let len = &mut 0;
        let bytes = &mut self.bytes;
        self.events
            .iter()
            .for_each(|event| extract_with_bytes(event, &mut select, buf, len, bytes));
        &self.buffer
    }

    /// 提取原文的段落文本，并以字符为单位记录段落分布。
    /// # 注意
    /// - 本方法返回提取后的所有文本，而且本方法一般只需要调用一次。
    /// - 本方法比 [`extract_with_bytes`][`Md::extract_with_bytes`]
    ///   多做了一件事：计算和记录每个段落的字符长度。 所以
    ///   [`bytes_paragraph`][`Md::bytes_paragraph`] 和 [`bytes`][`Md::bytes`] 方法均可调用。
    /// - 需要每个段落的字符长度或字节范围，请调用：[`chars`][`Md::chars`]、
    ///   [`chars_bytes_range`][`Md::chars_bytes_range`]。
    /// - 需要对段落按字符上限分批，请调用：[`chars_paragraph`][`Md::chars_paragraph`]。
    ///
    /// TODO: 尽可能保存原样式/结构
    pub fn extract_with_chars(&mut self) -> &str {
        self.buffer_clear();
        self.para_clear();
        let mut select = true;
        let buf = &mut self.buffer;
        let len = &mut 0;
        let cnt = &mut 0;
        let bytes = &mut self.bytes;
        let chars = &mut self.chars;
        self.events
            .iter()
            .for_each(|event| extract_with_chars(event, &mut select, buf, len, bytes, cnt, chars));
        &self.buffer
    }

    /// 提取的每个原文段落的字节数。
    pub fn bytes<'r>(&'r self) -> impl Iterator<Item = usize> + 'r { self.bytes.iter().copied() }

    /// 提取的每个原文段落的字符数。
    pub fn chars<'r>(&'r self) -> impl Iterator<Item = usize> + 'r { self.chars.iter().copied() }

    /// 提取的每个原文段落的字符数、字节数和字节范围。
    pub fn chars_bytes_range<'r>(&'r self) -> impl Iterator<Item = (usize, usize, Range)> + 'r {
        self.chars()
            .zip(self.bytes())
            .scan(0, |state, (c, l)| Some((c, l, replace(state, *state + l)..*state)))
    }

    /// 以字节数量分割段落批次。
    /// 分批策略如下：
    /// - 当返回 `Some` 时，意味着返回完整的段落，且至少返回一个段落；
    /// - 当返回 `None` 时，意味着已经返回所有段落。
    /// - 每次返回的段落有两种情况：
    ///
    ///   1. 不多于 limit 字节大小的完整段落（至少一个完整段落）；
    ///   2. 字节大小超过 limit 的**一个**段落。
    ///
    /// ## 注意
    /// - 请先调用一次 [`extract_with_bytes`][`Md::extract_with_bytes`] 或者
    ///   [`extract_with_chars`][`Md::extract_with_chars`] 再调用此方法。
    /// - 此方法可以多次调用：这在需要不同 limit 的分批时很有用。
    pub fn bytes_paragraph(&mut self, limit: usize) -> impl Iterator<Item = &str> {
        self.limit = Limit::new(limit);
        let limit = &mut self.limit;
        let f = |l: &usize| if let Some(i) = limit.bytes(*l) { self.buffer.get(i) } else { None };
        self.bytes.iter().chain(std::iter::once(&usize::MAX)).filter_map(f)
    }

    /// 以字符数量分割段落批次。
    /// 分批策略如下：
    /// - 当返回 `Some` 时，意味着返回完整的段落，且至少返回一个段落；
    /// - 当返回 `None` 时，意味着已经返回所有段落。
    /// - 每次返回的段落有两种情况：
    ///
    ///   1. 不多于 limit 字节大小的完整段落（至少一个完整段落）；
    ///   2. 字节大小超过 limit 的**一个**段落。
    ///
    /// ## 注意
    /// - 请先调用一次 [`extract_with_chars`][`Md::extract_with_chars`] 再调用此方法。
    /// - 此方法可以多次调用：这在需要不同 limit 的分批时很有用。
    pub fn chars_paragraph(&mut self, limit: usize) -> impl Iterator<Item = &str> {
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

    /// 完成并返回写入翻译内容。参数 `paragraph` 为按段落翻译的**译文**。
    pub fn done(mut self, mut paragraph: impl Iterator<Item = &'e str>) -> String {
        self.buffer_clear(); // 清除段落缓冲
                             // let events: Vec<_> = self.events.into();
        let output =
            self.events.into_vec().into_iter().map(|e| prepend(e, &mut paragraph)).flatten();
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

    fn buffer_clear(&mut self) {
        if !self.buffer.is_empty() {
            self.buffer.clear()
        }
    }

    fn para_clear(&mut self) {
        if !self.bytes.is_empty() {
            self.bytes.clear()
        }
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
        dbg!(len, &self);
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

pub fn prepend<'e>(event: Event<'e>, paragraph: &mut impl Iterator<Item = &'e str>)
                   -> ArrayVec<Event<'e>, MAXIMUM_EVENTS> {
    let mut arr = ArrayVec::<_, MAXIMUM_EVENTS>::new();
    match event {
        End(Paragraph | Heading(_)) => {
            arr.push(SoftBreak); // TODO: 是否空行
            arr.extend([SoftBreak, Text(paragraph.next().unwrap().into()), event]);
        }
        _ => arr.extend([event]),
    }
    arr
}

/// 取出需要被翻译的内容：按照段落或标题
pub fn extract(event: &Event, select: &mut bool, buf: &mut String) {
    match event {
        End(Paragraph | Heading(_)) => buf.push('\n'),
        Text(x) if *select => buf.push_str(x.as_ref()),
        SoftBreak | HardBreak => buf.push(' '),
        Code(x) => {
            buf.push('`');
            buf.push_str(x.as_ref());
            buf.push('`');
        }
        Start(CodeBlock(_)) => *select = false,
        End(CodeBlock(_)) => *select = true,
        _ => (),
    }
}

/// 取出需要被翻译的内容：按照段落或标题。
pub fn extract_with_bytes(event: &Event, select: &mut bool, buf: &mut String, len: &mut usize,
                          vec: &mut Vec<usize>) {
    match event {
        End(Paragraph | Heading(_)) => {
            buf.push('\n');
            vec.push(*len + 1);
            *len = 0;
        }
        Text(x) if *select => {
            buf.push_str(x.as_ref());
            *len += x.len();
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
        Start(CodeBlock(_)) => *select = false,
        End(CodeBlock(_)) => *select = true,
        _ => (),
    }
}

/// 取出需要被翻译的内容：按照段落或标题
pub fn extract_with_chars(event: &Event, select: &mut bool, buf: &mut String, len: &mut usize,
                          bytes: &mut Vec<usize>, cnt: &mut usize, chars: &mut Vec<usize>) {
    match event {
        End(Paragraph | Heading(_)) => {
            buf.push('\n');
            bytes.push(*len + 1);
            chars.push(*cnt + 1);
            *len = 0;
            *cnt = 0;
        }
        Text(x) if *select => {
            buf.push_str(x.as_ref());
            *len += x.len();
            *cnt += x.chars().count();
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
        Start(CodeBlock(_)) => *select = false,
        End(CodeBlock(_)) => *select = true,
        _ => (),
    }
}
