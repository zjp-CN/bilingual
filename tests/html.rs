use insta::assert_debug_snapshot;
use std::borrow::Cow;
use tl::{Parser, *};

fn parse(s: &str) -> VDom { tl::parse(s, tl::ParserOptions::default()).unwrap() }

#[test]
fn simple_test() {
    let dom = parse("<p>hi </p>");
    assert_debug_snapshot!(dom.nodes().iter().filter_map(|n| n.as_tag()).collect::<Vec<_>>(), @r###"
    [
        HTMLTag {
            _name: Some(
                Bytes(
                    "p",
                ),
            ),
            _attributes: Attributes {
                raw: {},
                id: None,
                class: None,
            },
            _children: [
                Raw(
                    Bytes(
                        "hi ",
                    ),
                ),
            ],
            _raw: Bytes(
                "<p>hi </p>",
            ),
        },
    ]
    "###);
    let parser = dom.parser();
    let tags = dom.nodes()
                  .iter()
                  .filter_map(|n| match n {
                      Node::Tag(t) => {
                          if t.name() == &Bytes::from("p") {
                              Some(t.inner_text(parser))
                          } else {
                              None
                          }
                      }
                      _ => None,
                  })
                  .collect::<Vec<_>>();
    assert_debug_snapshot!(tags, @r###"
    [
        "hi ",
    ]
    "###);
}

pub fn filter_script<'a>(dom: &'a VDom<'a>) -> Vec<Cow<'a, str>> {
    dom.nodes().iter().map(|n| inner_text(n, dom.parser())).collect()
}

// 排除掉 script 标签
pub fn inner_text_filter_script<'a, 'b: 'a>(tag: &HTMLTag<'a>, parser: &'b Parser<'b>)
                                            -> Cow<'a, str> {
    fn script(tag: &HTMLTag) -> bool { tag.name() == &Bytes::from("script") }
    fn pushdown(node: &Node, s: &mut String, parser: &Parser) {
        match node {
            Node::Tag(t) if !script(t) => {
                let text = inner_text_filter_script(t, parser);
                if !text.is_empty() {
                    s.push_str(&text);
                    s.push(' ');
                }
            }
            Node::Raw(e) => s.push_str(&e.as_utf8_str()),
            _ => { /* no op */ }
        }
    }

    let children = tag.children();
    let ch = children.top();
    let len = ch.len();

    if len == 0 {
        // If there are no subnodes, we can just return a static, empty, string slice
        return Cow::Borrowed("");
    }

    let first = &ch[0];

    if len == 1 {
        match first.get(parser) {
            Some(Node::Tag(t)) if !script(t) => return inner_text_filter_script(t, parser),
            Some(Node::Raw(e)) => return e.as_utf8_str(),
            _ => return Cow::Borrowed(""),
        }
    }

    // If there are >1 nodes, we need to allocate a new string and push each inner_text in it
    // TODO: check if String::with_capacity() is worth it
    let mut s =
        String::from(first.get(parser).map(|node| inner_text(node, parser)).unwrap_or_default());

    tag.children()
       .all(parser)
       .iter()
       .skip(1)
       .for_each(|node| pushdown(node, &mut s, parser));

    Cow::Owned(s)
}

pub fn inner_text<'a, 'b: 'a>(node: &'a Node<'a>, parser: &'b Parser<'b>) -> Cow<'a, str> {
    match node {
        Node::Comment(_) => Cow::Borrowed(""),
        Node::Raw(r) => r.as_utf8_str(),
        Node::Tag(t) => inner_text_filter_script(t, parser),
    }
}

