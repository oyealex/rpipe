use crate::Integer;
use std::fs::File;
use std::io;
use std::io::{BufRead, BufReader};

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum Item {
    Integer(Integer),
    String(String),
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum Input {
    /// 标准输入
    StdIn,
    /// 外部文件
    File { files: Vec<String> },
    /// 剪切板
    Clip,
    /// 直接字面值
    Of { values: Vec<String> },
    /// 整数生成器
    Gen { start: Integer, end: Integer, included: bool, step: Integer },
}

impl Input {
    pub(crate) fn iter(self) -> Box<dyn Iterator<Item = Item>> {
        match self {
            Input::StdIn => Box::new(
                io::stdin()
                    .lock()
                    .lines()
                    .into_iter()
                    .take_while(Result::is_ok)
                    .map(|line| Item::String(line.unwrap())),
            ),
            Input::File { files } => Box::new(
                files
                    .into_iter()
                    .map(File::open)
                    .take_while(Result::is_ok)
                    .map(Result::unwrap)
                    .map(BufReader::new)
                    .flat_map(|reader| BufRead::lines(reader).into_iter())
                    .take_while(Result::is_ok)
                    .map(|line| Item::String(line.unwrap())),
            ),
            Input::Clip => {
                todo!()
            }
            Input::Of { values } => Box::new(values.into_iter().map(Item::String)),
            Input::Gen { start, end, included, step } => Box::new(
                if step > 0 {
                    range_to_iter(start, end, included, step)
                } else {
                    range_to_iter(if included { end } else { end + 1 }, start, true, step)
                }
                .map(|x| Item::Integer(x)),
            ),
        }
    }
}

#[inline]
fn range_to_iter(start: Integer, end: Integer, included: bool, step: Integer) -> Box<dyn Iterator<Item = Integer>> {
    if step > 0 {
        Box::new(IntegerIter { start, end, included, step, next: start })
    } else {
        let (start, end) = (if included { end + 1 } else { end }, start);
        Box::new(
            IntegerIter { start: end, end: start, included, step: -step, next: if included { start } else { start - 1 } - step }
                .rev(),
        )
    }
}

#[test]
fn test_range_to_iter() {
    println!("{:?}", range_to_iter(0, 10, false, 1).collect::<Vec<_>>());
    println!("{:?}", range_to_iter(0, 10, true, 2).collect::<Vec<_>>());
    println!("{:?}", range_to_iter(0, 10, false, -1).collect::<Vec<_>>());
    println!("{:?}", range_to_iter(0, 10, true, -2).collect::<Vec<_>>());
}

#[derive(Debug, Eq, PartialEq)]
struct IntegerIter {
    start: Integer,
    end: Integer,
    included: bool,
    step: Integer,
    next: Integer,
}

impl Iterator for IntegerIter {
    type Item = Integer;

    fn next(&mut self) -> Option<Self::Item> {
        // dbg!(&self);
        let res = if self.included && self.next > self.end || !self.included && self.next >= self.end {
            None
        } else {
            Some(self.next)
        };
        self.next += if res.is_none() { 0 } else { self.step };
        res
    }
}

impl DoubleEndedIterator for IntegerIter {
    fn next_back(&mut self) -> Option<Self::Item> {
        // dbg!(&self);
        let pre = self.next - self.step;
        let res = if pre < self.start { None } else { Some(pre) };
        self.next = if res.is_none() { self.next } else { pre };
        res
    }
}
