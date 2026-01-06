#[derive(Debug, Eq, PartialEq)]
pub(crate) enum Config {
    /// 帮助 `-h`
    Help,
    /// 版本 `-V`
    Version,
    /// 打印流水线信息 `-v`
    Verbose,
    /// 仅解析，不执行 `-d`
    DryRun,
    /// 全局忽略大小写 `--nocase`
    Nocase,
    /// 以Token模式解析下一个参数 `--eval`
    Eval,
}

#[inline]
pub(crate) fn is_nocase(nocase: bool, configs: &[Config]) -> bool {
    nocase || configs.contains(&Config::Nocase)
}
