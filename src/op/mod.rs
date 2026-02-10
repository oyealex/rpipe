mod replace;
mod slice;
pub(crate) mod trim;

use crate::condition::Condition;
use crate::config::{Config, is_nocase};
use crate::err::RpErr;
use crate::fmt::{FmtArg, fmt_args};
use crate::op::replace::ReplaceArg;
use crate::op::slice::SliceIter;
use crate::op::trim::TrimArg;
use crate::pipe::Pipe;
use crate::{Float, Integer, Num, PipeRes};
use cmd_help::CmdHelp;
use itertools::Itertools;
use ordered_float::OrderedFloat;
use rand::seq::SliceRandom;
use rustc_hash::FxHashSet;
use std::borrow::Cow;
use std::cmp::Reverse;
use std::fs::OpenOptions;
use std::io::Write;
use unicase::UniCase;

#[derive(Debug, PartialEq, CmdHelp)]
pub(crate) enum Op {
    /* **************************************** 访问 **************************************** */
    /// :peek       打印每个值到标准输出或文件。
    ///             :peek[ <file>][ append][ lf|crlf]
    ///                 <file>  文件路径，可选。
    ///                 append  追加输出而不是覆盖，可选，如果未指定则覆盖源文件。
    ///                 lf|crlf 指定换行符为'LF'或'CRLF'，可选，如果未指定则默认使用'LF'。
    ///             例如：
    ///                 :peek
    ///                 :peek file.txt
    ///                 :peek file.txt append
    ///                 :peek file.txt lf
    ///                 :peek file.txt crlf
    ///                 :peek file.txt append crlf
    Peek(PeekArg),
    /* **************************************** 转换 **************************************** */
    /// :upper      转为ASCII大写。
    /// :lower      转为ASCII小写。
    /// :case       切换ASCII大小写。
    Case(CaseArg),
    /// :replace    替换字符串。
    ///             :replace <from> <to>[ <count>][ nocase]
    ///                 <from>  待替换的字符串，必选。
    ///                 <to>    待替换为的字符串，必选。
    ///                 <count> 对每个元素需要替换的次数，必须为正整数，可选，未指定则替换所有。
    ///                 nocase  替换时忽略大小写，可选，未指定时不忽略大小写。
    ///             例如：
    ///                 :replace abc xyz
    ///                 :replace abc xyz 10
    ///                 :replace abc xyz nocase
    ///                 :replace abc xyz 10 nocase
    Replace(ReplaceArg),
    /// :trim       去除首尾指定的子串。
    ///             :trim[ <pattern>[ nocase]]
    ///                 <pattern>   需要去除的子串，可选，留空则去除空白字符。
    ///                 nocase      忽略大小写，可选，仅当指定了<pattern>时生效。
    /// :ltrim      去除首部指定的子串。
    ///             :ltrim[ <pattern>[ nocase]]
    ///                 <pattern>   需要去除的子串，可选，留空则去除空白字符。
    ///                 nocase      忽略大小写，可选，仅当指定了<pattern>时生效。
    /// :rtrim      去除尾部指定的子串。
    ///             :rtrim[ <pattern>[ nocase]]
    ///                 <pattern>   需要去除的子串，可选，留空则去除空白字符。
    ///                 nocase      忽略大小写，可选，仅当指定了<pattern>时生效。
    /// :trimc      去除首尾指定范围内的字符。
    ///             :trimc[ <pattern>[ nocase]]
    ///                 <pattern>   需要去除的字符，可选，留空则去除空白字符。
    ///                 nocase      忽略大小写，可选，仅当指定了<pattern>时生效。
    /// :ltrimc     去除首部指定范围内的字符。
    ///             :ltrimc[ <pattern>[ nocase]]
    ///                 <pattern>   需要去除的字符，可选，留空则去除空白字符。
    ///                 nocase      忽略大小写，可选，仅当指定了<pattern>时生效。
    /// :rtrimc     去除尾部指定范围内的字符。
    ///             :rtrimc[ <pattern>[ nocase]]
    ///                 <pattern>   需要去除的字符，可选，留空则去除空白字符。
    ///                 nocase      忽略大小写，可选，仅当指定了<pattern>时生效。
    /// :trimr      去除首尾满足指定正则的字串。
    ///             :trimr <regex>
    ///                 <regex>     需要去除的正则，必选。
    /// :ltrimr     去除首部满足指定正则的字串。
    ///             :ltrimr <regex>
    ///                 <regex>     需要去除的正则，必选。
    /// :rtrimr     去除尾部满足指定正则的字串。
    ///             :rtrimr <regex>
    ///                 <regex>     需要去除的正则，必选。
    Trim(TrimArg),
    /* **************************************** 减少 **************************************** */
    /// :limit      保留前N个数据，丢弃后续的其他数据。
    ///             :limit <count>
    ///                 <count> 需要保留的数量，必须为非负整数，必选。
    /// :skip       丢弃前N个数据，保留后续的其他数据。
    ///             :skip <count>
    ///                 <count> 需要保留的数量，必须为非负整数，必选。
    /// :slice      对数据切片，保留指定索引范围内的数据，丢弃其他数据。
    ///             支持指定多个范围，操作不会对范围进行排序或合并，严格按照给定的范围选择数据。
    ///             如果一个范围无效，例如范围开始值大于结束值，此范围会被丢弃。
    ///             :slice [ <range>][...]
    ///                 <range> 切片范围，格式：<start>,<end>，如果不指定任何范围则丢弃全部数据。
    ///                     <start> 范围起始索引，包含，与<end>至少指定一个。
    ///                     <end>   范围起始索引，包含，与<start>至少指定一个。
    Slice { ranges: Vec<(Option<usize>, Option<usize>)> },
    /// :uniq       去重。
    ///             :uniq[ nocase]
    ///                 nocase  去重时忽略大小写，可选，未指定时不忽略大小写。
    ///             例如：
    ///                 :uniq
    ///                 :uniq nocase
    Uniq { nocase: bool },
    /// :sum        累加数据流中的数值，支持可选的格式化参数。
    ///             对输入流中的每个文本项，尝试转换为整数或浮点数，成功则累加，失败按 0 处理。
    ///             :sum[ <fmt>]
    ///                 <fmt>   格式化字符串，以{v}表示累加结果的数值。
    ///                         更多格式化信息参考`-h fmt`。
    ///             例如：
    ///                 :sum
    ///                 :sum "Result: {v}"
    ///                 :sum "Total: {v}"
    ///                 :sum "Sum = {v:#04x}"
    Sum { fmt: Option<String> },
    /// :join       合并数据。
    ///             :join<[ <delimiter>[ <prefix>[ <postfix>[ <batch>]]]]
    ///                 <delimiter> 分隔字符串，可选。
    ///                 <prefix>    前缀字符串，可选。
    ///                             指定前缀字符串时必须指定分割字符串。
    ///                 <postfix>   后缀字符串，可选。
    ///                             指定后缀字符串时必须指定分割字符串和前缀字符串。
    ///                 <batch>     分组大小，必须为正整数，可选，未指定时所有数据为一组。
    ///                             指定分组大小时必须指定分隔字符串、前缀字符串和后缀字符串。
    ///             例如：
    ///                 :join ,
    ///                 :join , [ ]
    ///                 :join , [ ] 3
    Join { join_info: JoinInfo, batch: Option<usize> },
    /// :drop       根据指定条件选择数据丢弃，其他数据保留。
    ///             :drop <condition>
    ///                 <condition> 条件表达式，参考`-h cond`或`-h condition`
    /// :take       根据指定条件选择数据保留，其他数据丢弃。
    ///             :take <condition>
    ///                 <condition> 条件表达式，参考`-h cond`或`-h condition`
    /// :drop while 根据指定条件选择数据持续丢弃，直到条件首次不满足。
    ///             :drop while <condition>
    ///                 <condition> 条件表达式，参考`-h cond`或`-h condition`
    /// :take while 根据指定条件选择数据持续保留，直到条件首次不满足。
    ///             :take while <condition>
    ///                 <condition> 条件表达式，参考`-h cond`或`-h condition`
    TakeDrop { mode: TakeDropMode, cond: Condition },
    /// :count      统计数据数量。
    ///             :count
    Count,
    /* **************************************** 增加 **************************************** */
    /* **************************************** 调整位置 **************************************** */
    /// :sort       排序。
    ///             :sort[ num [<default>]][ nocase][ desc][ random]
    ///                 num         按照数值排序，可选，未指定时按照字典序排序。
    ///                             尝试将文本解析为数值后排序，无法解析的按照<default>排序。
    ///                 <default>   仅按照数值排序时生效，无法解析为数值的文本的默认数值，可选，
    ///                             未指定时按照数值最大值处理。
    ///                 nocase      忽略大小写，仅按字典序排序时生效，可选，未指定时不忽略大小写。
    ///                 desc        逆序排序，可选，未指定时正序排序。
    ///                 random      随机排序，与按照数值排序和字典序排序互斥，且不支持逆序。
    ///             例如：
    ///                 :sort
    ///                 :sort desc
    ///                 :sort nocase
    ///                 :sort nocase desc
    ///                 :sort num
    ///                 :sort num desc
    ///                 :sort num 10
    ///                 :sort num 10 desc
    ///                 :sort num 10.5
    ///                 :sort num 10.5 desc
    ///                 :sort random
    Sort { sort_by: SortBy, desc: bool },
}

