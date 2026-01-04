use crate::err::RpErr;
use crate::output::Output;
use crate::parse::args;
use args::parse_general_file_info;
use std::iter::Peekable;

pub(in crate::parse::args) fn parse_output(args: &mut Peekable<impl Iterator<Item = String>>) -> Result<Output, RpErr> {
    if let Some(to_cmd) = args.peek()
        && to_cmd.eq_ignore_ascii_case("to")
    {
        args.next(); // 消耗`to`
        match args.peek() {
            Some(output) => {
                if output.eq_ignore_ascii_case("file") {
                    parse_file(args)
                } else if output.eq_ignore_ascii_case("clip") {
                    parse_clip(args)
                } else if output.eq_ignore_ascii_case("out") {
                    parse_std_out(args)
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

fn parse_file(args: &mut Peekable<impl Iterator<Item = String>>) -> Result<Output, RpErr> {
    args.next(); // 消耗`file`
    if let Some((file, append, crlf)) = parse_general_file_info(args) {
        Ok(Output::new_file(file, append, crlf))
    } else {
        Err(RpErr::MissingArg { cmd: "to file", arg: "file" })
    }
}

fn parse_clip(args: &mut Peekable<impl Iterator<Item = String>>) -> Result<Output, RpErr> {
    args.next(); // 消耗`clip`
    Ok(Output::new_clip())
}

fn parse_std_out(args: &mut Peekable<impl Iterator<Item = String>>) -> Result<Output, RpErr> {
    args.next(); // 消耗`out`
    Ok(Output::new_std_out())
}
