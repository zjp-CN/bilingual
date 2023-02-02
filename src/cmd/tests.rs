use super::*;
use insta::{assert_debug_snapshot as snap, assert_display_snapshot as shot};
use std::sync::Mutex;

// 存储以测试日志信息（为了明确在出错的情况下会产生什么记录）
// TODO: 如果需要测试更多日志，可公开此结构
struct Logger(Mutex<String>);

impl Logger {
    // 初始化：此函数已经设置了开启日志，无需再设置；已设置级别为 Debug
    fn init() -> &'static Self {
        let logger = Box::leak(Box::new(Logger(Mutex::new(String::with_capacity(1024)))));
        log::set_logger(logger).expect("日志初始化失败：已经设置了日志");
        log::set_max_level(log::LevelFilter::Debug);
        logger
    }

    // 获取存储的原日志（注意新的日志为空）
    fn take_content(&self) -> String {
        std::mem::replace(&mut *self.0.lock().unwrap(), String::with_capacity(1024))
    }
}

impl log::Log for Logger {
    fn enabled(&self, _metadata: &log::Metadata) -> bool { true }

    fn log(&self, record: &log::Record) {
        use std::fmt::Write;
        writeln!(&mut *self.0.lock().unwrap(), "LEVEL={}: {}", record.level(), record.args()).unwrap();
    }

    fn flush(&self) {}
}

#[test]
// 模拟无配置文件下，id 和 key 设置时返回的错误与日志记录
fn error_report_on_empty_config() {
    let logger = Logger::init();

    snap!(tencent("".into(), "".into(), &mut Config::default()), @r###"
    Err(
        "id 不应该为空",
    )
    "###);
    shot!(logger.take_content(), @r###"
    LEVEL=DEBUG: 由于未找到配置文件，先使用默认配置（无 id 和 key）
    "###);

    snap!(tencent("".into(), "key".into(), &mut Config::default()), @r###"
    Err(
        "id 不应该为空",
    )
    "###);
    shot!(logger.take_content(), @r###"
    LEVEL=DEBUG: 由于未找到配置文件，先使用默认配置（无 id 和 key）
    "###);

    snap!(tencent("id".into(), "".into(), &mut Config::default()), @r###"
    Err(
        "key 不应该为空",
    )
    "###);
    shot!(logger.take_content(), @r###"
    LEVEL=DEBUG: 由于未找到配置文件，先使用默认配置（无 id 和 key）
    LEVEL=DEBUG: id 被命令行参数覆盖
    "###);

    snap!(tencent("id".into(), "key".into(), &mut Config::default()), @r###"
    Ok(
        (),
    )
    "###);
    shot!(logger.take_content(), @r###"
    LEVEL=DEBUG: 由于未找到配置文件，先使用默认配置（无 id 和 key）
    LEVEL=DEBUG: id 被命令行参数覆盖
    LEVEL=DEBUG: key 被命令行参数覆盖
    "###);
}

#[test]
fn new_filename_dir() {
    assert_eq!(Path::new("/root/test-zh.md"), new_filename("/root/test.md".as_ref(), "zh"));
    assert_eq!(Path::new("/root/test-zh/"), new_dir("/root/test/".as_ref(), "zh"));
    assert_eq!(Path::new("/root/test-zh"), new_dir("/root/test".as_ref(), "zh"));
}
