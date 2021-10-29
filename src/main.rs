use anyhow::Result;
use bilingual::md;

mod cmd;
mod config;

fn main() -> Result<()> {
    let mut config = argh::from_env::<cmd::Bilingual>().run()?;
    dbg!(&config);
    while let Some(output) = config.single_query() {
        println!("{}", output);
    }
    while let Some(output) = config.single_file() {
        println!("{}", output);
    }
    Ok(())
}
