use pulldown_cmark::{CowStr, Event, Options, Tag};

pub fn cmark_opt() -> Options {
    let mut options = Options::all();
    options.remove(Options::ENABLE_SMART_PUNCTUATION);
    options
}

/// 初步排除不需要的 Event
pub fn filter(event: &Event, include: &mut bool) -> bool {
    match event {
        Event::Start(Tag::CodeBlock(_)) => {
            *include = false;
            false
        }
        Event::End(Tag::CodeBlock(_)) => {
            *include = true;
            false
        }
        Event::Start(Tag::Heading(_)) | Event::Start(Tag::Paragraph) => false,
        // 排除行间代码
        _ => *include,
    }
}

/// 取出需要被翻译的内容
pub fn extract<'a>((pos, event): (usize, Event<'a>)) -> (usize, CowStr<'a>, Option<usize>) {
    let mut para_new = None;
    let text = match event {
        Event::Text(x) => x,
        Event::End(Tag::Heading(_)) | Event::End(Tag::Paragraph) => {
            para_new = Some(pos);
            '\n'.into()
        }
        // Event::Code(x) => format!("\n{}\n", x).into(), // 行内代码
        _ => ' '.into(),
    };
    (pos, text, para_new)
}

pub fn append<'a>(event: Event<'a>, text: CowStr<'a>) -> [Event<'a>; 2] {
    match event {
        Event::End(Tag::Heading(_)) | Event::End(Tag::Paragraph) => {
            [event, Event::Text(text.into())]
        }
        _ => [event, Event::Text("".into())],
    }
}

/// 在每个标题后或者段落后插入
pub fn filter2(event: &Event) -> bool {
    match event {
        Event::End(Tag::Heading(_)) | Event::End(Tag::Paragraph) => true,
        _ => false,
    }
}
