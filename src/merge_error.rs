use std::ffi::OsString;
use std::io;
use std::path::PathBuf;
use thiserror::Error;

/// Errors that happen while creating the merged output static library from the extracted inputs
#[derive(Debug, Error)]
pub enum MergeError {
    #[error(
        "{reason}: {tool:?} {args:?})\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(stdout),
        String::from_utf8_lossy(stderr)
    )]
    ExternalToolError {
        reason: String,
        tool: String,
        args: Vec<OsString>,
        stdout: Vec<u8>,
        stderr: Vec<u8>,
    },
    #[error("failed to launch external tool `{tool}`: {inner})")]
    ExternalToolLaunchError { tool: String, inner: io::Error },
    #[error("failed to parse extracted object file at {path}: {inner}")]
    InvalidObject { path: PathBuf, inner: object::Error },
    #[error("zero objects left after filtering! Make sure to keep at least one public symbol")]
    NoObjectsLeft,
    #[error("failed to write merged output: {0}")]
    WritingArchive(io::Error),
    #[error("internal I/O error: {0}")]
    InternalIoError(#[from] io::Error),
    #[error("internal error while merging libraries: {0}")]
    InternalError(Box<dyn std::error::Error + Send + Sync + 'static>),
}
