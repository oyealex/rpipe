use crate::err::RpErr;
use crate::{Float, Integer};
use cmd_help::CmdHelp;
use regex::Regex;

#[derive(Debug, PartialEq)]
pub(crate) struct CondRangeArg<T> {
    min: Option<T>,
    max: Option<T>,
    not: bool,
}

impl<T> CondRangeArg<T> {
    pub(crate) fn new(min: Option<T>, max: Option<T>, not: bool) -> CondRangeArg<T> {
        CondRangeArg { min, max, not }
    }
}

#[derive(Debug, PartialEq)]
pub(crate) struct CondSpecArg<T> {
    spec: T,
    not: bool,
}

impl<T> CondSpecArg<T> {
    pub(crate) fn new(spec: T, not: bool) -> CondSpecArg<T> {
        CondSpecArg { spec, not }
    }
}

#[derive(Debug, CmdHelp)]
pub(crate) enum Cond {
    /// len [!][<min_len>],[<max_len>]
    ///     按照字符串长度范围选择，范围表达式最小值和最大值至少指定其一，支持可选的否定。
    TextLenRange(CondRangeArg<usize>),
    /// len [!]=<len>
    ///     按照字符串特定长度选择，支持可选的否定。
    TextLenSpec(CondSpecArg<usize>),
    /// num [!][<min_integer>],[<max_integer>]
    ///     按照整数值范围选择，范围表达式最小值和最大值至少指定其一，支持可选的否定。
    ///     如果无法解析为整数则不选择。
    IntegerRange(CondRangeArg<Integer>), // TODO 2026-01-13 02:13 合并float系列
    /// num [!]=<integer>
    ///     按照整数值特定值选择，支持可选的否定。
    ///     如果无法解析为整数则不选择。
    IntegerSpec(CondSpecArg<Integer>),
    /// num [!][<min_float>],[<max_float>]
    ///     按照浮点数值范围选择，范围表达式最小值和最大值至少指定其一，支持可选的否定。
    ///     如果无法解析为浮点数则不选择。
    FloatRange(CondRangeArg<Float>),
    /// num [!]=<float>
    ///     按照浮点数值特定值选择，支持可选的否定。
    ///     如果无法解析为浮点数则不选择。
    FloatSpec(CondSpecArg<Float>),
    /// num[ [!][integer|float]]
    ///     按照整数或浮点数选择，如果不指定则选择数值数据，支持可选的否定。
    Number { is_integer: Option<bool>, not: bool },
    /// upper|lower
    ///     选择全部为大写或小写字符的数据，不支持大小写的字符总是满足。
    TextAllCase(bool /*is_upper*/),
    /// empty|blank
    ///     选择没有任何字符或全部为空白字符的数据。
    TextEmptyOrBlank(bool /*is_empty*/),
    /// reg <exp>
    ///     选择匹配给定正则表达式的数据。
    RegMatch(Regex),
}

impl PartialEq for Cond {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            // Regex 比较模式字符串
            (Cond::RegMatch(re1), Cond::RegMatch(re2)) => re1.as_str() == re2.as_str(),
            (Cond::TextLenRange(l), Cond::TextLenRange(r)) => l == r,
            (Cond::TextLenSpec(l), Cond::TextLenSpec(r)) => l == r,
            (Cond::IntegerRange(l), Cond::IntegerRange(r)) => l == r,
            (Cond::IntegerSpec(l), Cond::IntegerSpec(r)) => l == r,
            (Cond::FloatRange(l), Cond::FloatRange(r)) => l == r,
            (Cond::FloatSpec(l), Cond::FloatSpec(r)) => l == r,
            (
                Cond::Number { is_integer: l_is_integer, not: l_not },
                Cond::Number { is_integer: r_is_integer, not: r_not },
            ) => l_is_integer == r_is_integer && l_not == r_not,
            (Cond::TextAllCase(l), Cond::TextAllCase(r)) => l == r,
            (Cond::TextEmptyOrBlank(l), Cond::TextEmptyOrBlank(r)) => l == r,
            // 其他情况都不相等
            _ => false,
        }
    }
}

#[inline]
fn with_not(res: bool, not: bool) -> bool {
    if not { !res } else { res }
}

