#[derive(Debug, Eq, PartialEq)]
pub(crate) enum Output {
    Out,
    File { file: String },
    Clip,
}
