use crate::output::Output;
use std::iter::Peekable;

pub(in crate::parse::token) fn parse_output(
    token: &mut Peekable<impl Iterator<Item = String>>,
) -> Result<Output, String> {
    if let Some(to_cmd) = token.peek()
        && to_cmd.eq_ignore_ascii_case("to")
    {
        token.next(); // 消耗`to`
        match token.peek() {
            Some(output) => {
                if output.eq_ignore_ascii_case("file") {
                    parse_file(token)
                } else if output.eq_ignore_ascii_case("clip") {
                    parse_clip(token)
                } else {
                    Ok(Output::new_std_out())
                }
            }
            None => Ok(Output::new_std_out()),
        }
    } else {
        Ok(Output::new_std_out())
    }
}

fn parse_file(token: &mut Peekable<impl Iterator<Item = String>>) -> Result<Output, String> {
    token.next(); // 消耗`file`
    if let Some(file) = token.next() {
        // 必须文件名，直接消耗
        let (append, crlf) = if let Some(append_or_ending) = token.peek() {
            if append_or_ending.eq_ignore_ascii_case("append") {
                token.next(); // 消耗`append`
                if let Some(crlf) = token.peek() {
                    if crlf.eq_ignore_ascii_case("crlf") {
                        token.next(); // 消耗`crlf`
                        (true, Some(true))
                    } else if crlf.eq_ignore_ascii_case("lf") {
                        token.next(); // 消耗`lf`
                        (true, Some(false))
                    } else {
                        (true, None)
                    }
                } else {
                    (true, None)
                }
            } else if append_or_ending.eq_ignore_ascii_case("crlf") {
                token.next(); // 消耗`crlf`
                (false, Some(true))
            } else if append_or_ending.eq_ignore_ascii_case("lf") {
                token.next(); // 消耗`lf`
                (false, Some(false))
            } else {
                (false, None)
            }
        } else {
            (false, None)
        };
        Ok(Output::new_file(file, append, crlf))
    } else {
        Err("`file` argument of cmd `to file` is required".to_string())
    }
}

fn parse_clip(token: &mut Peekable<impl Iterator<Item = String>>) -> Result<Output, String> {
    token.next(); // 消耗`clip`
    Ok(Output::new_clip())
}
