use bilingual::md::Md;
use insta::assert_debug_snapshot;
use pulldown_cmark::{Event::*, Parser, Tag::*};

const LINKS: &str = r#"
[link text](http://dev.nodeca.com)

[link with title](http://nodeca.github.io/pica/demo/ "title text!")

Autoconverted link https://github.com/nodeca/pica

![Minion](https://octodex.github.com/images/minion.png)

![Stormtroopocat](https://octodex.github.com/images/stormtroopocat.jpg "The Stormtroopocat")

### [Emojies](https://github.com/markdown-it/markdown-it-emoji)

Footnote 1 link[^first].

[^first]: Footnote **can have markup**

[<img alt="github" src="https://img.shields.io/github/license/zjp-CN/bilingual?color=blue" height="20">](https://github.com/zjp-CN/bilingual)

[![](https://img.shields.io/crates/d/rustdx.svg?label=downloads+rustdx&style=social)](https://crates.io/crates/rustdx)
"#;

#[test]
fn links_test() {
    let events = Parser::new_ext(LINKS, bilingual::md::cmark_opt()).collect::<Vec<_>>();
    let link = &mut false;
    let text = events.iter()
                     .filter_map(|e| match e {
                         Start(Link(_, _, _) | Image(_, _, _)) => {
                             *link = true;
                             None
                         }
                         End(Link(_, _, _) | Image(_, _, _)) => {
                             *link = true;
                             None
                         }
                         Text(s) if *link => Some(s.as_ref()),
                         _ => None,
                     })
                     .collect::<Vec<_>>();
    assert_debug_snapshot!(text, @r###"
    [
        "link text",
        "link with title",
        "Autoconverted link https://github.com/nodeca/pica",
        "Minion",
        "Stormtroopocat",
        "Emojies",
        "Footnote 1 link",
        ".",
        "Footnote ",
        "can have markup",
    ]
    "###);
    assert_debug_snapshot!(Md::new(LINKS).extract(), @r###""link text\nlink with title\nAutoconverted link https://github.com/nodeca/pica\nMinion\nStormtroopocat\nEmojies\nFootnote 1 link.\nFootnote can have markup\n\n\n""###);
    assert_debug_snapshot!(events, @r###"
    [
        Start(
            Paragraph,
        ),
        Start(
            Link(
                Inline,
                Borrowed(
                    "http://dev.nodeca.com",
                ),
                Borrowed(
                    "",
                ),
            ),
        ),
        Text(
            Borrowed(
                "link text",
            ),
        ),
        End(
            Link(
                Inline,
                Borrowed(
                    "http://dev.nodeca.com",
                ),
                Borrowed(
                    "",
                ),
            ),
        ),
        End(
            Paragraph,
        ),
        Start(
            Paragraph,
        ),
        Start(
            Link(
                Inline,
                Borrowed(
                    "http://nodeca.github.io/pica/demo/",
                ),
                Inlined(
                    InlineStr {
                        inner: [
                            116,
                            105,
                            116,
                            108,
                            101,
                            32,
                            116,
                            101,
                            120,
                            116,
                            33,
                            0,
                            0,
                            0,
                            0,
                            0,
                            0,
                            0,
                            0,
                            0,
                            0,
                            0,
                            11,
                        ],
                    },
                ),
            ),
        ),
        Text(
            Borrowed(
                "link with title",
            ),
        ),
        End(
            Link(
                Inline,
                Borrowed(
                    "http://nodeca.github.io/pica/demo/",
                ),
                Inlined(
                    InlineStr {
                        inner: [
                            116,
                            105,
                            116,
                            108,
                            101,
                            32,
                            116,
                            101,
                            120,
                            116,
                            33,
                            0,
                            0,
                            0,
                            0,
                            0,
                            0,
                            0,
                            0,
                            0,
                            0,
                            0,
                            11,
                        ],
                    },
                ),
            ),
        ),
        End(
            Paragraph,
        ),
        Start(
            Paragraph,
        ),
        Text(
            Borrowed(
                "Autoconverted link https://github.com/nodeca/pica",
            ),
        ),
        End(
            Paragraph,
        ),
        Start(
            Paragraph,
        ),
        Start(
            Image(
                Inline,
                Borrowed(
                    "https://octodex.github.com/images/minion.png",
                ),
                Borrowed(
                    "",
                ),
            ),
        ),
        Text(
            Borrowed(
                "Minion",
            ),
        ),
        End(
            Image(
                Inline,
                Borrowed(
                    "https://octodex.github.com/images/minion.png",
                ),
                Borrowed(
                    "",
                ),
            ),
        ),
        End(
            Paragraph,
        ),
        Start(
            Paragraph,
        ),
        Start(
            Image(
                Inline,
                Borrowed(
                    "https://octodex.github.com/images/stormtroopocat.jpg",
                ),
                Inlined(
                    InlineStr {
                        inner: [
                            84,
                            104,
                            101,
                            32,
                            83,
                            116,
                            111,
                            114,
                            109,
                            116,
                            114,
                            111,
                            111,
                            112,
                            111,
                            99,
                            97,
                            116,
                            0,
                            0,
                            0,
                            0,
                            18,
                        ],
                    },
                ),
            ),
        ),
        Text(
            Borrowed(
                "Stormtroopocat",
            ),
        ),
        End(
            Image(
                Inline,
                Borrowed(
                    "https://octodex.github.com/images/stormtroopocat.jpg",
                ),
                Inlined(
                    InlineStr {
                        inner: [
                            84,
                            104,
                            101,
                            32,
                            83,
                            116,
                            111,
                            114,
                            109,
                            116,
                            114,
                            111,
                            111,
                            112,
                            111,
                            99,
                            97,
                            116,
                            0,
                            0,
                            0,
                            0,
                            18,
                        ],
                    },
                ),
            ),
        ),
        End(
            Paragraph,
        ),
        Start(
            Heading(
                3,
            ),
        ),
        Start(
            Link(
                Inline,
                Borrowed(
                    "https://github.com/markdown-it/markdown-it-emoji",
                ),
                Borrowed(
                    "",
                ),
            ),
        ),
        Text(
            Borrowed(
                "Emojies",
            ),
        ),
        End(
            Link(
                Inline,
                Borrowed(
                    "https://github.com/markdown-it/markdown-it-emoji",
                ),
                Borrowed(
                    "",
                ),
            ),
        ),
        End(
            Heading(
                3,
            ),
        ),
        Start(
            Paragraph,
        ),
        Text(
            Borrowed(
                "Footnote 1 link",
            ),
        ),
        FootnoteReference(
            Borrowed(
                "first",
            ),
        ),
        Text(
            Borrowed(
                ".",
            ),
        ),
        End(
            Paragraph,
        ),
        Start(
            FootnoteDefinition(
                Borrowed(
                    "first",
                ),
            ),
        ),
        Start(
            Paragraph,
        ),
        Text(
            Borrowed(
                "Footnote ",
            ),
        ),
        Start(
            Strong,
        ),
        Text(
            Borrowed(
                "can have markup",
            ),
        ),
        End(
            Strong,
        ),
        End(
            Paragraph,
        ),
        End(
            FootnoteDefinition(
                Borrowed(
                    "first",
                ),
            ),
        ),
        Start(
            Paragraph,
        ),
        Start(
            Link(
                Inline,
                Borrowed(
                    "https://github.com/zjp-CN/bilingual",
                ),
                Borrowed(
                    "",
                ),
            ),
        ),
        Html(
            Borrowed(
                "<img alt=\"github\" src=\"https://img.shields.io/github/license/zjp-CN/bilingual?color=blue\" height=\"20\">",
            ),
        ),
        End(
            Link(
                Inline,
                Borrowed(
                    "https://github.com/zjp-CN/bilingual",
                ),
                Borrowed(
                    "",
                ),
            ),
        ),
        End(
            Paragraph,
        ),
        Start(
            Paragraph,
        ),
        Start(
            Link(
                Inline,
                Borrowed(
                    "https://crates.io/crates/rustdx",
                ),
                Borrowed(
                    "",
                ),
            ),
        ),
        Start(
            Image(
                Inline,
                Borrowed(
                    "https://img.shields.io/crates/d/rustdx.svg?label=downloads+rustdx&style=social",
                ),
                Borrowed(
                    "",
                ),
            ),
        ),
        End(
            Image(
                Inline,
                Borrowed(
                    "https://img.shields.io/crates/d/rustdx.svg?label=downloads+rustdx&style=social",
                ),
                Borrowed(
                    "",
                ),
            ),
        ),
        End(
            Link(
                Inline,
                Borrowed(
                    "https://crates.io/crates/rustdx",
                ),
                Borrowed(
                    "",
                ),
            ),
        ),
        End(
            Paragraph,
        ),
    ]
    "###);
}
