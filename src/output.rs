use crate::input::{Item, Pipe};
use std::fs::OpenOptions;
use std::io::Write;

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum Output {
    /// 输出到标准输出：
    /// ```
    /// to out
    /// ```
    StdOut,
    /// 输出到文件：
    /// ```
    /// to file <file_name>[ append][ cr|crlf]
    ///
    /// to file file_name
    /// to file file_name append
    /// to file file_name crlf
    /// to file file_name lf
    /// to file file_name append crlf
    /// to file file_name append lf
    /// ```
    File { file: String, append: bool, crlf: Option<bool> },
    /// 输出到剪切板：
    /// ```
    /// to clip
    /// ```
    Clip,
}

impl Output {
    pub(crate) fn new_std_out() -> Self {
        Output::StdOut
    }
    pub(crate) fn new_file(file: String, append: bool, crlf: Option<bool>) -> Self {
        Output::File { file, append, crlf }
    }
    pub(crate) fn new_clip() -> Self {
        Output::Clip
    }

    pub(crate) fn handle(self, pipe: Pipe) {
        match self {
            Output::StdOut => {
                for item in pipe {
                    match item {
                        Item::String(string) => println!("{}", string),
                        Item::Integer(integer) => println!("{}", integer),
                    }
                }
            }
            Output::File { file, append, crlf } => {
                match OpenOptions::new().write(true).truncate(!append).append(append).create(true).open(&file) {
                    Ok(mut writer) => match crlf {
                        Some(true) => {
                            for x in pipe {
                                if let Err(err) = write!(writer, "{}\r\n", String::from(x)) {
                                    on_save_failed(&file, &err);
                                    return;
                                }
                            }
                        }
                        _ => {
                            for x in pipe {
                                if let Err(err) = write!(writer, "{}\n", String::from(x)) {
                                    on_save_failed(&file, &err);
                                    return;
                                }
                            }
                        }
                    },
                    Err(err) => on_save_failed(&file, &err),
                }
            }
            Output::Clip => {}
        }
    }
}

fn on_save_failed(file: &str, err: &std::io::Error) {
    eprintln!("Save to File {file} error: {}", err);
}
