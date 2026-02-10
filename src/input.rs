use crate::config::{Config, skip_err};
use crate::err::RpErr;
use crate::fmt::{FmtArg, fmt_args};
use crate::pipe::Pipe;
use crate::{Integer, PipeRes};
use cmd_help::CmdHelp;
use std::fs::File;
use std::io;
use std::io::{BufRead, BufReader};
use std::iter::repeat;
use std::rc::Rc;

#[derive(Debug, Eq, PartialEq, CmdHelp)]
pub(crate) enum Input {
    /// :in         从标准输入读取输入。
    ///             未指定元素输入时的默认输入。
    StdIn,
    /// :file       从文件读取输入。
    ///             :file <file>[ <file>][...]
    ///                 <file>  文件路径，至少指定一个。
    ///             例如：
    ///                 :file input.txt
    ///                 :file input1.txt input2.txt input3.txt
    File { files: Vec<String> },
    /// :clip       从剪切板读取输入。
    #[cfg(windows)]
    Clip,
    /// :of         使用直接字面值作为输入。
    ///             :of <text>[ <text][...]
    ///                 <text>  字面值，至少指定一个，如果以':'开头，需要使用'\:'转义。
    ///             例如：
    ///                 :of line
    ///                 :of line1 "line 2" 'line 3'
    Of { values: Vec<String> },
    /// :gen        生成指定范围内的整数作为输入，支持进一步格式化。
    ///             :gen <start>[,[<end>][,<step>]][ <fmt>]
    ///                 <start> 起始值，包含，必须。
    ///                 <end>   结束值，包含，可选。
    ///                         未指定时生成到整数最大值（取决于构建版本）。
    ///                         如果范围为空（起始值大于结束值），则无数据生成。
    ///                 <step>  步长，不能为0，可选，未指定时取步长为1。
    ///                         如果步长为正值，表示正序生成；
    ///                         如果步长为负值，表示逆序生成。
    ///                 <fmt>   格式化字符串，以{v}表示生成的整数值。
    ///                         更多格式化信息参考`-h fmt`。
    ///             例如：
    ///                 :gen 0          生成：0 1 2 3 4 5 ...
    ///                 :gen 0,         生成：0 1 2 3 4 5 ...
    ///                 :gen 0,10       生成：0 1 2 3 4 5 6 7 8 9
    ///                 :gen 0,10,2     生成：0 2 4 6 8
    ///                 :gen 0,,2       生成：0 2 4 6 8 10 12 14 ...
    ///                 :gen 10,0       无数据生成
    ///                 :gen 0,10,-1    生成：9 8 7 6 5 4 3 2 1
    ///                 :gen 0,10 n{v}  生成：n0 n1 n2 n3 n4 n5 n6 n7 n8 n9
    ///                 :gen 0,10 "Hex of {v} is {v:#04x}" 生成：
    ///                                 "Hex of 0 is 0x00"
    ///                                 "Hex of 1 is 0x01"
    ///                                 ...
    Gen { start: Integer, end: Integer, step: Integer, fmt: Option<String> },
    /// :repeat     重复字面值作为输入。
    ///             :repeat <value>[ <count>]
    ///                 <value> 需要重复的字面值，必选。
    ///                 <count> 需要重复的次数，必须为非负数，可选，未指定时重复无限次数。
    Repeat { value: String, count: Option<usize> },
}

impl Input {
    pub(crate) fn new_std_in() -> Input {
        Input::StdIn
    }
    pub(crate) fn new_file(files: Vec<String>) -> Input {
        Input::File { files }
    }

    #[cfg(windows)]
    pub(crate) fn new_clip() -> Input {
        Input::Clip
    }
    pub(crate) fn new_of(values: Vec<String>) -> Input {
        Input::Of { values }
    }
    pub(crate) fn new_gen(start: Integer, end: Integer, step: Integer, fmt: Option<String>) -> Input {
        Input::Gen { start, end, step, fmt }
    }
    pub(crate) fn new_repeat(value: String, count: Option<usize>) -> Input {
        Input::Repeat { value, count }
    }
}

