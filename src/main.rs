#![allow(unused)] // TODO 2025-12-26 22:47 移除告警禁用

use crate::config::Config;
use crate::err::RpErr;
use crate::input::Pipe;
use itertools::Itertools;

mod config;
mod err;
mod input;
mod op;
mod output;
mod parse;

/// 整数类型
pub(crate) type Integer = i64;

pub(crate) type RpRes = Result<Pipe, RpErr>;

fn main() {
    if let Err(e) = run() {
        e.termination();
    }
}

fn run() -> Result<(), RpErr> {
    let mut args = std::env::args().skip(1).peekable();
    let configs = parse::args::parse_configs(&mut args);
    let (input, ops, output) = parse::args::parse(args)?;
    if configs.contains(&Config::Verbose) {
        println!("Input:");
        println!("    {:?}", input);
        println!("Op:");
        println!("{}", ops.iter().map(|op| format!("    {:?}", op)).join("\n"));
        println!("Output:");
        println!("    {:?}", output);
    }
    let configs: &'static mut [Config] = configs.leak();
    let mut pipe = input.pipe()?;
    for op in ops {
        pipe = op.wrap(pipe, configs)?;
    }
    if !configs.contains(&Config::DryRun) { output.handle(pipe) } else { Ok(()) }
}
