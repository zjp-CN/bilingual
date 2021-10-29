use anyhow::Result;
use bilingual::md;

mod cmd;
mod config;

fn main() -> Result<()> {
    let mut config = argh::from_env::<cmd::Bilingual>().run()?;
    dbg!(&config);
    if let Some(output) = config.do_query() {
        println!("{}", output);
    }
    while let Some(output) = config.do_single_file() {
        println!("{}", output);
    }
    Ok(())
}