impl Input {
    pub(crate) fn try_into(self, configs: &'static [Config]) -> PipeRes {
        match self {
            Input::StdIn => Ok(Pipe {
                iter: Box::new(io::stdin().lock().lines().take_while(Result::is_ok).map(|line| line.unwrap())),
            }),
            Input::File { files } => Ok(Pipe {
                iter: Box::new(
                    files
                        .into_iter()
                        .map(|f| (File::open(&f), f))
                        .filter_map(|(r, f)| match r {
                            Ok(fin) => Some((fin, f)),
                            Err(err) => {
                                if skip_err(configs) {
                                    None
                                } else {
                                    RpErr::OpenFileErr { file: f, err: err.to_string() }.termination();
                                }
                            }
                        })
                        .map(|(fin, f)| (BufReader::new(fin), Rc::new(f)))
                        .flat_map(|(reader, f)| BufRead::lines(reader).enumerate().map(move |l| (l, f.clone())))
                        .filter_map(|((line, lr), f)| match lr {
                            Ok(line) => Some(line),
                            Err(err) => {
                                if skip_err(configs) {
                                    None
                                } else {
                                    RpErr::ReadFromFileErr { file: (*f).clone(), line_no: line, err: err.to_string() }
                                        .termination();
                                }
                            }
                        }),
                ),
            }),
            #[cfg(windows)]
            Input::Clip => match clipboard_win::get_clipboard_string() {
                Ok(text) => Ok(Pipe { iter: Box::new(OwnedSplitLines::new(text)) }),
                Err(err) => Err(RpErr::ReadClipboardTextErr(err.to_string())),
            },
            Input::Of { values } => Ok(Pipe { iter: Box::new(values.into_iter()) }),
            Input::Gen { start, end, step, fmt } => {
                if let Some(fmt) = fmt {
                    Ok(Pipe {
                        iter: Box::new(range_to_iter(start, end, step).map(move |x| {
                            match fmt_args(&fmt, &[("v", FmtArg::from(x))]) {
                                Ok(string) => string,
                                Err(err) => err.termination(),
                            }
                        })),
                    })
                } else {
                    Ok(Pipe { iter: Box::new(range_to_iter(start, end, step).map(|s| s.to_string())) })
                }
            }
            Input::Repeat { value, count } => Ok(if let Some(count_value) = count {
                Pipe { iter: Box::new(std::iter::repeat_n(value, count_value)) }
            } else {
                Pipe { iter: Box::new(repeat(value)) }
            }),
        }
    }
}

fn range_to_iter(start: Integer, end: Integer, step: Integer) -> Box<dyn DoubleEndedIterator<Item = Integer>> {
    let iter = RangeIter { start, end, step: Integer::abs(step), next: start, next_back: end };
    if step < 0 { Box::new(iter.rev()) } else { Box::new(iter) }
}

#[derive(Debug, Eq, PartialEq)]
struct RangeIter {
    start: Integer,
    end: Integer,
    step: Integer,
    next: Integer,
    next_back: Integer,
}

impl Iterator for RangeIter {
    type Item = Integer;

    fn next(&mut self) -> Option<Self::Item> {
        if self.next >= self.start && self.next <= self.end && self.next <= self.next_back {
            let res = Some(self.next);
            self.next += self.step;
            res
        } else {
            None
        }
    }
}

impl DoubleEndedIterator for RangeIter {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.next_back >= self.start && self.next_back <= self.end && self.next_back >= self.next {
            let res = Some(self.next_back);
            self.next_back -= self.step;
            res
        } else {
            None
        }
    }
}

#[derive(Debug)]
struct OwnedSplitLines {
    text: String,
    pos: usize,
}

impl OwnedSplitLines {
    fn new(text: String) -> Self {
        Self { text, pos: 0 }
    }
}

