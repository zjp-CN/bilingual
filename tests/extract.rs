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
    let mut select = true;
    events.iter().map(|event| extract(event, &mut select, &mut buf)).last();
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
     > 
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
fn bytes_next_paragraph_test() {
    let mut md = Md::new(MD);
    let mut limit = Limit::new(400);
    assert_display_snapshot!(md.extract_with_bytes().len(), @"826"); // 段落文本总长度
    assert_debug_snapshot!(md.bytes_next_range().collect::<Vec<_>>(), @r###"
    [
        (
            16,
            0..16,
        ),
        (
            317,
            16..333,
        ),
        (
            4,
            333..337,
        ),
        (
            305,
            337..642,
        ),
        (
            184,
            642..826,
        ),
    ]
    "###);
    assert_debug_snapshot!(md.bytes_next_paragraph(&mut limit).collect::<Vec<_>>(), @r###"
    [
        "I/O event queue\nWe add the `callback_id` to the collection of callbacks to run. We pass in `Js::Undefined` since we'll not actually pass any data along here. You'll see why when we reach the Http module chapter, but the main point is that the I/O queue doesn't return any data itself, it just tells us that data is ready to be read.\nHi!\n",
        "Hi! Why even keep track of how many `epoll_events` are pending? We don't use this value here, but I added it to make it easier to create some `print` statements showing the status of our runtime at different points. However, there are good reasons to keep track of these events even if we don't use them.\n",
        "One area we're taking shortcuts on all the way here is security. If someone were to build a public facing server out of this, we need to account for slow networks and malicious users.\n",
    ]
    "###);
}

#[test]
fn md_limit_test() {
    let mut md = Md::new(MD);
    let buf = md.extract_with_bytes().to_owned();
    {
        assert_debug_snapshot!(md.bytes_next_range().next().unwrap().1, @"0..16");
    }
    {
        assert_debug_snapshot!(md.bytes_next_range().next().unwrap().1, @"0..16");
        assert_debug_snapshot!(md.bytes_next_range().next().unwrap().1, @"0..16");
    }
    {
        let mut range = md.bytes_next_range();
        assert_debug_snapshot!(range.next().unwrap().1, @"0..16");
        assert_debug_snapshot!(range.next().unwrap().1, @"16..333");
    }
    assert_debug_snapshot!(md.bytes_next_range().map(|(l, i)| (l, &buf[i])).collect::<Vec<_>>(),
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
            4,
            "Hi!\n",
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
        buffer: "I/O event queue\nWe add the `callback_id` to the collection of callbacks to run. We pass in `Js::Undefined` since we'll not actually pass any data along here. You'll see why when we reach the Http module chapter, but the main point is that the I/O queue doesn't return any data itself, it just tells us that data is ready to be read.\nHi!\nHi! Why even keep track of how many `epoll_events` are pending? We don't use this value here, but I added it to make it easier to create some `print` statements showing the status of our runtime at different points. However, there are good reasons to keep track of these events even if we don't use them.\nOne area we're taking shortcuts on all the way here is security. If someone were to build a public facing server out of this, we need to account for slow networks and malicious users.\n",
        para: [
            16,
            317,
            4,
            305,
            184,
        ],
    }
    "###);
}

#[test]
fn md_split_append() {
    fn split(raw: &str) -> String {
        let mut md = Md::new(raw);
        let buf = md.extract().to_owned();
        let output = md.done(buf.split('\n'));
        // println!("{}", output);
        output
    }

    assert_display_snapshot!(split(MD), @r###"
    I/O event queue
    We add the `callback_id` to the collection of callbacks to run. We pass in `Js::Undefined` since we'll not actually pass any data along here. You'll see why when we reach the Http module chapter, but the main point is that the I/O queue doesn't return any data itself, it just tells us that data is ready to be read.
    Hi!
    Hi! Why even keep track of how many `epoll_events` are pending? We don't use this value here, but I added it to make it easier to create some `print` statements showing the status of our runtime at different points. However, there are good reasons to keep track of these events even if we don't use them.
    One area we're taking shortcuts on all the way here is security. If someone were to build a public facing server out of this, we need to account for slow networks and malicious users.
    # I/O event queue

    I/O event queue

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

    Hi!

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
