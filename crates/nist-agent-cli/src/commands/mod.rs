//! CLI subcommand modules. Each module exposes a `run` entry
//! point that returns an exit code (`Result<i32>`); the binary's
//! main() dispatches and propagates the exit code via
//! `std::process::exit`.

pub mod daemon;
pub mod doctor;
pub mod install;
pub mod model;
pub mod status;
pub mod wizard;
