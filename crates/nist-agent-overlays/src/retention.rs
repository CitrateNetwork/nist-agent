//! Retention floor table per RFC §6.4.
//!
//! "Retention periods are overlay-driven, expressed in the policy
//! bundle, and enforced by the harness's audit-record retention
//! scheduler. Records MUST NOT be deleted before their overlay-
//! derived retention period. Deletion of a record after expiration
//! is itself an audited event requiring AU-9(5) dual authorization."
//!
//! Values cross-checked against `features/core/audit-retention.feature`:
//!
//! | overlay        | floor   |
//! |----------------|---------|
//! | CMMC-L3        | 6 years |
//! | HIPAA / HITECH | 6 years |
//! | FedRAMP-High   | 3 years |
//! | FERPA          | 5 years |
//! | COPPA / CIPA   | 5 years (school context — same floor as FERPA) |
//!
//! Per the planset's S-6b RETRO, future PolicyBundle v2 incorporates
//! this table directly into the signed bundle. For v1, the table is
//! a constant the harness consults at retention-scheduler time.

use nist_agent_prelude::Overlay;
use std::time::Duration;

const ONE_YEAR_SECONDS: u64 = 365 * 24 * 60 * 60;

/// The minimum retention period an audit record written under
/// `overlay` MUST observe before deletion. If multiple overlays
/// are active, the harness takes the MAX of each active overlay's
/// floor — the most-restrictive overlay wins.
pub fn retention_floor(overlay: Overlay) -> Duration {
    let years = match overlay {
        Overlay::CmmcL3 => 6,
        Overlay::Ferpa => 5,
        Overlay::Coppa => 5,
        Overlay::Cipa => 5,
        Overlay::HipaaHitech => 6,
        Overlay::FedrampHigh => 3,
    };
    Duration::from_secs(years * ONE_YEAR_SECONDS)
}

/// Compute the effective retention floor across an active overlay
/// set — the max of each member's floor. Empty set returns the
/// CMMC-L3 baseline (6 years) because that overlay is always
/// active per RFC §2.1.
pub fn effective_floor<I>(active: I) -> Duration
where
    I: IntoIterator<Item = Overlay>,
{
    let mut max = retention_floor(Overlay::CmmcL3);
    for o in active {
        let f = retention_floor(o);
        if f > max {
            max = f;
        }
    }
    max
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retention_matches_audit_retention_feature() {
        // Pin the values to the feature file:
        //   CMMC-L3:      6y
        //   HIPAA:        6y
        //   FedRAMP-High: 3y
        //   FERPA:        5y
        assert_eq!(
            retention_floor(Overlay::CmmcL3).as_secs() / ONE_YEAR_SECONDS,
            6
        );
        assert_eq!(
            retention_floor(Overlay::HipaaHitech).as_secs() / ONE_YEAR_SECONDS,
            6
        );
        assert_eq!(
            retention_floor(Overlay::FedrampHigh).as_secs() / ONE_YEAR_SECONDS,
            3
        );
        assert_eq!(
            retention_floor(Overlay::Ferpa).as_secs() / ONE_YEAR_SECONDS,
            5
        );
    }

    #[test]
    fn coppa_and_cipa_match_ferpa_floor() {
        // School context — same retention floor as FERPA.
        assert_eq!(
            retention_floor(Overlay::Coppa),
            retention_floor(Overlay::Ferpa)
        );
        assert_eq!(
            retention_floor(Overlay::Cipa),
            retention_floor(Overlay::Ferpa)
        );
    }

    #[test]
    fn effective_floor_takes_max() {
        // CMMC-L3 baseline + FERPA = MAX(6y, 5y) = 6y.
        let floor = effective_floor([Overlay::CmmcL3, Overlay::Ferpa]);
        assert_eq!(floor.as_secs() / ONE_YEAR_SECONDS, 6);

        // CMMC-L3 + FedRAMP-High = MAX(6y, 3y) = 6y (the longer floor wins).
        let floor = effective_floor([Overlay::CmmcL3, Overlay::FedrampHigh]);
        assert_eq!(floor.as_secs() / ONE_YEAR_SECONDS, 6);
    }

    #[test]
    fn effective_floor_empty_falls_back_to_cmmc_baseline() {
        // No overlays in the iterator → CMMC-L3 baseline is the
        // implicit floor (RFC §2.1).
        let floor = effective_floor(std::iter::empty());
        assert_eq!(floor.as_secs() / ONE_YEAR_SECONDS, 6);
    }
}
