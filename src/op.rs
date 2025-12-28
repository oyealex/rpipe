#[derive(Debug, Eq, PartialEq)]
pub(crate) enum Op {
    Upper,
    Lower,
    Replace { from: &'static str, to: &'static str, count: u32 },
}
