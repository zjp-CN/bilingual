use bilingual::md::*;
use insta::{assert_debug_snapshot, assert_display_snapshot};
use pulldown_cmark::{CowStr, Event, Parser, Tag};
use pulldown_cmark_to_cmark::cmark;

/// 初步排除不需要的 Event；可能废弃
pub fn filter_text(event: &Event) -> bool { matches!(event, Event::Text(_)) }

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
    let events: Vec<_> = Parser::new_ext(md, cmark_opt()).collect();
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

    let content = events.clone().into_iter().filter(filter_text);
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
    let select = &mut true;
    let table = &mut false;
    events.iter().map(|event| extract(event, select, table, &mut buf)).last();
    assert_display_snapshot!(buf, @r###"
    level one
    one paragraph `Inline code`
    quote block
    A
    "###);

    let mut paragraphs = buf.split('\n');
    let table = &mut false;
    let output = events.into_iter().flat_map(|event| prepend(event, table, &mut paragraphs));
    let mut output_md = String::with_capacity(capacity * 2);
    cmark(output, &mut output_md).unwrap();
    assert_display_snapshot!(output_md, @r###"
    # level one

    # level one

    one paragraph `Inline code`

    one paragraph `Inline code`

    ````RUST
    code block
    ````

     > 
     > quote block
     > 
     > quote block

    <a>A</a>

    A
    "###);
}

#[test]
fn size() {
    use std::mem::size_of;
    assert_debug_snapshot!(size_of::<Option<Event>>(),  @"64");
    assert_debug_snapshot!(size_of::<Option<CowStr>>(), @"24");
    assert_debug_snapshot!(size_of::<Option<Tag>>(),    @"56");

    assert_debug_snapshot!(size_of::<Md>(),             @"128");
    assert_debug_snapshot!(size_of::<Vec<usize>>(),     @"24");
    assert_debug_snapshot!(size_of::<String>(),         @"24");
    assert_debug_snapshot!(size_of::<Box<[usize]>>(),   @"16");
}

#[rustfmt::skip]
static MD: &str = "
# I/O event queue

We add the `callback_id` to the collection of callbacks to run. We pass
in `Js::Undefined` since we'll not actually pass any data along here. You'll see
why when we reach the [Http module](./8_3_http_module.md) chapter, but the main
point is that the I/O queue doesn't return any data itself, it just tells us that
data is ready to be read.

```rust, ignored
fn process_epoll_events(&mut self, event_id: usize) {
    self.callbacks_to_run.push((event_id, Js::Undefined));
    self.epoll_pending_events -= 1;
}
```


Hi!
你好！这里是中文！



> Hi!
> **Why even keep track of how many `epoll_events` are pending?**
> We don't use this value here, but I added it to make it easier to create
> some `print` statements showing the status of our runtime at different points.
> However, there are good reasons to keep track of these events even if we don't use them.
>
> One area we're taking shortcuts on all the way here is security. If someone were
> to build a public facing server out of this, we need to account for slow networks
> and malicious users.
";

#[test]
fn batch_paragraph() {
    let mut md = Md::new(MD);

    // 极端情况
    assert_debug_snapshot!(md.chars_paragraph(0).collect::<Vec<_>>(), @r###"
    [
        "I/O event queue\n",
        "We add the `callback_id` to the collection of callbacks to run. We pass in `Js::Undefined` since we'll not actually pass any data along here. You'll see why when we reach the Http module chapter, but the main point is that the I/O queue doesn't return any data itself, it just tells us that data is ready to be read.\n",
        "Hi! 你好！这里是中文！\n",
        "Hi! Why even keep track of how many `epoll_events` are pending? We don't use this value here, but I added it to make it easier to create some `print` statements showing the status of our runtime at different points. However, there are good reasons to keep track of these events even if we don't use them.\n",
        "One area we're taking shortcuts on all the way here is security. If someone were to build a public facing server out of this, we need to account for slow networks and malicious users.\n",
    ]
    "###);
    assert_debug_snapshot!(md.bytes_paragraph(0).collect::<Vec<_>>(), @r###"
    [
        "I/O event queue\n",
        "We add the `callback_id` to the collection of callbacks to run. We pass in `Js::Undefined` since we'll not actually pass any data along here. You'll see why when we reach the Http module chapter, but the main point is that the I/O queue doesn't return any data itself, it just tells us that data is ready to be read.\n",
        "Hi! 你好！这里是中文！\n",
        "Hi! Why even keep track of how many `epoll_events` are pending? We don't use this value here, but I added it to make it easier to create some `print` statements showing the status of our runtime at different points. However, there are good reasons to keep track of these events even if we don't use them.\n",
        "One area we're taking shortcuts on all the way here is security. If someone were to build a public facing server out of this, we need to account for slow networks and malicious users.\n",
    ]
    "###);

    assert_display_snapshot!(md.extract().len(), @"854"); // 段落文本总字节数
    assert_debug_snapshot!(md.chars_bytes_range().collect::<Vec<_>>(), @r###"
    [
        (
            16,
            16,
            0..16,
        ),
        (
            317,
            317,
            16..333,
        ),
        (
            14,
            32,
            333..365,
        ),
        (
            305,
            305,
            365..670,
        ),
        (
            184,
            184,
            670..854,
        ),
    ]
    "###);

    assert_debug_snapshot!(md.bytes_paragraph(1 << 10).collect::<Vec<_>>(), @r###"
    [
        "I/O event queue\nWe add the `callback_id` to the collection of callbacks to run. We pass in `Js::Undefined` since we'll not actually pass any data along here. You'll see why when we reach the Http module chapter, but the main point is that the I/O queue doesn't return any data itself, it just tells us that data is ready to be read.\nHi! 你好！这里是中文！\nHi! Why even keep track of how many `epoll_events` are pending? We don't use this value here, but I added it to make it easier to create some `print` statements showing the status of our runtime at different points. However, there are good reasons to keep track of these events even if we don't use them.\nOne area we're taking shortcuts on all the way here is security. If someone were to build a public facing server out of this, we need to account for slow networks and malicious users.\n",
    ]
    "###);
    assert_debug_snapshot!(md.bytes_paragraph(400).collect::<Vec<_>>(), @r###"
    [
        "I/O event queue\nWe add the `callback_id` to the collection of callbacks to run. We pass in `Js::Undefined` since we'll not actually pass any data along here. You'll see why when we reach the Http module chapter, but the main point is that the I/O queue doesn't return any data itself, it just tells us that data is ready to be read.\nHi! 你好！这里是中文！\n",
        "Hi! Why even keep track of how many `epoll_events` are pending? We don't use this value here, but I added it to make it easier to create some `print` statements showing the status of our runtime at different points. However, there are good reasons to keep track of these events even if we don't use them.\n",
        "One area we're taking shortcuts on all the way here is security. If someone were to build a public facing server out of this, we need to account for slow networks and malicious users.\n",
    ]
    "###);
    assert_debug_snapshot!(md.bytes_paragraph(16).collect::<Vec<_>>(), @r###"
    [
        "I/O event queue\n",
        "We add the `callback_id` to the collection of callbacks to run. We pass in `Js::Undefined` since we'll not actually pass any data along here. You'll see why when we reach the Http module chapter, but the main point is that the I/O queue doesn't return any data itself, it just tells us that data is ready to be read.\n",
        "Hi! 你好！这里是中文！\n",
        "Hi! Why even keep track of how many `epoll_events` are pending? We don't use this value here, but I added it to make it easier to create some `print` statements showing the status of our runtime at different points. However, there are good reasons to keep track of these events even if we don't use them.\n",
        "One area we're taking shortcuts on all the way here is security. If someone were to build a public facing server out of this, we need to account for slow networks and malicious users.\n",
    ]
    "###);

    assert_display_snapshot!(md.chars().sum::<usize>(), @"836"); // 段落文本总字符数
    assert_debug_snapshot!(md.chars_paragraph(347).collect::<Vec<_>>(), @r###"
    [
        "I/O event queue\nWe add the `callback_id` to the collection of callbacks to run. We pass in `Js::Undefined` since we'll not actually pass any data along here. You'll see why when we reach the Http module chapter, but the main point is that the I/O queue doesn't return any data itself, it just tells us that data is ready to be read.\nHi! 你好！这里是中文！\n",
        "Hi! Why even keep track of how many `epoll_events` are pending? We don't use this value here, but I added it to make it easier to create some `print` statements showing the status of our runtime at different points. However, there are good reasons to keep track of these events even if we don't use them.\n",
        "One area we're taking shortcuts on all the way here is security. If someone were to build a public facing server out of this, we need to account for slow networks and malicious users.\n",
    ]
    "###);
}

#[test]
fn md_and_range() {
    let mut md = Md::new(MD);
    md.chars_paragraph(0).last();
    let buf = md.paragraphs();
    {
        assert_debug_snapshot!(md.chars_bytes_range().next().unwrap(), @r###"
        (
            16,
            16,
            0..16,
        )
        "###);
    }
    {
        assert_debug_snapshot!(md.chars_bytes_range().next().unwrap(), @r###"
        (
            16,
            16,
            0..16,
        )
        "###);
        assert_debug_snapshot!(md.chars_bytes_range().next().unwrap(), @r###"
        (
            16,
            16,
            0..16,
        )
        "###);
    }
    {
        let mut range = md.chars_bytes_range();
        assert_debug_snapshot!(range.next().unwrap(), @r###"
        (
            16,
            16,
            0..16,
        )
        "###);
        assert_debug_snapshot!(range.next().unwrap(), @r###"
        (
            317,
            317,
            16..333,
        )
        "###);
    }
    assert_debug_snapshot!(md.chars_bytes_range().map(|(_, _, i)| (i.len(), &buf[i])).collect::<Vec<_>>(),
    @r###"
    [
        (
            16,
            "I/O event queue\n",
        ),
        (
            317,
            "We add the `callback_id` to the collection of callbacks to run. We pass in `Js::Undefined` since we'll not actually pass any data along here. You'll see why when we reach the Http module chapter, but the main point is that the I/O queue doesn't return any data itself, it just tells us that data is ready to be read.\n",
        ),
        (
            32,
            "Hi! 你好！这里是中文！\n",
        ),
        (
            305,
            "Hi! Why even keep track of how many `epoll_events` are pending? We don't use this value here, but I added it to make it easier to create some `print` statements showing the status of our runtime at different points. However, there are good reasons to keep track of these events even if we don't use them.\n",
        ),
        (
            184,
            "One area we're taking shortcuts on all the way here is security. If someone were to build a public facing server out of this, we need to account for slow networks and malicious users.\n",
        ),
    ]
    "###);

    assert_debug_snapshot!(md, @r###"
    Md {
        events: [
            Start(
                Heading(
                    1,
                ),
            ),
            Text(
                Borrowed(
                    "I/O event queue",
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
                    "We add the ",
                ),
            ),
            Code(
                Borrowed(
                    "callback_id",
                ),
            ),
            Text(
                Borrowed(
                    " to the collection of callbacks to run. We pass",
                ),
            ),
            SoftBreak,
            Text(
                Borrowed(
                    "in ",
                ),
            ),
            Code(
                Borrowed(
                    "Js::Undefined",
                ),
            ),
            Text(
                Borrowed(
                    " since we'll not actually pass any data along here. You'll see",
                ),
            ),
            SoftBreak,
            Text(
                Borrowed(
                    "why when we reach the ",
                ),
            ),
            Start(
                Link(
                    Inline,
                    Borrowed(
                        "./8_3_http_module.md",
                    ),
                    Borrowed(
                        "",
                    ),
                ),
            ),
            Text(
                Borrowed(
                    "Http module",
                ),
            ),
            End(
                Link(
                    Inline,
                    Borrowed(
                        "./8_3_http_module.md",
                    ),
                    Borrowed(
                        "",
                    ),
                ),
            ),
            Text(
                Borrowed(
                    " chapter, but the main",
                ),
            ),
            SoftBreak,
            Text(
                Borrowed(
                    "point is that the I/O queue doesn't return any data itself, it just tells us that",
                ),
            ),
            SoftBreak,
            Text(
                Borrowed(
                    "data is ready to be read.",
                ),
            ),
            End(
                Paragraph,
            ),
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
                    "fn process_epoll_events(&mut self, event_id: usize) {\n    self.callbacks_to_run.push((event_id, Js::Undefined));\n    self.epoll_pending_events -= 1;\n}\n",
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
            Start(
                Paragraph,
            ),
            Text(
                Borrowed(
                    "Hi!",
                ),
            ),
            SoftBreak,
            Text(
                Borrowed(
                    "你好！这里是中文！",
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
                    "Hi!",
                ),
            ),
            SoftBreak,
            Start(
                Strong,
            ),
            Text(
                Borrowed(
                    "Why even keep track of how many ",
                ),
            ),
            Code(
                Borrowed(
                    "epoll_events",
                ),
            ),
            Text(
                Borrowed(
                    " are pending?",
                ),
            ),
            End(
                Strong,
            ),
            SoftBreak,
            Text(
                Borrowed(
                    "We don't use this value here, but I added it to make it easier to create",
                ),
            ),
            SoftBreak,
            Text(
                Borrowed(
                    "some ",
                ),
            ),
            Code(
                Borrowed(
                    "print",
                ),
            ),
            Text(
                Borrowed(
                    " statements showing the status of our runtime at different points.",
                ),
            ),
            SoftBreak,
            Text(
                Borrowed(
                    "However, there are good reasons to keep track of these events even if we don't use them.",
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
                    "One area we're taking shortcuts on all the way here is security. If someone were",
                ),
            ),
            SoftBreak,
            Text(
                Borrowed(
                    "to build a public facing server out of this, we need to account for slow networks",
                ),
            ),
            SoftBreak,
            Text(
                Borrowed(
                    "and malicious users.",
                ),
            ),
            End(
                Paragraph,
            ),
            End(
                BlockQuote,
            ),
        ],
        buffer: "I/O event queue\nWe add the `callback_id` to the collection of callbacks to run. We pass in `Js::Undefined` since we'll not actually pass any data along here. You'll see why when we reach the Http module chapter, but the main point is that the I/O queue doesn't return any data itself, it just tells us that data is ready to be read.\nHi! 你好！这里是中文！\nHi! Why even keep track of how many `epoll_events` are pending? We don't use this value here, but I added it to make it easier to create some `print` statements showing the status of our runtime at different points. However, there are good reasons to keep track of these events even if we don't use them.\nOne area we're taking shortcuts on all the way here is security. If someone were to build a public facing server out of this, we need to account for slow networks and malicious users.\n",
        bytes: [
            16,
            317,
            32,
            305,
            184,
        ],
        chars: [
            16,
            317,
            14,
            305,
            184,
        ],
        limit: Limit {
            limit: 0,
            cnt: 0,
            len: 0,
            pos: 854,
        },
    }
    "###);
}

#[test]
fn md_split_append() {
    fn split(raw: &str) -> String {
        let mut md = Md::new(raw);
        let buf = md.extract().to_owned();
        // println!("{}", buf);
        let output = md.done(buf.split('\n'));
        // println!("{}", output);
        output
    }

    assert_display_snapshot!(split(MD), @r###"
    # I/O event queue

    # I/O event queue

    We add the `callback_id` to the collection of callbacks to run. We pass
    in `Js::Undefined` since we'll not actually pass any data along here. You'll see
    why when we reach the [Http module](./8_3_http_module.md) chapter, but the main
    point is that the I/O queue doesn't return any data itself, it just tells us that
    data is ready to be read.

    We add the `callback_id` to the collection of callbacks to run. We pass in `Js::Undefined` since we'll not actually pass any data along here. You'll see why when we reach the Http module chapter, but the main point is that the I/O queue doesn't return any data itself, it just tells us that data is ready to be read.

    ```rust, ignored
    fn process_epoll_events(&mut self, event_id: usize) {
        self.callbacks_to_run.push((event_id, Js::Undefined));
        self.epoll_pending_events -= 1;
    }
    ```

    Hi!
    你好！这里是中文！

    Hi! 你好！这里是中文！

     > 
     > Hi!
     > **Why even keep track of how many `epoll_events` are pending?**
     > We don't use this value here, but I added it to make it easier to create
     > some `print` statements showing the status of our runtime at different points.
     > However, there are good reasons to keep track of these events even if we don't use them.
     > 
     > Hi! Why even keep track of how many `epoll_events` are pending? We don't use this value here, but I added it to make it easier to create some `print` statements showing the status of our runtime at different points. However, there are good reasons to keep track of these events even if we don't use them.
     > 
     > One area we're taking shortcuts on all the way here is security. If someone were
     > to build a public facing server out of this, we need to account for slow networks
     > and malicious users.
     > 
     > One area we're taking shortcuts on all the way here is security. If someone were to build a public facing server out of this, we need to account for slow networks and malicious users.
    "###);
}
