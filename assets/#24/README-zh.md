<!-- 本文件由 ./readme.make.md 自动生成，请不要直接修改此文件 -->

# mdbx

# MDBX

rust wrapper for [libmdbx](https://github.com/erthink/libmdbx)

libmdbx的铁锈包装器

## use example

## 使用示例

```
#![allow(non_upper_case_globals)]
use anyhow::Result;
use lazy_static::lazy_static;
use mdbx::{db, env::Env, Db};

lazy_static! {
  pub static ref MDBX: Env = {
    let mut dir = std::env::current_exe().unwrap();
    dir.pop();
    dir.push("test");
    println!("mdbx file path {}", dir.display());
    dir.try_into().unwrap()
  };
}

Db!(MDBX, UserName);

// [mdbx db flag list link](https://erthink.github.io/libmdbx/group__c__dbi.html#gafe3bddb297b3ab0d828a487c5726f76a)
// MDBX_DUPSORT 为一个键可以对应多个值
Db!(MDBX, Tag, db::Flag::MDBX_DUPSORT);

fn main() -> Result<()> {
  unsafe {
    println!(
      "mdbx version https://github.com/erthink/libmdbx/releases/tag/v{}.{}.{}",
      mdbx::mdbx_version.major,
      mdbx::mdbx_version.minor,
      mdbx::mdbx_version.release
    );
  }
  let t = std::thread::spawn(|| {
    let tx = &MDBX.w()?;
    let user_name = UserName & tx;
    user_name.set(&[3], &[4])?;
    print!("thread {:?}", user_name.get(&[2])?);
    Ok::<(), anyhow::Error>(())
  });

  {
    let tx = &MDBX.w()?;
    let user_name = UserName & tx;
    user_name.set(&[2], &[5])?;
    println!("main get {:?}", user_name.get(&[2])?);
    (user_name - [2])?;
    println!("main get after del {:?}", user_name.get(&[2])?);

    let tag = Tag & tx;

    // 一个键可以对应多个值
    tag.set(&[1], &[1])?;
    tag.set(&[1], &[2])?;
    tag.set(&[1], &[3])?;
    tag.set(&[1], &[4])?;

    dbg!(tag.get(&[1])?);

    // del需要传入val，只删除精确匹配到的
    dbg!(tag.del(&[1], &[2])?);

    dbg!(tag.get(&[1])?);

    // 删除这个key所有的val
    (tag - [1])?;

    dbg!(tag.get(&[1])?);
  }

  t.join().unwrap()?;

  Ok(())
}
```

output as below

输出如下

```
mdbx file path /root/git/mdbx/target/debug/examples/test
mdbx version https://github.com/erthink/libmdbx/releases/tag/v0.11.1
main get Some([5])
main get after del None
thread None
```

## 引子

## 引子

因为[mdbx-rs(mdbx-sys)不支持windows](https://github.com/vorot93/mdbx-rs/issues/1)，于是我自己动手封装一个支持windows版本。

因为mdbx-rs(mdbx-sys)不支持窗口，于是我自己动手封装一个支持窗口版本。

[mdbx](https://github.com/erthink/libmdbx)是基于lmdb魔改的数据库 ，作者是俄罗斯人[Леонид Юрьев (Leonid Yuriev)](https://vk.com/erthink)。

是基于lmdb魔改的数据库，作者是俄罗斯人ЛеонидЮрьев(列昂尼德·尤里耶夫)

lmdb是一个超级快的嵌入式键值数据库，[性能测试对比如下图](http://www.lmdb.tech/bench/inmem/)。

lmdb是一个超级快的嵌入式键值数据库，性能测试对比如下图。

![](http://www.lmdb.tech/bench/inmem/InMem20Mperf.png)



全文搜索引擎[MeiliSearch](https://docs.meilisearch.com/reference/under_the_hood/storage.html#measured-disk-usage)就是基于lmdb开发的。

全文搜索引擎meili搜索就是基于lmdb开发的。

[深度学习框架caffe也用lmdb作为数据存储](https://docs.nvidia.com/deeplearning/dali/user-guide/docs/examples/general/data_loading/dataloading_lmdb.html)。

深度学习框架Caffe也用lmdb作为数据存储.

mdbx在嵌入式性能测试基准[ioarena](https://github.com/pmwkaa/ioarena)中lmdb还要快30% 。

在嵌入式性能测试基准ioarena中lmdb还要快30%。

![](https://raw.githubusercontent.com/wiki/erthink/libmdbx/img/perf-slide-1.png)
![](https://raw.githubusercontent.com/wiki/erthink/libmdbx/img/perf-slide-3.png)
![](https://raw.githubusercontent.com/wiki/erthink/libmdbx/img/perf-slide-4.png)
![](https://raw.githubusercontent.com/wiki/erthink/libmdbx/img/perf-slide-5.png)



[mdbx改进了不少lmdb的缺憾](https://github.com/erthink/libmdbx#improvements-beyond-lmdb)。

mdbx改进了不少lmdb的缺憾.

[Erigon（下一代以太坊客户端）最近从 LMDB 切换到了 MDBX。](https://github.com/ledgerwatch/erigon/wiki/Criteria-for-transitioning-from-Alpha-to-Beta#switch-from-lmdb-to-mdbx)

Erigon(下一代以太坊客户端)最近从lmdb切换到了mdbx.

他们列举了从 LMDB 过渡到 MDBX 的好处：

他们列举了从lmdb过渡到mdbx的好处：

 > 
 > Erigon started off with the BoltDB database backend, then adding the support for BadgerDB, and then eventually migrating exclusively to LMDB. At some point we have encountered stability issues that were caused by our usage of LMDB that was not envisaged by the creators. We have since then been looking at a well-supported derivative of LMDB, called MDBX, and hoping to use their stability improvement, and potentially working more together in the future. The integration of MDBX is done, now it is time for more testing and documentation.
 > 
 > Erigon从BoltDB数据库后端开始，然后添加对BadgerDB的支持，最后专门迁移到LMDB。在某种程度上，我们遇到了由于使用LMDB而导致的稳定性问题，这是创建者没有想到的。从那时起，我们一直在研究LMDB的一个得到很好支持的衍生品，称为MDBX，并希望利用它们的稳定性改善，并有可能在未来进行更多合作。MDBX的集成已经完成，现在是进行更多测试和文档编制的时候了。
 > 
 > Erigon 从 BoltDB 数据库后端开始，然后添加对 BadgerDB 的支持，最终专门迁移到 LMDB。在某种程度上，我们遇到了稳定性问题，这是由于我们使用了创建者没有预料到的 LMDB 而引起的。从那时起，我们一直在研究一种受到良好支持的 LMDB 衍生物，称为 MDBX，并希望利用它们对稳定性的改善，并可能在未来进行更多的合作。MDBX 的集成已经完成，现在是进行更多测试和文档化的时候了。
 > 
 > Erigon从BoltDB数据库后端开始，然后添加对BadgerDB的支持，最终专门迁移到lmdb.在某种程度上，我们遇到了稳定性问题，这是由于我们使用了创建者没有预料到的lmdb而引起的。从那时起，我们一直在研究一种受到良好支持的lmdb衍生物，称为mdbx，并希望利用它们对稳定性的改善，并可能在未来进行更多的合作。MDBX的集成已经完成，现在是进行更多测试和文档化的时候了。
 > 
 > Benefits of transitioning from LMDB to MDBX:
 > 
 > 从LMDB过渡到MDBX的优势：
 > 
 > 从 LMDB 过渡到 MDBX 的好处:
 > 
 > 从lmdb过渡到mdbx的好处：
 > 
 > 1. Database file growth "geometry" works properly. This is important especially on Windows. In LMDB, one has to specify the memory map size once in advance (currently we use 2Tb by default), and if the database file grows over that limit, one has to restart the process. On Windows, setting memory map size to 2Tb makes database file 2Tb large on the onset, which is not very convenient. With MDBX, memory map size is increased in 2Gb increments. This means occasional remapping, but results in a better user experience.
 >    
 >    数据库文件增长“几何体”工作正常。这一点在Windows上尤其重要。在LMDB中，必须预先指定一次内存映射大小(当前我们默认使用2TB)，如果数据库文件增长超过该限制，则必须重新启动进程。在Windows上，将内存映射大小设置为2TB会使数据库文件一开始就大2TB，这不是很方便。使用MDBX，内存映射大小以2 GB为增量增加。这意味着偶尔会重新映射，但会带来更好的用户体验。
 >    
 >    数据库文件增长“几何”工程正常。这一点非常重要，尤其是在 Windows 上。在 LMDB 中，必须提前一次指定内存映射大小(目前我们默认使用2tb) ，如果数据库文件超过这个限制，则必须重新启动进程。在 Windows 上，将内存映射大小设置为2tb 会使数据库文件在开始时就变大，这并不十分方便。使用 MDBX，内存映射大小以2gb 的增量增加。这意味着偶尔的重新映射，但是会带来更好的用户体验。
 >    
 >    数据库文件增长“几何”工程正常.这一点非常重要，尤其是在Windows上。在lmdb中，必须提前一次指定内存映射大小(目前我们默认使用2TB)，如果数据库文件超过这个限制，则必须重新启动进程。在Windows上，将内存映射大小设置为2TB会使数据库文件在开始时就变大，这并不十分方便。使用mdbx，内存映射大小以2 GB的增量增加。这意味着偶尔的重新映射，但是会带来更好的用户体验.
 > 
 > 1. MDBX has more strict checks on concurrent use of the transaction handles, as well as overlap read and write transaction within the same thread of execution. This allowed us to find some non-obvious bugs and make behaviour more predictable.
 >    
 >    MDBX对事务句柄的并发使用有更严格的检查，并在同一执行线程中重叠读和写事务。这使我们能够发现一些不明显的错误，并使行为更可预测。
 >    
 >    MDBX 对事务句柄的并发使用有更严格的检查，以及在同一执行线程中重叠读写事务。这使我们能够发现一些不明显的错误，并使行为更可预测。
 >    
 >    MDBX对事务句柄的并发使用有更严格的检查，以及在同一执行线程中重叠读写事务。这使我们能够发现一些不明显的错误，并使行为更可预测.
 >    
 >    Over the period of more than 5 years (since it split from LMDB), MDBX accumulated a lot of safety fixes and heisenbug fixes that are still present in LMDB to the best of our knowledge. Some of them we have discovered during our testing, and MDBX maintainer took them seriously and worked on the fixes promptly.
 >    
 >    在5年多的时间里(从LMDB分离出来)，据我们所知，MDBX积累了大量的安全修复程序和海森漏洞修复程序，它们仍然存在于LMDB中。其中一些是我们在测试过程中发现的，MDBX维护人员认真对待它们，并迅速进行修复。
 >    
 >    在超过5年的时间里(从 LMDB 中分离出来以后) ，MDBX 积累了大量的安全修复和 heisenberg bug 修复，据我们所知，这些修复仍然存在于 LMDB 中。我们在测试期间发现了其中的一些问题，MDBX 维护人员认真对待了这些问题，并及时修复了这些问题。
 >    
 >    在超过5年的时间里(从lmdb中分离出来以后)，mdbx积累了大量的安全修复和heisenberg bug修复，据我们所知，这些修复仍然存在于lmdb中。我们在测试期间发现了其中的一些问题、mdbx维护人员认真对待了这些问题，并及时修复了这些问题。
 > 
 > 1. When it comes to databases that constantly modify data, they generate quite a lot of reclaimable space (also known as "freelist" in LMDB terminology). We had to patch LMDB to fix most serious drawbacks when working with reclaimable space (analysis here: https://github.com/ledgerwatch/erigon/wiki/LMDB-freelist-illustrated-guide). MDBX takes special care of efficient handling of reclaimable space and so far no patches were required.
 >    
 >    当涉及到不断修改数据的数据库时，它们会生成相当多的可回收空间(在LMDB术语中也称为“自由列表”)。在使用可回收空间时，我们必须修补lmdb以解决最严重的缺陷(此处分析：https://github.com/ledgerwatch/erigon/wiki/LMDB-freelist-illustrated-guide).MDBX特别注意有效地处理可回收空间，到目前为止不需要打补丁。
 >    
 >    当涉及到不断修改数据的数据库时，它们会产生大量可回收空间(在 LMDB 术语中也称为“自由职业者”)。当使用可回收空间时，我们不得不修补 LMDB 来修复最严重的缺陷(这里分析: https://github.com/ledgerwatch/erigon/wiki/LMDB-freelist-illustrated-guide 空间)。MDBX 特别注意有效处理可回收空间，迄今为止没有补丁需要。
 >    
 >    当涉及到不断修改数据的数据库时，它们会产生大量可回收空间(在lmdb术语中也称为“自由职业者”)。https://github.com/ledgerwatch/erigon/wiki/LMDB-freelist-illustrated-guide当使用可回收空间时，我们不得不修补来修复最严重的缺陷(这里分析：Lmdb空间)。MDBX特别注意有效处理可回收空间，迄今为止没有补丁需要。
 > 
 > 1. According to our tests, MDBX performs slightly better on our workloads.
 >    
 >    根据我们的测试，MDBX在我们的工作负载上的性能略好一些。
 >    
 >    根据我们的测试，MDBX 在我们的工作负载上表现稍好一些。
 >    
 >    根据我们的测试、mdbx在我们的工作负载上表现稍好一些。
 > 
 > 1. MDBX exposes more internal telemetry - more metrics of what happening inside DB. And we have them in Grafana - to make better decisions on app design. For example, after complete transition to MDBX (removing LMDB support) we will implement "commit half-full transactions" strategy to avoid spill/unspill disk touches. This will simplify our code further without affecting performance.
 >    
 >    MDBX公开了更多的内部遥测-更多关于数据库内部发生的事情的度量。我们在Grafana有他们--在应用程序设计上做出更好的决定。例如，在完全过渡到MDBX之后(取消LMDB支持)，我们将实施“提交半满事务”策略，以避免磁盘溢出/未溢出。这将在不影响性能的情况下进一步简化我们的代码。
 >    
 >    MDBX 公开了更多的内部遥测数据——关于 DB 内部发生的情况的更多指标。而且我们在 Grafana 也有这样的机构——它们可以在应用程序设计上做出更好的决策。例如，在完全转换到 MDBX (删除 LMDB 支持)之后，我们将实现“提交半满事务”策略，以避免溢出/未溢出磁盘。这将进一步简化我们的代码，而不会影响性能。
 >    
 >    mdbx公开了更多的内部遥测数据--关于DB内部发生的情况的更多指标。而且我们在grafana也有这样的机构--它们可以在应用程序设计上做出更好的决策.例如，在完全转换到mdbx(删除lmdb支持)之后，我们将实现“提交半满事务”策略，以避免溢出/未溢出磁盘.这将进一步简化我们的代码，而不会影响性能.
 > 
 > 1. MDBX has support for "Exclusive open" mode - we using it for DB migrations, to prevent any other reader from accessing the database while DB migration is in progress.
 >    
 >    MDBX支持“独占开放”模式-我们将其用于数据库迁移，以防止任何其他读者在数据库迁移过程中访问数据库。
 >    
 >    MDBX 支持“ Exclusive open”模式——我们使用它进行 DB 迁移，以防止任何其他读取器在 DB 迁移过程中访问数据库。
 >    
 >    mdbx支持独占打开模式--我们使用它进行DB迁移，以防止任何其他读取器在DB迁移过程中访问数据库。

## 关于

## 关于

本项目隶属于**人民网络([rmw.link](//rmw.link))** 代码计划。

本项目隶属于人民网络(rmw.link)代码计划.

<a href="//rmw.link">
![人民网络](https://raw.githubusercontent.com/rmw-link/logo/master/rmw.red.bg.svg)</a>

人民网络