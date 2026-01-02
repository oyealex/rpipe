use crate::input::Input;
use crate::parse::token::parse_arg_or_arg1;
use std::iter::Peekable;

pub(in crate::parse::token) fn parse_input(
    token: &mut Peekable<impl Iterator<Item = String>>,
) -> Result<Input, String> {
    match token.peek() {
        Some(cmd) => {
            if cmd.eq_ignore_ascii_case("in") {
                parse_std_in(token)
            } else if cmd.eq_ignore_ascii_case("file") {
                parse_file(token)
            } else if cmd.eq_ignore_ascii_case("clip") {
                parse_clip(token)
            } else if cmd.eq_ignore_ascii_case("of") {
                parse_of(token)
            } else if cmd.eq_ignore_ascii_case("gen") {
                parse_gen(token)
            } else if cmd.eq_ignore_ascii_case("repeat") {
                parse_repeat(token)
            } else {
                Ok(Input::StdIn)
            }
        }
        None => Ok(Input::StdIn),
    }
}

fn parse_std_in(token: &mut Peekable<impl Iterator<Item = String>>) -> Result<Input, String> {
    token.next(); // 消耗`in`
    Ok(Input::StdIn)
}

fn parse_file(token: &mut Peekable<impl Iterator<Item = String>>) -> Result<Input, String> {
    token.next(); // 消耗`file`
    Ok(Input::File { files: parse_arg_or_arg1(token)? })
}

fn parse_clip(token: &mut Peekable<impl Iterator<Item = String>>) -> Result<Input, String> {
    token.next(); // 消耗`clip`
    Ok(Input::Clip)
}

fn parse_of(token: &mut Peekable<impl Iterator<Item = String>>) -> Result<Input, String> {
    token.next(); // 消耗`of`
    Ok(Input::Of { values: parse_arg_or_arg1(token)? })
}

fn parse_gen(token: &mut Peekable<impl Iterator<Item = String>>) -> Result<Input, String> {
    token.next(); // 消耗`gen`
    let range = token.next().ok_or("missing `range` argument for gen".to_string())?;
    match crate::parse::text::input::parse_range_in_gen(&range) {
        Ok((remaining, input)) => {
            if !remaining.is_empty() {
                Err(format!("unexpected token in `range` argument of `gen`: {remaining}"))
            } else {
                Ok(input)
            }
        }
        Err(e) => Err(format!("failed to parse `range` argument of `gen`: {e}")),
    }
}

fn parse_repeat(token: &mut Peekable<impl Iterator<Item = String>>) -> Result<Input, String> {
    token.next(); // 消耗`repeat`
    let value = token.next().ok_or("missing `repeat` argument for `repeat`".to_string())?;
    let count = match token.next() {
        Some(string) => match string.parse::<usize>() {
            Ok(count) => Ok(Some(count)),
            Err(e) => Err(format!("failed to parse `count` argument of `repeat`: {e}")),
        },
        None => Ok(None),
    }?;
    Ok(Input::Repeat { value, count })
}
