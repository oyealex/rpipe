use crate::op::Op;
use std::iter::Peekable;

pub(in crate::parse::token) fn parse_ops(
    token: &mut Peekable<impl Iterator<Item = String>>,
) -> Result<Vec<Op>, String> {
    let mut ops = vec![];
    while let Some(op) = parse_op(token)? {
        ops.push(op);
    }
    Ok(ops)
}

fn parse_op(token: &mut Peekable<impl Iterator<Item = String>>) -> Result<Option<Op>, String> {
    match token.peek() {
        Some(cmd) => {
            if cmd.eq_ignore_ascii_case("upper") {
                parse_upper(token)
            } else if cmd.eq_ignore_ascii_case("lower") {
                parse_lower(token)
            } else if cmd.eq_ignore_ascii_case("replace") {
                parse_replace(token)
            } else if cmd.eq_ignore_ascii_case("uniq") {
                parse_uniq(token)
            } else {
                Ok(None)
            }
        }
        None => Ok(None),
    }
}

fn parse_upper(token: &mut Peekable<impl Iterator<Item = String>>) -> Result<Option<Op>, String> {
    token.next();
    Ok(Some(Op::new_upper()))
}

fn parse_lower(token: &mut Peekable<impl Iterator<Item = String>>) -> Result<Option<Op>, String> {
    token.next();
    Ok(Some(Op::new_lower()))
}

fn parse_replace(token: &mut Peekable<impl Iterator<Item = String>>) -> Result<Option<Op>, String> {
    token.next();
    if let Some(from) = token.next() {
        if let Some(to) = token.next() {
            if let Some(count_or_nocase) = token.next() {
                if count_or_nocase.eq_ignore_ascii_case("nocase") {
                    Ok(Some(Op::new_replace(from, Some(to), None, true)))
                } else {
                    let count = count_or_nocase
                        .parse::<usize>()
                        .map_err(|e| format!("failed to parse `count` argument of `replace`: {e}"))?;
                    let nocase = token.peek().map(|nocase| nocase.eq_ignore_ascii_case("nocase")).unwrap_or(false);
                    if nocase {
                        token.next();
                    }
                    Ok(Some(Op::new_replace(from, Some(to), Some(count), nocase)))
                }
            } else {
                Ok(Some(Op::new_replace(from, Some(to), None, false)))
            }
        } else {
            Ok(Some(Op::new_replace(from, None, None, false)))
        }
    } else {
        Err("`from` argument of cmd `replace` is required".to_string())
    }
}

fn parse_uniq(token: &mut Peekable<impl Iterator<Item = String>>) -> Result<Option<Op>, String> {
    token.next();
    let nocase = token.peek().map(|nocase| nocase.eq_ignore_ascii_case("nocase")).unwrap_or(false);
    if nocase {
        token.next();
    }
    Ok(Some(Op::new_uniq(nocase)))
}
