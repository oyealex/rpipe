#![allow(unused)] // TODO 2025-12-26 22:47 移除告警禁用

use crate::err::RpErr;
use crate::input::Pipe;

mod err;
mod input;
mod op;
mod output;
mod parse;

/// 整数类型
pub(crate) type Integer = i64;

pub(crate) type PipeRes = Result<Pipe, RpErr>;

fn main() -> Result<(), RpErr> {
    match run() {
        Ok(_) => Ok(()),
        Err(err) => {
            println!("### {err}");
            Err(err)
        }
    }
}

fn run() -> Result<(), RpErr> {
    let mut args = std::env::args().skip(1).peekable();
    let (input, ops, output) = parse::args::parse(args)?;
    // TODO 2026-01-05 01:41 仅选项要求打印时才打印
    println!("input: {:?}", input);
    println!("ops: {:?}", ops);
    println!("output: {:?}", output);
    let mut pipe = input.pipe()?;
    for op in ops {
        pipe = op.wrap(pipe)?;
    }
    output.handle(pipe)
}