impl Cond {
    pub(crate) fn new_text_len_range(range: (Option<usize>, Option<usize>), not: bool) -> Cond {
        Cond::TextLenRange(CondRangeArg { min: range.0, max: range.1, not })
    }
    pub(crate) fn new_text_len_spec(len: usize, not: bool) -> Cond {
        Cond::TextLenSpec(CondSpecArg { spec: len, not })
    }
    pub(crate) fn new_integer_range(range: (Option<Integer>, Option<Integer>), not: bool) -> Cond {
        Cond::IntegerRange(CondRangeArg { min: range.0, max: range.1, not })
    }
    pub(crate) fn new_integer_spec(val: Integer, not: bool) -> Cond {
        Cond::IntegerSpec(CondSpecArg { spec: val, not })
    }
    pub(crate) fn new_float_range(range: (Option<Float>, Option<Float>), not: bool) -> Cond {
        Cond::FloatRange(CondRangeArg { min: range.0, max: range.1, not })
    }
    pub(crate) fn new_float_spec(val: Float, not: bool) -> Cond {
        Cond::FloatSpec(CondSpecArg { spec: val, not })
    }
    pub(crate) fn new_number(is_integer: Option<bool>, not: bool) -> Cond {
        Cond::Number { is_integer, not }
    }
    pub(crate) fn new_text_all_case(is_upper: bool) -> Cond {
        Cond::TextAllCase(is_upper)
    }
    pub(crate) fn new_text_empty_or_blank(is_empty: bool) -> Cond {
        Cond::TextEmptyOrBlank(is_empty)
    }
    pub(crate) fn new_reg_match(regex: &str) -> Result<Cond, RpErr> {
        let reg = format!(r"\A(?:{})\z", regex);
        Regex::new(&reg)
            .map(|reg| Cond::RegMatch(reg))
            .map_err(|err| RpErr::ParseRegexErr { reg, err: err.to_string() })
    }

