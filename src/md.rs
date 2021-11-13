use arrayvec::ArrayVec;
use pulldown_cmark::{
    Event::{self, *},
    Options,
    Tag::*,
};
use pulldown_cmark_to_cmark::Options as OutOptions;

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
    /// output 的 bytes 分布
    para:   Vec<usize>,
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
               para:   Vec::with_capacity(128), // 预先分配 128 个段落
               limit:  Limit::default(), }
    }

    /// 提取文本
    ///
    /// TODO: 尽可能保存原样式/结构
    pub fn extract(&mut self) -> &str {
        self.buffer_clear();
        let mut select = true;
        let buf = &mut self.buffer;
        self.events.iter().for_each(|event| extract(event, &mut select, buf));
        &self.buffer
    }

    /// 提取文本，并以字节单位记录段落分布。
    /// # 注意
    /// - 本方法返回提取后的所有文本，而且本方法一般只需要调用一次。
    /// - 需要分批段落，请调用：[`bytes_paragraph`][`Md::bytes_paragraph`]。
    /// - 需要分批段落的字节长度或范围，请调用：[`bytes_range`][`Md::bytes_range`]。
    ///
    /// TODO: 尽可能保存原样式/结构
    pub fn extract_with_bytes(&mut self) -> &str {
        self.buffer_clear();
        self.para_clear();
        let mut select = true;
        let buf = &mut self.buffer;
        let mut len = 0;
        let vec = &mut self.para;
        self.events
            .iter()
            .for_each(|event| extract_with_bytes(event, &mut select, buf, &mut len, vec));
        &self.buffer
    }

    /// 以字节数量分割段落批次。
    ///
    /// ## 注意
    /// - 请先调用一次 [`extract_with_bytes`][`Md::extract_with_bytes`] 再调用此方法。
    /// - 此方法可以多次调用：这在需要不同 limit 的分批时很有用。
    pub fn bytes_paragraph(&mut self, limit: usize) -> impl Iterator<Item = &str> {
        self.limit = Limit::new(limit);
        let limit = &mut self.limit;
        let f = |l: &usize| if let Some(i) = limit.batch(*l) { self.buffer.get(i) } else { None };
        self.para.iter().chain(std::iter::once(&usize::MAX)).filter_map(f)
    }

    /// 返回每个分批段落的字节数和范围。
    ///
    /// ## 注意
    /// - 请先调用一次 [`extract_with_bytes`][`Md::extract_with_bytes`] 再调用此方法。
    /// - 此方法可以多次调用。
    #[rustfmt::skip]
    pub fn bytes_range<'r>(&'r self) -> impl Iterator<Item = (usize, Range)> + 'r {
        self.para.iter().scan(0, |state, &l| Some((l, std::mem::replace(state, *state+l)..*state)))
    }

    /// 完成并返回写入翻译内容。参数 `paragraph` 为按段落翻译的译文。
    pub fn done(mut self, mut paragraph: impl Iterator<Item = &'e str>) -> String {
        self.buffer_clear(); // 清除段落缓冲
        let output = self.events.into_iter().map(|event| prepend(event, &mut paragraph)).flatten();
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
        if !self.para.is_empty() {
            self.para.clear()
        }
    }
}

type Range = std::ops::Range<usize>;

#[derive(Debug, Default)]
struct Limit {
    limit: usize,
    bat:   usize,
    pos:   usize,
}

impl Limit {
    #[rustfmt::skip]
    fn new(limit: usize) -> Self { Self { limit, bat: 0, pos: 0 } }

    fn batch(&mut self, len: usize) -> Option<Range> {
        if let Some(add) = self.bat.checked_add(len) {
            if add < self.limit {
                self.bat += len;
                None
            } else {
                let p = self.pos;
                self.pos += if self.bat == 0 { len } else { std::mem::replace(&mut self.bat, len) };
                Some(p..self.pos)
            }
        } else {
            // 返回最后一个段落
            Some(self.pos..self.pos + self.bat)
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

/// 取出需要被翻译的内容：按照段落或标题
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
