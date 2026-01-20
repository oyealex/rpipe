use crate::config::{is_nocase, Config};
use crate::err::RpErr;
use regex::Regex;
use std::collections::HashSet;

#[derive(Debug, PartialEq, Clone)]
pub(crate) enum TrimPos {
    Head,
    Tail,
    Both,
}

#[derive(Debug)]
pub(crate) enum TrimParam {
    Blank,
    Str(String),
    Chars(Vec<char>),
    Regex { primary: Regex, secondary: Option<Regex> /*仅用于Both时匹配Tail*/ },
}

#[derive(Debug, PartialEq)]
pub(crate) struct TrimArg {
    pos: TrimPos,
    param: TrimParam,
    nocase: bool,
}

impl PartialEq for TrimParam {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (TrimParam::Blank, TrimParam::Blank) => true,
            (TrimParam::Str(l), TrimParam::Str(r)) => l == r,
            (TrimParam::Chars(l), TrimParam::Chars(r)) => l == r,
            (TrimParam::Regex { primary: l_p, secondary: l_s }, TrimParam::Regex { primary: r_p, secondary: r_s }) => {
                l_p.as_str() == r_p.as_str()
                    && match (l_s, r_s) {
                        (Some(l), Some(r)) => l.as_str() == r.as_str(),
                        (None, None) => true,
                        (_, _) => false,
                    }
            }
            (_, _) => false,
        }
    }
}

impl TrimArg {
    pub(crate) fn new_blank(pos: TrimPos) -> TrimArg {
        TrimArg { pos, param: TrimParam::Blank, nocase: false }
    }
    pub(crate) fn new_str(pos: TrimPos, mut pattern: String, nocase: bool) -> TrimArg {
        let pattern = if nocase {
            pattern.make_ascii_lowercase();
            pattern
        } else {
            pattern
        };
        TrimArg { pos, param: TrimParam::Str(pattern), nocase }
    }
    pub(crate) fn new_chars(pos: TrimPos, mut pattern: String, nocase: bool) -> TrimArg {
        let pattern = if nocase {
            pattern.make_ascii_lowercase();
            pattern
        } else {
            pattern
        };
        let mut seen = HashSet::new();
        TrimArg { pos, param: TrimParam::Chars(pattern.chars().filter(|&c| seen.insert(c)).collect()), nocase }
    }
    pub(crate) fn new_regex(pos: TrimPos, reg: String) -> Result<TrimArg, RpErr> {
        let (primary, secondary) = match pos {
            TrimPos::Head => (Regex::new(&format!(r"\A(?:{})", reg)), None),
            TrimPos::Tail => (Regex::new(&format!(r"(?:{})\z", reg)), None),
            TrimPos::Both => (
                Regex::new(&format!(r"\A(?:{})", reg)),
                Some(
                    Regex::new(&format!(r"(?:{})\z", reg))
                        .map_err(|err| RpErr::ParseRegexErr { reg: reg.clone(), err: err.to_string() })?,
                ),
            ),
        };
        Ok(TrimArg {
            pos,
            param: TrimParam::Regex {
                primary: primary.map_err(|err| RpErr::ParseRegexErr { reg, err: err.to_string() })?,
                secondary,
            },
            nocase: false,
        })
    }

