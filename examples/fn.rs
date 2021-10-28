fn main() { a(vec![0, 1].into_iter()).last(); }

fn a<T: Iterator<Item = u8>>(t: T) -> impl Iterator<Item = u8> {
    t.map(|x| x + 1).collect::<Vec<u8>>().into_iter()
}

// failed: mismatched types
// fn b<T: Iterator<Item = u8>, U: Iterator<Item = u8>>(t: T) -> U { t.map(|x| x + 1) }
