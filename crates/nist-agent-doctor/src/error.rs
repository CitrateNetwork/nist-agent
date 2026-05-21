//! `DoctorError` — typed errors for the doctor checks.
//!
//! Note: doctor checks themselves return `CheckResult` (with a
//! `Severity` enum) rather than `Result`. This error type is used
//! for context construction failures only.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum DoctorError {
    #[error("context: {0}")]
    Context(String),

    #[error("io: {0}")]
    Io(String),
}