impl Op {
    pub(crate) fn new_replace(from: String, to: String, count: Option<usize>, nocase: bool) -> Op {
        Op::Replace(ReplaceArg::new(from, to, count, nocase))
    }
    pub(crate) fn new_join(join_info: JoinInfo, count: Option<usize>) -> Op {
        Op::Join { join_info, batch: count }
    }
    pub(crate) fn new_take_drop(mode: TakeDropMode, cond: Condition) -> Op {
        Op::TakeDrop { mode, cond }
    }
    pub(crate) fn new_sort(sort_by: SortBy, desc: bool) -> Op {
        Op::Sort { sort_by, desc }
    }

    pub(crate) fn wrap(self, mut pipe: Pipe, configs: &'static [Config]) -> PipeRes {
        match self {
            Op::Peek(peek) => match peek {
                PeekArg::StdOut => Ok(pipe.op_inspect(|item| println!("{item}"))),
                PeekArg::File { file, append, crlf } => {
                    match OpenOptions::new().write(true).truncate(!append).append(append).create(true).open(&file) {
                        Ok(mut writer) => {
                            let postfix = if crlf.unwrap_or(false) { "\r\n" } else { "\n" };
                            Ok(pipe.op_inspect(move |item| {
                                if let Err(err) = write!(writer, "{item}{postfix}") {
                                    RpErr::WriteToFileErr {
                                        file: file.clone(),
                                        item: item.to_string(),
                                        err: err.to_string(),
                                    }
                                    .termination()
                                }
                            }))
                        }
                        Err(err) => RpErr::OpenFileErr { file, err: err.to_string() }.termination(),
                    }
                }
            },
            Op::Case(case_arg) => match case_arg {
                CaseArg::Lower => Ok(pipe.op_map(|mut item|
                    // OPT 2026-12-29 01:24 Pipe增加属性以优化重复大小写。
                    if item.chars().all(|c| c.is_ascii_lowercase()) {
                        item
                    } else {
                        item.make_ascii_lowercase();
                        item
                    }
                )),
                CaseArg::Upper => Ok(pipe.op_map(|mut item|
                    // OPT 2026-12-29 01:24 Pipe增加属性以优化重复大小写。
                    if item.chars().all(|c| c.is_ascii_uppercase()) {
                        item
                    } else {
                        item.make_ascii_uppercase();
                        item
                    }
                )),
                CaseArg::Switch => Ok(pipe.op_map(|mut item| {
                    // 只修改ASCII字母（范围A-Z/a-z），而ASCII字符在UTF-8中就是单字节，
                    // 且切换大小写后仍是合法ASCII（从而合法UTF-8）。
                    for b in unsafe { item.as_bytes_mut() } {
                        match b {
                            b'A'..=b'Z' => *b += b'a' - b'A',
                            b'a'..=b'z' => *b -= b'a' - b'A',
                            _ => {}
                        }
                    }
                    item
                })),
            },
            Op::Replace(replace_arg) => {
                if replace_arg.count == Some(0) {
                    Ok(pipe)
                } else {
                    Ok(pipe.op_map(move |item| {
                        let cow = replace_arg.replace(&item, configs);
                        match cow {
                            Cow::Borrowed(_) => item,
                            Cow::Owned(string) => string,
                        }
                    }))
                }
            }
            Op::Trim(trim_arg) => Ok(pipe.op_map(move |s| trim_arg.trim(s, configs))),
            // OPT 2026-01-22 01:10 针对 limit 0、skip 0 等命令进行优化
            Op::Slice { ranges } => Ok(Pipe { iter: Box::new(SliceIter::new(pipe, ranges)) }),
            Op::Uniq { nocase } => {
                let mut seen = FxHashSet::default();
                Ok(pipe.op_filter(move |item| {
                    let key = if is_nocase(nocase, configs) { item.to_ascii_uppercase() } else { item.clone() };
                    seen.insert(key)
                }))
            }
            Op::Sum { fmt } => {
                // 使用 Num::sum 进行流式累加，更符合 Rust 惯用法
                let acc = pipe
                    .filter_map(|s| s.parse::<Num>().ok()) // 解析失败的项目被过滤掉（视为0）
                    .sum::<Num>();
                // 根据是否有格式化参数决定输出格式
                let out = if let Some(fmt_str) = fmt {
                    match fmt_args(&fmt_str, &[("v", FmtArg::from(acc))]) {
                        Ok(string) => string,
                        Err(err) => err.termination(),
                    }
                } else {
                    // 根据类型决定输出格式
                    match acc {
                        Num::Integer(i) => i.to_string(),
                        Num::Float(f) => {
                            // 如果小数部分为 0，显示为整数
                            if f.fract() == 0.0 { (f as Integer).to_string() } else { f.to_string() }
                        }
                    }
                };
                Ok(Pipe { iter: Box::new(std::iter::once(out)) })
            }
            Op::Join { join_info, batch: count } => {
                if let Some(count) = count {
                    if count > 0 {
                        return Ok(Pipe { iter: Box::new(ChunkJoin { source: pipe, group_size: count, join_info }) });
                    } else {
                        unreachable!("join count must be greater than zero");
                    }
                }
                Ok(Pipe {
                    iter: Box::new(std::iter::once(format!(
                        "{}{}{}",
                        join_info.prefix,
                        pipe.join(&join_info.delimiter),
                        join_info.postfix
                    ))),
                })
            }
            Op::TakeDrop { mode, cond } => match mode {
                TakeDropMode::Take => Ok(Pipe { iter: Box::new(pipe.filter(move |s| cond.test(s))) }),
                TakeDropMode::Drop => Ok(Pipe { iter: Box::new(pipe.filter(move |s| !cond.test(s))) }),
                TakeDropMode::TakeWhile => Ok(Pipe { iter: Box::new(pipe.take_while(move |s| cond.test(s))) }),
                TakeDropMode::DropWhile => Ok(Pipe { iter: Box::new(pipe.skip_while(move |s| cond.test(s))) }),
            },
            Op::Count => Ok(Pipe { iter: Box::new(std::iter::once(pipe.count().to_string())) }),
            Op::Sort { sort_by, desc } => match sort_by {
                SortBy::Num(def_integer, def_float) => {
                    if let Some(def) = def_integer {
                        let key_fn = move |item: &String| item.parse().unwrap_or(def);
                        let new_pipe = if desc {
                            pipe.sorted_by_key(|item| Reverse(key_fn(item)))
                        } else {
                            pipe.sorted_by_key(key_fn)
                        };
                        return Ok(Pipe { iter: Box::new(new_pipe) });
                    }
                    let def = def_float.unwrap_or(Float::MAX); // 默认按照浮点最大值
                    let key_fn = move |item: &String| OrderedFloat(item.parse().unwrap_or(def));
                    let new_pipe = if desc {
                        pipe.sorted_by_key(|item| Reverse(key_fn(item)))
                    } else {
                        pipe.sorted_by_key(key_fn)
                    };
                    Ok(Pipe { iter: Box::new(new_pipe) })
                }
                SortBy::Text(nocase) => {
                    // TODO 2026-01-08 02:34 使用UniCase优化其他nocase场景
                    let iter = if is_nocase(nocase, configs) {
                        if desc {
                            pipe.sorted_by_key(|item| Reverse(UniCase::new(item.to_string())))
                        } else {
                            pipe.sorted_by_key(|item| UniCase::new(item.to_string()))
                        }
                    } else if desc {
                        pipe.sorted_by_key(|item| Reverse(item.to_string()))
                    } else {
                        pipe.sorted_by_key(|item| item.to_string())
                    };
                    Ok(Pipe { iter: Box::new(iter) })
                }
                SortBy::Random => {
                    let mut v = pipe.collect::<Vec<_>>();
                    v.shuffle(&mut rand::rng());
                    Ok(Pipe { iter: Box::new(v.into_iter()) })
                }
            },
        }
    }
}

