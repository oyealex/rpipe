use cmd_help::CmdHelp;

#[derive(Debug, Eq, PartialEq, CmdHelp)]
pub(crate) enum Config {
    /// -V,--version    打印版本信息。
    Version,
    /// -h,--help       打印帮助信息。
    Help,
    /// -v,--verbose    执行之前打印流水线详情。
    Verbose,
    /// -d,--dry-run    仅解析流水线，不执行。
    DryRun,
    /// -n,--nocase     全局忽略大小写。
    Nocase,
    /// -e,--eval       以Token模式解析下一个参数。
    ///                 除了紧跟的第一个参数外，其他参数会被忽略。
    ///                 -e|--eval <token>
    ///                     <token> 需要解析的文本参数，必选。
    ///                 例如：
    ///                     -e ':in :uniq :to out'
    Eval,
}

#[inline]
pub(crate) fn is_nocase(nocase: bool, configs: &[Config]) -> bool {
    nocase || configs.contains(&Config::Nocase)
}
