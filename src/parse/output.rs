use crate::output::Output;
use crate::parse::arg;
use crate::parse::ParserError;
use nom::branch::alt;
use nom::bytes::complete::tag_no_case;
use nom::character::complete::space1;
use nom::combinator::{map, opt, success};
use nom::error::context;
use nom::sequence::{preceded, terminated};
use nom::IResult;
use nom::Parser;

pub(super) type OutputResult<'a> = IResult<&'a str, Output, ParserError<'a>>;

pub(super) fn parse_out(input: &'static str) -> OutputResult<'static> {
    context(
        "Output",
        alt((
            parse_to_file,
            parse_to_clip,
            context("Output::Out", map(success(()), |_| Output::Out)), // 最后默认使用`Output::Out`
        )),
    )
    .parse(input)
}

/// 解析：
/// ```
/// to file file_name
/// to file file_name append
/// to file file_name lf
/// to file file_name crlf
/// to file file_name append lf
/// to file file_name append crlf
/// ```
fn parse_to_file(input: &'static str) -> OutputResult<'static> {
    context(
        "Output::File",
        map(
            terminated(
                preceded(
                    (tag_no_case("to"), space1, tag_no_case("file"), space1), // 丢弃：`to file `
                    (
                        arg,                                                                  // 文件
                        opt((space1, tag_no_case("append"))),                                 // 是否追加
                        opt(preceded(space1, alt((tag_no_case("lf"), tag_no_case("crlf"))))), // 换行符
                    ),
                ),
                space1, // 丢弃：结尾空格
            ),
            |(file, append_opt, ending_opt): (String, Option<_>, Option<&str>)| Output::File {
                file,
                append: append_opt.is_some(),
                crlf: ending_opt.map(|s| s.eq_ignore_ascii_case("crlf")),
            },
        ),
    )
    .parse(input)
}

fn parse_to_clip(input: &str) -> OutputResult<'_> {
    context(
        "Output::Clip",
        map(
            (tag_no_case("to"), space1, tag_no_case("clip"), space1), // 丢弃：`to clip `
            |_| Output::Clip,
        ),
    )
    .parse(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_to_file() {
        assert_eq!(
            parse_to_file("to file out.txt "),
            Ok(("", Output::File { file: "out.txt".to_string(), append: false, crlf: None }))
        );
        assert_eq!(
            parse_to_file("to file out.txt append "),
            Ok(("", Output::File { file: "out.txt".to_string(), append: true, crlf: None }))
        );
        assert_eq!(
            parse_to_file("to file out.txt append crlf "),
            Ok(("", Output::File { file: "out.txt".to_string(), append: true, crlf: Some(true) }))
        );
        assert_eq!(
            parse_to_file("to file out.txt crlf "),
            Ok(("", Output::File { file: "out.txt".to_string(), append: false, crlf: Some(true) }))
        );
        assert_eq!(
            parse_to_file(r#"to file "out .txt" "#),
            Ok(("", Output::File { file: "out .txt".to_string(), append: false, crlf: None }))
        );
        assert!(parse_to_file("to").is_err());
        assert!(parse_to_file("to file ").is_err());
        assert!(parse_to_file("to file [").is_err());
    }

    #[test]
    fn test_parse_to_clip() {
        assert_eq!(parse_to_clip("to clip "), Ok(("", Output::Clip)));
        assert_eq!(parse_to_clip("to  clip  "), Ok(("", Output::Clip)));
        assert!(parse_to_clip("to ").is_err());
    }
}
