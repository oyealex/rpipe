#[derive(Debug, Eq, PartialEq)]
pub(crate) enum Config {
    /// `-h`
    Help,
    /// `-V`
    Version,
    /// `-v`
    Verbose,
    /// `-d`
    DryRun,
    /// `--nocase`
    Nocase,
}

#[inline]
pub(crate) fn is_nocase(nocase: bool, configs: &[Config]) -> bool {
    nocase || configs.contains(&Config::Nocase)
}
