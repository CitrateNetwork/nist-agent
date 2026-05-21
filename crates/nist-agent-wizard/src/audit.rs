//! Per-step audit-record builders.
//!
//! Each wizard transition emits one typed `StepAudit` value.
//! The harness (or test) translates it into an
//! `AuditRecord` for `citrate_agent_core::audit::AuditSink::append`.
//! The conversion lives at the integration site — this crate
//! stays free of the upstream `AuditRecord` dependency so the
//! wizard can compile and test without the full audit chain
//! wired up.

use nist_agent_policy::types::Role;
use nist_agent_prelude::Overlay;
use serde::{Deserialize, Serialize};

/// One audit-worthy event during the wizard's run.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StepAudit {
    /// Unix epoch seconds at write time.
    pub at_unix: i64,
    /// What happened. The variant tells the audit pipeline which
    /// `EventType` the upstream `AuditRecord` should carry.
    pub kind: StepAuditKind,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "event")]
pub enum StepAuditKind {
    /// RFC §8.1 step 1 — operator identifies the deploying
    /// organization.
    OrgIdentitySet { org_did: String, org_name: String },
    /// RFC §8.1 step 2 — operator assigned an identity to a role.
    RoleAssigned { role: Role, did: String },
    /// RFC §8.1 step 3 — operator enrolled a hardware key.
    HardwareKeyEnrolled {
        role: Role,
        did: String,
        surface: String,
        fingerprint: String,
    },
    /// RFC §2.3 + §8.1 step 4 — operator activated an overlay.
    OverlayActivated { overlay: Overlay },
    /// RFC §3.1 + §8.1 step 5 — the SecurityOfficer signed the
    /// final PolicyBundle.
    PolicyBundleSigned {
        bundle_name: String,
        canonical_hash: [u8; 32],
    },
}

impl StepAudit {
    pub fn new(at_unix: i64, kind: StepAuditKind) -> Self {
        Self { at_unix, kind }
    }
}
