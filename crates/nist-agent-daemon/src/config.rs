//! `DaemonConfig` — TOML schema the operator supplies to
//! `citrate-agent daemon --config <path>`.
//!
//! Example:
//!
//! ```toml
//! [daemon]
//! audit_sink_path  = "/var/lib/citrate-agent/audit"
//! ipc_socket_path  = "/run/citrate-agent.sock"
//!
//! [anchor]
//! strategy        = "hybrid"        # one of: none, nightly, per-install, hybrid
//! nightly_at_iso  = "02:00"          # local time, 24h, for nightly / hybrid
//!
//! [policy]
//! bundle_path     = "/etc/citrate-agent/policy.cbor"
//! so_pubkey_hex   = "0x..."          # SecurityOfficer trust root
//! ```
//!
//! Validation happens at load time; the daemon refuses to start
//! against a config that would silently misbehave (empty paths,
//! sub-second cadence, etc.).

use crate::error::DaemonError;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Anchor cadence strategy. Matches RFC §6.3.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum AnchorStrategy {
    /// No on-chain anchoring. Operator runs fully offline.
    None,
    /// One Merkle root per day, written at `nightly_at_iso`.
    Nightly,
    /// One anchor per capsule install event. No nightly roll-up.
    PerInstall,
    /// Both: nightly root + per-install events. v1.0 default.
    #[default]
    Hybrid,
}

impl AnchorStrategy {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Nightly => "nightly",
            Self::PerInstall => "per-install",
            Self::Hybrid => "hybrid",
        }
    }
}

