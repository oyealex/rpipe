use crate::op::Op;
use crate::parse::token::{arg, general_file_info, ParserError};
use nom::branch::alt;
use nom::bytes::complete::tag_no_case;
use nom::character::complete::{space1, usize};
use nom::combinator::{map, opt};
use nom::error::context;
use nom::multi::many0;
use nom::sequence::{delimited, preceded, terminated};
use nom::{IResult, Parser};

pub(in crate::parse) type OpsResult<'a> = IResult<&'a str, Vec<Op>, ParserError<'a>>;
pub(in crate::parse) type OpResult<'a> = IResult<&'a str, Op, ParserError<'a>>;

pub(in crate::parse) fn parse_ops(input: &str) -> OpsResult<'_> {
    context("Op", many0(alt((parse_upper, parse_lower, parse_case, parse_replace, parse_uniq, parse_peek))))
        .parse(input)
}

fn parse_upper(input: &str) -> OpResult<'_> {
    context("Op::Upper", map(terminated(tag_no_case(":upper"), space1), |_| Op::new_upper())).parse(input)
}

fn parse_lower(input: &str) -> OpResult<'_> {
    context("Op::Lower", map(terminated(tag_no_case(":lower"), space1), |_| Op::new_lower())).parse(input)
}

fn parse_case(input: &str) -> OpResult<'_> {
    context("Op::Case", map(terminated(tag_no_case(":case"), space1), |_| Op::new_case())).parse(input)
}

fn parse_replace(input: &str) -> OpResult<'_> {
    context(
        "Op::Replace",
        map(
            preceded(
                (tag_no_case(":replace"), space1), // 丢弃：命令+空格
                terminated(
                    // 兼容:
                    //  from to
                    //  from to 10
                    //  from to 10 nocase
                    //  from to    nocase
                    (
                        arg, // 被替换文本
                        preceded(
                            space1,
                            (
                                arg,                                          // 替换为文本
                                opt(preceded(space1, usize)),                 // 替换次数
                                opt(preceded(space1, tag_no_case("nocase"))), // 忽略大小写
                            ),
                        ),
                    ),
                    space1,
                ),
            ), // 丢弃：结尾空格
            |(from, (to, count_opt, nocase_opt))| Op::new_replace(from, to, count_opt, nocase_opt.is_some()),
        ),
    )
    .parse(input)
}

fn parse_uniq(input: &str) -> OpResult<'_> {
    context(
        "Op::Uniq",
        map(
            delimited(
                tag_no_case(":uniq"),                         // 丢弃：命令
                opt(preceded(space1, tag_no_case("nocase"))), // 可选：空格+nocase选项
                space1,
            ), // 丢弃：结尾空格
            |nocase_opt| Op::new_uniq(nocase_opt.is_some()),
        ),
    )
    .parse(input)
}

fn parse_peek(input: &str) -> OpResult<'_> {
    context(
        "Op::Peek",
        map(
            preceded(
                (tag_no_case(":peek"), space1), // 丢弃命令和空格
                opt(general_file_info(true)),   // 可选文件信息
            ),
            |file_info| match file_info {
                Some((file, append_opt, ending_opt)) => {
                    Op::new_peek_to_file(file, append_opt.is_some(), ending_opt.map(|s| s.eq_ignore_ascii_case("crlf")))
                }
                None => Op::new_peek_to_std_out(),
            },
        ),
    )
    .parse(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_upper() {
        assert_eq!(parse_upper(":upper "), Ok(("", Op::new_upper())));
    }

    #[test]
    fn test_parse_lower() {
        assert_eq!(parse_lower(":lower "), Ok(("", Op::new_lower())));
    }

    #[test]
    fn test_parse_case() {
        assert_eq!(parse_case(":case "), Ok(("", Op::new_case())));
    }

    #[test]
    fn test_parse_replace() {
        assert_eq!(
            parse_replace(r#":replace abc "" "#),
            Ok(("", Op::new_replace("abc".to_string(), "".to_string(), None, false)))
        );
        assert_eq!(
            parse_replace(":replace abc 123 "),
            Ok(("", Op::new_replace("abc".to_string(), "123".to_string(), None, false)))
        );
        assert_eq!(
            parse_replace(":replace abc 123 5 "),
            Ok(("", Op::new_replace("abc".to_string(), "123".to_string(), Some(5), false)))
        );
        assert_eq!(
            parse_replace(":replace abc 123 5 nocase "),
            Ok(("", Op::new_replace("abc".to_string(), "123".to_string(), Some(5), true)))
        );
        assert_eq!(
            parse_replace(r#":replace abc "" 5 nocase "#),
            Ok(("", Op::new_replace("abc".to_string(), "".to_string(), Some(5), true)))
        );
        assert_eq!(
            parse_replace(r#":replace abc "" nocase "#),
            Ok(("", Op::new_replace("abc".to_string(), "".to_string(), None, true)))
        );
        assert_eq!(
            parse_replace(r#":replace abc '' nocase "#),
            Ok(("", Op::new_replace("abc".to_string(), "".to_string(), None, true)))
        );
        assert_eq!(
            parse_replace(r#":replace abc def nocase "#),
            Ok(("", Op::new_replace("abc".to_string(), "def".to_string(), None, true)))
        );
    }

    #[test]
    fn test_parse_uniq() {
        assert_eq!(parse_uniq(":uniq "), Ok(("", Op::new_uniq(false))));
        assert_eq!(parse_uniq(":uniq nocase "), Ok(("", Op::new_uniq(true))));
    }

    #[test]
    fn test_parse_peek() {
        assert_eq!(parse_peek(":peek "), Ok(("", Op::new_peek_to_std_out())));
        assert_eq!(parse_peek(":peek :abc"), Ok((":abc", Op::new_peek_to_std_out())));
        assert_eq!(parse_peek(":peek out.txt"), Ok(("", Op::new_peek_to_file("out.txt".to_string(), false, None))));
        assert_eq!(
            parse_peek(":peek out.txt append"),
            Ok(("", Op::new_peek_to_file("out.txt".to_string(), true, None)))
        );
        assert_eq!(
            parse_peek(":peek out.txt append crlf"),
            Ok(("", Op::new_peek_to_file("out.txt".to_string(), true, Some(true))))
        );
        assert_eq!(
            parse_peek(":peek out.txt crlf"),
            Ok(("", Op::new_peek_to_file("out.txt".to_string(), false, Some(true))))
        );
        assert_eq!(
            parse_peek(r#":peek "out .txt" "#),
            Ok((" ", Op::new_peek_to_file("out .txt".to_string(), false, None)))
        );
    }
}
