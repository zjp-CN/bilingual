use bilingual::md::*;
use insta::{assert_debug_snapshot, assert_display_snapshot};
use pulldown_cmark::{CowStr, Event, Parser, Tag};
use pulldown_cmark_to_cmark::cmark;

/// 初步排除不需要的 Event；可能废弃
pub fn filter_text(event: &Event) -> bool {
    match event {
        Event::Text(_) => true,
        // 排除行间代码
        _ => false,
    }
}

#[test]
fn base() {
    let md = "# level one
one paragraph `Inline code`
```RUST
code block
```

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
        End(
            Paragraph,
        ),
        Start(
            CodeBlock(
                Fenced(
                    Borrowed(
                        "RUST",
                    ),
                ),
            ),
        ),
        Text(
            Borrowed(
                "code block\n",
            ),
        ),
        End(
            CodeBlock(
                Fenced(
                    Borrowed(
                        "RUST",
                    ),
                ),
            ),
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
                "code block\n",
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
    let mut select = true;
    events.iter().map(|event| extract(event, &mut &mut select, &mut buf)).last();
    assert_display_snapshot!(buf, @r###"
    level one
    one paragraph `Inline code`
    quote block
    A
    "###);

    let mut paragraphs = buf.split('\n');
    let output = events.into_iter().map(|event| prepend(event, &mut paragraphs)).flatten();
    let mut output_md = String::with_capacity(capacity * 2);
    cmark(output, &mut output_md, None).unwrap();
    assert_display_snapshot!(output_md, @r###"
    # level one
    level one

    one paragraph `Inline code`
    one paragraph `Inline code`

    ````RUST
    code block
    ````

     > 
     > quote block
     > quote block

    <a>A</a>
    A
    "###);
}

#[test]
fn size() {
    use std::mem::size_of;
    assert_debug_snapshot!(size_of::<Option<Event>>(), @"64");
    assert_debug_snapshot!(size_of::<Option<CowStr>>(), @"24");
    assert_debug_snapshot!(size_of::<Option<Tag>>(), @"56");
}
