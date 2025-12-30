use crate::input::{Item, Pipe};
use std::fs::OpenOptions;
use std::io::Write;

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum Output {
    Out,
    File { file: String, append: bool, crlf: Option<bool> },
    Clip,
}

impl Output {
    pub(crate) fn handle(self, pipe: Pipe) {
        match self {
            Output::Out => {
                for item in pipe {
                    match item {
                        Item::String(string) => println!("{}", string),
                        Item::Integer(integer) => println!("{}", integer),
                    }
                }
            }
            Output::File { file, append, crlf } => {
                match OpenOptions::new().write(true).truncate(!append).create(true).open(&file) {
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