    pub(crate) fn trim(&self, to_trim: String, configs: &[Config]) -> String {
        let trimmed = match &self.param {
            TrimParam::Blank => to_trim.trim(),
            TrimParam::Str(pattern) => {
                if is_nocase(self.nocase, configs) {
                    match self.pos {
                        TrimPos::Head => Self::trim_head_str_nocase(&to_trim, &pattern),
                        TrimPos::Tail => Self::trim_tail_str_nocase(&to_trim, &pattern),
                        TrimPos::Both => {
                            Self::trim_tail_str_nocase(Self::trim_head_str_nocase(&to_trim, &pattern), &pattern)
                        }
                    }
                } else {
                    match self.pos {
                        TrimPos::Head => to_trim.strip_prefix(pattern).unwrap_or(&to_trim),
                        TrimPos::Tail => to_trim.strip_suffix(pattern).unwrap_or(&to_trim),
                        TrimPos::Both => {
                            let stripped = to_trim.strip_prefix(pattern).unwrap_or(&to_trim);
                            stripped.strip_suffix(pattern).unwrap_or(stripped)
                        }
                    }
                }
            }
            TrimParam::Chars(chars) => {
                if is_nocase(self.nocase, configs) {
                    match self.pos {
                        TrimPos::Head => Self::trim_head_char_nocase(&to_trim, &chars[..]),
                        TrimPos::Tail => Self::trim_tail_char_nocase(&to_trim, &chars[..]),
                        TrimPos::Both => {
                            Self::trim_tail_char_nocase(Self::trim_head_char_nocase(&to_trim, &chars[..]), &chars[..])
                        }
                    }
                } else {
                    match self.pos {
                        TrimPos::Head => Self::trim_head_char(&to_trim, &chars[..]),
                        TrimPos::Tail => Self::trim_tail_char(&to_trim, &chars[..]),
                        TrimPos::Both => Self::trim_tail_char(Self::trim_head_char(&to_trim, &chars[..]), &chars[..]),
                    }
                }
            }
            TrimParam::Regex { primary, secondary } => match self.pos {
                TrimPos::Head => Self::trim_head_regex(&to_trim, &primary),
                TrimPos::Tail => Self::trim_tail_regex(&to_trim, &primary),
                TrimPos::Both => {
                    let to_trim = Self::trim_head_regex(&to_trim, &primary);
                    if let Some(regex) = secondary { Self::trim_tail_regex(&to_trim, &regex) } else { to_trim }
                }
            },
        };
        if trimmed == &to_trim { to_trim } else { trimmed.to_owned() }
    }

    fn trim_head_str_nocase<'a>(to_trim: &'a str, pattern: &'a str) -> &'a str {
        let mut to_trim_chars = to_trim.char_indices();
        let mut pattern_chars = pattern.chars();
        loop {
            match (to_trim_chars.next(), pattern_chars.next()) {
                (Some((_, tc)), Some(pc)) => {
                    if tc.to_ascii_lowercase() != pc {
                        return to_trim; // 匹配失败，不截取
                    }
                }
                (None, Some(_)) => return to_trim,            // to_trim太短，不截取
                (Some((i, _)), None) => return &to_trim[i..], // 匹配完成
                (None, None) => return "",                    // 完全匹配，全部截取
            }
        }
    }

    fn trim_tail_str_nocase<'a>(to_trim: &'a str, pattern: &'a str) -> &'a str {
        let mut to_trim_chars = to_trim.char_indices().rev();
        let mut pattern_chars = pattern.chars().rev();
        loop {
            match (to_trim_chars.next(), pattern_chars.next()) {
                (Some((_, tc)), Some(pc)) => {
                    if tc.to_ascii_lowercase() != pc {
                        return to_trim; // 匹配失败，不截取
                    }
                }
                (None, Some(_)) => return to_trim, // to_trim太短，不截取
                (Some((i, tc)), None) => return &to_trim[..(i + tc.len_utf8())], // 匹配完成
                (None, None) => return "",         // 完全匹配，全部截取
            }
        }
    }

