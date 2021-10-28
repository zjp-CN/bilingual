use pulldown_cmark::{
    Event::{self, *},
    Options,
    Tag::*,
};

pub struct Md<'e> {
    /// 解析 md 文件的事件
    events:  Vec<Event<'e>>,
    raw_len: usize,
    // /// 提取的、待翻译的段落
    // extract: String,
    // /// 已翻译的段落
    // translation: Vec<String>,
    /// 填充翻译内容之后的 md 文件的内容
    output:  String,
}

impl<'e> Md<'e> {
    pub fn new(md: &'e str) -> Self {
        let capacity = md.len();
        Self { events:  pulldown_cmark::Parser::new_ext(md, cmark_opt()).collect(),
               raw_len: capacity,
               output:  String::with_capacity(capacity * 2), }
    }

    pub fn extract(&mut self) -> String {
        let mut select = true;
        let mut buf = String::with_capacity(self.raw_len);
        self.events.iter().map(|event| extract(event, &mut select, &mut buf)).last();
        buf
    }

    pub fn done(mut self, mut paragraph: impl Iterator<Item = &'e str>) -> String {
        let output = self.events.into_iter().map(|event| prepend(event, &mut paragraph)).flatten();
        pulldown_cmark_to_cmark::cmark(output, &mut self.output, None).unwrap();
        self.output
    }
}

pub fn process(raw: &str) -> String {
    let mut md = Md::new(&raw);
    let buf = md.extract();
    let output = md.done(buf.split('\n'));
    println!("{}", output);
    output
}

pub fn cmark_opt() -> Options {
    let mut options = Options::all();
    options.remove(Options::ENABLE_SMART_PUNCTUATION);
    options
}

const MAXIMUM: usize = 4;
use arrayvec::ArrayVec;

pub fn prepend<'e>(event: Event<'e>, paragraph: &mut impl Iterator<Item = &'e str>)
                   -> ArrayVec<Event<'e>, MAXIMUM> {
    let mut arr = ArrayVec::<_, MAXIMUM>::new();
    match event {
        End(Paragraph | Heading(_)) => {
            arr.push(SoftBreak); // TODO: 控制换行行数，1 或 2 行（默认 2 行）
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
