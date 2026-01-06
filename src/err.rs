use std::process::{ExitCode, Termination};
use thiserror::Error;

#[derive(Error, Debug, Eq, PartialEq)]
pub(crate) enum RpErr {
    #[error("[ParseConfigTokenErr] Invalid args: {0}")]
    ParseConfigTokenErr(String),

    #[error("[ParseInputTokenErr] Invalid args: {0}")]
    ParseInputTokenErr(String),

    #[error("[ParseOpTokenErr] Invalid args: {0}")]
    ParseOpTokenErr(String),

    #[error("[ParseOutputTokenErr] Invalid args: {0}")]
    ParseOutputTokenErr(String),

    #[error("[ArgParseErr] Unable to parse `{arg_value}` in argument `{arg}` of cmd `{cmd}`, error: {error}")]
    ArgParseErr { cmd: &'static str, arg: &'static str, arg_value: String, error: String },

    #[error("[UnexpectedRemaining] Unexpected remaining value `{remaining}` in argument `{arg}` of cmd `{cmd}`")]
    UnexpectedRemaining { cmd: &'static str, arg: &'static str, remaining: String },

    #[error("[MissingArg] Missing argument `{arg}` of cmd `{cmd}`")]
    MissingArg { cmd: &'static str, arg: &'static str },

    #[error("[UnknownArgs] Unknown arguments: {args:?}")]
    UnknownArgs { args: Vec<String> },

    #[error("[ReadClipboardTextErr] Read text from clipboard error: {0}")]
    ReadClipboardTextErr(String),

    #[error("[OpenInputFileErr] Open input file `{file}` error: {err}")]
    OpenInputFileErr { file: String, err: String },

    #[error("[ReadFromInputFileErr] Read line `{line_no}` of input file `{file}` error: {err}")]
    ReadFromInputFileErr { file: String, line_no: usize, err: String },

    #[error("[WriteToClipboardErr] Write result to clipboard error: {0}")]
    WriteToClipboardErr(String),

    #[error("[OpenFileErr] Open output file `{file}` error: {err}")]
    OpenFileErr { file: String, err: String },

    #[error("[WriteToFileErr] Write item `{item}` to file `{file}` error: {err}")]
    WriteToFileErr { file: String, item: String, err: String },
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
            RpErr::UnknownArgs { .. } => code.next().unwrap(),
            RpErr::ReadClipboardTextErr(_) => code.next().unwrap(),
            RpErr::OpenInputFileErr { .. } => code.next().unwrap(),
            RpErr::ReadFromInputFileErr { .. } => code.next().unwrap(),
            RpErr::WriteToClipboardErr(_) => code.next().unwrap(),
            RpErr::OpenFileErr { .. } => code.next().unwrap(),
            RpErr::WriteToFileErr { .. } => code.next().unwrap(),
        }
    }
}
