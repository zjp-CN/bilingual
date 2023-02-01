use anyhow::Result;
use bilingual::md;
#[macro_use]
extern crate log;

mod cmd;
mod config;

#[cfg(test)]
mod tests;

fn main() -> Result<()> {
    log_init()?;
    let mut config = argh::from_env::<cmd::Bilingual>().run()?;
    debug!("\n{:#?}", config);

    while let Some(output) = config.do_single_query_write() {
        log::trace!("{:?}", output);
    }

    Ok(())
}

#[rustfmt::skip]
fn log_init() -> Result<()> {
    use simplelog::*;
    use std::env::var;

    let (term, file) = if let Ok(ref s) = var("LOG") {
        (s.parse().unwrap_or(LevelFilter::Warn),
         s.parse().unwrap_or(LevelFilter::Off))
    } else {
        (var("TERM_LOG").ok().and_then(|s| s.parse().ok()).unwrap_or(LevelFilter::Warn),
         var("FILE_LOG").ok().and_then(|s| s.parse().ok()).unwrap_or(LevelFilter::Off))
    };
    let logf = var("FILE").unwrap_or("bilingual.log".into());
    let info = format!("log-level: term => {term}, file => {file}; log-file => {logf}");
    let mut config = ConfigBuilder::default();
    config.set_time_offset_to_local().map_err(|_| anyhow::anyhow!("simplelog 无法确定 time offset 来获取本地时间"))?;
    let config_term = config.clone().set_time_level(LevelFilter::Debug).build();

    let logger: Vec<Box<dyn SharedLogger>> = if file != LevelFilter::Off {
        vec![TermLogger::new(term, config_term, TerminalMode::Mixed, ColorChoice::Auto),
             WriteLogger::new(file, config.build(), std::fs::File::create(logf)?)]
    } else {
        vec![TermLogger::new(term, config_term, TerminalMode::Mixed, ColorChoice::Auto)]
    };
    CombinedLogger::init(logger).map(|_| log::info!("{}", info)).map_err(|e| e.into())
}