    pub(crate) fn test(&self, input: &str) -> bool {
        match self {
            Cond::TextLenRange(CondRangeArg { min, max, not }) => {
                let len = *&input.chars().count();
                with_not(min.map_or(true, |min_len| len >= min_len) && max.map_or(true, |max_len| len <= max_len), *not)
            }
            Cond::TextLenSpec(CondSpecArg { spec: len, not }) => with_not(input.chars().count() == *len, *not),
            Cond::IntegerRange(CondRangeArg { min, max, not }) => input
                .parse::<Integer>()
                .map(|i| {
                    with_not(min.map_or(true, |min_len| i >= min_len) && max.map_or(true, |max_len| i <= max_len), *not)
                })
                .unwrap_or(false),
            Cond::IntegerSpec(CondSpecArg { spec: val, not }) => {
                input.parse::<Integer>().ok().map(|i| with_not(&i == val, *not)).unwrap_or(false)
            }
            Cond::FloatRange(CondRangeArg { min, max, not }) => input
                .parse::<Float>()
                .map(|f| {
                    if !f.is_finite() {
                        return false;
                    }
                    with_not(min.map_or(true, |min_len| f >= min_len) && max.map_or(true, |max_len| f <= max_len), *not)
                })
                .unwrap_or(false),
            Cond::FloatSpec(CondSpecArg { spec: val, not }) => input
                .parse::<Float>()
                .ok()
                .map(|f| {
                    if !f.is_finite() {
                        return false;
                    }
                    with_not(&f == val, *not)
                })
                .unwrap_or(false),
            Cond::Number { is_integer, not } => match is_integer {
                Some(integer) => {
                    if *integer {
                        with_not(input.parse::<Integer>().is_ok(), *not)
                    } else {
                        with_not(
                            input.parse::<Integer>().is_err()
                                && input.parse::<Float>().map_or(false, |v| v.is_finite()),
                            *not,
                        )
                    }
                }
                None => with_not(input.parse::<Float>().map_or(false, |v| v.is_finite()), *not),
            },
            Cond::TextAllCase(upper) => {
                if *upper {
                    !input.chars().any(|c| c.is_lowercase())
                } else {
                    !input.chars().any(|c| c.is_uppercase())
                }
            }
            Cond::TextEmptyOrBlank(empty) => {
                if *empty {
                    input.is_empty()
                } else {
                    input.chars().all(|c| c.is_whitespace())
                }
            }
            Cond::RegMatch(regex) => regex.is_match(input),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_len_range() {
        assert!(!Cond::new_text_len_range((Some(3), Some(5)), false).test("12"));
        assert!(Cond::new_text_len_range((Some(3), Some(5)), false).test("123"));
        assert!(Cond::new_text_len_range((Some(3), Some(5)), false).test("1234"));
        assert!(Cond::new_text_len_range((Some(3), Some(5)), false).test("12345"));
        assert!(!Cond::new_text_len_range((Some(3), Some(5)), false).test("123456"));
        assert!(!Cond::new_text_len_range((Some(3), None), false).test("12"));
        assert!(Cond::new_text_len_range((Some(3), None), false).test("123"));
        assert!(Cond::new_text_len_range((Some(3), None), false).test("1234"));
        assert!(Cond::new_text_len_range((None, Some(3)), false).test("12"));
        assert!(Cond::new_text_len_range((None, Some(3)), false).test("123"));
        assert!(!Cond::new_text_len_range((None, Some(3)), false).test("1234"));
        assert!(Cond::new_text_len_range((None, None), false).test("123"));
        // not
        assert!(Cond::new_text_len_range((Some(3), Some(5)), true).test("12"));
        assert!(!Cond::new_text_len_range((Some(3), Some(5)), true).test("123"));
        assert!(!Cond::new_text_len_range((Some(3), Some(5)), true).test("1234"));
        assert!(!Cond::new_text_len_range((Some(3), Some(5)), true).test("12345"));
        assert!(Cond::new_text_len_range((Some(3), Some(5)), true).test("123456"));
        assert!(Cond::new_text_len_range((Some(3), None), true).test("12"));
        assert!(!Cond::new_text_len_range((Some(3), None), true).test("123"));
        assert!(!Cond::new_text_len_range((Some(3), None), true).test("1234"));
        assert!(!Cond::new_text_len_range((None, Some(3)), true).test("12"));
        assert!(!Cond::new_text_len_range((None, Some(3)), true).test("123"));
        assert!(Cond::new_text_len_range((None, Some(3)), true).test("1234"));
        assert!(!Cond::new_text_len_range((None, None), true).test("123"));
    }

    #[test]
    fn test_text_len_spec() {
        assert!(Cond::new_text_len_spec(0, false).test(""));
        assert!(!Cond::new_text_len_spec(0, false).test("1"));
        assert!(!Cond::new_text_len_spec(3, false).test(""));
        assert!(!Cond::new_text_len_spec(3, false).test("12"));
        assert!(Cond::new_text_len_spec(3, false).test("123"));
        assert!(!Cond::new_text_len_spec(3, false).test("1234"));
        // not
        assert!(!Cond::new_text_len_spec(0, true).test(""));
        assert!(Cond::new_text_len_spec(0, true).test("1"));
        assert!(Cond::new_text_len_spec(3, true).test(""));
        assert!(Cond::new_text_len_spec(3, true).test("12"));
        assert!(!Cond::new_text_len_spec(3, true).test("123"));
        assert!(Cond::new_text_len_spec(3, true).test("1234"));
    }

    #[test]
    fn test_integer_range() {
        assert!(!Cond::new_integer_range((Some(3), Some(5)), false).test("2"));
        assert!(Cond::new_integer_range((Some(3), Some(5)), false).test("3"));
        assert!(Cond::new_integer_range((Some(3), Some(5)), false).test("4"));
        assert!(Cond::new_integer_range((Some(3), Some(5)), false).test("5"));
        assert!(!Cond::new_integer_range((Some(3), Some(5)), false).test("6"));
        assert!(!Cond::new_integer_range((Some(3), None), false).test("2"));
        assert!(Cond::new_integer_range((Some(3), None), false).test("3"));
        assert!(Cond::new_integer_range((Some(3), None), false).test("4"));
        assert!(Cond::new_integer_range((None, Some(3)), false).test("2"));
        assert!(Cond::new_integer_range((None, Some(3)), false).test("3"));
        assert!(!Cond::new_integer_range((None, Some(3)), false).test("4"));
        assert!(Cond::new_integer_range((None, None), false).test("3"));
        assert!(!Cond::new_integer_range((None, None), false).test("abc"));
        assert!(!Cond::new_integer_range((None, None), false).test(""));
        // not
        assert!(Cond::new_integer_range((Some(3), Some(5)), true).test("2"));
        assert!(!Cond::new_integer_range((Some(3), Some(5)), true).test("3"));
        assert!(!Cond::new_integer_range((Some(3), Some(5)), true).test("4"));
        assert!(!Cond::new_integer_range((Some(3), Some(5)), true).test("5"));
        assert!(Cond::new_integer_range((Some(3), Some(5)), true).test("6"));
        assert!(Cond::new_integer_range((Some(3), None), true).test("2"));
        assert!(!Cond::new_integer_range((Some(3), None), true).test("3"));
        assert!(!Cond::new_integer_range((Some(3), None), true).test("4"));
        assert!(!Cond::new_integer_range((None, Some(3)), true).test("2"));
        assert!(!Cond::new_integer_range((None, Some(3)), true).test("3"));
        assert!(Cond::new_integer_range((None, Some(3)), true).test("4"));
        assert!(!Cond::new_integer_range((None, None), true).test("3"));
        assert!(!Cond::new_integer_range((None, None), true).test("abc"));
        assert!(!Cond::new_integer_range((None, None), true).test(""));
    }

    #[test]
    fn test_integer_spec() {
        assert!(Cond::new_integer_spec(0, false).test("0"));
        assert!(!Cond::new_integer_spec(0, false).test("1"));
        assert!(!Cond::new_integer_spec(3, false).test("1"));
        assert!(Cond::new_integer_spec(3, false).test("3"));
        assert!(!Cond::new_integer_spec(3, false).test("abc"));
        assert!(!Cond::new_integer_spec(3, false).test(""));
        // not
        assert!(!Cond::new_integer_spec(0, true).test("0"));
        assert!(Cond::new_integer_spec(0, true).test("1"));
        assert!(Cond::new_integer_spec(3, true).test("1"));
        assert!(!Cond::new_integer_spec(3, true).test("3"));
        assert!(!Cond::new_integer_spec(3, true).test("abc"));
        assert!(!Cond::new_integer_spec(3, true).test(""));
    }

    #[test]
    fn test_float_range() {
        assert!(!Cond::new_float_range((Some(3.0), Some(5.0)), false).test("2"));
        assert!(Cond::new_float_range((Some(3.0), Some(5.0)), false).test("3"));
        assert!(Cond::new_float_range((Some(3.0), Some(5.0)), false).test("4"));
        assert!(Cond::new_float_range((Some(3.0), Some(5.0)), false).test("5"));
        assert!(!Cond::new_float_range((Some(3.0), Some(5.0)), false).test("6"));
        assert!(!Cond::new_float_range((Some(3.0), None), false).test("2"));
        assert!(Cond::new_float_range((Some(3.0), None), false).test("3"));
        assert!(Cond::new_float_range((Some(3.0), None), false).test("4"));
        assert!(Cond::new_float_range((None, Some(3.0)), false).test("2"));
        assert!(Cond::new_float_range((None, Some(3.0)), false).test("3"));
        assert!(!Cond::new_float_range((None, Some(3.0)), false).test("4"));
        assert!(Cond::new_float_range((None, None), false).test("3"));
        assert!(!Cond::new_float_range((None, None), false).test("abc"));
        assert!(!Cond::new_float_range((None, None), false).test("NaN"));
        assert!(!Cond::new_float_range((None, None), false).test("nan"));
        assert!(!Cond::new_float_range((None, None), false).test("inf"));
        assert!(!Cond::new_float_range((None, None), false).test("Inf"));
        assert!(!Cond::new_float_range((None, None), false).test("-inf"));
        assert!(!Cond::new_float_range((None, None), false).test("-Inf"));
        assert!(!Cond::new_float_range((None, None), false).test(""));
        // not
        assert!(Cond::new_float_range((Some(3.0), Some(5.0)), true).test("2"));
        assert!(!Cond::new_float_range((Some(3.0), Some(5.0)), true).test("3"));
        assert!(!Cond::new_float_range((Some(3.0), Some(5.0)), true).test("4"));
        assert!(!Cond::new_float_range((Some(3.0), Some(5.0)), true).test("5"));
        assert!(Cond::new_float_range((Some(3.0), Some(5.0)), true).test("6"));
        assert!(Cond::new_float_range((Some(3.0), None), true).test("2"));
        assert!(!Cond::new_float_range((Some(3.0), None), true).test("3"));
        assert!(!Cond::new_float_range((Some(3.0), None), true).test("4"));
        assert!(!Cond::new_float_range((None, Some(3.0)), true).test("2"));
        assert!(!Cond::new_float_range((None, Some(3.0)), true).test("3"));
        assert!(Cond::new_float_range((None, Some(3.0)), true).test("4"));
        assert!(!Cond::new_float_range((None, None), true).test("3"));
        assert!(!Cond::new_float_range((None, None), true).test("abc"));
        assert!(!Cond::new_float_range((None, None), true).test("NaN"));
        assert!(!Cond::new_float_range((None, None), true).test("nan"));
        assert!(!Cond::new_float_range((None, None), true).test("inf"));
        assert!(!Cond::new_float_range((None, None), true).test("Inf"));
        assert!(!Cond::new_float_range((None, None), true).test("-inf"));
        assert!(!Cond::new_float_range((None, None), true).test("-Inf"));
        assert!(!Cond::new_float_range((None, None), true).test(""));
    }

    #[test]
    fn test_float_spec() {
        assert!(Cond::new_float_spec(0.0, false).test("0"));
        assert!(!Cond::new_float_spec(0.0, false).test("1"));
        assert!(!Cond::new_float_spec(3.0, false).test("1"));
        assert!(Cond::new_float_spec(3.0, false).test("3"));
        assert!(!Cond::new_float_spec(3.0, false).test("abc"));
        assert!(!Cond::new_float_spec(3.0, false).test("NaN"));
        assert!(!Cond::new_float_spec(3.0, false).test("nan"));
        assert!(!Cond::new_float_spec(3.0, false).test("inf"));
        assert!(!Cond::new_float_spec(3.0, false).test("Inf"));
        assert!(!Cond::new_float_spec(3.0, false).test("-inf"));
        assert!(!Cond::new_float_spec(3.0, false).test("-Inf"));
        assert!(!Cond::new_float_spec(3.0, false).test(""));
        // not
        assert!(!Cond::new_float_spec(0.0, true).test("0"));
        assert!(Cond::new_float_spec(0.0, true).test("1"));
        assert!(Cond::new_float_spec(3.0, true).test("1"));
        assert!(!Cond::new_float_spec(3.0, true).test("3"));
        assert!(!Cond::new_float_spec(3.0, true).test("abc"));
        assert!(!Cond::new_float_spec(3.0, true).test("NaN"));
        assert!(!Cond::new_float_spec(3.0, true).test("nan"));
        assert!(!Cond::new_float_spec(3.0, true).test("inf"));
        assert!(!Cond::new_float_spec(3.0, true).test("Inf"));
        assert!(!Cond::new_float_spec(3.0, true).test("-inf"));
        assert!(!Cond::new_float_spec(3.0, true).test("-Inf"));
        assert!(!Cond::new_float_spec(3.0, true).test(""));
    }

    #[test]
    fn test_number_not() {
        // integer
        assert!(Cond::new_number(Some(true), true).test("abc"));
        assert!(!Cond::new_number(Some(true), true).test("123"));
        assert!(Cond::new_number(Some(true), true).test("123.1"));
        assert!(Cond::new_number(Some(true), true).test("123.0"));
        assert!(Cond::new_number(Some(true), true).test("NaN"));
        assert!(Cond::new_number(Some(true), true).test("nan"));
        assert!(Cond::new_number(Some(true), true).test("inf"));
        assert!(Cond::new_number(Some(true), true).test("Inf"));
        assert!(Cond::new_number(Some(true), true).test("-inf"));
        assert!(Cond::new_number(Some(true), true).test("-Inf"));
        assert!(Cond::new_number(Some(true), true).test(""));
        assert!(!Cond::new_number(Some(true), false).test("abc"));
        assert!(Cond::new_number(Some(true), false).test("123"));
        assert!(!Cond::new_number(Some(true), false).test("123.1"));
        assert!(!Cond::new_number(Some(true), false).test("123.0"));
        assert!(!Cond::new_number(Some(true), false).test("NaN"));
        assert!(!Cond::new_number(Some(true), false).test("nan"));
        assert!(!Cond::new_number(Some(true), false).test("inf"));
        assert!(!Cond::new_number(Some(true), false).test("Inf"));
        assert!(!Cond::new_number(Some(true), false).test("-inf"));
        assert!(!Cond::new_number(Some(true), false).test("-Inf"));
        assert!(!Cond::new_number(Some(true), false).test(""));
        // float
        assert!(Cond::new_number(Some(false), true).test("abc"));
        assert!(Cond::new_number(Some(false), true).test("123"));
        assert!(!Cond::new_number(Some(false), true).test("123.1"));
        assert!(!Cond::new_number(Some(false), true).test("123.0"));
        assert!(Cond::new_number(Some(false), true).test("NaN"));
        assert!(Cond::new_number(Some(false), true).test("nan"));
        assert!(Cond::new_number(Some(false), true).test("inf"));
        assert!(Cond::new_number(Some(false), true).test("Inf"));
        assert!(Cond::new_number(Some(false), true).test("-inf"));
        assert!(Cond::new_number(Some(false), true).test("-Inf"));
        assert!(Cond::new_number(Some(false), true).test(""));
        assert!(!Cond::new_number(Some(false), false).test("abc"));
        assert!(!Cond::new_number(Some(false), false).test("123"));
        assert!(Cond::new_number(Some(false), false).test("123.1"));
        assert!(Cond::new_number(Some(false), false).test("123.0"));
        assert!(!Cond::new_number(Some(false), false).test("NaN"));
        assert!(!Cond::new_number(Some(false), false).test("nan"));
        assert!(!Cond::new_number(Some(false), false).test("inf"));
        assert!(!Cond::new_number(Some(false), false).test("Inf"));
        assert!(!Cond::new_number(Some(false), false).test("-inf"));
        assert!(!Cond::new_number(Some(false), false).test("-Inf"));
        assert!(!Cond::new_number(Some(false), false).test(""));
        // number
        assert!(Cond::new_number(None, true).test("abc"));
        assert!(!Cond::new_number(None, true).test("123"));
        assert!(!Cond::new_number(None, true).test("123.1"));
        assert!(!Cond::new_number(None, true).test("123.0"));
        assert!(Cond::new_number(None, true).test("NaN"));
        assert!(Cond::new_number(None, true).test("nan"));
        assert!(Cond::new_number(None, true).test("inf"));
        assert!(Cond::new_number(None, true).test("Inf"));
        assert!(Cond::new_number(None, true).test("-inf"));
        assert!(Cond::new_number(None, true).test("-Inf"));
        assert!(Cond::new_number(None, true).test(""));
        assert!(!Cond::new_number(None, false).test("abc"));
        assert!(Cond::new_number(None, false).test("123"));
        assert!(Cond::new_number(None, false).test("123.1"));
        assert!(Cond::new_number(None, false).test("123.0"));
        assert!(!Cond::new_number(None, false).test("NaN"));
        assert!(!Cond::new_number(None, false).test("nan"));
        assert!(!Cond::new_number(None, false).test("inf"));
        assert!(!Cond::new_number(None, false).test("Inf"));
        assert!(!Cond::new_number(None, false).test("-inf"));
        assert!(!Cond::new_number(None, false).test("-Inf"));
        assert!(!Cond::new_number(None, false).test(""));
    }

    #[test]
    fn test_text_all_case() {
        // upper
        assert!(!Cond::new_text_all_case(true).test("abc"));
        assert!(Cond::new_text_all_case(true).test("ABC"));
        assert!(!Cond::new_text_all_case(true).test("abcABC"));
        assert!(Cond::new_text_all_case(true).test("你好123.#!@"));
        // lower
        assert!(Cond::new_text_all_case(false).test("abc"));
        assert!(!Cond::new_text_all_case(false).test("ABC"));
        assert!(!Cond::new_text_all_case(false).test("abcABC"));
        assert!(Cond::new_text_all_case(false).test("你好123.#!@"));
    }

    #[test]
    fn test_text_empty_or_blank() {
        // empty
        assert!(Cond::new_text_empty_or_blank(true).test(""));
        assert!(!Cond::new_text_empty_or_blank(true).test("abc"));
        assert!(!Cond::new_text_empty_or_blank(true).test(" "));
        assert!(!Cond::new_text_empty_or_blank(true).test(" \n\t\r "));
        // blank
        assert!(Cond::new_text_empty_or_blank(false).test(""));
        assert!(!Cond::new_text_empty_or_blank(false).test("abc"));
        assert!(Cond::new_text_empty_or_blank(false).test(" "));
        assert!(Cond::new_text_empty_or_blank(false).test(" \n\t\r "));
    }

    #[test]
    fn test_reg_match() {
        assert!(Cond::new_reg_match(r"[").is_err());
        assert!(Cond::new_reg_match(r"\d+").unwrap().test("123"));
        assert!(!Cond::new_reg_match(r"\d+").unwrap().test("123abc"));
        assert!(!Cond::new_reg_match(r"\d+").unwrap().test("123\n123"));
        assert!(!Cond::new_reg_match(r"(?m)\d+").unwrap().test("123\n123"));
        assert!(Cond::new_reg_match(r"(?m)[\d\n]+").unwrap().test("123\n123"));
    }
}
