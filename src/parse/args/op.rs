use crate::err::RpErr;
use crate::op::Op;
use crate::parse::args::{consume_if, consume_if_some};
use std::iter::Peekable;

pub(in crate::parse::args) fn parse_ops(args: &mut Peekable<impl Iterator<Item = String>>) -> Result<Vec<Op>, RpErr> {
    let mut ops = vec![];
    while let Some(op) = parse_op(args)? {
        ops.push(op);
    }
    Ok(ops)
}

fn parse_op(args: &mut Peekable<impl Iterator<Item = String>>) -> Result<Option<Op>, RpErr> {
    match args.peek() {
        Some(cmd) => {
            if cmd.eq_ignore_ascii_case("upper") {
                parse_upper(args)
            } else if cmd.eq_ignore_ascii_case("lower") {
                parse_lower(args)
            } else if cmd.eq_ignore_ascii_case("case") {
                parse_case(args)
            } else if cmd.eq_ignore_ascii_case("replace") {
                parse_replace(args)
            } else if cmd.eq_ignore_ascii_case("uniq") {
                parse_uniq(args)
            } else if cmd.eq_ignore_ascii_case("peek") {
                parse_peek(args)
            } else {
                Ok(None)
            }
        }
        None => Ok(None),
    }
}

fn parse_upper(args: &mut Peekable<impl Iterator<Item = String>>) -> Result<Option<Op>, RpErr> {
    args.next();
    Ok(Some(Op::new_upper()))
}

fn parse_lower(args: &mut Peekable<impl Iterator<Item = String>>) -> Result<Option<Op>, RpErr> {
    args.next();
    Ok(Some(Op::new_lower()))
}

fn parse_case(args: &mut Peekable<impl Iterator<Item = String>>) -> Result<Option<Op>, RpErr> {
    args.next();
    Ok(Some(Op::new_case()))
}

fn parse_replace(args: &mut Peekable<impl Iterator<Item = String>>) -> Result<Option<Op>, RpErr> {
    args.next();
    // 被替换字符串必选，直接消耗
    if let Some(from) = args.next() {
        // 替换目标字符串必选，直接消耗
        if let Some(to) = args.next() {
            let count_opt = consume_if_some(args, |s| s.parse::<usize>().ok());
            let nocase = consume_if(args, |s| s.eq_ignore_ascii_case("nocase")).is_some();
            Ok(Some(Op::new_replace(from, to, count_opt, nocase)))
        } else {
            Err(RpErr::MissingArg { cmd: "replace", arg: "to" })
        }
    } else {
        Err(RpErr::MissingArg { cmd: "replace", arg: "from" })
    }
}

fn parse_uniq(args: &mut Peekable<impl Iterator<Item = String>>) -> Result<Option<Op>, RpErr> {
    args.next();
    let nocase = args.peek().map(|nocase| nocase.eq_ignore_ascii_case("nocase")).unwrap_or(false);
    if nocase {
        args.next();
    }
    Ok(Some(Op::new_uniq(nocase)))
}

fn parse_peek(args: &mut Peekable<impl Iterator<Item = String>>) -> Result<Option<Op>, RpErr> {
    args.next();
    let nocase = args.peek().map(|nocase| nocase.eq_ignore_ascii_case("nocase")).unwrap_or(false);
    if nocase {
        args.next();
    }
    Ok(Some(Op::new_uniq(nocase)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse::args::build_args;

    #[test]
    fn test_non_match() {
        let mut args = build_args("");
        assert_eq!(Ok(None), parse_op(&mut args));
        assert_eq!(Some("".to_string()), args.next());
    }

    #[test]
    fn test_parse_upper() {
        let mut args = build_args("upper");
        assert_eq!(Ok(Some(Op::new_upper())), parse_op(&mut args));
        assert!(args.next().is_none());
    }

    #[test]
    fn test_parse_lower() {
        let mut args = build_args("lower");
        assert_eq!(Ok(Some(Op::new_lower())), parse_op(&mut args));
        assert!(args.next().is_none());
    }

    #[test]
    fn test_parse_case() {
        let mut args = build_args("case");
        assert_eq!(Ok(Some(Op::new_case())), parse_op(&mut args));
        assert!(args.next().is_none());
    }

    #[test]
    fn test_parse_replace() {
        let mut args = build_args("replace 123 abc");
        assert_eq!(Ok(Some(Op::new_replace("123".to_string(), "abc".to_string(), None, false))), parse_op(&mut args));
        assert!(args.next().is_none());

        let mut args = build_args("replace 123 abc 10");
        assert_eq!(
            Ok(Some(Op::new_replace("123".to_string(), "abc".to_string(), Some(10), false))),
            parse_op(&mut args)
        );
        assert!(args.next().is_none());

        let mut args = build_args("replace 123 abc nocase");
        assert_eq!(Ok(Some(Op::new_replace("123".to_string(), "abc".to_string(), None, true))), parse_op(&mut args));
        assert!(args.next().is_none());

        let mut args = build_args("replace 123 abc 10 nocase");
        assert_eq!(
            Ok(Some(Op::new_replace("123".to_string(), "abc".to_string(), Some(10), true))),
            parse_op(&mut args)
        );
        assert!(args.next().is_none());

        let mut args = build_args("replace 123");
        assert_eq!(Err(RpErr::MissingArg { cmd: "replace", arg: "to" }), parse_op(&mut args));
        assert!(args.next().is_none());

        let mut args = build_args("replace");
        assert_eq!(Err(RpErr::MissingArg { cmd: "replace", arg: "from" }), parse_op(&mut args));
        assert!(args.next().is_none());
    }

    #[test]
    fn test_parse_uniq() {
        let mut args = build_args("uniq");
        assert_eq!(Ok(Some(Op::new_uniq(false))), parse_op(&mut args));
        assert!(args.next().is_none());

        let mut args = build_args("uniq nocase");
        assert_eq!(Ok(Some(Op::new_uniq(true))), parse_op(&mut args));
        assert!(args.next().is_none());
    }

    #[test]
    fn test_parse_peek() {
        let mut args = build_args("uniq");
        assert_eq!(Ok(Some(Op::new_uniq(false))), parse_op(&mut args));
        assert!(args.next().is_none());

        let mut args = build_args("uniq nocase");
        assert_eq!(Ok(Some(Op::new_uniq(true))), parse_op(&mut args));
        assert!(args.next().is_none());
    }
}
