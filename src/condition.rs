use crate::err::RpErr;
use crate::{Float, Integer, Num};
use cmd_help::CmdHelp;
use regex::Regex;

/// 条件
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Condition {
    Yes(Select),
    No(Select),
}

impl Condition {
    pub(crate) fn new(select: Select, not: bool) -> Condition {
        if not { Condition::No(select) } else { Condition::Yes(select) }
    }

    pub(crate) fn test(&self, input: &str) -> bool {
        match self {
            Condition::Yes(select) => select.select(input),
            Condition::No(select) => !select.select(input),
        }
    }
}

/// 选择
#[derive(Debug, Clone, CmdHelp)]
pub(crate) enum Select {
    /// [!]len [<min>],[<max>]
    ///     按照字符串长度范围选择，范围表达式最小值和最大值至少指定其一，支持可选否定。
    ///     例如：
    ///         len 2,
    ///         len 2,5
    ///         len ,5
    ///         !len ,5
    ///         !len 2,5
    TextLenRange { min: Option<usize>, max: Option<usize> },
    /// [!]len <len>
    ///     按照字符串特定长度选择，支持可选否定。
    ///     例如：
    ///         len 3
    ///         !len 3
    TextLenSpec { spec: usize },
    /// [!]num [<min>],[<max>]
    ///     按照数值范围选择，范围表达式最小值和最大值至少指定其一，支持可选否定。
    ///     如果无法解析为数则不选择。
    ///     例如：
    ///         num 2,5
    ///         num -2.1,5
    ///         num 2,5.3
    ///         num ,5.3
    ///         !num 1,5.3
    NumRange { min: Option<Num>, max: Option<Num> },
    /// [!]num <spec>
    ///     按照数值特定值选择，支持可选否定。
    ///     如果无法解析为数则不选择。
    ///     例如：
    ///         num 3
    ///         num 3.3
    ///         !num 3.3
    NumSpec { spec: Num },
    /// [!]num[ [integer|float]]
    ///     按照整数或浮点数选择，如果不指定则选择数值数据，支持可选否定。
    ///     例如：
    ///         num
    ///         num integer
    ///         num float
    ///         !num
    ///         !num integer
    ///         !num float
    Num { integer: Option<bool> },
    /// [!]upper
    ///     选择全部为ASCII大写字符的数据，包括空字符串和不支持大小写的字符。
    /// [!]lower
    ///     选择全部为ASCII小写字符的数据，包括空字符串和不支持大小写的字符。
    TextAllCase { upper: bool },
    /// [!]ascii
    ///     选择全部为ASCII字符的数据，包括空字符串。
    /// [!]nonascii
    ///     选择全部不为ASCII字符的数据，包括空字符串。
    Ascii { ascii: bool },
    /// [!]empty
    ///     选择空字符串数据。
    /// [!]blank
    ///     选择全部为空白字符的数据，不包括空字符串。
    TextEmptyOrBlank { empty: bool },
    /// [!]reg <exp>
    ///     选择匹配给定正则表达式的数据。
    ///     <exp>   正则表达式，必选。
    ///     例如：
    ///         reg '\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}'
    RegMatch { regex: Regex },
}

impl PartialEq for Select {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Select::TextLenRange { min: l_min, max: l_max }, Select::TextLenRange { min: r_min, max: r_max }) => {
                l_min == r_min && l_max == r_max
            }
            (Select::TextLenSpec { spec: l }, Select::TextLenSpec { spec: r }) => l == r,
            (Select::NumRange { min: l_min, max: l_max }, Select::NumRange { min: r_min, max: r_max }) => {
                l_min == r_min && l_max == r_max
            }
            (Select::NumSpec { spec: l }, Select::NumSpec { spec: r }) => l == r,
            (Select::Num { integer: l }, Select::Num { integer: r }) => l == r,
            (Select::TextAllCase { upper: l }, Select::TextAllCase { upper: r }) => l == r,
            (Select::Ascii { ascii: l }, Select::Ascii { ascii: r }) => l == r,
            (Select::TextEmptyOrBlank { empty: l }, Select::TextEmptyOrBlank { empty: r }) => l == r,
            // Regex 比较模式字符串
            (Select::RegMatch { regex: l }, Select::RegMatch { regex: r }) => l.as_str() == r.as_str(),
            // 其他情况都不相等
            _ => false,
        }
    }
}

