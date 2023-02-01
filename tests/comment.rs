use bilingual::md::Md;
use insta::{assert_debug_snapshot, assert_display_snapshot};
use pulldown_cmark::{Event, Parser};

pub struct MdComment<'t> {
    text:     &'t str,
    language: String,
    cm:       Vec<comment::CommentMatch>,
}

impl<'t> MdComment<'t> {
    fn new(text: &'t str, l: &str) -> Self {
        Self { text,
               language: l[..l.find(',').unwrap_or(l.len())].to_lowercase(),
               cm: Vec::new() }
    }

    fn matches(&mut self) -> Result<(String, usize), &'static str> {
        match self.language.as_bytes() {
            b"c" | b"c++" | b"cpp" | b"rust" => {
                self.cm = comment::c::find_comments(self.text)?;
                Ok((self.trim_c(), self.cm.len()))
            }
            _ => unimplemented!(),
        }
    }

    // 去掉开头的 `/` 和之后首尾的空格：适合于 C 系注释
    fn trim_c(&self) -> String {
        self.cm
            .iter()
            .map(|m| {
                unsafe { self.text.get_unchecked(m.from..m.to) }.trim_start_matches('/').trim()
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn insert<'s: 't>(&self, translation: impl Iterator<Item = &'s str>) -> String {
        let mut buf = String::with_capacity(self.text.len() * 2);
        buf.push_str(self.text);
        let pos = &mut 0;
        self.cm
            .iter()
            .map(|m| m.to)
            .zip(translation)
            .for_each(|(t, s)| {
                // TODO: 不足
                // 1. 目前只针对 C 系语言注释，且主要识别 `//` 和 `/*` 开头的注释
                // 2. 翻译内容统一另起一行，以 `// ` 注释插入
                const NEWLINE: &str = "\n// ";
                const LEN: usize = NEWLINE.len();
                buf.insert_str(t + *pos, NEWLINE);
                buf.insert_str(t + LEN + std::mem::replace(pos, *pos + LEN + s.len()), s);
            });
        buf
    }
}

const CODEBLOCK: &str = "
```rust, ignored
//! module doc comment

/// doc comment
fn process_epoll_events(&mut self, event_id: usize) {
    // hidden comment
    self.callbacks_to_run.push((event_id, Js::Undefined) /* inline comment */ );
    self.epoll_pending_events -= 1;
}
```
";

#[test]
fn comment_test() {
    let mut md = Md::new(CODEBLOCK);
    let text = md.extract().to_owned();
    assert_debug_snapshot!(text, @r###""""###);

    let events = Parser::new_ext(CODEBLOCK, bilingual::md::cmark_opt()).collect::<Vec<_>>();
    let text = match &events[1] {
        Event::Text(x) => x.as_ref(),
        _ => "",
    };
    assert_debug_snapshot!(comment::c::find_comments(text).unwrap(), @r###"
    [
        CommentMatch {
            from: 0,
            to: 22,
        },
        CommentMatch {
            from: 24,
            to: 39,
        },
        CommentMatch {
            from: 98,
            to: 115,
        },
        CommentMatch {
            from: 173,
            to: 193,
        },
    ]
    "###);

    let mut mdcomment = MdComment::new(text, "rust");
    let (matched_text, matched_cnt) = mdcomment.matches().unwrap();
    assert_debug_snapshot!(matched_text, @r###""! module doc comment\ndoc comment\nhidden comment\n* inline comment */""###);
    assert_display_snapshot!(
    mdcomment.insert(["模块文档注释","文档注释","隐藏的注释", "行内注释"].iter().copied().take(matched_cnt)), 
    @r###"
    //! module doc comment
    // 模块文档注释

    /// doc comment
    // 文档注释
    fn process_epoll_events(&mut self, event_id: usize) {
        // hidden comment
    // 隐藏的注释
        self.callbacks_to_run.push((event_id, Js::Undefined) /* inline comment */
    // 行内注释 );
        self.epoll_pending_events -= 1;
    }
    "###);
    assert_debug_snapshot!(events, @r###"
    [
        Start(
            CodeBlock(
                Fenced(
                    Borrowed(
                        "rust, ignored",
                    ),
                ),
            ),
        ),
        Text(
            Borrowed(
                "//! module doc comment\n\n/// doc comment\nfn process_epoll_events(&mut self, event_id: usize) {\n    // hidden comment\n    self.callbacks_to_run.push((event_id, Js::Undefined) /* inline comment */ );\n    self.epoll_pending_events -= 1;\n}\n",
            ),
        ),
        End(
            CodeBlock(
                Fenced(
                    Borrowed(
                        "rust, ignored",
                    ),
                ),
            ),
        ),
    ]
    "###);
}
