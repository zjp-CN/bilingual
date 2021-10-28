use bilingual::md;

fn main() { md::process(MD); }

#[rustfmt::skip]
pub static MD: &str = "
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
