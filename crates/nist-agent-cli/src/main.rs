//! `citrate-agent` — operator-facing CLI binary for nist-agent.
//!
//! Composed from the workspace's library crates per ADR-001
//! (workspace + runtime consumption) and ADR-011 (release
//! verifier). The CLI does not own logic; it owns dispatch.
//! Every subcommand is a thin wrapper around the relevant
//! library function.
//!
//! Subcommand shape:
//!
//! ```text
//! citrate-agent doctor                 # 11 pre-flight checks
//! citrate-agent model verify ...       # SI-7 GGUF hash check
//! citrate-agent model infer  ...       # llama.cpp inference (feat-flagged)
//! citrate-agent install     ...        # release-bundle install
//! citrate-agent wizard                 # Slint concierge (feat-ui)
//! citrate-agent daemon                 # minimal daemon loop
//! citrate-agent status                 # daemon status
//! ```
//!
//! The exit code convention: `0` = success / pass, `1` = warn,
//! `2` = blocker / refusal. Doctor + install both honor this so
//! shell scripts can branch on severity.

use anyhow::Result;
use clap::{Parser, Subcommand};

mod commands;

/// `citrate-agent` — NIST-compliant agent harness CLI per RFC-CIT-AGENT-0001.
#[derive(Parser, Debug)]
#[command(name = "citrate-agent", version, about, long_about = None)]
struct Cli {
    /// Increase log verbosity (repeat for more).
    #[arg(short, long, action = clap::ArgAction::Count, global = true)]
    verbose: u8,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Run the 11 pre-flight checks (RFC §10.2).
    Doctor(commands::doctor::DoctorArgs),

    /// Model operations — SI-7 hash check + (optional) inference.
    Model {
        #[command(subcommand)]
        cmd: commands::model::ModelCmd,
    },

    /// Verify and install a signed release bundle.
    Install(commands::install::InstallArgs),

    /// Connect to the daemon's IPC socket and print live status.
    /// Falls back to a static readout when no daemon is reachable.
    Status(commands::status::StatusArgs),

    /// Start the daemon loop (minimal in this sprint).
    Daemon(commands::daemon::DaemonArgs),

    /// Launch the Slint concierge wizard. Requires `--features
    /// feat-ui` at build time.
    Wizard,
}

fn init_tracing(verbose: u8) {
    use tracing_subscriber::{fmt, EnvFilter};
    let default = match verbose {
        0 => "warn",
        1 => "info",
        2 => "debug",
        _ => "trace",
    };
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(default));
    fmt().with_env_filter(filter).with_target(false).init();
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    init_tracing(cli.verbose);

    let exit = match cli.command {
        Command::Doctor(args) => commands::doctor::run(args).await?,
        Command::Model { cmd } => commands::model::run(cmd).await?,
        Command::Install(args) => commands::install::run(args)?,
        Command::Status(args) => commands::status::run(args).await?,
        Command::Daemon(args) => commands::daemon::run(args).await?,
        Command::Wizard => commands::wizard::run()?,
    };

    std::process::exit(exit);
}
