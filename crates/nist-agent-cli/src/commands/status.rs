//! `citrate-agent status` — connects to the daemon's IPC
//! socket and prints the response. Falls back to a static
//! readout (build-time identity + feature flags) when no
//! socket path is given AND no daemon is running.

use anyhow::{Context, Result};
use clap::Args;
use nist_agent_daemon::{IpcRequest, IpcResponse};
use std::path::PathBuf;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;

#[derive(Args, Debug)]
pub struct StatusArgs {
    /// Path to the daemon's IPC socket. Defaults to
    /// `/run/citrate-agent.sock` (matching the v1 default
    /// daemon config). Falls back to a static readout when
    /// no socket is reachable.
    #[arg(long)]
    pub socket: Option<PathBuf>,

    /// Skip the IPC probe; just print the static readout.
    #[arg(long)]
    pub offline: bool,
}

pub async fn run(args: StatusArgs) -> Result<i32> {
    let socket = args
        .socket
        .unwrap_or_else(|| PathBuf::from("/run/citrate-agent.sock"));

    if args.offline {
        print_static_readout();
        return Ok(0);
    }

    match query_daemon(&socket).await {
        Ok(resp) => {
            print_daemon_response(&resp);
            Ok(0)
        }
        Err(_e) => {
            // The socket isn't there or the daemon isn't running.
            // Fall back to the static readout + a one-line note.
            print_static_readout();
            println!(
                "(daemon not reachable at {}; static readout)",
                socket.display()
            );
            Ok(0)
        }
    }
}

async fn query_daemon(socket: &std::path::Path) -> Result<IpcResponse> {
    let stream = UnixStream::connect(socket)
        .await
        .with_context(|| format!("connect to {}", socket.display()))?;
    let (r, mut w) = stream.into_split();
    let mut line = IpcRequest::Status.to_line();
    line.push('\n');
    w.write_all(line.as_bytes())
        .await
        .context("write request")?;
    w.shutdown().await.context("shutdown write half")?;
    let mut reader = BufReader::new(r);
    let mut buf = String::new();
    reader.read_line(&mut buf).await.context("read response")?;
    let resp = IpcResponse::from_line(&buf).context("decode response")?;
    Ok(resp)
}

fn print_static_readout() {
    println!("citrate-agent {}", env!("CARGO_PKG_VERSION"));
    println!("workspace: nist-agent");
    println!(
        "features: feat-ui={}, feat-model-llamacpp={}",
        cfg!(feature = "feat-ui"),
        cfg!(feature = "feat-model-llamacpp"),
    );
}

fn print_daemon_response(resp: &IpcResponse) {
    match resp {
        IpcResponse::Status {
            protocol_version,
            crate_version,
            queue_depth,
            last_anchor_unix,
        } => {
            println!("citrate-agent daemon {crate_version} (ipc protocol v{protocol_version})");
            println!("queue depth:     {queue_depth}");
            match last_anchor_unix {
                Some(ts) => println!("last anchor:     unix {ts}"),
                None => println!("last anchor:     never"),
            }
        }
        other => {
            // QueueDepth / RecentAudit / Error responses to a
            // Status request shouldn't happen against a v1.0
            // daemon, but print whatever we got rather than
            // panicking.
            println!("{other:?}");
        }
    }
}
