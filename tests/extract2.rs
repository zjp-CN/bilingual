use insta::assert_display_snapshot;
use pulldown_cmark::{
    Event::{self, *},
    Options, Parser,
    Tag::*,
};
use pulldown_cmark_to_cmark::cmark;

#[test]
fn test1() {
    let md = "# level one
one paragraph `Inline code`
        Indented code block

> quote block

<a>A</a>";
    let capacity = md.len();
    let events: Vec<_> = Parser::new_ext(&md, cmark_opt()).collect();
    let mut buf = String::with_capacity(capacity);

    events.iter().map(|event| extract(event, &mut buf)).last();

    let mut paragraphs = buf.split('\n');
    let output =
        events.into_iter()
              .map(|event| match event {
                  End(Paragraph | Heading(_)) => {
                      [Some(SoftBreak), Some(Text(paragraphs.next().unwrap().into())), Some(event)]
                  }
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

pub fn append<'a, 'b: 'a>(event: Event<'a>, paragraphs: &'a mut impl Iterator<Item = &'b str>)
                          -> [Option<Event<'a>>; 3] {
    match event {
        End(Paragraph | Heading(_)) => {
            [Some(SoftBreak), Some(Text(paragraphs.next().unwrap().into())), Some(event)]
        }
        _ => [Some(event), None, None],
    }
}

#[test]
fn test2() {
    let md = "# level one
one paragraph `Inline code`
        Indented code block

> quote block

<a>A</a>";
    let capacity = md.len();
    let events: Vec<_> = Parser::new_ext(&md, cmark_opt()).collect();
    let mut buf = String::with_capacity(capacity);

    events.iter().map(|event| extract(event, &mut buf)).last();

    let mut paragraphs = buf.split('\n');
    let output = events.into_iter()
                       .map(|event| append(event, &mut paragraphs))
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

pub fn cmark_opt() -> Options {
    let mut options = Options::all();
    options.remove(Options::ENABLE_SMART_PUNCTUATION);
    options
}

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