/// Top-level config struct. Serde derives match the TOML layout
/// in this module's docs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DaemonConfig {
    pub daemon: DaemonSection,
    #[serde(default)]
    pub anchor: AnchorSection,
    #[serde(default)]
    pub policy: PolicySection,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DaemonSection {
    pub audit_sink_path: PathBuf,
    pub ipc_socket_path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AnchorSection {
    #[serde(default)]
    pub strategy: AnchorStrategy,
    /// Local-time `HH:MM` (24h) for nightly / hybrid strategies.
    /// Ignored when strategy is `none` or `per-install`.
    #[serde(default = "default_nightly")]
    pub nightly_at_iso: String,
}

impl Default for AnchorSection {
    // Manual impl so `#[serde(default)]` on a missing section
    // yields the same shape as a present empty section.
    fn default() -> Self {
        Self {
            strategy: AnchorStrategy::default(),
            nightly_at_iso: default_nightly(),
        }
    }
}

fn default_nightly() -> String {
    "02:00".to_string()
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct PolicySection {
    /// PolicyBundle CBOR file path. `None` boots the daemon in
    /// minimal mode (no policy enforcement; useful for the air-
    /// gap smoke test).
    #[serde(default)]
    pub bundle_path: Option<PathBuf>,
    /// SecurityOfficer trust-root public key (hex). Required
    /// when `bundle_path` is set.
    #[serde(default)]
    pub so_pubkey_hex: Option<String>,
}

impl DaemonConfig {
    /// Parse a TOML string. Stable error class on decode failure.
    pub fn from_toml(s: &str) -> Result<Self, DaemonError> {
        toml::from_str(s).map_err(|e| DaemonError::ConfigDecode(e.to_string()))
    }

    /// Load + parse from a file. Convenience for the CLI.
    pub fn from_path(path: impl AsRef<Path>) -> Result<Self, DaemonError> {
        let s = std::fs::read_to_string(path.as_ref())
            .map_err(|e| DaemonError::Filesystem(format!("read config: {e}")))?;
        Self::from_toml(&s)
    }

    /// Validate at start time so the daemon refuses to come up
    /// against a misconfigured operator (rather than silently
    /// degrading mid-run). The CLI's `daemon` subcommand calls
    /// this before any IPC bring-up.
    pub fn validate(&self) -> Result<(), DaemonError> {
        if self.daemon.audit_sink_path.as_os_str().is_empty() {
            return Err(DaemonError::ConfigInvalid {
                field: "daemon.audit_sink_path".into(),
                reason: "empty path".into(),
            });
        }
        if self.daemon.ipc_socket_path.as_os_str().is_empty() {
            return Err(DaemonError::ConfigInvalid {
                field: "daemon.ipc_socket_path".into(),
                reason: "empty path".into(),
            });
        }
        if matches!(
            self.anchor.strategy,
            AnchorStrategy::Nightly | AnchorStrategy::Hybrid
        ) && parse_hhmm(&self.anchor.nightly_at_iso).is_none()
        {
            return Err(DaemonError::ConfigInvalid {
                field: "anchor.nightly_at_iso".into(),
                reason: format!("expected HH:MM 24h, got '{}'", self.anchor.nightly_at_iso),
            });
        }
        if self.policy.bundle_path.is_some() && self.policy.so_pubkey_hex.is_none() {
            return Err(DaemonError::ConfigInvalid {
                field: "policy.so_pubkey_hex".into(),
                reason: "required when policy.bundle_path is set".into(),
            });
        }
        Ok(())
    }

    /// Fixture used by tests + the CLI's `--smoke` path. Points
    /// every path at the supplied scratch directory so callers
    /// don't pollute system paths.
    pub fn fixture(scratch: &Path) -> Self {
        Self {
            daemon: DaemonSection {
                audit_sink_path: scratch.join("audit"),
                ipc_socket_path: scratch.join("daemon.sock"),
            },
            anchor: AnchorSection {
                strategy: AnchorStrategy::None,
                nightly_at_iso: "02:00".into(),
            },
            policy: PolicySection::default(),
        }
    }
}

/// Parse a `HH:MM` time string. Returns `(hour, minute)` if
/// valid, `None` otherwise.
pub(crate) fn parse_hhmm(s: &str) -> Option<(u8, u8)> {
    let (h, m) = s.split_once(':')?;
    let h: u8 = h.parse().ok()?;
    let m: u8 = m.parse().ok()?;
    if h < 24 && m < 60 {
        Some((h, m))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn anchor_strategy_serializes_kebab_case() {
        assert_eq!(
            serde_json::to_string(&AnchorStrategy::PerInstall).unwrap(),
            "\"per-install\""
        );
        assert_eq!(AnchorStrategy::Hybrid.as_str(), "hybrid");
    }

    #[test]
    fn default_anchor_strategy_is_hybrid_per_rfc() {
        assert_eq!(AnchorStrategy::default(), AnchorStrategy::Hybrid);
    }

    #[test]
    fn minimal_toml_parses_with_defaults() {
        let s = r#"
        [daemon]
        audit_sink_path = "/var/lib/audit"
        ipc_socket_path = "/run/daemon.sock"
        "#;
        let cfg = DaemonConfig::from_toml(s).unwrap();
        assert_eq!(cfg.anchor.strategy, AnchorStrategy::Hybrid);
        assert_eq!(cfg.anchor.nightly_at_iso, "02:00");
        assert!(cfg.policy.bundle_path.is_none());
        cfg.validate().expect("valid minimal config");
    }

    #[test]
    fn empty_paths_refused_at_validate() {
        let mut cfg = DaemonConfig::fixture(std::path::Path::new("/tmp/x"));
        cfg.daemon.audit_sink_path = PathBuf::new();
        match cfg.validate() {
            Err(DaemonError::ConfigInvalid { field, .. }) => {
                assert_eq!(field, "daemon.audit_sink_path")
            }
            other => panic!("expected ConfigInvalid, got {other:?}"),
        }
    }

    #[test]
    fn nightly_strategy_requires_valid_time() {
        let mut cfg = DaemonConfig::fixture(std::path::Path::new("/tmp/x"));
        cfg.anchor.strategy = AnchorStrategy::Nightly;
        cfg.anchor.nightly_at_iso = "not-a-time".into();
        let err = cfg.validate().unwrap_err();
        match err {
            DaemonError::ConfigInvalid { field, .. } => {
                assert_eq!(field, "anchor.nightly_at_iso")
            }
            other => panic!("expected ConfigInvalid, got {other:?}"),
        }
    }

    #[test]
    fn nightly_strategy_accepts_boundary_times() {
        for t in ["00:00", "23:59", "02:00", "12:30"] {
            let mut cfg = DaemonConfig::fixture(std::path::Path::new("/tmp/x"));
            cfg.anchor.strategy = AnchorStrategy::Nightly;
            cfg.anchor.nightly_at_iso = t.into();
            cfg.validate()
                .unwrap_or_else(|e| panic!("'{t}' should be valid: {e:?}"));
        }
    }

    #[test]
    fn none_strategy_skips_time_validation() {
        let mut cfg = DaemonConfig::fixture(std::path::Path::new("/tmp/x"));
        cfg.anchor.strategy = AnchorStrategy::None;
        cfg.anchor.nightly_at_iso = "garbage".into();
        cfg.validate().expect("None strategy ignores time field");
    }

    #[test]
    fn bundle_path_without_pubkey_refused() {
        let mut cfg = DaemonConfig::fixture(std::path::Path::new("/tmp/x"));
        cfg.policy.bundle_path = Some(PathBuf::from("/etc/policy.cbor"));
        cfg.policy.so_pubkey_hex = None;
        let err = cfg.validate().unwrap_err();
        match err {
            DaemonError::ConfigInvalid { field, .. } => {
                assert_eq!(field, "policy.so_pubkey_hex")
            }
            other => panic!("expected ConfigInvalid, got {other:?}"),
        }
    }

    #[test]
    fn parse_hhmm_round_trips() {
        assert_eq!(parse_hhmm("00:00"), Some((0, 0)));
        assert_eq!(parse_hhmm("23:59"), Some((23, 59)));
        assert_eq!(parse_hhmm("24:00"), None);
        assert_eq!(parse_hhmm("12:60"), None);
        assert_eq!(parse_hhmm("not"), None);
    }
}
