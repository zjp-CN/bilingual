# I/O event queue

#I/O事件队列

The I/O event queue is what handles most of our I/O tasks. Now we'll go through
how we register events to that queue later on, but once an event is ready we
it sends the `event_id` through our channel.

I/O事件队列处理大多数I/O任务。现在，我们稍后将介绍如何向该队列注册事件，但一旦事件准备就绪，它就会通过通道发送“event_id”。

```rust, ignored
fn process_epoll_events(&mut self, event_id: usize) {
    self.callbacks_to_run.push((event_id, Js::Undefined));
    self.epoll_pending_events -= 1;
}
```

As we'll see later on, the way we designed this, we actually made our `event_id`
and `callback_id` the same value since both represents an unique value for this
event. It simplifies things slightly for us.

正如我们稍后将看到的，按照我们设计它的方式，我们实际上将'event_id'和'callback_id'设置为相同的值，因为两者都表示此事件的唯一值。对我们来说，它稍微简化了一些事情。

We add the `callback_id` to the collection of callbacks to run. We pass
in `Js::Undefined` since we'll not actually pass any data along here. You'll see
why when we reach the `[Http module](./8_3_http_module.md) chapter, but the main
point is that the I/O queue doesn't return any data itself, it just tells us that
data is ready to be read.

我们将'callback_id'添加到要运行的回调集合中。我们传入'Js:：Undefined'，因为我们实际上不会在这里传递任何数据。当我们到达`[Http模块]（/8_3 _http_模块。md）章节，但主要的一点是I/O队列本身不返回任何数据，它只是告诉我们数据已经准备好读取。

Lastly it's only for our own bookkeeping we decrement the count of outstanding
`epoll_pending_events` so we keep track of how many events we have pending.

最后，这只是为了我们自己的簿记，我们减少了未完成的“epoll_pending_events”的数量，以便跟踪我们有多少未完成的事件。

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
>**为什么还要跟踪有多少“epoll_事件”处于挂起状态？**我们在这里不使用这个值，但我添加它是为了更容易地创建一些“print”语句，显示运行时在不同点的状态。然而，即使我们不使用它们，也有很好的理由跟踪这些事件。我们一路走捷径的一个领域是安全。如果有人要用它来构建一个面向公众的服务器，我们需要考虑慢速网络和恶意用户。由于我们使用“IOCP”，这是一种基于完成的模型，因此我们为每个“Read”或“Write”事件分配内存作为缓冲区。当我们把这些内存借给操作系统时，我们处于一种奇怪的境地。我们拥有它，但我们不能碰它。有几种方法可以让我们对比发生的事件更多的事件感兴趣，从而分配只保存在缓冲区中的内存。现在，如果有人想破坏我们的服务器，他们可能会故意造成这种情况。因此，一个好的做法是通过跟踪挂起事件的数量来创建“高水位线”，当我们到达该水位线时，我们将事件排队，而不是向操作系统注册它们。通过扩展，这也是为什么您应该**始终**对这些事件有一个超时，以便在某个时候可以回收内存并在必要时返回超时错误的原因。

