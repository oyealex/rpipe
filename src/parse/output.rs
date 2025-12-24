use crate::output::Output;
use crate::parse::base_parser::arg;
use nom::branch::alt;
use nom::bytes::tag_no_case;
use nom::character::complete::space1;
use nom::combinator::{map, success};
use nom::sequence::{preceded, terminated};
use nom::IResult;
use nom::Parser;

pub(super) type OutputResult<'a> = IResult<&'a str, Output>;

pub(super) fn parse_out(input: &str) -> OutputResult<'_> {
    alt((
        parse_to_file,
        parse_to_clip,
        map(success(()), |_| Output::Out), // 最后默认使用`Output::Out`
    ))
    .parse(input)
}

fn parse_to_file(input: &str) -> OutputResult<'_> {
    map(
        terminated(
            preceded(
                (tag_no_case("to"), space1, tag_no_case("file"), space1), // 丢弃：`to file `
                arg,                                                      // 文件
            ),
            space1, // 丢弃：结尾空格
        ),
        |file| Output::File { file },
    )
    .parse(input)
}

fn parse_to_clip(input: &str) -> OutputResult<'_> {
    map(
        (tag_no_case("to"), space1, tag_no_case("clip"), space1), // 丢弃：`to clip `
        |_| Output::Clip,
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
            Ok((
                "",
                Output::File {
                    file: "out.txt".to_string()
                }
            ))
        );
        assert_eq!(
            parse_to_file(r#"to file "out .txt" "#),
            Ok((
                "",
                Output::File {
                    file: "out .txt".to_string()
                }
            ))
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
