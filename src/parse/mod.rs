use crate::input::Input;
use crate::op::Op;
use crate::output::Output;
use crate::parse::input::parse_input;
use crate::parse::op::parse_ops;
use crate::parse::output::parse_out;
use nom::{IResult, Parser};

mod base_parser;
mod input;
mod op;
mod output;

pub(crate) fn parse(input: &str) -> IResult<&str, (Input, Vec<Op>, Output)> {
    (parse_input, parse_ops, parse_out).parse(input)
}
