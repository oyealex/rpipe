use crate::config::Config;
use crate::err::RpErr;
use crate::input::{Input, Pipe};
use crate::op::Op;
use crate::output::Output;
use itertools::Itertools;
use std::env::Args;
use std::iter::{Peekable, Skip};

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
    let (input, ops, output) =
        if configs.contains(&Config::Eval) { parse_eval_token(&mut args)? } else { parse::args::parse(args)? };
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

fn parse_eval_token(args: &mut Peekable<Skip<Args>>) -> Result<(Input, Vec<Op>, Output), RpErr> {
    if let Some(mut token) = args.next() {
        token.push(' ');
        match parse::token::parse_without_configs(&token) {
            Ok((remaining, res)) => {
                if !remaining.is_empty() {
                    Err(RpErr::UnexpectedRemaining { cmd: "--eval", arg: "token", remaining: remaining.to_owned() })?
                }
                Ok(res)
            }
            Err(err) => {
                Err(RpErr::ArgParseErr { cmd: "--eval", arg: "token", arg_value: token, error: err.to_string() })?
            }
        }
    } else {
        Err(RpErr::MissingArg { cmd: "--eval", arg: "token" })?
    }
}
