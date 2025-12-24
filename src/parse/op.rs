use crate::op::Op;
use nom::IResult;

pub(super) type OpsResult<'a> = IResult<&'a str, Vec<Op>>;

pub(super) fn parse_ops(input: &str) -> OpsResult<'_> {
    todo!()
}
