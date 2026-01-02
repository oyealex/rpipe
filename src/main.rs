#![allow(unused)] // TODO 2025-12-26 22:47 移除告警禁用

mod input;
mod op;
mod output;
mod parse;

/// 整数类型
pub(crate) type Integer = i64;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1).peekable();
    let (input, ops, output) = parse::token::parse(&mut args)?;
    println!("remaining: {:?}", args.collect::<Vec<_>>());
    println!("input: {:?}", input);
    println!("ops: {:?}", ops);
    println!("output: {:?}", output);
    output.handle(ops.into_iter().fold(input.pipe(), |pipe, op| op.wrap(pipe)));
    Ok(())
}
