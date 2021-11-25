# I/O event queue

# I/O事件队列

We add the `callback_id` to the collection of callbacks to run. We pass
in `Js::Undefined` since we'll not actually pass any data along here. You'll see
why when we reach the [Http module](./8_3_http_module.md) chapter, but the main
point is that the I/O queue doesn't return any data itself, it just tells us that
data is ready to be read.

我们将'callback_id'添加到要运行的回调集合中。我们传入'Js:：Undefined'，因为我们实际上不会在这里传递任何数据。当我们谈到Http模块一章时，您将看到为什么，但主要的一点是I/O队列本身不返回任何数据，它只是告诉我们数据已经准备好读取。

```rust, ignored
fn process_epoll_events(&mut self, event_id: usize) {
    self.callbacks_to_run.push((event_id, Js::Undefined));
    self.epoll_pending_events -= 1;
}
```

Hi!

你好

 > 
 > Hi!
 > **Why even keep track of how many `epoll_events` are pending?**
 > We don't use this value here, but I added it to make it easier to create
 > some `print` statements showing the status of our runtime at different points.
 > However, there are good reasons to keep track of these events even if we don't use them.
 > 
 > 你好为什么还要记录有多少“epoll_事件”悬而未决？我们这里不使用这个值，但我添加它是为了更容易地创建一些“print”语句来显示运行时在不同点的状态。然而，即使我们不使用它们，也有很好的理由跟踪这些事件。
 > 
 > One area we're taking shortcuts on all the way here is security. If someone were
 > to build a public facing server out of this, we need to account for slow networks
 > and malicious users.
 > 
 > 我们一路走捷径的一个领域是安全。如果有人要用它来构建一个面向公众的服务器，我们需要考虑慢速网络和恶意用户。