#[test]
fn html_text_test() {
    let html = &std::fs::read_to_string("assets/markdown-it.html").unwrap();
    let dom = parse(html);
    assert_debug_snapshot!(filter_script(&dom), @r###"
    [
        "markdown-it demomarkdown-it demo html\n          xhtmlOut\n           breaks\n           linkify\n           typographer\n           highlight\n           CommonMark strict\n            clearpermalink ---\n__Advertisement :)__\n\n- __[pica](https://nodeca.github.io/pica/demo/)__ - high quality and fast image\n  resize in browser.\n- __[babelfish](https://github.com/nodeca/babelfish/)__ - developer friendly\n  i18n with plurals support and easy syntax.\n\nYou will like those projects!\n\n---\n\n# h1 Heading 8-)\n## h2 Heading\n### h3 Heading\n#### h4 Heading\n##### h5 Heading\n###### h6 Heading\n\n\n## Horizontal Rules\n\n___\n\n---\n\n***\n\n\n## Typographic replacements\n\nEnable typographer option to see result.\n\n(c) (C) (r) (R) (tm) (TM) (p) (P) +-\n\ntest.. test... test..... test?..... test!....\n\n!!!!!! ???? ,,  -- ---\n\n&quot;Smartypants, double quotes&quot; and 'single quotes'\n\n\n## Emphasis\n\n**This is bold text**\n\n__This is bold text__\n\n*This is italic text*\n\n_This is italic text_\n\n~~Strikethrough~~\n\n\n## Blockquotes\n\n\n&gt; Blockquotes can also be nested...\n&gt;&gt; ...by using additional greater-than signs right next to each other...\n&gt; &gt; &gt; ...or with spaces between arrows.\n\n\n## Lists\n\nUnordered\n\n+ Create a list by starting a line with `+`, `-`, or `*`\n+ Sub-lists are made by indenting 2 spaces:\n  - Marker character change forces new list start:\n    * Ac tristique libero volutpat at\n    + Facilisis in pretium nisl aliquet\n    - Nulla volutpat aliquam velit\n+ Very easy!\n\nOrdered\n\n1. Lorem ipsum dolor sit amet\n2. Consectetur adipiscing elit\n3. Integer molestie lorem at massa\n\n\n1. You can use sequential numbers...\n1. ...or keep all the numbers as `1.`\n\nStart numbering with offset:\n\n57. foo\n1. bar\n\n\n## Code\n\nInline `code`\n\nIndented code\n\n    // Some comments\n    line 1 of code\n    line 2 of code\n    line 3 of code\n\n\nBlock code &quot;fences&quot;\n\n```\nSample text here...\n```\n\nSyntax highlighting\n\n``` js\nvar foo = function (bar) {\n  return bar++;\n};\n\nconsole.log(foo(5));\n```\n\n## Tables\n\n| Option | Description |\n| ------ | ----------- |\n| data   | path to data files to supply the data that will be passed into templates. |\n| engine | engine to be used for processing templates. Handlebars is the default. |\n| ext    | extension to be used for dest files. |\n\nRight aligned columns\n\n| Option | Description |\n| ------:| -----------:|\n| data   | path to data files to supply the data that will be passed into templates. |\n| engine | engine to be used for processing templates. Handlebars is the default. |\n| ext    | extension to be used for dest files. |\n\n\n## Links\n\n[link text](http://dev.nodeca.com)\n\n[link with title](http://nodeca.github.io/pica/demo/ &quot;title text!&quot;)\n\nAutoconverted link https://github.com/nodeca/pica (enable linkify to see)\n\n\n## Images\n\n![Minion](https://octodex.github.com/images/minion.png)\n![Stormtroopocat](https://octodex.github.com/images/stormtroopocat.jpg &quot;The Stormtroopocat&quot;)\n\nLike links, Images also have a footnote style syntax\n\n![Alt text][id]\n\nWith a reference later in the document defining the URL location:\n\n[id]: https://octodex.github.com/images/dojocat.jpg  &quot;The Dojocat&quot;\n\n\n## Plugins\n\nThe killer feature of `markdown-it` is very effective support of\n[syntax plugins](https://www.npmjs.org/browse/keyword/markdown-it-plugin).\n\n\n### [Emojies](https://github.com/markdown-it/markdown-it-emoji)\n\n&gt; Classic markup: :wink: :crush: :cry: :tear: :laughing: :yum:\n&gt;\n&gt; Shortcuts (emoticons): :-) :-( 8-) ;)\n\nsee [how to change output](https://github.com/markdown-it/markdown-it-emoji#change-output) with twemoji.\n\n\n### [Subscript](https://github.com/markdown-it/markdown-it-sub) / [Superscript](https://github.com/markdown-it/markdown-it-sup)\n\n- 19^th^\n- H~2~O\n\n\n### [\\&lt;ins&gt;](https://github.com/markdown-it/markdown-it-ins)\n\n++Inserted text++\n\n\n### [\\&lt;mark&gt;](https://github.com/markdown-it/markdown-it-mark)\n\n==Marked text==\n\n\n### [Footnotes](https://github.com/markdown-it/markdown-it-footnote)\n\nFootnote 1 link[^first].\n\nFootnote 2 link[^second].\n\nInline footnote^[Text of inline footnote] definition.\n\nDuplicated footnote reference[^second].\n\n[^first]: Footnote **can have markup**\n\n    and multiple paragraphs.\n\n[^second]: Footnote text.\n\n\n### [Definition lists](https://github.com/markdown-it/markdown-it-deflist)\n\nTerm 1\n\n:   Definition 1\nwith lazy continuation.\n\nTerm 2 with *inline markup*\n\n:   Definition 2\n\n        { some code, part of Definition 2 }\n\n    Third paragraph of definition 2.\n\n_Compact style:_\n\nTerm 1\n  ~ Definition 1\n\nTerm 2\n  ~ Definition 2a\n  ~ Definition 2b\n\n\n### [Abbreviations](https://github.com/markdown-it/markdown-it-abbr)\n\nThis is HTML abbreviation example.\n\nIt converts &quot;HTML&quot;, but keep intact partial entries like &quot;xxxHTMLyyy&quot; and so on.\n\n*[HTML]: Hyper Text Markup Language\n\n### [Custom containers](https://github.com/markdown-it/markdown-it-container)\n\n::: warning\n*here be dragons*\n:::\n htmlsource debug   Fork me on GitHub  ",
    ]
    "###);
}

const SCRIPT: &str = r#"
  <body>
    <script src="index.js"></script>
    <!-- Ancient IE support - load shiv & kill broken highlighter--><!--[if lt IE 9]>
<script src="https://oss.maxcdn.com/html5shiv/3.7.2/html5shiv.min.js"></script>
<script>window.hljs = null;</script>
<![endif]-->
    <!-- GA counter-->
    <script>
      (function(i,s,o,g,r,a,m){i['GoogleAnalyticsObject']=r;i[r]=i[r]||function(){
      (i[r].q=i[r].q||[]).push(arguments)},i[r].l=1*new Date();a=s.createElement(o),
      m=s.getElementsByTagName(o)[0];a.async=1;a.src=g;m.parentNode.insertBefore(a,m)
      })(window,document,'script','//www.google-analytics.com/analytics.js','ga');
      
      ga('create', 'UA-26895916-4', 'auto');
      ga('require', 'displayfeatures');
      ga('require', 'linkid', 'linkid.js');
      ga('send', 'pageview');
      
    </script>
    <p>no scirpt from 1</p>
    <div class="container">
      <p>no scirpt from 2</p>
    </div>
  </body>
  <div class="container">
    <p>content outside body</p>
  </div>
"#;

#[test]
fn script_test() {
    let dom = parse(SCRIPT);
    assert_debug_snapshot!(filter_script(&dom), @r###"
    [
        "no scirpt from 1 no scirpt from 2 ",
        "content outside body",
    ]
    "###);
    assert_debug_snapshot!(dom.children(), @r###"
    [
        Tag(
            HTMLTag {
                _name: Some(
                    Bytes(
                        "body",
                    ),
                ),
                _attributes: Attributes {
                    raw: {},
                    id: None,
                    class: None,
                },
                _children: [
                    Tag(
                        HTMLTag {
                            _name: Some(
                                Bytes(
                                    "script",
                                ),
                            ),
                            _attributes: Attributes {
                                raw: {
                                    Bytes(
                                        "src",
                                    ): Some(
                                        Bytes(
                                            "index.js",
                                        ),
                                    ),
                                },
                                id: None,
                                class: None,
                            },
                            _children: [],
                            _raw: Bytes(
                                "<script src=\"index.js\"></script>",
                            ),
                        },
                    ),
                    Comment(
                        Bytes(
                            " Ancient IE support - load shiv & kill broken highlighter-->",
                        ),
                    ),
                    Comment(
                        Bytes(
                            "[if lt IE 9]>\n<script src=\"https://oss.maxcdn.com/html5shiv/3.7.2/html5shiv.min.js\"></script>\n<script>window.hljs = null;</script>\n<![endif]-->",
                        ),
                    ),
                    Comment(
                        Bytes(
                            " GA counter-->",
                        ),
                    ),
                    Tag(
                        HTMLTag {
                            _name: Some(
                                Bytes(
                                    "script",
                                ),
                            ),
                            _attributes: Attributes {
                                raw: {},
                                id: None,
                                class: None,
                            },
                            _children: [
                                Raw(
                                    Bytes(
                                        "(function(i,s,o,g,r,a,m){i['GoogleAnalyticsObject']=r;i[r]=i[r]||function(){\n      (i[r].q=i[r].q||[]).push(arguments)},i[r].l=1*new Date();a=s.createElement(o),\n      m=s.getElementsByTagName(o)[0];a.async=1;a.src=g;m.parentNode.insertBefore(a,m)\n      })(window,document,'script','//www.google-analytics.com/analytics.js','ga');\n      \n      ga('create', 'UA-26895916-4', 'auto');\n      ga('require', 'displayfeatures');\n      ga('require', 'linkid', 'linkid.js');\n      ga('send', 'pageview');\n      \n    ",
                                    ),
                                ),
                            ],
                            _raw: Bytes(
                                "<script>\n      (function(i,s,o,g,r,a,m){i['GoogleAnalyticsObject']=r;i[r]=i[r]||function(){\n      (i[r].q=i[r].q||[]).push(arguments)},i[r].l=1*new Date();a=s.createElement(o),\n      m=s.getElementsByTagName(o)[0];a.async=1;a.src=g;m.parentNode.insertBefore(a,m)\n      })(window,document,'script','//www.google-analytics.com/analytics.js','ga');\n      \n      ga('create', 'UA-26895916-4', 'auto');\n      ga('require', 'displayfeatures');\n      ga('require', 'linkid', 'linkid.js');\n      ga('send', 'pageview');\n      \n    </script>",
                            ),
                        },
                    ),
                    Tag(
                        HTMLTag {
                            _name: Some(
                                Bytes(
                                    "p",
                                ),
                            ),
                            _attributes: Attributes {
                                raw: {},
                                id: None,
                                class: None,
                            },
                            _children: [
                                Raw(
                                    Bytes(
                                        "no scirpt from 1",
                                    ),
                                ),
                            ],
                            _raw: Bytes(
                                "<p>no scirpt from 1</p>",
                            ),
                        },
                    ),
                    Tag(
                        HTMLTag {
                            _name: Some(
                                Bytes(
                                    "div",
                                ),
                            ),
                            _attributes: Attributes {
                                raw: {
                                    Bytes(
                                        "class",
                                    ): Some(
                                        Bytes(
                                            "container",
                                        ),
                                    ),
                                },
                                id: None,
                                class: Some(
                                    Bytes(
                                        "container",
                                    ),
                                ),
                            },
                            _children: [
                                Tag(
                                    HTMLTag {
                                        _name: Some(
                                            Bytes(
                                                "p",
                                            ),
                                        ),
                                        _attributes: Attributes {
                                            raw: {},
                                            id: None,
                                            class: None,
                                        },
                                        _children: [
                                            Raw(
                                                Bytes(
                                                    "no scirpt from 2",
                                                ),
                                            ),
                                        ],
                                        _raw: Bytes(
                                            "<p>no scirpt from 2</p>",
                                        ),
                                    },
                                ),
                            ],
                            _raw: Bytes(
                                "<div class=\"container\">\n      <p>no scirpt from 2</p>\n    </div>",
                            ),
                        },
                    ),
                ],
                _raw: Bytes(
                    "<body>\n    <script src=\"index.js\"></script>\n    <!-- Ancient IE support - load shiv & kill broken highlighter--><!--[if lt IE 9]>\n<script src=\"https://oss.maxcdn.com/html5shiv/3.7.2/html5shiv.min.js\"></script>\n<script>window.hljs = null;</script>\n<![endif]-->\n    <!-- GA counter-->\n    <script>\n      (function(i,s,o,g,r,a,m){i['GoogleAnalyticsObject']=r;i[r]=i[r]||function(){\n      (i[r].q=i[r].q||[]).push(arguments)},i[r].l=1*new Date();a=s.createElement(o),\n      m=s.getElementsByTagName(o)[0];a.async=1;a.src=g;m.parentNode.insertBefore(a,m)\n      })(window,document,'script','//www.google-analytics.com/analytics.js','ga');\n      \n      ga('create', 'UA-26895916-4', 'auto');\n      ga('require', 'displayfeatures');\n      ga('require', 'linkid', 'linkid.js');\n      ga('send', 'pageview');\n      \n    </script>\n    <p>no scirpt from 1</p>\n    <div class=\"container\">\n      <p>no scirpt from 2</p>\n    </div>\n  </body>",
                ),
            },
        ),
        Tag(
            HTMLTag {
                _name: Some(
                    Bytes(
                        "div",
                    ),
                ),
                _attributes: Attributes {
                    raw: {
                        Bytes(
                            "class",
                        ): Some(
                            Bytes(
                                "container",
                            ),
                        ),
                    },
                    id: None,
                    class: Some(
                        Bytes(
                            "container",
                        ),
                    ),
                },
                _children: [
                    Tag(
                        HTMLTag {
                            _name: Some(
                                Bytes(
                                    "p",
                                ),
                            ),
                            _attributes: Attributes {
                                raw: {},
                                id: None,
                                class: None,
                            },
                            _children: [
                                Raw(
                                    Bytes(
                                        "content outside body",
                                    ),
                                ),
                            ],
                            _raw: Bytes(
                                "<p>content outside body</p>",
                            ),
                        },
                    ),
                ],
                _raw: Bytes(
                    "<div class=\"container\">\n    <p>content outside body</p>\n  </div>",
                ),
            },
        ),
    ]
    "###);
}
