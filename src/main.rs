mod md;
use md::*;

use pulldown_cmark::{Event, Parser};
use pulldown_cmark_to_cmark::cmark;
use std::io::{stdout, Write};

fn main() {
    // let file = std::env::args().skip(1).next().unwrap();
    // let md = std::fs::read_to_string(file).unwrap();
    let md = MD;
    let parser: Vec<_> = Parser::new_ext(&md, cmark_opt()).collect();
    dbg!(parser.len());

    let mut include = true;
    let mut content = {
        parser.clone()
              .into_iter()
              .filter(|event| filter(event, &mut include))
              .enumerate()
              .map(extract)
    };
    let capacity = md.len();
    let mut buf = String::with_capacity(capacity * 2);

    // cmark(parser.into_iter(), &mut buf, None).unwrap();

    // let mut include2 = true;
    // println!("{}", content.join(""));
    // let mut text = content.into_iter().enumerate();
    // let mut para = paragraphs.iter();

    cmark(parser.into_iter()
                .map(move |event| {
                    if filter2(&event) {
                        let mut text = String::with_capacity(capacity);
                        while let Some((pos, s, para)) = content.next() {
                            text += s.as_ref();
                            if para.map(|p| pos == p).unwrap_or(false) {
                                break;
                            }
                        }
                        text += "=============\n\n";
                        dbg!(&text);
                        append(event, text.into())
                    } else {
                        [dbg!(event), Event::Text(String::new().into())]
                    }
                })
                .flatten(),
          &mut buf,
          None).unwrap();
    stdout().write_all(buf.as_bytes()).unwrap();

    // dbg!(paragraphs);
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
