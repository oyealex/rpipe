mod input;
mod op;
mod output;
mod parse;

fn main() {
    let (_, (input, ops, output)) = parse::parse("").unwrap();
}
