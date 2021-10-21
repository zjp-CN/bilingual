use std::{
    env,
    fs::read_to_string,
    io::{stdout, Write},
};

use pulldown_cmark::{Options, Parser};
use pulldown_cmark_to_cmark::cmark;

fn main() {
    let path = env::args().skip(1).next().expect("First argument is markdown file to display");

    let md = read_to_string(path).unwrap();
    let mut buf = String::with_capacity(md.len() + 128);
    let mut options = Options::all();
    options.remove(Options::ENABLE_SMART_PUNCTUATION);
    cmark(Parser::new_ext(&md, options), &mut buf, None).unwrap();
    stdout().write_all(buf.as_bytes()).unwrap();
}
