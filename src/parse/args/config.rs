use crate::config::Config;
use std::iter::Peekable;

pub fn parse_configs(args: &mut Peekable<impl Iterator<Item = String>>) -> Vec<Config> {
    let mut configs = Vec::new();
    while let Some(config) = parse_config(args.peek()) {
        args.next();
        configs.push(config);
    }
    configs
}

fn parse_config(arg: Option<&String>) -> Option<Config> {
    match arg {
        Some(arg) => match arg.as_str() {
            "-h" => Some(Config::Help),
            "-V" => Some(Config::Version),
            "-v" => Some(Config::Verbose),
            "-d" => Some(Config::DryRun),
            "--nocase" => Some(Config::Nocase),
            "--eval" => Some(Config::Eval),
            _ => None, // 遇到未知参数，停止解析（由调用者处理）
        },
        None => None,
    }
}