impl Iterator for OwnedSplitLines {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos > self.text.len() {
            return None;
        }
        let rest = &self.text[self.pos..];
        let newline_pos = rest.find('\n');
        match newline_pos {
            Some(idx) => {
                let line = rest[..idx].to_string();
                self.pos += idx + 1;
                Some(line)
            }
            None if self.pos < self.text.len() => {
                let line = rest.to_string();
                self.pos = self.text.len() + 1;
                Some(line)
            }
            None if self.pos == self.text.len() => {
                self.pos += 1;
                Some(String::new())
            }
            None => None,
        }
    }
}

#[cfg(test)]
mod iter_tests {
    use super::*;

    #[test]
    fn test_owned_split_lines_basic() {
        let text = String::from("line1\nline2\nline3");
        let iter = OwnedSplitLines::new(text);
        assert_eq!(iter.collect::<Vec<_>>(), vec!["line1", "line2", "line3"]);
    }

    #[test]
    fn test_owned_split_lines_empty() {
        let text = String::new();
        let iter = OwnedSplitLines::new(text);
        assert_eq!(iter.collect::<Vec<_>>(), vec![String::new()]);
    }

    #[test]
    fn test_owned_split_lines_trailing_newline() {
        let text = String::from("line1\nline2\n");
        let iter = OwnedSplitLines::new(text);
        assert_eq!(iter.collect::<Vec<_>>(), vec!["line1", "line2", ""]);
    }

    #[test]
    fn test_owned_split_lines_single_line() {
        let text = String::from("single");
        let iter = OwnedSplitLines::new(text);
        assert_eq!(iter.collect::<Vec<_>>(), vec!["single"]);
    }

    #[test]
    fn test_range_to_iter_positive() {
        assert_eq!(range_to_iter(0, 10, 1).collect::<Vec<_>>(), (0..=10).collect::<Vec<_>>());
        assert_eq!(range_to_iter(0, 10, 2).collect::<Vec<_>>(), (0..=10).step_by(2).collect::<Vec<_>>());
    }

    #[test]
    fn test_range_to_iter_negative() {
        assert_eq!(range_to_iter(0, 10, -1).collect::<Vec<_>>(), (0..=10).rev().collect::<Vec<_>>());
        assert_eq!(range_to_iter(0, 10, -2).collect::<Vec<_>>(), (0..=10).rev().step_by(2).collect::<Vec<_>>());
    }

    #[test]
    fn test_range_to_iter_empty() {
        assert_eq!(range_to_iter(0, 0, 1).collect::<Vec<_>>(), (0..=0).collect::<Vec<_>>());
        assert_eq!(range_to_iter(0, 0, 2).collect::<Vec<_>>(), (0..=0).step_by(2).collect::<Vec<_>>());
    }

    #[allow(clippy::reversed_empty_ranges)]
    #[test]
    fn test_range_to_iter_reverted_range_and_positive() {
        assert_eq!(range_to_iter(10, 0, 1).collect::<Vec<_>>(), (10..=0).collect::<Vec<_>>());
        assert_eq!(range_to_iter(10, 0, 2).collect::<Vec<_>>(), (10..=0).step_by(2).collect::<Vec<_>>());
    }

    #[allow(clippy::reversed_empty_ranges)]
    #[test]
    fn test_range_to_iter_reverted_range_and_negative() {
        assert_eq!(range_to_iter(10, 0, -1).collect::<Vec<_>>(), (10..=0).rev().collect::<Vec<_>>());
        assert_eq!(range_to_iter(10, 0, -2).collect::<Vec<_>>(), (10..=0).rev().step_by(2).collect::<Vec<_>>());
    }

    #[test]
    fn test_range_to_iter_zero_step() {
        assert_eq!(Some(0), range_to_iter(0, 0, 0).next());
        assert_eq!(range_to_iter(0, 1, 0).take(10).collect::<Vec<_>>(), vec![0; 10].into_iter().collect::<Vec<_>>());
        assert_eq!(range_to_iter(0, 1, 0).take(100).collect::<Vec<_>>(), vec![0; 100].into_iter().collect::<Vec<_>>());
    }
}
