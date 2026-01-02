use crate::input::Input;
use crate::op::Op;
use crate::output::Output;
use crate::parse::token::input::parse_input;
use crate::parse::token::op::parse_ops;
use crate::parse::token::output::parse_output;
use std::iter::Peekable;
use std::str::FromStr;

mod input;
mod op;
mod output;

pub(crate) fn parse(token: &mut Peekable<impl Iterator<Item = String>>) -> Result<(Input, Vec<Op>, Output), String> {
    let input = parse_input(token)?;
    let ops = parse_ops(token)?;
    let output = parse_output(token)?;
    Ok((input, ops, output))
}

fn parse_arg_or_arg1(token: &mut Peekable<impl Iterator<Item = String>>) -> Result<Vec<String>, String> {
    match token.next() {
        // 至少有一个值，直接消耗
        Some(arg) => {
            if arg == "[" {
                // 多值开始
                let mut args = Vec::new();
                while let Some(arg) = token.next() {
                    if arg == "]" {
                        // 多值结束
                        return if args.is_empty() {
                            Err("at least one arg is required".to_string())
                        } else {
                            Ok(args)
                        };
                    } else {
                        args.push(escaped(arg))
                    }
                }
                Err("closing bracket is required".to_string())
            } else if arg == "]" {
                // 未开启的多值结束
                Err("unexpected closing bracket".to_string())
            } else {
                Ok(vec![escaped(arg)])
            }
        }
        None => Err("no more args available".to_string()),
    }
}

fn escaped(arg: String) -> String {
    if arg == "\\[" || arg == "\\]" { arg[1..].to_string() } else { arg }
}
