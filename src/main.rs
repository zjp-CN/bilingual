use anyhow::Result;
use bilingual::md;

mod cmd;
mod config;

#[cfg(test)]
mod tests;

fn main() -> Result<()> {
    let mut config = argh::from_env::<cmd::Bilingual>().run()?;
    dbg!(&config);
    while let Some(output) = config.do_single_query_write() {
        println!("{:?}", output);
    }
    Ok(())
}
