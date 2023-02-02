use super::*;
use insta::{assert_debug_snapshot as snap, assert_display_snapshot as shot};
use log::{Metadata, Record};
use std::sync::Mutex;

struct Logger(Mutex<String>);

impl Logger {
    fn init() -> &'static Self {
        let logger = Box::leak(Box::new(Logger(Mutex::new(String::with_capacity(1024)))));
        log::set_logger(logger).expect("日志初始化失败：已经设置了日志");
        log::set_max_level(log::LevelFilter::Debug);
        logger
    }

    fn take_content(&self) -> String { std::mem::take(&mut self.0.lock().unwrap()) }
}

impl log::Log for Logger {
    fn enabled(&self, _metadata: &Metadata) -> bool { true }

    fn log(&self, record: &Record) {
        use std::fmt::Write;
        let s = &mut *self.0.lock().unwrap();
        writeln!(s, "LEVEL={}: {}", record.level(), record.args()).unwrap();
    }

    fn flush(&self) {}
}

#[test]
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