#[derive(Debug, PartialEq)]
pub(crate) enum CaseArg {
    Upper,
    Lower,
    Switch,
}

#[derive(Debug, PartialEq)]
pub(crate) enum PeekArg {
    StdOut,
    File { file: String, append: bool, crlf: Option<bool> },
}

#[derive(Debug, PartialEq)]
pub(crate) enum SortBy {
    Num(Option<Integer>, Option<Float>),
    Text(bool /*nocase*/),
    Random,
}

#[derive(Debug, PartialEq)]
pub(crate) enum TakeDropMode {
    Take,
    Drop,
    TakeWhile,
    DropWhile,
}

#[derive(Debug, PartialEq, Default)]
pub(crate) struct JoinInfo {
    pub(crate) delimiter: String,
    pub(crate) prefix: String,
    pub(crate) postfix: String,
}

struct ChunkJoin<I: Iterator<Item = String>> {
    source: I,
    group_size: usize,
    join_info: JoinInfo,
}

impl<I> Iterator for ChunkJoin<I>
where
    I: Iterator<Item = String>,
{
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        let mut chunk = Vec::with_capacity(self.group_size);
        for _ in 0..self.group_size {
            if let Some(item) = self.source.next() {
                chunk.push(item);
            } else {
                break;
            }
        }
        if chunk.is_empty() {
            None
        } else {
            Some(format!(
                "{}{}{}",
                self.join_info.prefix,
                chunk.join(&self.join_info.delimiter),
                self.join_info.postfix
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pipe::Pipe;

    #[test]
    fn test_sum_without_fmt() {
        let input = Pipe { iter: Box::new(vec!["1", "2", "3"].into_iter().map(|s| s.to_string())) };
        let result = Op::Sum { fmt: None }.wrap(input, &[]).unwrap();
        let output: Vec<String> = result.collect();
        assert_eq!(output, vec!["6"]);
    }

    #[test]
    fn test_sum_with_fmt() {
        let input = Pipe { iter: Box::new(vec!["1", "2", "3"].into_iter().map(|s| s.to_string())) };
        let result = Op::Sum { fmt: Some("Result: {v}".to_string()) }.wrap(input, &[]).unwrap();
        let output: Vec<String> = result.collect();
        assert_eq!(output, vec!["Result: 6"]);
    }

    #[test]
    fn test_sum_with_custom_fmt() {
        let input = Pipe { iter: Box::new(vec!["10", "20", "30"].into_iter().map(|s| s.to_string())) };
        let result = Op::Sum { fmt: Some("Total: {v}".to_string()) }.wrap(input, &[]).unwrap();
        let output: Vec<String> = result.collect();
        assert_eq!(output, vec!["Total: 60"]);
    }

    #[test]
    fn test_sum_with_hex_fmt() {
        let input = Pipe { iter: Box::new(vec!["10", "20", "30"].into_iter().map(|s| s.to_string())) };
        let result = Op::Sum { fmt: Some("Sum = {v}".to_string()) }.wrap(input, &[]).unwrap();
        let output: Vec<String> = result.collect();
        assert_eq!(output, vec!["Sum = 60"]);
    }

    #[test]
    fn test_sum_with_float_input() {
        let input = Pipe { iter: Box::new(vec!["1.5", "2.5", "3.0"].into_iter().map(|s| s.to_string())) };
        let result = Op::Sum { fmt: Some("{v}".to_string()) }.wrap(input, &[]).unwrap();
        let output: Vec<String> = result.collect();
        assert_eq!(output, vec!["7"]);
    }

    #[test]
    fn test_sum_with_mixed_input() {
        let input = Pipe { iter: Box::new(vec!["1", "2.5", "abc", "3"].into_iter().map(|s| s.to_string())) };
        let result = Op::Sum { fmt: None }.wrap(input, &[]).unwrap();
        let output: Vec<String> = result.collect();
        assert_eq!(output, vec!["6.5"]);
    }
}
