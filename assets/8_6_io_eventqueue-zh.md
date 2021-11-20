# I/O event queue

# I/O事件队列

The I/O event queue is what handles most of our I/O tasks. Now we'll go through
how we register events to that queue later on, but once an event is ready we
it sends the `event_id` through our channel.

I/O事件队列处理我们的大多数I/O任务。现在，我们稍后将介绍如何将事件注册到该队列，但是一旦事件准备就绪，它就会通过我们的通道发送`event_id‘。

```rust, ignored
fn process_epoll_events(&mut self, event_id: usize) {
    self.callbacks_to_run.push((event_id, Js::Undefined));
    self.epoll_pending_events -= 1;
}
```

As we'll see later on, the way we designed this, we actually made our `event_id`
and `callback_id` the same value since both represents an unique value for this
event. It simplifies things slightly for us.

正如我们稍后将看到的，在我们的设计方法中，我们实际上将`event_id`和`callback_id`设置为相同的值，因为它们都表示该事件的唯一值。它稍微简化了我们的事情。

We add the `callback_id` to the collection of callbacks to run. We pass
in `Js::Undefined` since we'll not actually pass any data along here. You'll see
why when we reach the [Http module](./8_3_http_module.md) chapter, but the main
point is that the I/O queue doesn't return any data itself, it just tells us that
data is ready to be read.

我们将`callback_id`添加到要运行的回调集合中。我们传入`js：：unfined`，因为我们不会在这里实际传递任何数据。当我们讲到http模块一章时，您就会明白为什么，但是主要的观点是I/O队列本身并不返回任何数据，它只是告诉我们数据已经准备好可以读取了。

Lastly it's only for our own bookkeeping we decrement the count of outstanding
`epoll_pending_events` so we keep track of how many events we have pending.

最后，这只是为了我们自己的记账，我们会减少未完成的‘epolpending_events’的数量，这样我们就可以跟踪我们有多少待处理的事件。

 > 
 > **Why even keep track of how many `epoll_events` are pending?**
 > We don't use this value here, but I added it to make it easier to create
 > some `print` statements showing the status of our runtime at different points.
 > However, there are good reasons to keep track of these events even if we don't use them.
 > 
 > 为什么还要跟踪有多少`epol_event`挂起呢？我们在这里没有使用这个值，但我添加了它，以便更容易创建一些显示运行时在不同点的状态的“print`”语句。然而，即使我们不使用这些事件，也有很好的理由跟踪它们。
 > 
 > One area we're taking shortcuts on all the way here is security. If someone were
 > to build a public facing server out of this, we need to account for slow networks
 > and malicious users.
 > 
 > 我们一路走捷径的一个领域就是安保。如果有人要用它来构建面向公众的服务器，我们需要考虑网络速度慢和恶意用户的问题。
 > 
 > Since we use `IOCP`, which is a completion based model, we allocate memory for
 > a buffer for each `Read` or `Write` event. When we lend this memory to the OS,
 > we're in a weird situation. We own it, but we can't touch it. There are several
 > ways in which we could register interest in more events than occur, and thereby
 > allocating memory that is just held in the buffers. Now if someone wanted to crash
 > our server, they could cause this intentionally.
 > 
 > 由于我们使用的是`IOCP`，这是一个基于完成的模型，所以我们为每个`Read`或`Write`事件分配内存作为缓冲区。当我们把这个内存借给操作系统时，我们就处于一种奇怪的境地。我们拥有它，但我们不能碰它。有几种方法可以让我们注册对比发生的事件更多的事件感兴趣，从而分配刚刚保存在缓冲区中的内存。现在，如果有人想要使我们的服务器崩溃，他们可能是故意造成的。
 > 
 > A good practice is therefore to create a "high watermark" by keeping track of
 > the number of pending events, and when we reach that watermark, we queue events
 > instead of registering them with the OS.
 > 
 > 因此，一种好的做法是通过跟踪挂起事件的数量来创建“高水位线”，当我们达到该水位线时，我们将事件排入队列，而不是将它们注册到操作系统。
 > 
 > By extension, this is also why you should **always** have a timeout on these events
 > so that you at some point can reclaim the memory and return an timeout error if
 > necessary.
 > 
 > 通过扩展，这也是为什么您应该始终对这些事件设置超时，以便您可以在某个点回收内存，并在必要时返回超时错误。