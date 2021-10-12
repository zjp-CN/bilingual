# I/O event queue

The I/O event queue is what handles most of our I/O tasks. Now we'll go through
how we register events to that queue later on, but once an event is ready we
it sends the `event_id` through our channel.

```rust, ignored
fn process_epoll_events(&mut self, event_id: usize) {
    self.callbacks_to_run.push((event_id, Js::Undefined));
    self.epoll_pending_events -= 1;
}
```

As we'll see later on, the way we designed this, we actually made our `event_id`
and `callback_id` the same value since both represents an unique value for this
event. It simplifies things slightly for us.

We add the `callback_id` to the collection of callbacks to run. We pass
in `Js::Undefined` since we'll not actually pass any data along here. You'll see
why when we reach the `[Http module](./8_3_http_module.md) chapter, but the main
`point is that the I/O queue doesn't return any data itself, it just tells us that
data is ready to be read.

Lastly it's only for our own bookkeeping we decrement the count of outstanding
`epoll_pending_events` so we keep track of how many events we have pending.

> **Why even keep track of how many `epoll_events` are pending?**
> We don't use this value here, but I added it to make it easier to create
> some `print` statements showing the status of our runtime at different points.
> However, there are good reasons to keep track of these events even if we don't use them.
>
> One area we're taking shortcuts on all the way here is security. If someone were
> to build a public facing server out of this, we need to account for slow networks
> and malicious users.
>
> Since we use `IOCP`, which is a completion based model, we allocate memory for
> a buffer for each `Read` or `Write` event. When we lend this memory to the OS,
> we're in a weird situation. We own it, but we can't touch it. There are several
> ways in which we could register interest in more events than occur, and thereby
> allocating memory that is just held in the buffers. Now if someone wanted to crash
> our server, they could cause this intentionally.
>
> A good practice is therefore to create a "high watermark" by keeping track of
> the number of pending events, and when we reach that watermark, we queue events
> instead of registering them with the OS.
>
> By extension, this is also why you should **always** have a timeout on these events
> so that you at some point can reclaim the memory and return an timeout error if
> necessary.
