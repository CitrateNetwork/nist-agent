//! `WizardState` — accumulated inputs across the wizard's steps.
//!
//! The state is intentionally *value-only* — no callbacks, no
//! UI handles. Transitions are pure functions
//! `(WizardState, Input) -> Result<WizardState>`. This is what
//! lets the same state machine drive a Slint UI, a CLI prompt
//! loop, and the test suite.

use nist_agent_policy::types::Role;
use nist_agent_prelude::Overlay;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// One enrolled identity for one role.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoleAssignment {
    /// DID of the identity, e.g. `did:citrate:role:0xab12...`.
    pub did: String,
    /// Human label for the operator UI (e.g. "Alice — Compliance").
    pub label: String,
}

/// One hardware key enrolled for an identity. Future S-11 will
/// add device-attestation chains; for v0.x the operator types in
/// the public-key fingerprint they'll use during HIC signing.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HardwareKey {
    /// Role the key belongs to.
    pub role: Role,
    /// DID this key authenticates (matches one entry in
    /// `WizardState::roles[role]`).
    pub did: String,
    /// Surface: "fido2" | "piv-cac" | "secure-enclave" | "tpm".
    pub surface: String,
    /// Public-key fingerprint (sha256:HEX). The operator pastes
    /// this in from their authenticator's enrollment screen.
    pub fingerprint: String,
}

/// The wizard's accumulated state. Cleared per session; written
/// to disk only at the `Signed` step.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct WizardState {
    /// Operator's chosen org name (RFC §8.1 first-run step).
    pub org_name: String,
    /// `did:citrate:org:0x...` — operator-supplied or generated.
    pub org_did: String,
    /// One entry per role; each role has ≥1 identity at sign time.
    pub roles: BTreeMap<Role, Vec<RoleAssignment>>,
    /// One entry per enrolled hardware key. May have ≥1 key per
    /// role per identity (e.g. a YubiKey + a backup).
    pub hardware_keys: Vec<HardwareKey>,
    /// Operator-chosen overlay set ON TOP of the CMMC-L3 baseline
    /// (which is always-active per RFC §2.1).
    pub additional_overlays: Vec<Overlay>,
    /// Output path the signed bundle will be written to.
    pub output_path: Option<std::path::PathBuf>,
}

impl WizardState {
    /// Helper for tests + UI: a complete set of role assignments
    /// (all five base roles have ≥1 identity).
    pub fn roles_complete(&self) -> bool {
        for r in Role::ALL {
            let count = self.roles.get(r).map(|v| v.len()).unwrap_or(0);
            if count == 0 {
                return false;
            }
        }
        true
    }
}
