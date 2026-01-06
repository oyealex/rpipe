use std::process::{ExitCode, Termination};
use thiserror::Error;

#[derive(Error, Debug, Eq, PartialEq)]
pub(crate) enum RpErr {
    #[error("[Config Token] Invalid args: {0}")]
    ParseConfigTokenErr(String),

    #[error("[Input Token] Invalid args: {0}")]
    ParseInputTokenErr(String),

    #[error("[Op Token] Invalid args: {0}")]
    ParseOpTokenErr(String),

    #[error("[Output Token] Invalid args: {0}")]
    ParseOutputTokenErr(String),

    #[error("[Arg Parse Err] Unable to parse `{arg_value}` in argument `{arg}` of cmd `{cmd}`, error: {error}")]
    ArgParseErr { cmd: &'static str, arg: &'static str, arg_value: String, error: String },

    #[error("[Bad Arg] Unexpected remaining value `{remaining}` in argument `{arg}` of cmd `{cmd}`")]
    UnexpectedRemaining { cmd: &'static str, arg: &'static str, remaining: String },

    #[error("[Missing Arg] Missing argument `{arg}` of cmd `{cmd}`")]
    MissingArg { cmd: &'static str, arg: &'static str },

    #[error("[Missing Arg] At least one value for argument `{arg}` is required for cmd `{cmd}`")]
    ArgNotEnough { cmd: &'static str, arg: &'static str },

    #[error("[Bad Arg] Closing bracket (`]`) for argument `{arg}` is required for cmd `{cmd}`")]
    UnclosingMultiArg { cmd: &'static str, arg: &'static str },

    #[error("[Bad Arg] Unexpected closing bracket of argument `{arg}` for cmd `{cmd}`")]
    UnexpectedClosingBracket { cmd: &'static str, arg: &'static str },

    #[error("[Bad Arg] Unknown arguments: {args:?}")]
    UnknownArgs { args: Vec<String> },

    #[error("[Input] Read text from clipboard error: {0}")]
    ReadClipboardTextErr(String),

    #[error("[Input] Open input file `{file}` error: {err}")]
    OpenInputFileErr { file: String, err: String },

    #[error("[Input] Read line `{line_no}` of input file `{file}` error: {err}")]
    ReadFromInputFileErr { file: String, line_no: usize, err: String },

    #[error("[Output] Write result to clipboard error: {0}")]
    WriteToClipboardErr(String),

    #[error("[Output] Open output file `{file}` error: {err}")]
    OpenOutputFileErr { file: String, err: String },

    #[error("[Output] Write item `{item}` to file `{file}` error: {err}")]
    WriteToOutputFileErr { file: String, item: String, err: String },
}

impl Termination for RpErr {
    fn report(self) -> ExitCode {
        eprintln!("{}", self);
        ExitCode::from(self.exit_code())
    }
}

impl RpErr {
    pub fn termination(self) -> ! {
        let exit_code = self.exit_code();
        self.report();
        std::process::exit(exit_code as i32);
    }

    fn exit_code(&self) -> u8 {
        let mut code = 0u8..;
        match self {
            RpErr::ParseConfigTokenErr(_) => code.next().unwrap(),
            RpErr::ParseInputTokenErr(_) => code.next().unwrap(),
            RpErr::ParseOpTokenErr(_) => code.next().unwrap(),
            RpErr::ParseOutputTokenErr(_) => code.next().unwrap(),
            RpErr::ArgParseErr { .. } => code.next().unwrap(),
            RpErr::UnexpectedRemaining { .. } => code.next().unwrap(),
            RpErr::MissingArg { .. } => code.next().unwrap(),
            RpErr::ArgNotEnough { .. } => code.next().unwrap(),
            RpErr::UnclosingMultiArg { .. } => code.next().unwrap(),
            RpErr::UnexpectedClosingBracket { .. } => code.next().unwrap(),
            RpErr::UnknownArgs { .. } => code.next().unwrap(),
            RpErr::ReadClipboardTextErr(_) => code.next().unwrap(),
            RpErr::OpenInputFileErr { .. } => code.next().unwrap(),
            RpErr::ReadFromInputFileErr { .. } => code.next().unwrap(),
            RpErr::WriteToClipboardErr(_) => code.next().unwrap(),
            RpErr::OpenOutputFileErr { .. } => code.next().unwrap(),
            RpErr::WriteToOutputFileErr { .. } => code.next().unwrap(),
        }
    }
}
