use insta::{assert_debug_snapshot, assert_display_snapshot};
use pulldown_cmark::{
    Event::{self, *},
    Options, Parser,
    Tag::*,
};
use pulldown_cmark_to_cmark::cmark;

pub fn cmark_opt() -> Options {
    let mut options = Options::all();
    options.remove(Options::ENABLE_SMART_PUNCTUATION);
    options
}

#[test]
fn title() {
    let md = "# level one
one paragraph `Inline code`
        Indented code block

> quote block

<a>A</a>";
    let capacity = md.len();
    let events: Vec<_> = Parser::new_ext(&md, cmark_opt()).collect();
    assert_debug_snapshot!(events, @r###"
    [
        Start(
            Heading(
                1,
            ),
        ),
        Text(
            Borrowed(
                "level one",
            ),
        ),
        End(
            Heading(
                1,
            ),
        ),
        Start(
            Paragraph,
        ),
        Text(
            Borrowed(
                "one paragraph ",
            ),
        ),
        Code(
            Borrowed(
                "Inline code",
            ),
        ),
        SoftBreak,
        Text(
            Borrowed(
                "Indented code block",
            ),
        ),
        End(
            Paragraph,
        ),
        Start(
            BlockQuote,
        ),
        Start(
            Paragraph,
        ),
        Text(
            Borrowed(
                "quote block",
            ),
        ),
        End(
            Paragraph,
        ),
        End(
            BlockQuote,
        ),
        Start(
            Paragraph,
        ),
        Html(
            Borrowed(
                "<a>",
            ),
        ),
        Text(
            Borrowed(
                "A",
            ),
        ),
        Html(
            Borrowed(
                "</a>",
            ),
        ),
        End(
            Paragraph,
        ),
    ]
    "###);

    let content = events.clone().into_iter().filter(|event| filter_text(event));
    assert_debug_snapshot!(content.collect::<Vec<_>>(), @r###"
    [
        Text(
            Borrowed(
                "level one",
            ),
        ),
        Text(
            Borrowed(
                "one paragraph ",
            ),
        ),
        Text(
            Borrowed(
                "Indented code block",
            ),
        ),
        Text(
            Borrowed(
                "quote block",
            ),
        ),
        Text(
            Borrowed(
                "A",
            ),
        ),
    ]
    "###);

    let mut buf = String::with_capacity(capacity);
    events.iter().map(|event| extract(event, &mut buf)).last();
    assert_display_snapshot!(buf, @r###"
    level one
    one paragraph `Inline code` Indented code block
    quote block
    A
    "###);

    let mut split = buf.split('\n');
    let output =
        events.into_iter()
              .map(|event| match event {
                  End(Paragraph | Heading(_)) => [Some(Text('\n'.into())),
                                                  Some(Text(split.next().unwrap().into())),
                                                  Some(event)],
                  _ => [Some(event), None, None],
              })
              .flatten()
              .flatten();
    let mut output_md = String::with_capacity(capacity * 2);
    cmark(output, &mut output_md, None).unwrap();
    assert_display_snapshot!(output_md, @r###"
    # level one
    level one

    one paragraph `Inline code`
    Indented code block
    one paragraph `Inline code` Indented code block

     > 
     > quote block
     > quote block

    <a>A</a>
    A
    "###);
}

/// 初步排除不需要的 Event
pub fn filter_text(event: &Event) -> bool {
    match event {
        Text(_) => true,
        // 排除行间代码
        _ => false,
    }
}

/// 取出需要被翻译的内容：按照段落或标题
pub fn extract(event: &Event, buf: &mut String) {
    match event {
        End(Paragraph | Heading(_)) => buf.push('\n'),
        Text(x) => buf.push_str(x.as_ref()),
        SoftBreak | HardBreak => buf.push(' '),
        Code(x) => {
            buf.push('`');
            buf.push_str(x.as_ref());
            buf.push('`');
        }
        _ => (),
    }
}
