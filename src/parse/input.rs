use crate::input::Input;
use crate::parse::base_parser::cmd_arg_or_args1;
use nom::branch::alt;
use nom::bytes::complete::tag_no_case;
use nom::character::complete::space1;
use nom::combinator::map;
use nom::{IResult, Parser};

pub(super) type InputResult<'a> = IResult<&'a str, Input>;

pub(super) fn parse_input(input: &str) -> InputResult<'_> {
    alt((parse_std_in, parse_file, parse_clip, parse_of)).parse(input)
}

fn parse_std_in(input: &str) -> InputResult<'_> {
    map((tag_no_case("in"), space1), |_| Input::StdIn).parse(input)
}

fn parse_file(input: &str) -> InputResult<'_> {
    map(cmd_arg_or_args1("file"), |files| Input::File { files }).parse(input)
}

fn parse_clip(input: &str) -> InputResult<'_> {
    map((tag_no_case("clip"), space1), |_| Input::Clip).parse(input)
}

fn parse_of(input: &str) -> InputResult<'_> {
    map(cmd_arg_or_args1("of"), |values| Input::Of { values }).parse(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_std_in() {
        assert_eq!(parse_std_in("in "), Ok(("", Input::StdIn)));
        assert_eq!(parse_std_in("IN "), Ok(("", Input::StdIn)));
        assert!(parse_std_in("ina ").is_err());
    }

    #[test]
    fn test_parse_file() {
        assert_eq!(
            parse_file("file f.txt "),
            Ok((
                "",
                Input::File {
                    files: vec!["f.txt".to_string()]
                }
            ))
        );
        assert_eq!(
            parse_file(r#"file "f .txt" "#),
            Ok((
                "",
                Input::File {
                    files: vec!["f .txt".to_string()]
                }
            ))
        );
        assert_eq!(
            parse_file("file [ f.txt ] "),
            Ok((
                "",
                Input::File {
                    files: vec!["f.txt".to_string()]
                }
            ))
        );
        assert_eq!(
            parse_file(r#"file [ f.txt "f .txt" ] "#),
            Ok((
                "",
                Input::File {
                    files: vec!["f.txt".to_string(), "f .txt".to_string()]
                }
            ))
        );
        assert!(parse_file("files f.txt ").is_err());
        assert!(parse_file("file [ ] ").is_err());
        assert!(parse_file("file [  ] ").is_err());
        assert!(parse_file("file [ [ ] ").is_err());
        assert!(parse_file("file [ ] ] ").is_err());
        assert!(parse_file("file [ f.txt [ ] ").is_err());
    }

    #[test]
    fn test_parse_clip() {
        assert_eq!(parse_clip("clip "), Ok(("", Input::Clip)));
        assert!(parse_clip("clip").is_err());
    }

    #[test]
    fn test_parse_of() {
        assert_eq!(
            parse_of("of str "),
            Ok((
                "",
                Input::Of {
                    values: vec!["str".to_string()]
                }
            ))
        );
        assert_eq!(
            parse_of(r#"of "s tr" "#),
            Ok((
                "",
                Input::Of {
                    values: vec!["s tr".to_string()]
                }
            ))
        );
        assert_eq!(
            parse_of("of [ str ] "),
            Ok((
                "",
                Input::Of {
                    values: vec!["str".to_string()]
                }
            ))
        );
        assert_eq!(
            parse_of(r#"of [ str "s tr" ] "#),
            Ok((
                "",
                Input::Of {
                    values: vec!["str".to_string(), "s tr".to_string()]
                }
            ))
        );
        assert!(parse_of("ofs str ").is_err());
        assert!(parse_of("of [ ] ").is_err());
        assert!(parse_of("of [  ] ").is_err());
        assert!(parse_of("of [ [ ] ").is_err());
        assert!(parse_of("of [ ] ] ").is_err());
        assert!(parse_of("of [ str [ ] ").is_err());
    }
}
