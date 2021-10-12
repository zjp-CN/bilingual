fn main() {
    for path in std::env::args().skip(1) {
        let md = std::fs::read_to_string(path).unwrap();
        let res: Vec<_> = pulldown_cmark::Parser::new(&md).collect();
        println!("{:#?}", res);
    }
}
