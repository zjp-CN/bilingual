use arrayvec::ArrayVec;
use pulldown_cmark::{
    Event::{self, *},
    Options,
    Tag::*,
};

pub struct Md<'e> {
    /// 解析 md 文件的事件
    events:  Vec<Event<'e>>,
    /// md 原文的长度
    raw_len: usize,
    /// 填充翻译内容之后的 md 文件的内容
    /// TODO: 比较是否超出 output's capacity
    output:  String,
}

impl<'e> Md<'e> {
    pub fn new(md: &'e str) -> Self {
        let capacity = md.len();
        Self { events:  pulldown_cmark::Parser::new_ext(md, cmark_opt()).collect(),
               raw_len: capacity,
               output:  String::with_capacity(capacity * 2), }
    }

    pub fn extract(&self) -> String {
        let mut select = true;
        let mut buf = String::with_capacity(self.raw_len);
        self.events.iter().for_each(|event| extract(event, &mut select, &mut buf));
        buf
    }

    pub fn done(mut self, mut paragraph: impl Iterator<Item = &'e str>) -> String {
        let output = self.events.into_iter().map(|event| prepend(event, &mut paragraph)).flatten();
        let opt = cmark_to_cmark_opt();
        pulldown_cmark_to_cmark::cmark_with_options(output, &mut self.output, None, opt).unwrap();
        dbg!(self.output.len(),
             self.output.capacity(),
             self.raw_len * 2,
             self.output.len() <= self.raw_len * 2);
        self.output
    }
}

/// 开启 `pulldown_cmark::Options` 除 `SMART_PUNCTUATION` 之外的所有功能
pub fn cmark_opt() -> Options {
    let mut options = Options::all();
    options.remove(Options::ENABLE_SMART_PUNCTUATION);
    options
}

/// 把 `pulldown_cmark_to_cmark::Options` 的 `code_block_backticks` 设置为 3
pub fn cmark_to_cmark_opt() -> pulldown_cmark_to_cmark::Options {
    let mut opt = pulldown_cmark_to_cmark::Options::default();
    opt.code_block_backticks = 3;
    opt
}

const MAXIMUM: usize = 4;

pub fn prepend<'e>(event: Event<'e>, paragraph: &mut impl Iterator<Item = &'e str>)
                   -> ArrayVec<Event<'e>, MAXIMUM> {
    let mut arr = ArrayVec::<_, MAXIMUM>::new();
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
