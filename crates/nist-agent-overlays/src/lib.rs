//! nist-agent-overlays — PolicyBundle factories per RFC §2.3.
//!
//! Each overlay's `factory` function returns a canonical
//! `PolicyBundle` ready to be signed by the SecurityOfficer. The
//! factories take role assignments and validity window as
//! parameters; everything else is overlay-prescribed.
//!
//! S-8 ships the first four overlays:
//!
//!   - [`cmmc_l3_baseline`] — RFC §2.1 baseline applies to every
//!     deployment regardless of overlay selection
//!   - [`ferpa`] — CMMC-L3 + Family Educational Rights and Privacy Act
//!   - [`coppa`] — CMMC-L3 + Children's Online Privacy Protection Act
//!   - [`cipa`] — CMMC-L3 + Children's Internet Protection Act
//!
//! HIPAA + FedRAMP-High ship in S-9; ITAR / CJIS / IRS 1075 / IL5
//! ship in the v1.1 planset (RFC §11.2).
//!
//! Plus the retention-floor table (`retention`) per RFC §6.4.

pub mod profiles;
pub mod retention;

pub use profiles::{cipa, cmmc_l3_baseline, coppa, ferpa, OverlayBuilder};
pub use retention::retention_floor;
