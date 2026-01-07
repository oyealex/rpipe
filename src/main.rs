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

pub(crate) type Integer = i64;
pub(crate) type Float = f64;

pub(crate) type RpRes = Result<Pipe, RpErr>;

fn main() {
    if let Err(e) = run() {
        e.termination();
    }
}

fn run() -> Result<(), RpErr> {
    let mut args = std::env::args().skip(1).peekable();
    let configs = parse::args::parse_configs(&mut args);
    if configs.contains(&Config::Help) {
        print_help();
        return Ok(());
    } else if configs.contains(&Config::Version) {
        print_version();
        return Ok(());
    }
    let (input, ops, output) =
        if configs.contains(&Config::Eval) { parse_eval_token(&mut args)? } else { parse::args::parse(args)? };
    if configs.contains(&Config::Verbose) {
        print_pipe_info(&input, &ops, &output);
    }
    let configs: &'static mut [Config] = configs.leak();
    let mut pipe = input.pipe()?;
    for op in ops {
        pipe = op.wrap(pipe, configs)?;
    }
    if configs.contains(&Config::DryRun) { Ok(()) } else { output.handle(pipe) }
}

fn print_pipe_info(input: &Input, ops: &Vec<Op>, output: &Output) {
    println!("Input:");
    println!("    {:?}", input);
    println!("Op:");
    println!("{}", ops.iter().map(|op| format!("    {:?}", op)).join("\n"));
    println!("Output:");
    println!("    {:?}", output);
}

fn parse_eval_token(args: &mut Peekable<Skip<Args>>) -> Result<(Input, Vec<Op>, Output), RpErr> {
    if let Some(mut token) = args.next() {
        token.push(' ');
        match parse::token::parse_without_configs(&token.trim_start()) {
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

fn print_help() {
    print_version();
    println!("\nrp [options] [input_cmd] [operate_cmd] [...] [output_cmd]");
    println!("\noptions 选项：");
    for (_, help) in Config::all_help() {
        println!("{}", help);
    }
    println!("\ninput_cmd 数据输入命令：");
    for (_, help) in Input::all_help() {
        println!("{}", help);
    }
    println!("\noperate_cmd 数据操作命令：");
    for (_, help) in Op::all_help() {
        println!("{}", help);
    }
    println!("\noutput_cmd 数据输出命令：");
    for (_, help) in Output::all_help() {
        println!("{}", help);
    }
    println!("\n命令退出码：");
    for (_, help) in RpErr::all_help() {
        println!("{}", help);
    }
}

fn print_version() {
    println!("rp (rust pipe) - v0.1.0");
}
