use nom::branch::alt;
use nom::bytes::complete::{tag_no_case, take_until, take_while1};
use nom::character::complete::char;
use nom::character::complete::space1;
use nom::combinator::{map, verify};
use nom::error::Error;
use nom::multi::many_till;
use nom::sequence::{delimited, preceded, terminated};
use nom::{IResult, Parser};

/// 构造一个解析器，支持解析：
///  - `cmd arg `：命令+单个参数；
///  - `cmd [ arg ] `：命令+单个参数，中括号包围；
///  - `cmd [ arg0 arg1 ] `：命令+一个以上的参数，中括号包围；
pub(super) fn cmd_arg_or_args1<'a>(
    cmd: &'static str,
) -> impl Parser<&'a str, Output = Vec<String>, Error = Error<&'a str>> {
    alt((
        map(cmd_arg(cmd), |arg| vec![arg]), // 单个参数
        cmd_args1(cmd),                     // 多个参数
    ))
}

/// 构造一个解析器，支持解析：
///  - `cmd arg `：命令+单个参数；
pub(super) fn cmd_arg<'a>(
    cmd: &'static str,
) -> impl Parser<&'a str, Output = String, Error = Error<&'a str>> {
    terminated(
        preceded(
            (tag_no_case(cmd), space1), // 丢弃：命令标记和空格
            arg,                        // 参数
        ),
        space1, // 丢弃：结尾空格
    )
}

/// 构造一个解析器，支持解析：
///  - `cmd [ arg ] `：命令+单个参数，中括号包围；
///  - `cmd [ arg0 arg1 ] `：命令+一个以上的参数，中括号包围；
pub(super) fn cmd_args1<'a>(
    cmd: &'static str,
) -> impl Parser<&'a str, Output = Vec<String>, Error = Error<&'a str>> {
    map(
        terminated(
            preceded(
                // 丢弃： 命令标记、空格、左括号、空格
                (tag_no_case(cmd), space1, char('['), space1),
                verify(
                    many_till(
                        terminated(arg, space1), // 参数、空格（丢弃）
                        char(']'),               // 忽略：右括号
                    ),
                    // OPT 2025-12-25 00:57 是否支持空的参数列表？
                    |(args, _)| !args.is_empty(), // 验证：参数非空
                ),
            ),
            space1, // 丢弃：结尾空格
        ),
        |(args, _)| args,
    )
}

/// 解析器，支持解析单个参数。
pub(super) fn arg(input: &str) -> IResult<&str, String> {
    // TODO 2025-12-24 23:29 实现完整的单个参数解析
    let result = map(
        verify(
            alt((
                delimited(char('"'), take_until("\""), char('"')), // 带引号的参数
                take_while1(|c: char| !c.is_whitespace() && c != '"'), // 不带引号的文件名
            )),
            |arg: &str| arg != "[" && arg != "]", // 验证：不能是单个括号
        ),
        |arg: &str| arg.to_string(),
    )
    .parse(input);
    dbg!(&result);
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cmd_arg_or_args1() {
        assert_eq!(
            cmd_arg_or_args1("cmd").parse("cmd arg "),
            Ok(("", vec!["arg".to_string()]))
        );
        assert_eq!(
            cmd_arg_or_args1("cmd").parse("cmd [ arg ] "),
            Ok(("", vec!["arg".to_string()]))
        );
        assert_eq!(
            cmd_arg_or_args1("cmd").parse("cmd [ arg arg1 ] "),
            Ok(("", vec!["arg".to_string(), "arg1".to_string()]))
        );
        assert_eq!(
            cmd_arg_or_args1("cmd").parse(r#"cmd [ arg "arg 1" ] "#),
            Ok(("", vec!["arg".to_string(), "arg 1".to_string()]))
        );
        assert!(cmd_arg_or_args1("cmd").parse("cmd").is_err());
        assert!(cmd_arg_or_args1("cmd").parse("cmd ").is_err());
        assert!(cmd_arg_or_args1("cmd").parse("cmd [ arg ").is_err());
        assert!(cmd_arg_or_args1("cmd").parse("cmd [ ] ").is_err());
        assert!(cmd_arg_or_args1("cmd").parse("cmd [ [ ] ").is_err());
        assert!(cmd_arg_or_args1("cmd").parse("cmd ] ").is_err());
    }

    #[test]
    fn test_cmd_arg() {
        assert_eq!(
            cmd_arg("cmd").parse("cmd arg "),
            Ok(("", "arg".to_string()))
        );
        assert_eq!(
            cmd_arg("cmd").parse(r#"cmd "ar g" "#),
            Ok(("", "ar g".to_string()))
        );
        assert!(cmd_arg("cmd1").parse("cmd arg ").is_err());
    }

    #[test]
    fn test_cmd_args1() {
        assert_eq!(
            cmd_args1("cmd").parse("cmd [ arg ] "),
            Ok(("", vec!["arg".to_string()]))
        );
        assert_eq!(
            cmd_args1("cmd").parse("cmd [ arg1 arg2 ] "),
            Ok(("", vec!["arg1".to_string(), "arg2".to_string()]))
        );
        assert_eq!(
            cmd_args1("cmd").parse(r#"cmd [ arg1 arg2 "arg 3" ] "#),
            Ok((
                "",
                vec!["arg1".to_string(), "arg2".to_string(), "arg 3".to_string()]
            ))
        );
        assert!(cmd_args1("cmd").parse(r#"cmd [ ] "#).is_err());
        assert!(cmd_args1("cmd").parse(r#"cmd [  ] "#).is_err());
    }

    #[test]
    fn test_arg() {
        assert_eq!(arg("hello"), Ok(("", "hello".to_string())));
        assert_eq!(arg("hello "), Ok((" ", "hello".to_string())));
        assert_eq!(arg("hello world"), Ok((" world", "hello".to_string())));
        assert_eq!(
            arg(r#"hello" world"#),
            Ok((r#"" world"#, "hello".to_string()))
        );
        assert_eq!(
            arg(r#""hello " world"#),
            Ok((r#" world"#, "hello ".to_string()))
        );
        assert!(arg(r#""hello "#).is_err());
        assert!(arg("[ ").is_err());
        assert!(arg("] ").is_err());
    }
}
