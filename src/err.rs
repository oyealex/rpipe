use std::process::{ExitCode, Termination};
use thiserror::Error;

#[derive(Error, Debug)]
pub(crate) enum RpErr {
    #[error("[Input Token] Invalid args: {0}")]
    ParseInputTokenErr(String),

    #[error("[Op Token] Invalid args: {0}")]
    ParseOpTokenErr(String),

    #[error("[Output Token] Invalid args: {0}")]
    ParseOutputTokenErr(String),

    #[error("[Arg Parse Err] Unable to parse `{arg_value}` in argument `{arg}` of cmd `{cmd}`, error: {error}")]
    ArgParseErr { cmd: &'static str, arg: &'static str, arg_value: String, error: String },

    #[error("[Bad Arg] Bad value `{arg_value}` in argument `{arg}` of cmd `{cmd}`")]
    BadArg { cmd: &'static str, arg: &'static str, arg_value: String },

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
        eprintln!(">>{}", self);
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
        match self {
            RpErr::ParseInputTokenErr(_) => 1,
            RpErr::ParseOpTokenErr(_) => 2,
            RpErr::ParseOutputTokenErr(_) => 3,
            RpErr::ArgParseErr { .. } => 4,
            RpErr::BadArg { .. } => 5,
            RpErr::UnexpectedRemaining { .. } => 6,
            RpErr::MissingArg { .. } => 7,
            RpErr::ArgNotEnough { .. } => 8,
            RpErr::UnclosingMultiArg { .. } => 9,
            RpErr::UnexpectedClosingBracket { .. } => 10,
            RpErr::UnknownArgs { .. } => 11,
            RpErr::ReadClipboardTextErr(_) => 12,
            RpErr::OpenInputFileErr { .. } => 13,
            RpErr::ReadFromInputFileErr { .. } => 14,
            RpErr::WriteToClipboardErr(_) => 15,
            RpErr::OpenOutputFileErr { .. } => 16,
            RpErr::WriteToOutputFileErr { .. } => 17,
        }
    }
}
