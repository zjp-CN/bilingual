use anyhow::Result;
use bilingual::md;

mod cmd;
mod config;

#[cfg(test)]
mod tests;

fn main() -> Result<()> {
    log_init()?;
    let mut config = argh::from_env::<cmd::Bilingual>().run()?;
    log::debug!("\n{:#?}", config);

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
         s.parse().unwrap_or(LevelFilter::Info))
    } else {
        (var("TERM_LOG").ok().map(|s| s.parse().ok()).flatten().unwrap_or(LevelFilter::Warn),
         var("FILE_LOG").ok().map(|s| s.parse().ok()).flatten().unwrap_or(LevelFilter::Info))
    };
    let logf = var("FILE").unwrap_or("bilingual.log".into());
    let info = format!("log-level: term => {}, file => {}; log-file => {}", term, file, logf);
    let mut config = ConfigBuilder::default();
    config.set_time_to_local(true);
    let config_term = config.clone().set_time_level(LevelFilter::Debug).build();

    CombinedLogger::init(
        vec![TermLogger::new(term, config_term, TerminalMode::Mixed, ColorChoice::Auto),
             WriteLogger::new(file, config.build(), std::fs::File::create(logf)?)]
        )
        .map(|_| log::info!("{}", info))
        .map_err(|e| e.into())
}
