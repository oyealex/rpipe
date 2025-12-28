use crate::op::Op;
use crate::parse::ParserError;
use nom::IResult;

pub(super) type OpsResult<'a> = IResult<&'a str, Vec<Op>, ParserError<'a>>;

pub(super) fn parse_ops(input: &str) -> OpsResult<'_> {
    Ok((input, vec![Op::Upper]))
}