    fn trim_head_char_nocase<'a>(to_trim: &'a str, chars: &[char]) -> &'a str {
        let mut start_idx = 0;
        for ch in to_trim.chars() {
            if chars.iter().any(|p| p.eq(&ch.to_ascii_lowercase())) {
                start_idx += ch.len_utf8();
            } else {
                break;
            }
        }
        &to_trim[start_idx..]
    }

    fn trim_tail_char_nocase<'a>(to_trim: &'a str, chars: &[char]) -> &'a str {
        let mut end_idx = to_trim.len();
        for ch in to_trim.chars().rev() {
            if chars.iter().any(|p| p.eq(&ch.to_ascii_lowercase())) {
                end_idx -= ch.len_utf8();
            } else {
                break;
            }
        }
        &to_trim[..end_idx]
    }

    fn trim_head_char<'a>(to_trim: &'a str, chars: &[char]) -> &'a str {
        let start = to_trim.char_indices().find(|(_, c)| !chars.contains(c)).map_or(to_trim.len(), |(i, _)| i);
        if start == to_trim.len() { "" } else { &to_trim[start..] }
    }

    fn trim_tail_char<'a>(to_trim: &'a str, chars: &[char]) -> &'a str {
        let end = to_trim.char_indices().rfind(|(_, c)| !chars.contains(c)).map_or(0, |(i, c)| i + c.len_utf8());
        if end == 0 { "" } else { &to_trim[..end] }
    }

    fn trim_head_regex<'a>(text: &'a str, regex: &'a Regex) -> &'a str {
        if let Some(mat) = regex.find(text) { &text[mat.end()..] } else { text }
    }
    fn trim_tail_regex<'a>(text: &'a str, regex: &'a Regex) -> &'a str {
        if let Some(mat) = regex.find(text) { &text[..mat.start()] } else { text }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trim_blank() {
        let configs = vec![];
        assert_eq!("abc", TrimArg::new_blank(TrimPos::Head).trim("abc".to_owned(), &configs));
        assert_eq!("abc", TrimArg::new_blank(TrimPos::Head).trim(" \n  abc\n\t".to_owned(), &configs));
    }

    #[test]
    fn test_trim_char_nocase() {
        let configs = vec![];
        // head
        assert_eq!(
            "abc123abc",
            TrimArg::new_chars(TrimPos::Head, "_;+-=".to_owned(), true).trim("abc123abc".to_owned(), &configs)
        );
        assert_eq!(
            "23ABC",
            TrimArg::new_chars(TrimPos::Head, "cBAa1".to_owned(), true).trim("abc123ABC".to_owned(), &configs)
        );
        assert_eq!(
            "啊你好",
            TrimArg::new_chars(TrimPos::Head, "你好好".to_owned(), true).trim("你好你好啊你好".to_owned(), &configs)
        );
        assert_eq!(
            "1c好啊你好",
            TrimArg::new_chars(TrimPos::Head, "你好aBc".to_owned(), true)
                .trim("你a好b你c1c好啊你好".to_owned(), &configs)
        );
        assert_eq!(
            "",
            TrimArg::new_chars(TrimPos::Head, "你好啊abc".to_owned(), true).trim("a你".to_owned(), &configs)
        );
        // tail
        assert_eq!(
            "abc123abc",
            TrimArg::new_chars(TrimPos::Tail, "_;+-=".to_owned(), true).trim("abc123abc".to_owned(), &configs)
        );
        assert_eq!(
            "abc123",
            TrimArg::new_chars(TrimPos::Tail, "cBAa1".to_owned(), true).trim("abc123ABC".to_owned(), &configs)
        );
        assert_eq!(
            "你好你好啊",
            TrimArg::new_chars(TrimPos::Tail, "你好好".to_owned(), true).trim("你好你好啊你好".to_owned(), &configs)
        );
        assert_eq!(
            "你a好b你c1c好啊",
            TrimArg::new_chars(TrimPos::Tail, "你好aBc".to_owned(), true)
                .trim("你a好b你c1c好啊你好".to_owned(), &configs)
        );
        assert_eq!(
            "",
            TrimArg::new_chars(TrimPos::Tail, "你好啊abc".to_owned(), true).trim("a你".to_owned(), &configs)
        );
        // both
        assert_eq!(
            "abc123abc",
            TrimArg::new_chars(TrimPos::Both, "_;+-=".to_owned(), true).trim("abc123abc".to_owned(), &configs)
        );
        assert_eq!(
            "23",
            TrimArg::new_chars(TrimPos::Both, "cBAa1".to_owned(), true).trim("abc123ABC".to_owned(), &configs)
        );
        assert_eq!(
            "啊",
            TrimArg::new_chars(TrimPos::Both, "你好好".to_owned(), true).trim("你好你好啊你好".to_owned(), &configs)
        );
        assert_eq!(
            "1c好啊",
            TrimArg::new_chars(TrimPos::Both, "你好aBc".to_owned(), true)
                .trim("你a好b你c1c好啊你好".to_owned(), &configs)
        );
        assert_eq!(
            "",
            TrimArg::new_chars(TrimPos::Both, "你好啊abc".to_owned(), true).trim("a你".to_owned(), &configs)
        );
    }

    #[test]
    fn test_trim_char() {
        let configs = vec![];
        // head
        assert_eq!(
            "abc123abc",
            TrimArg::new_chars(TrimPos::Head, "_;+-=".to_owned(), false).trim("abc123abc".to_owned(), &configs)
        );
        assert_eq!(
            "23aBc",
            TrimArg::new_chars(TrimPos::Head, "aBc1".to_owned(), false).trim("acB123aBc".to_owned(), &configs)
        );
        assert_eq!(
            "啊你好",
            TrimArg::new_chars(TrimPos::Head, "你好好".to_owned(), false).trim("你好你好啊你好".to_owned(), &configs)
        );
        assert_eq!(
            "b你c1c好啊你好",
            TrimArg::new_chars(TrimPos::Head, "你好aBc".to_owned(), false)
                .trim("你a好b你c1c好啊你好".to_owned(), &configs)
        );
        assert_eq!(
            "",
            TrimArg::new_chars(TrimPos::Head, "你好啊abc".to_owned(), false).trim("a你".to_owned(), &configs)
        );
        // tail
        assert_eq!(
            "abc123abc",
            TrimArg::new_chars(TrimPos::Tail, "_;+-=".to_owned(), false).trim("abc123abc".to_owned(), &configs)
        );
        assert_eq!(
            "abc123ab",
            TrimArg::new_chars(TrimPos::Tail, "aBc1".to_owned(), false).trim("abc123abc".to_owned(), &configs)
        );
        assert_eq!(
            "你好你好啊",
            TrimArg::new_chars(TrimPos::Tail, "你好好".to_owned(), false).trim("你好你好啊你好".to_owned(), &configs)
        );
        assert_eq!(
            "你a好b你c1c好啊",
            TrimArg::new_chars(TrimPos::Tail, "你好aBc".to_owned(), false)
                .trim("你a好b你c1c好啊你好".to_owned(), &configs)
        );
        assert_eq!(
            "",
            TrimArg::new_chars(TrimPos::Tail, "你好啊abc".to_owned(), false).trim("a你".to_owned(), &configs)
        );
        // both
        assert_eq!(
            "abc123abc",
            TrimArg::new_chars(TrimPos::Both, "_;+-=".to_owned(), false).trim("abc123abc".to_owned(), &configs)
        );
        assert_eq!(
            "bc123ab",
            TrimArg::new_chars(TrimPos::Both, "aBc1".to_owned(), false).trim("abc123abc".to_owned(), &configs)
        );
        assert_eq!(
            "啊",
            TrimArg::new_chars(TrimPos::Both, "你好好".to_owned(), false).trim("你好你好啊你好".to_owned(), &configs)
        );
        assert_eq!(
            "b你c1c好啊",
            TrimArg::new_chars(TrimPos::Both, "你好aBc".to_owned(), false)
                .trim("你a好b你c1c好啊你好".to_owned(), &configs)
        );
        assert_eq!(
            "",
            TrimArg::new_chars(TrimPos::Both, "你好啊abc".to_owned(), false).trim("a你".to_owned(), &configs)
        );
    }

    #[test]
    fn test_trim_str_nocase() {
        let configs = vec![];
        // head
        assert_eq!(
            "abc123abc",
            TrimArg::new_str(TrimPos::Head, "_;+-=".to_owned(), true).trim("abc123abc".to_owned(), &configs)
        );
        assert_eq!(
            "abc123abCABC",
            TrimArg::new_str(TrimPos::Head, "abc".to_owned(), true).trim("abcabc123abCABC".to_owned(), &configs)
        );
        assert_eq!(
            "123aBc",
            TrimArg::new_str(TrimPos::Head, "acB".to_owned(), true).trim("acB123aBc".to_owned(), &configs)
        );
        assert_eq!(
            "好啊你好",
            TrimArg::new_str(TrimPos::Head, "你好你".to_owned(), true).trim("你好你好啊你好".to_owned(), &configs)
        );
        assert_eq!(
            "你a好b你c1c好啊你好",
            TrimArg::new_str(TrimPos::Head, "你好aBc".to_owned(), true)
                .trim("你a好b你c1c好啊你好".to_owned(), &configs)
        );
        assert_eq!(
            "啊你好你好aBc",
            TrimArg::new_str(TrimPos::Head, "你好aBc".to_owned(), true)
                .trim("你好aBc啊你好你好aBc".to_owned(), &configs)
        );
        assert_eq!(
            "a你",
            TrimArg::new_str(TrimPos::Head, "你好啊abc".to_owned(), true).trim("a你".to_owned(), &configs)
        );
        // tail
        assert_eq!(
            "abc123abc",
            TrimArg::new_str(TrimPos::Tail, "_;+-=".to_owned(), true).trim("abc123abc".to_owned(), &configs)
        );
        assert_eq!(
            "abcabc123abC",
            TrimArg::new_str(TrimPos::Tail, "abc".to_owned(), true).trim("abcabc123abCABC".to_owned(), &configs)
        );
        assert_eq!(
            "abc123abc",
            TrimArg::new_str(TrimPos::Tail, "aBc1".to_owned(), true).trim("abc123abc".to_owned(), &configs)
        );
        assert_eq!(
            "你好你好啊你好",
            TrimArg::new_str(TrimPos::Tail, "你好你".to_owned(), true).trim("你好你好啊你好".to_owned(), &configs)
        );
        assert_eq!(
            "你a好b你c1c好啊你好",
            TrimArg::new_str(TrimPos::Tail, "你好aBc".to_owned(), true)
                .trim("你a好b你c1c好啊你好".to_owned(), &configs)
        );
        assert_eq!(
            "你好aBc啊你好",
            TrimArg::new_str(TrimPos::Tail, "你好aBc".to_owned(), true)
                .trim("你好aBc啊你好你好aBc".to_owned(), &configs)
        );
        assert_eq!(
            "a你",
            TrimArg::new_str(TrimPos::Tail, "你好啊abc".to_owned(), true).trim("a你".to_owned(), &configs)
        );
        // both
        assert_eq!(
            "abc123abc",
            TrimArg::new_str(TrimPos::Both, "_;+-=".to_owned(), true).trim("abc123abc".to_owned(), &configs)
        );
        assert_eq!(
            "abc123abC",
            TrimArg::new_str(TrimPos::Both, "abc".to_owned(), true).trim("abcabc123abCABC".to_owned(), &configs)
        );
        assert_eq!(
            "23abc",
            TrimArg::new_str(TrimPos::Both, "aBc1".to_owned(), true).trim("abc123abc".to_owned(), &configs)
        );
        assert_eq!(
            "好啊你好",
            TrimArg::new_str(TrimPos::Both, "你好你".to_owned(), true).trim("你好你好啊你好".to_owned(), &configs)
        );
        assert_eq!(
            "你a好b你c1c好啊你好",
            TrimArg::new_str(TrimPos::Both, "你好aBc".to_owned(), true)
                .trim("你a好b你c1c好啊你好".to_owned(), &configs)
        );
        assert_eq!(
            "啊你好",
            TrimArg::new_str(TrimPos::Both, "你好aBc".to_owned(), true)
                .trim("你好aBc啊你好你好aBc".to_owned(), &configs)
        );
        assert_eq!(
            "a你",
            TrimArg::new_str(TrimPos::Both, "你好啊abc".to_owned(), true).trim("a你".to_owned(), &configs)
        );
    }

    #[test]
    fn test_trim_str() {
        let configs = vec![];
        // head
        assert_eq!(
            "abc123abc",
            TrimArg::new_str(TrimPos::Head, "_;+-=".to_owned(), false).trim("abc123abc".to_owned(), &configs)
        );
        assert_eq!(
            "aBcabc123abcabc",
            TrimArg::new_str(TrimPos::Head, "abc".to_owned(), false).trim("aBcabc123abcabc".to_owned(), &configs)
        );
        assert_eq!(
            "123acb",
            TrimArg::new_str(TrimPos::Head, "acB".to_owned(), false).trim("acB123acb".to_owned(), &configs)
        );
        assert_eq!(
            "好啊你好",
            TrimArg::new_str(TrimPos::Head, "你好你".to_owned(), false).trim("你好你好啊你好".to_owned(), &configs)
        );
        assert_eq!(
            "你a好b你c1c好啊你好",
            TrimArg::new_str(TrimPos::Head, "你好aBc".to_owned(), false)
                .trim("你a好b你c1c好啊你好".to_owned(), &configs)
        );
        assert_eq!(
            "啊你好你好abc",
            TrimArg::new_str(TrimPos::Head, "你好aBc".to_owned(), false)
                .trim("你好aBc啊你好你好abc".to_owned(), &configs)
        );
        assert_eq!(
            "a你",
            TrimArg::new_str(TrimPos::Head, "你好啊abc".to_owned(), false).trim("a你".to_owned(), &configs)
        );
        // tail
        assert_eq!(
            "abc123abc",
            TrimArg::new_str(TrimPos::Tail, "_;+-=".to_owned(), false).trim("abc123abc".to_owned(), &configs)
        );
        assert_eq!(
            "aBcabc123abc",
            TrimArg::new_str(TrimPos::Tail, "abc".to_owned(), false).trim("aBcabc123abcabc".to_owned(), &configs)
        );
        assert_eq!(
            "abc123abc",
            TrimArg::new_str(TrimPos::Tail, "aBc1".to_owned(), false).trim("abc123abc".to_owned(), &configs)
        );
        assert_eq!(
            "你好你好啊你好",
            TrimArg::new_str(TrimPos::Tail, "你好你".to_owned(), false).trim("你好你好啊你好".to_owned(), &configs)
        );
        assert_eq!(
            "你a好b你c1c好啊你好",
            TrimArg::new_str(TrimPos::Tail, "你好aBc".to_owned(), false)
                .trim("你a好b你c1c好啊你好".to_owned(), &configs)
        );
        assert_eq!(
            "你好aBc啊你好你好abc",
            TrimArg::new_str(TrimPos::Tail, "你好aBc".to_owned(), false)
                .trim("你好aBc啊你好你好abc".to_owned(), &configs)
        );
        assert_eq!(
            "a你",
            TrimArg::new_str(TrimPos::Tail, "你好啊abc".to_owned(), false).trim("a你".to_owned(), &configs)
        );
        // both
        assert_eq!(
            "abc123abc",
            TrimArg::new_str(TrimPos::Both, "_;+-=".to_owned(), false).trim("abc123abc".to_owned(), &configs)
        );
        assert_eq!(
            "aBcabc123abc",
            TrimArg::new_str(TrimPos::Both, "abc".to_owned(), false).trim("aBcabc123abcabc".to_owned(), &configs)
        );
        assert_eq!(
            "abc123abc",
            TrimArg::new_str(TrimPos::Both, "aBc1".to_owned(), false).trim("abc123abc".to_owned(), &configs)
        );
        assert_eq!(
            "好啊你好",
            TrimArg::new_str(TrimPos::Both, "你好你".to_owned(), false).trim("你好你好啊你好".to_owned(), &configs)
        );
        assert_eq!(
            "你a好b你c1c好啊你好",
            TrimArg::new_str(TrimPos::Both, "你好aBc".to_owned(), false)
                .trim("你a好b你c1c好啊你好".to_owned(), &configs)
        );
        assert_eq!(
            "啊你好你好abc",
            TrimArg::new_str(TrimPos::Both, "你好aBc".to_owned(), false)
                .trim("你好aBc啊你好你好abc".to_owned(), &configs)
        );
        assert_eq!(
            "a你",
            TrimArg::new_str(TrimPos::Both, "你好啊abc".to_owned(), false).trim("a你".to_owned(), &configs)
        );
    }

    #[test]
    fn test_trim_regex() {
        let configs = vec![];
        // head
        assert_eq!("", TrimArg::new_regex(TrimPos::Head, "\\d+".to_string()).unwrap().trim("".to_owned(), &configs));
        assert_eq!(
            "abc123",
            TrimArg::new_regex(TrimPos::Head, "\\d+".to_string()).unwrap().trim("123abc123".to_owned(), &configs)
        );
        // tail
        assert_eq!("", TrimArg::new_regex(TrimPos::Tail, "\\d+".to_string()).unwrap().trim("".to_owned(), &configs));
        assert_eq!(
            "123abc",
            TrimArg::new_regex(TrimPos::Tail, "\\d+".to_string()).unwrap().trim("123abc123".to_owned(), &configs)
        );
        // both
        assert_eq!("", TrimArg::new_regex(TrimPos::Both, "\\d+".to_string()).unwrap().trim("".to_owned(), &configs));
        assert_eq!(
            "abc",
            TrimArg::new_regex(TrimPos::Both, "\\d+".to_string()).unwrap().trim("123abc123".to_owned(), &configs)
        );
    }
}
