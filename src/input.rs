#[derive(Debug, Eq, PartialEq)]
pub(crate) enum Input {
    StdIn,
    File { files: Vec<String> },
    Clip,
    Of { values: Vec<String> },
}
