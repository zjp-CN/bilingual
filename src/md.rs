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
    /// 填充翻译内容之后的 md 文件的内容。
    /// 为了减少分配，小于 1024B 的文本以 1024B 字节长度初始化；
    /// 大于 1024B 的文本以原文 2 倍字节长度初始化。
    buffer: String,
    /// output 的 bytes 分布
    para:   Vec<usize>,
}

type Range = std::ops::Range<usize>;

impl<'e> Md<'e> {
    pub fn new(md: &'e str) -> Self {
        const PARAGRAPHS: usize = 128;
        let capacity = md.len();
        Self { events: pulldown_cmark::Parser::new_ext(md, cmark_opt()).collect(),
               buffer: {
                   const MINIMUM_CAPACITY: usize = 1 << 10;
                   let capacity =
                       if capacity < MINIMUM_CAPACITY { MINIMUM_CAPACITY } else { capacity * 2 };
                   String::with_capacity(capacity)
               },
               para:   Vec::with_capacity(PARAGRAPHS), }
    }

    /// 提取文本
    ///
    /// TODO: 尽可能保存原样式/结构
    pub fn extract(&mut self) -> &str {
        let mut select = true;
        let buf = &mut self.buffer;
        self.events.iter().for_each(|event| extract(event, &mut select, buf));
        &self.buffer
    }

    /// 提取文本，并以字节单位记录段落分布。
    ///
    /// TODO: 尽可能保存原样式/结构
    pub fn extract_with_bytes(&mut self) -> &str {
        let mut select = true;
        let buf = &mut self.buffer;
        let mut len = 0;
        let vec = &mut self.para;
        self.events
            .iter()
            .for_each(|event| extract_with_bytes(event, &mut select, buf, &mut len, vec));
        &self.buffer
    }

    pub fn bytes_next_paragraph<'i>(&'i self, limit: &'i mut Limit)
                                    -> impl Iterator<Item = &str> + 'i {
        let closure =
            |l: &usize| if let Some(idx) = limit.batch(*l) { self.buffer.get(idx) } else { None };
        self.para.iter().chain(std::iter::once(&usize::MAX)).filter_map(closure)
    }

    #[rustfmt::skip]
    pub fn bytes_next_range<'r>(&'r self) -> impl Iterator<Item = (usize, Range)> + 'r {
        self.para.iter().scan(0, |state, &l| { Some((l, std::mem::replace(state, *state+l)..*state)) })
    }

    /// 完成并返回写入翻译内容。参数 `paragraph` 为按段落翻译的译文。
    pub fn done(mut self, mut paragraph: impl Iterator<Item = &'e str>) -> String {
        self.buffer.clear(); // 清除段落缓冲
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
}

#[derive(Debug)]
pub struct Limit {
    limit: usize,
    bat:   usize,
    pos:   usize,
}

impl Limit {
    #[rustfmt::skip]
    pub fn new(limit: usize) -> Self {
        Self { limit, bat: 0, pos: 0 }
    }

    pub fn batch(&mut self, len: usize) -> Option<Range> {
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
            // 最后一位
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
