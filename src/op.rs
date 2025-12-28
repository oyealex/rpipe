use crate::input::{Item, Pipe};
use std::borrow::Cow;
use std::ops::Deref;

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum Op {
    Upper, // OPT 2026-12-29 01:23 使用Unicode的大小写。
    Lower, // OPT 2026-12-29 01:23 使用Unicode的大小写。
    Replace { from: &'static str, to: &'static str, count: Option<usize>, nocase: bool },
}

impl Op {
    pub(crate) fn wrap(self, pipe: Pipe) -> Pipe {
        match self {
            Op::Upper => pipe.op_map(|mut item| match &mut item {
                // OPT 2026-12-29 01:24 Pipe增加属性以优化重复大小写。
                Item::String(string) => {
                    if string.chars().all(|c| c.is_ascii_uppercase()) {
                        item
                    } else {
                        string.make_ascii_uppercase();
                        item
                    }
                }
                Item::Str(string) => {
                    if string.chars().all(|c| c.is_ascii_uppercase()) {
                        item
                    } else {
                        Item::String(string.to_ascii_uppercase())
                    }
                }
                Item::Integer(_) => item,
            }),
            Op::Lower => pipe.op_map(|mut item| match &mut item {
                // OPT 2026-12-29 01:24 Pipe增加属性以优化重复大小写。
                Item::String(string) => {
                    if string.chars().all(|c| c.is_ascii_lowercase()) {
                        item
                    } else {
                        string.make_ascii_lowercase();
                        item
                    }
                }
                Item::Str(string) => {
                    if string.chars().all(|c| c.is_ascii_lowercase()) {
                        item
                    } else {
                        Item::String(string.to_ascii_lowercase())
                    }
                }
                Item::Integer(_) => item,
            }),
            Op::Replace { from, to, count, nocase } => {
                if to == "" || count == Some(0) {
                    pipe
                } else {
                    pipe.op_map(move |mut item| match &item {
                        Item::String(string) => {
                            let cow = replace_with_count_and_nocase(string, from, to, count, nocase);
                            match cow {
                                Cow::Borrowed(_) => item,
                                Cow::Owned(string) => Item::String(string),
                            }
                        }
                        Item::Str(string) => {
                            let cow = replace_with_count_and_nocase(string, from, to, count, nocase);
                            match cow {
                                Cow::Borrowed(_) => item,
                                Cow::Owned(string) => Item::String(string),
                            }
                        }
                        Item::Integer(integer) => {
                            let integer_str = integer.to_string();
                            let cow = replace_with_count_and_nocase(&integer_str, from, to, count, nocase);
                            match cow {
                                Cow::Borrowed(_) => item,
                                Cow::Owned(string) => Item::String(string),
                            }
                        }
                    })
                }
            }
        }
    }
}

/// 替换字符串
///
/// # Arguments
/// * `text` - 原始字符串
/// * `from` - 要被替换的子串
/// * `to` - 替换后的字符串
/// * `count` - 替换次数
/// * `nocase` - 是否忽略大小写
///
/// # Returns
/// 返回替换后的字符串（如果无替换发生，返回原字符串的引用以避免分配）
fn replace_with_count_and_nocase<'a>(
    text: &'a str, from: &str, to: &str, count: Option<usize>, nocase: bool,
) -> Cow<'a, str> {
    let mut result = String::new();
    let mut last_end = 0;
    let mut replaced_count = 0;
    let max_replacements = count.unwrap_or(usize::MAX);

    let (lower_text_holder, lower_from_holder); // 保持下方的&str引用有效
    // 根据是否忽略大小写选择匹配函数
    let (actual_text, actual_from) = if nocase {
        lower_text_holder = text.to_lowercase();
        lower_from_holder = from.to_lowercase();
        (&lower_text_holder as &str, &lower_from_holder as &str)
    } else {
        (text, from)
    };

    let matches = actual_text.match_indices(from);
    for (start, end) in matches {
        if replaced_count >= max_replacements {
            break;
        }
        result.push_str(&text[last_end..start]); // 添加从上一个结束位置到当前匹配开始位置的文本
        result.push_str(to); // 添加替换文本
        last_end = start + end.len();
        replaced_count += 1;
    }

    if replaced_count == 0 {
        Cow::Borrowed(text) // 无替换发生，直接返回原字符串
    } else {
        result.push_str(&text[last_end..]); // 添加剩余文本
        Cow::Owned(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        let item = Item::String("abc".to_string());
        println!("{:p}", &item);
        let item = match item {
            Item::String(string) => Item::String(string),
            Item::Str(string) => Item::Str(string),
            Item::Integer(integer) => Item::Integer(integer),
        };
        println!("{:p}", &item);
    }

    #[test]
    fn test_replace_with_count_and_nocase() {
        assert_eq!(replace_with_count_and_nocase("abc ABC abc abc", "abc", "1234", None, false), "1234 ABC 1234 1234");
        assert_eq!(replace_with_count_and_nocase("abc ABC abc abc", "abc", "1234", None, true), "1234 1234 1234 1234");
        assert_eq!(replace_with_count_and_nocase("abc ABC abc abc", "abc", "1234", Some(0), false), "abc ABC abc abc");
        assert_eq!(replace_with_count_and_nocase("abc ABC abc abc", "abc", "1234", Some(0), true), "abc ABC abc abc");
        assert_eq!(
            replace_with_count_and_nocase("abc ABC abc abc", "abc", "1234", Some(2), false),
            "1234 ABC 1234 abc"
        );
        assert_eq!(replace_with_count_and_nocase("abc ABC abc abc", "abc", "1234", Some(2), true), "1234 1234 abc abc");
        assert_eq!(
            replace_with_count_and_nocase("abc ABC abc abc", "", "1234", Some(2), true),
            "1234a1234bc ABC abc abc"
        );
        assert_eq!(replace_with_count_and_nocase("abc", "", "_", None, true), "_a_b_c_");
    }
}