impl Select {
    pub(crate) fn new_text_len_range(min: Option<usize>, max: Option<usize>) -> Select {
        Select::TextLenRange { min, max }
    }
    pub(crate) fn new_num_range(min: Option<Num>, max: Option<Num>) -> Select {
        Select::NumRange { min, max }
    }
    pub(crate) fn new_reg_match(regex: &str) -> Result<Select, RpErr> {
        let reg = format!(r"\A(?:{})\z", regex);
        Regex::new(&reg)
            .map(|regex| Select::RegMatch { regex })
            .map_err(|err| RpErr::ParseRegexErr { reg, err: err.to_string() })
    }

    pub(crate) fn yes(self) -> Condition {
        Condition::Yes(self)
    }

    pub(crate) fn no(self) -> Condition {
        Condition::No(self)
    }

    fn select(&self, input: &str) -> bool {
        match self {
            Select::TextLenRange { min, max } => {
                let len = *&input.chars().count();
                min.map_or(true, |min_len| len >= min_len) && max.map_or(true, |max_len| len <= max_len)
            }
            Select::TextLenSpec { spec } => input.chars().count() == *spec,
            Select::NumRange { min, max } => input
                .parse::<Num>()
                .map(|i| min.map_or(true, |min_len| i >= min_len) && max.map_or(true, |max_len| i <= max_len))
                .unwrap_or(false),
            Select::NumSpec { spec } => input.parse::<Num>().ok().map(|i| &i == spec).unwrap_or(false),
            Select::Num { integer } => match integer {
                Some(integer) => {
                    if *integer {
                        input.parse::<Integer>().is_ok()
                    } else {
                        input.parse::<Integer>().is_err() && input.parse::<Float>().map_or(false, |v| v.is_finite())
                    }
                }
                None => input.parse::<Float>().map_or(false, |v| v.is_finite()),
            },
            Select::TextAllCase { upper } => {
                if *upper {
                    !input.chars().any(|c| c.is_lowercase())
                } else {
                    !input.chars().any(|c| c.is_uppercase())
                }
            }
            Select::Ascii { ascii } => {
                if *ascii {
                    input.is_ascii()
                } else {
                    input.chars().all(|c| !c.is_ascii())
                }
            }
            Select::TextEmptyOrBlank { empty } => {
                if *empty {
                    input.is_empty()
                } else {
                    input.chars().all(|c| c.is_whitespace())
                }
            }
            Select::RegMatch { regex } => regex.is_match(input),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_len_range() {
        assert!(!Select::new_text_len_range(Some(3), Some(5)).yes().test("12"));
        assert!(Select::new_text_len_range(Some(3), Some(5)).yes().test("123"));
        assert!(Select::new_text_len_range(Some(3), Some(5)).yes().test("1234"));
        assert!(Select::new_text_len_range(Some(3), Some(5)).yes().test("12345"));
        assert!(!Select::new_text_len_range(Some(3), Some(5)).yes().test("123456"));
        assert!(!Select::new_text_len_range(Some(3), None).yes().test("12"));
        assert!(Select::new_text_len_range(Some(3), None).yes().test("123"));
        assert!(Select::new_text_len_range(Some(3), None).yes().test("1234"));
        assert!(Select::new_text_len_range(None, Some(3)).yes().test("12"));
        assert!(Select::new_text_len_range(None, Some(3)).yes().test("123"));
        assert!(!Select::new_text_len_range(None, Some(3)).yes().test("1234"));
        assert!(Select::new_text_len_range(None, None).yes().test("123"));
        // not
        assert!(Select::new_text_len_range(Some(3), Some(5)).no().test("12"));
        assert!(!Select::new_text_len_range(Some(3), Some(5)).no().test("123"));
        assert!(!Select::new_text_len_range(Some(3), Some(5)).no().test("1234"));
        assert!(!Select::new_text_len_range(Some(3), Some(5)).no().test("12345"));
        assert!(Select::new_text_len_range(Some(3), Some(5)).no().test("123456"));
        assert!(Select::new_text_len_range(Some(3), None).no().test("12"));
        assert!(!Select::new_text_len_range(Some(3), None).no().test("123"));
        assert!(!Select::new_text_len_range(Some(3), None).no().test("1234"));
        assert!(!Select::new_text_len_range(None, Some(3)).no().test("12"));
        assert!(!Select::new_text_len_range(None, Some(3)).no().test("123"));
        assert!(Select::new_text_len_range(None, Some(3)).no().test("1234"));
        assert!(!Select::new_text_len_range(None, None).no().test("123"));
    }

    #[test]
    fn test_text_len_spec() {
        assert!(Select::TextLenSpec { spec: 0 }.yes().test(""));
        assert!(!Select::TextLenSpec { spec: 0 }.yes().test("1"));
        assert!(!Select::TextLenSpec { spec: 3 }.yes().test(""));
        assert!(!Select::TextLenSpec { spec: 3 }.yes().test("12"));
        assert!(Select::TextLenSpec { spec: 3 }.yes().test("123"));
        assert!(!Select::TextLenSpec { spec: 3 }.yes().test("1234"));
        // not
        assert!(!Select::TextLenSpec { spec: 0 }.no().test(""));
        assert!(Select::TextLenSpec { spec: 0 }.no().test("1"));
        assert!(Select::TextLenSpec { spec: 3 }.no().test(""));
        assert!(Select::TextLenSpec { spec: 3 }.no().test("12"));
        assert!(!Select::TextLenSpec { spec: 3 }.no().test("123"));
        assert!(Select::TextLenSpec { spec: 3 }.no().test("1234"));
    }

    #[test]
    fn test_integer_range() {
        assert!(!Select::new_num_range(Some(Num::from(3)), Some(Num::from(5))).yes().test("2"));
        assert!(Select::new_num_range(Some(Num::from(3)), Some(Num::from(5))).yes().test("3"));
        assert!(Select::new_num_range(Some(Num::from(3)), Some(Num::from(5))).yes().test("4"));
        assert!(Select::new_num_range(Some(Num::from(3)), Some(Num::from(5))).yes().test("5"));
        assert!(!Select::new_num_range(Some(Num::from(3)), Some(Num::from(5))).yes().test("6"));
        assert!(!Select::new_num_range(Some(Num::from(3)), None).yes().test("2"));
        assert!(Select::new_num_range(Some(Num::from(3)), None).yes().test("3"));
        assert!(Select::new_num_range(Some(Num::from(3)), None).yes().test("4"));
        assert!(Select::new_num_range(None, Some(Num::from(3))).yes().test("2"));
        assert!(Select::new_num_range(None, Some(Num::from(3))).yes().test("3"));
        assert!(!Select::new_num_range(None, Some(Num::from(3))).yes().test("4"));
        assert!(Select::new_num_range(None, None).yes().test("3"));
        assert!(!Select::new_num_range(None, None).yes().test("abc"));
        assert!(!Select::new_num_range(None, None).yes().test(""));
        // not
        assert!(Select::new_num_range(Some(Num::from(3)), Some(Num::from(5))).no().test("2"));
        assert!(!Select::new_num_range(Some(Num::from(3)), Some(Num::from(5))).no().test("3"));
        assert!(!Select::new_num_range(Some(Num::from(3)), Some(Num::from(5))).no().test("4"));
        assert!(!Select::new_num_range(Some(Num::from(3)), Some(Num::from(5))).no().test("5"));
        assert!(Select::new_num_range(Some(Num::from(3)), Some(Num::from(5))).no().test("6"));
        assert!(Select::new_num_range(Some(Num::from(3)), None).no().test("2"));
        assert!(!Select::new_num_range(Some(Num::from(3)), None).no().test("3"));
        assert!(!Select::new_num_range(Some(Num::from(3)), None).no().test("4"));
        assert!(!Select::new_num_range(None, Some(Num::from(3))).no().test("2"));
        assert!(!Select::new_num_range(None, Some(Num::from(3))).no().test("3"));
        assert!(Select::new_num_range(None, Some(Num::from(3))).no().test("4"));
        assert!(!Select::new_num_range(None, None).no().test("3"));
        assert!(Select::new_num_range(None, None).no().test("abc"));
        assert!(Select::new_num_range(None, None).no().test(""));
    }

    #[test]
    fn test_integer_spec() {
        assert!(Select::NumSpec { spec: Num::from(0) }.yes().test("0"));
        assert!(!Select::NumSpec { spec: Num::from(0) }.yes().test("1"));
        assert!(!Select::NumSpec { spec: Num::from(3) }.yes().test("1"));
        assert!(Select::NumSpec { spec: Num::from(3) }.yes().test("3"));
        assert!(!Select::NumSpec { spec: Num::from(3) }.yes().test("abc"));
        assert!(!Select::NumSpec { spec: Num::from(3) }.yes().test(""));
        // not
        assert!(!Select::NumSpec { spec: Num::from(0) }.no().test("0"));
        assert!(Select::NumSpec { spec: Num::from(0) }.no().test("1"));
        assert!(Select::NumSpec { spec: Num::from(3) }.no().test("1"));
        assert!(!Select::NumSpec { spec: Num::from(3) }.no().test("3"));
        assert!(Select::NumSpec { spec: Num::from(3) }.no().test("abc"));
        assert!(Select::NumSpec { spec: Num::from(3) }.no().test(""));
    }

    #[test]
    fn test_float_range() {
        assert!(!Select::new_num_range(Some(Num::from(3.0)), Some(Num::from(5.0))).yes().test("2"));
        assert!(Select::new_num_range(Some(Num::from(3.0)), Some(Num::from(5.0))).yes().test("3"));
        assert!(Select::new_num_range(Some(Num::from(3.0)), Some(Num::from(5.0))).yes().test("4"));
        assert!(Select::new_num_range(Some(Num::from(3.0)), Some(Num::from(5.0))).yes().test("5"));
        assert!(!Select::new_num_range(Some(Num::from(3.0)), Some(Num::from(5.0))).yes().test("6"));
        assert!(!Select::new_num_range(Some(Num::from(3.0)), None).yes().test("2"));
        assert!(Select::new_num_range(Some(Num::from(3.0)), None).yes().test("3"));
        assert!(Select::new_num_range(Some(Num::from(3.0)), None).yes().test("4"));
        assert!(Select::new_num_range(None, Some(Num::from(3.0))).yes().test("2"));
        assert!(Select::new_num_range(None, Some(Num::from(3.0))).yes().test("3"));
        assert!(!Select::new_num_range(None, Some(Num::from(3.0))).yes().test("4"));
        assert!(Select::new_num_range(None, None).yes().test("3"));
        assert!(!Select::new_num_range(None, None).yes().test("abc"));
        assert!(!Select::new_num_range(None, None).yes().test("NaN"));
        assert!(!Select::new_num_range(None, None).yes().test("nan"));
        assert!(!Select::new_num_range(None, None).yes().test("inf"));
        assert!(!Select::new_num_range(None, None).yes().test("Inf"));
        assert!(!Select::new_num_range(None, None).yes().test("-inf"));
        assert!(!Select::new_num_range(None, None).yes().test("-Inf"));
        assert!(!Select::new_num_range(None, None).yes().test(""));
        // not
        assert!(Select::new_num_range(Some(Num::from(3.0)), Some(Num::from(5.0))).no().test("2"));
        assert!(!Select::new_num_range(Some(Num::from(3.0)), Some(Num::from(5.0))).no().test("3"));
        assert!(!Select::new_num_range(Some(Num::from(3.0)), Some(Num::from(5.0))).no().test("4"));
        assert!(!Select::new_num_range(Some(Num::from(3.0)), Some(Num::from(5.0))).no().test("5"));
        assert!(Select::new_num_range(Some(Num::from(3.0)), Some(Num::from(5.0))).no().test("6"));
        assert!(Select::new_num_range(Some(Num::from(3.0)), None).no().test("2"));
        assert!(!Select::new_num_range(Some(Num::from(3.0)), None).no().test("3"));
        assert!(!Select::new_num_range(Some(Num::from(3.0)), None).no().test("4"));
        assert!(!Select::new_num_range(None, Some(Num::from(3.0))).no().test("2"));
        assert!(!Select::new_num_range(None, Some(Num::from(3.0))).no().test("3"));
        assert!(Select::new_num_range(None, Some(Num::from(3.0))).no().test("4"));
        assert!(!Select::new_num_range(None, None).no().test("3"));
        assert!(Select::new_num_range(None, None).no().test("abc"));
        assert!(Select::new_num_range(None, None).no().test("NaN"));
        assert!(Select::new_num_range(None, None).no().test("nan"));
        assert!(Select::new_num_range(None, None).no().test("inf"));
        assert!(Select::new_num_range(None, None).no().test("Inf"));
        assert!(Select::new_num_range(None, None).no().test("-inf"));
        assert!(Select::new_num_range(None, None).no().test("-Inf"));
        assert!(Select::new_num_range(None, None).no().test(""));
    }

    #[test]
    fn test_float_spec() {
        assert!(Select::NumSpec { spec: Num::from(0.0) }.yes().test("0"));
        assert!(!Select::NumSpec { spec: Num::from(0.0) }.yes().test("1"));
        assert!(!Select::NumSpec { spec: Num::from(3.0) }.yes().test("1"));
        assert!(Select::NumSpec { spec: Num::from(3.0) }.yes().test("3"));
        assert!(!Select::NumSpec { spec: Num::from(3.0) }.yes().test("abc"));
        assert!(!Select::NumSpec { spec: Num::from(3.0) }.yes().test("NaN"));
        assert!(!Select::NumSpec { spec: Num::from(3.0) }.yes().test("nan"));
        assert!(!Select::NumSpec { spec: Num::from(3.0) }.yes().test("inf"));
        assert!(!Select::NumSpec { spec: Num::from(3.0) }.yes().test("Inf"));
        assert!(!Select::NumSpec { spec: Num::from(3.0) }.yes().test("-inf"));
        assert!(!Select::NumSpec { spec: Num::from(3.0) }.yes().test("-Inf"));
        assert!(!Select::NumSpec { spec: Num::from(3.0) }.yes().test(""));
        // not
        assert!(!Select::NumSpec { spec: Num::from(0.0) }.no().test("0"));
        assert!(Select::NumSpec { spec: Num::from(0.0) }.no().test("1"));
        assert!(Select::NumSpec { spec: Num::from(3.0) }.no().test("1"));
        assert!(!Select::NumSpec { spec: Num::from(3.0) }.no().test("3"));
        assert!(Select::NumSpec { spec: Num::from(3.0) }.no().test("abc"));
        assert!(Select::NumSpec { spec: Num::from(3.0) }.no().test("NaN"));
        assert!(Select::NumSpec { spec: Num::from(3.0) }.no().test("nan"));
        assert!(Select::NumSpec { spec: Num::from(3.0) }.no().test("inf"));
        assert!(Select::NumSpec { spec: Num::from(3.0) }.no().test("Inf"));
        assert!(Select::NumSpec { spec: Num::from(3.0) }.no().test("-inf"));
        assert!(Select::NumSpec { spec: Num::from(3.0) }.no().test("-Inf"));
        assert!(Select::NumSpec { spec: Num::from(3.0) }.no().test(""));
    }

    #[test]
    fn test_number_not() {
        // integer
        assert!(!Select::Num { integer: Some(true) }.yes().test("abc"));
        assert!(Select::Num { integer: Some(true) }.yes().test("123"));
        assert!(!Select::Num { integer: Some(true) }.yes().test("123.1"));
        assert!(!Select::Num { integer: Some(true) }.yes().test("123.0"));
        assert!(!Select::Num { integer: Some(true) }.yes().test("NaN"));
        assert!(!Select::Num { integer: Some(true) }.yes().test("nan"));
        assert!(!Select::Num { integer: Some(true) }.yes().test("inf"));
        assert!(!Select::Num { integer: Some(true) }.yes().test("Inf"));
        assert!(!Select::Num { integer: Some(true) }.yes().test("-inf"));
        assert!(!Select::Num { integer: Some(true) }.yes().test("-Inf"));
        assert!(!Select::Num { integer: Some(true) }.yes().test(""));
        assert!(Select::Num { integer: Some(true) }.no().test("abc"));
        assert!(!Select::Num { integer: Some(true) }.no().test("123"));
        assert!(Select::Num { integer: Some(true) }.no().test("123.1"));
        assert!(Select::Num { integer: Some(true) }.no().test("123.0"));
        assert!(Select::Num { integer: Some(true) }.no().test("NaN"));
        assert!(Select::Num { integer: Some(true) }.no().test("nan"));
        assert!(Select::Num { integer: Some(true) }.no().test("inf"));
        assert!(Select::Num { integer: Some(true) }.no().test("Inf"));
        assert!(Select::Num { integer: Some(true) }.no().test("-inf"));
        assert!(Select::Num { integer: Some(true) }.no().test("-Inf"));
        assert!(Select::Num { integer: Some(true) }.no().test(""));
        // float
        assert!(!Select::Num { integer: Some(false) }.yes().test("abc"));
        assert!(!Select::Num { integer: Some(false) }.yes().test("123"));
        assert!(Select::Num { integer: Some(false) }.yes().test("123.1"));
        assert!(Select::Num { integer: Some(false) }.yes().test("123.0"));
        assert!(!Select::Num { integer: Some(false) }.yes().test("NaN"));
        assert!(!Select::Num { integer: Some(false) }.yes().test("nan"));
        assert!(!Select::Num { integer: Some(false) }.yes().test("inf"));
        assert!(!Select::Num { integer: Some(false) }.yes().test("Inf"));
        assert!(!Select::Num { integer: Some(false) }.yes().test("-inf"));
        assert!(!Select::Num { integer: Some(false) }.yes().test("-Inf"));
        assert!(!Select::Num { integer: Some(false) }.yes().test(""));
        assert!(Select::Num { integer: Some(false) }.no().test("abc"));
        assert!(Select::Num { integer: Some(false) }.no().test("123"));
        assert!(!Select::Num { integer: Some(false) }.no().test("123.1"));
        assert!(!Select::Num { integer: Some(false) }.no().test("123.0"));
        assert!(Select::Num { integer: Some(false) }.no().test("NaN"));
        assert!(Select::Num { integer: Some(false) }.no().test("nan"));
        assert!(Select::Num { integer: Some(false) }.no().test("inf"));
        assert!(Select::Num { integer: Some(false) }.no().test("Inf"));
        assert!(Select::Num { integer: Some(false) }.no().test("-inf"));
        assert!(Select::Num { integer: Some(false) }.no().test("-Inf"));
        assert!(Select::Num { integer: Some(false) }.no().test(""));
        // number
        assert!(!Select::Num { integer: None }.yes().test("abc"));
        assert!(Select::Num { integer: None }.yes().test("123"));
        assert!(Select::Num { integer: None }.yes().test("123.1"));
        assert!(Select::Num { integer: None }.yes().test("123.0"));
        assert!(!Select::Num { integer: None }.yes().test("NaN"));
        assert!(!Select::Num { integer: None }.yes().test("nan"));
        assert!(!Select::Num { integer: None }.yes().test("inf"));
        assert!(!Select::Num { integer: None }.yes().test("Inf"));
        assert!(!Select::Num { integer: None }.yes().test("-inf"));
        assert!(!Select::Num { integer: None }.yes().test("-Inf"));
        assert!(!Select::Num { integer: None }.yes().test(""));
        assert!(Select::Num { integer: None }.no().test("abc"));
        assert!(!Select::Num { integer: None }.no().test("123"));
        assert!(!Select::Num { integer: None }.no().test("123.1"));
        assert!(!Select::Num { integer: None }.no().test("123.0"));
        assert!(Select::Num { integer: None }.no().test("NaN"));
        assert!(Select::Num { integer: None }.no().test("nan"));
        assert!(Select::Num { integer: None }.no().test("inf"));
        assert!(Select::Num { integer: None }.no().test("Inf"));
        assert!(Select::Num { integer: None }.no().test("-inf"));
        assert!(Select::Num { integer: None }.no().test("-Inf"));
        assert!(Select::Num { integer: None }.no().test(""));
    }

    #[test]
    fn test_text_all_case() {
        // upper
        assert!(!Select::TextAllCase { upper: true }.yes().test("abc"));
        assert!(Select::TextAllCase { upper: true }.yes().test("ABC"));
        assert!(!Select::TextAllCase { upper: true }.yes().test("abcABC"));
        assert!(Select::TextAllCase { upper: true }.yes().test("你好123.#!@"));
        assert!(Select::TextAllCase { upper: true }.no().test("abc"));
        assert!(!Select::TextAllCase { upper: true }.no().test("ABC"));
        assert!(Select::TextAllCase { upper: true }.no().test("abcABC"));
        assert!(!Select::TextAllCase { upper: true }.no().test("你好123.#!@"));
        // lower
        assert!(Select::TextAllCase { upper: false }.yes().test("abc"));
        assert!(!Select::TextAllCase { upper: false }.yes().test("ABC"));
        assert!(!Select::TextAllCase { upper: false }.yes().test("abcABC"));
        assert!(Select::TextAllCase { upper: false }.yes().test("你好123.#!@"));
        assert!(!Select::TextAllCase { upper: false }.no().test("abc"));
        assert!(Select::TextAllCase { upper: false }.no().test("ABC"));
        assert!(Select::TextAllCase { upper: false }.no().test("abcABC"));
        assert!(!Select::TextAllCase { upper: false }.no().test("你好123.#!@"));
    }

    #[test]
    fn test_ascii() {
        assert!(Select::Ascii { ascii: true }.yes().test("abc"));
        assert!(Select::Ascii { ascii: true }.yes().test(""));
        assert!(Select::Ascii { ascii: true }.yes().test("\n"));
        assert!(!Select::Ascii { ascii: true }.yes().test("你好"));
        assert!(!Select::Ascii { ascii: false }.yes().test("abc"));
        assert!(Select::Ascii { ascii: false }.yes().test(""));
        assert!(!Select::Ascii { ascii: false }.yes().test("\n"));
        assert!(Select::Ascii { ascii: false }.yes().test("你好"));
        // not
        assert!(!Select::Ascii { ascii: true }.no().test("abc"));
        assert!(!Select::Ascii { ascii: true }.no().test(""));
        assert!(!Select::Ascii { ascii: true }.no().test("\n"));
        assert!(Select::Ascii { ascii: true }.no().test("你好"));
        assert!(Select::Ascii { ascii: false }.no().test("abc"));
        assert!(!Select::Ascii { ascii: false }.no().test(""));
        assert!(Select::Ascii { ascii: false }.no().test("\n"));
        assert!(!Select::Ascii { ascii: false }.no().test("你好"));
    }

    #[test]
    fn test_text_empty_or_blank() {
        // empty
        assert!(Select::TextEmptyOrBlank { empty: true }.yes().test(""));
        assert!(!Select::TextEmptyOrBlank { empty: true }.yes().test("abc"));
        assert!(!Select::TextEmptyOrBlank { empty: true }.yes().test(" "));
        assert!(!Select::TextEmptyOrBlank { empty: true }.yes().test(" \n\t\r "));
        assert!(!Select::TextEmptyOrBlank { empty: true }.no().test(""));
        assert!(Select::TextEmptyOrBlank { empty: true }.no().test("abc"));
        assert!(Select::TextEmptyOrBlank { empty: true }.no().test(" "));
        assert!(Select::TextEmptyOrBlank { empty: true }.no().test(" \n\t\r "));
        // blank
        assert!(Select::TextEmptyOrBlank { empty: false }.yes().test(""));
        assert!(!Select::TextEmptyOrBlank { empty: false }.yes().test("abc"));
        assert!(Select::TextEmptyOrBlank { empty: false }.yes().test(" "));
        assert!(Select::TextEmptyOrBlank { empty: false }.yes().test(" \n\t\r "));
        assert!(!Select::TextEmptyOrBlank { empty: false }.no().test(""));
        assert!(Select::TextEmptyOrBlank { empty: false }.no().test("abc"));
        assert!(!Select::TextEmptyOrBlank { empty: false }.no().test(" "));
        assert!(!Select::TextEmptyOrBlank { empty: false }.no().test(" \n\t\r "));
    }

    #[test]
    fn test_reg_match() {
        assert!(Select::new_reg_match(r"[").is_err());
        // yes
        assert!(Select::new_reg_match(r"\d+").unwrap().yes().test("123"));
        assert!(!Select::new_reg_match(r"\d+").unwrap().yes().test("123abc"));
        assert!(!Select::new_reg_match(r"\d+").unwrap().yes().test("123\n123"));
        assert!(!Select::new_reg_match(r"(?m)\d+").unwrap().yes().test("123\n123"));
        assert!(Select::new_reg_match(r"(?m)[\d\n]+").unwrap().yes().test("123\n123"));
        // not
        assert!(!Select::new_reg_match(r"\d+").unwrap().no().test("123"));
        assert!(Select::new_reg_match(r"\d+").unwrap().no().test("123abc"));
        assert!(Select::new_reg_match(r"\d+").unwrap().no().test("123\n123"));
        assert!(Select::new_reg_match(r"(?m)\d+").unwrap().no().test("123\n123"));
        assert!(!Select::new_reg_match(r"(?m)[\d\n]+").unwrap().no().test("123\n123"));
    }
}
