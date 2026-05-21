//! Capsule marketplace browser render model — RFC §8.3.
//!
//! Three tabs:
//!
//! - **Bundled** — capsules that ship inside the signed nist-agent
//!   distribution (TIER `Bundled`). Always available; no egress
//!   required.
//! - **Site Mirror** — capsules from an operator-configured local
//!   path (typically an organization's internal mirror). Available
//!   in any egress posture that permits read access to the mirror.
//! - **On-Chain** — capsules read from the chain's CapsuleRegistry
//!   via `dyn ChainClient` (S-4). The tab is *disabled* when the
//!   active overlay's egress posture forbids on-chain reads.
//!
//! Filtering matches the feature scenario "filter by overlay
//! certification": a FERPA-only deployment must not see capsules
//! whose manifest does not list FERPA under `certified`. The
//! filter is applied at the view-construction site so the UI only
//! ever iterates compatible listings.
//!
//! The Install action hands the listing's manifest off to the
//! Capsule Inspector pane (S-10b crate `nist-agent-hitl`); that
//! pane carries the load-bearing AC-6 install gate.

use nist_agent_hitl::SigningTierBadge;
use nist_agent_prelude::Overlay;
use serde::{Deserialize, Serialize};

use crate::error::MarketplaceUiError;

/// Which source a listing came from. Drives the tab the UI renders
/// it under and the styling badge.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum MarketplaceSource {
    /// Bundled inside the signed nist-agent distribution.
    #[default]
    Bundled,
    /// Operator-configured local mirror.
    SiteMirror,
    /// Read from CapsuleRegistry on the configured chain.
    OnChain,
}

impl MarketplaceSource {
    /// Stable tab id for the Slint tab-strip.
    pub fn tab_id(self) -> &'static str {
        match self {
            Self::Bundled => "bundled",
            Self::SiteMirror => "site-mirror",
            Self::OnChain => "on-chain",
        }
    }
}

/// One capsule the marketplace can offer.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapsuleListing {
    /// Stable id within the active source; for OnChain this is the
    /// CapsuleRegistry capsule_id, for the others it's
    /// `"{name}@{version}"`.
    pub id: String,
    pub name: String,
    pub version: String,
    /// Hex-encoded SHA-256 of the capsule content. Surfaced for
    /// per-capsule verification (RFC §8.3) — the Inspector pane
    /// re-displays it for the AC-6 audit artifact.
    pub content_hash_hex: String,
    /// Publisher DID — matches the manifest's `publisher` field.
    pub publisher_did: String,
    pub summary: String,
    pub source: MarketplaceSource,
    pub signing_tier: SigningTierBadge,
    /// Overlays this capsule declares it's certified for. Empty
    /// means "no certified overlays" — such a capsule is hidden
    /// from every active overlay's view via [`MarketplaceView::filter`].
    pub certified_overlays: Vec<Overlay>,
    /// Optional risk hint — surfaced inline so the operator
    /// doesn't have to open the Inspector to triage by risk.
    pub risk_tier: String,
}

impl CapsuleListing {
    /// Whether this listing is certified for at least one of the
    /// supplied active overlays.
    pub fn certified_for_any(&self, active: &[Overlay]) -> bool {
        active.iter().any(|o| self.certified_overlays.contains(o))
    }
}

/// Operator-driven filter applied to the marketplace listing set.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MarketplaceFilter {
    /// Free-form search; matches name + summary case-insensitively.
    pub search: String,
    /// If non-empty, restrict to capsules certified for *all* of
    /// these overlays. A FERPA-only deployment passes `[Ferpa]`
    /// here; a multi-overlay deployment may pass more.
    pub required_overlays: Vec<Overlay>,
    /// Active tab — only listings from this source are returned.
    pub active_tab: MarketplaceSource,
}

/// User action returned from the marketplace UI.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MarketplaceAction {
    /// Open the Capsule Inspector (S-10b) on this listing. The
    /// harness loads the manifest, builds a `CapsuleInspectorView`,
    /// and routes the operator through the install gate.
    Install { listing_id: String },
    /// Switch active tab.
    SwitchTab { source: MarketplaceSource },
    /// Update the filter.
    Filter { filter: MarketplaceFilter },
}

/// Top-level marketplace view. Built from the harness's snapshots
/// of the three listing sources; the UI re-renders when this
/// changes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MarketplaceView {
    pub bundled: Vec<CapsuleListing>,
    pub site_mirror: Vec<CapsuleListing>,
    pub on_chain: Vec<CapsuleListing>,
    /// When the egress posture forbids on-chain reads, this is
    /// `Some(reason)` and the UI greys out the tab. RFC §8.3.
    pub on_chain_disabled_reason: Option<String>,
    pub filter: MarketplaceFilter,
}

impl MarketplaceView {
    /// Construct from raw per-source listings + a snapshot of the
    /// active overlay set. The constructor pre-filters by
    /// overlay-certification so the UI never sees a listing it
    /// must not offer.
    pub fn new(
        bundled: Vec<CapsuleListing>,
        site_mirror: Vec<CapsuleListing>,
        on_chain: Vec<CapsuleListing>,
        active_overlays: &[Overlay],
        on_chain_disabled_reason: Option<String>,
    ) -> Self {
        let f = |list: Vec<CapsuleListing>| -> Vec<CapsuleListing> {
            list.into_iter()
                .filter(|l| l.certified_for_any(active_overlays))
                .collect()
        };
        Self {
            bundled: f(bundled),
            site_mirror: f(site_mirror),
            on_chain: f(on_chain),
            on_chain_disabled_reason,
            filter: MarketplaceFilter {
                search: String::new(),
                required_overlays: active_overlays.to_vec(),
                active_tab: MarketplaceSource::Bundled,
            },
        }
    }

    /// Listings on the active tab that pass the current filter.
    /// Search is case-insensitive over name + summary. The
    /// per-overlay pre-filter already ran in the constructor; the
    /// optional `required_overlays` filter is an additional
    /// tightening (operator can ask "show me only capsules
    /// certified for BOTH HIPAA AND CMMC-L3", for example).
    pub fn visible_listings(&self) -> Vec<&CapsuleListing> {
        let src = match self.filter.active_tab {
            MarketplaceSource::Bundled => &self.bundled,
            MarketplaceSource::SiteMirror => &self.site_mirror,
            MarketplaceSource::OnChain => &self.on_chain,
        };
        let q = self.filter.search.to_lowercase();
        src.iter()
            .filter(|l| {
                if !q.is_empty()
                    && !l.name.to_lowercase().contains(&q)
                    && !l.summary.to_lowercase().contains(&q)
                {
                    return false;
                }
                if !self.filter.required_overlays.is_empty()
                    && !self
                        .filter
                        .required_overlays
                        .iter()
                        .all(|req| l.certified_overlays.contains(req))
                {
                    return false;
                }
                true
            })
            .collect()
    }

    /// Lookup by id across all tabs. Used by the Install action
    /// handler to find the manifest the operator clicked.
    pub fn find_listing(&self, listing_id: &str) -> Option<&CapsuleListing> {
        self.bundled
            .iter()
            .chain(self.site_mirror.iter())
            .chain(self.on_chain.iter())
            .find(|l| l.id == listing_id)
    }

    /// Switch the active tab. Returns an error if the operator
    /// tries to switch to OnChain while it's disabled.
    pub fn switch_tab(&mut self, source: MarketplaceSource) -> Result<(), MarketplaceUiError> {
        if source == MarketplaceSource::OnChain {
            if let Some(reason) = &self.on_chain_disabled_reason {
                return Err(MarketplaceUiError::OnChainTabDisabled {
                    reason: reason.clone(),
                });
            }
        }
        self.filter.active_tab = source;
        Ok(())
    }

    /// Validate an Install action before the harness routes it to
    /// the Inspector pane. Confirms the listing exists and that
    /// it's certified for at least one active overlay (which is
    /// already enforced by the constructor's pre-filter; this is
    /// belt-and-braces against an out-of-band caller).
    pub fn validate_install(
        &self,
        listing_id: &str,
    ) -> Result<&CapsuleListing, MarketplaceUiError> {
        let listing = self.find_listing(listing_id).ok_or_else(|| {
            MarketplaceUiError::CapsuleOverlayMismatch {
                capsule: listing_id.to_string(),
            }
        })?;
        if !listing.certified_for_any(&self.filter.required_overlays) {
            return Err(MarketplaceUiError::CapsuleOverlayMismatch {
                capsule: listing.name.clone(),
            });
        }
        Ok(listing)
    }

    /// Test/example fixture: a deterministic three-tab marketplace
    /// with one capsule per tab certified for CMMC-L3 + FERPA.
    pub fn fixture() -> Self {
        let bundled = vec![CapsuleListing {
            id: "ferpa-redact@0.3.1".into(),
            name: "ferpa-redact".into(),
            version: "0.3.1".into(),
            content_hash_hex: "1".repeat(64),
            publisher_did: "did:citrate:org:0xcitrate".into(),
            summary: "Redact PII per FERPA exclusion rules.".into(),
            source: MarketplaceSource::Bundled,
            signing_tier: SigningTierBadge::Bundled,
            certified_overlays: vec![Overlay::CmmcL3, Overlay::Ferpa],
            risk_tier: "low".into(),
        }];
        let site_mirror = vec![CapsuleListing {
            id: "campus-grade-export@1.0.0".into(),
            name: "campus-grade-export".into(),
            version: "1.0.0".into(),
            content_hash_hex: "2".repeat(64),
            publisher_did: "did:citrate:org:0xcampus".into(),
            summary: "Export gradebook to SIS over a FERPA-clean path.".into(),
            source: MarketplaceSource::SiteMirror,
            signing_tier: SigningTierBadge::Workspace,
            certified_overlays: vec![Overlay::Ferpa],
            risk_tier: "medium".into(),
        }];
        let on_chain = vec![CapsuleListing {
            id: "0xcap0001".into(),
            name: "consent-attest".into(),
            version: "0.2.0".into(),
            content_hash_hex: "3".repeat(64),
            publisher_did: "did:citrate:agent:0xattest".into(),
            summary: "Anchor a consent receipt on AnchorRegistry.".into(),
            source: MarketplaceSource::OnChain,
            signing_tier: SigningTierBadge::Managed,
            certified_overlays: vec![Overlay::CmmcL3],
            risk_tier: "medium".into(),
        }];
        Self::new(
            bundled,
            site_mirror,
            on_chain,
            &[Overlay::CmmcL3, Overlay::Ferpa],
            None,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constructor_filters_listings_not_certified_for_any_active_overlay() {
        // FERPA-only deployment with one ITAR-only capsule mixed
        // in. The ITAR capsule must not appear in any tab.
        let bundled = vec![
            CapsuleListing {
                id: "ok@1".into(),
                name: "ok".into(),
                version: "1".into(),
                content_hash_hex: "0".repeat(64),
                publisher_did: "did:x".into(),
                summary: "fine".into(),
                source: MarketplaceSource::Bundled,
                signing_tier: SigningTierBadge::Bundled,
                certified_overlays: vec![Overlay::Ferpa],
                risk_tier: "low".into(),
            },
            CapsuleListing {
                id: "bad@1".into(),
                name: "bad".into(),
                version: "1".into(),
                content_hash_hex: "0".repeat(64),
                publisher_did: "did:y".into(),
                summary: "hidden".into(),
                source: MarketplaceSource::Bundled,
                signing_tier: SigningTierBadge::Bundled,
                certified_overlays: vec![], // no certified overlays
                risk_tier: "low".into(),
            },
        ];
        let v = MarketplaceView::new(bundled, vec![], vec![], &[Overlay::Ferpa], None);
        let ids: Vec<&str> = v.bundled.iter().map(|l| l.id.as_str()).collect();
        assert_eq!(ids, vec!["ok@1"]);
    }

    #[test]
    fn on_chain_tab_disabled_blocks_switch() {
        let mut v = MarketplaceView::new(
            vec![],
            vec![],
            vec![],
            &[Overlay::Ferpa],
            Some("egress forbidden by FedRAMP High posture".into()),
        );
        match v.switch_tab(MarketplaceSource::OnChain) {
            Err(MarketplaceUiError::OnChainTabDisabled { reason }) => {
                assert!(reason.contains("egress"));
            }
            other => panic!("expected OnChainTabDisabled, got {other:?}"),
        }
        // Other tabs still switch fine.
        v.switch_tab(MarketplaceSource::SiteMirror)
            .expect("non-on-chain tab switch should succeed");
    }

    #[test]
    fn search_filter_is_case_insensitive_and_matches_summary() {
        let mut v = MarketplaceView::fixture();
        v.filter.search = "REDACT".into();
        let visible: Vec<&str> = v
            .visible_listings()
            .iter()
            .map(|l| l.name.as_str())
            .collect();
        assert_eq!(visible, vec!["ferpa-redact"]);
    }

    #[test]
    fn required_overlays_filter_requires_all_to_be_certified() {
        let mut v = MarketplaceView::fixture();
        v.filter.required_overlays = vec![Overlay::CmmcL3, Overlay::Ferpa];
        // Only ferpa-redact is certified for BOTH.
        let visible: Vec<&str> = v
            .visible_listings()
            .iter()
            .map(|l| l.name.as_str())
            .collect();
        assert_eq!(visible, vec!["ferpa-redact"]);
    }

    #[test]
    fn validate_install_rejects_unknown_id() {
        let v = MarketplaceView::fixture();
        match v.validate_install("nope") {
            Err(MarketplaceUiError::CapsuleOverlayMismatch { capsule }) => {
                assert_eq!(capsule, "nope");
            }
            other => panic!("expected CapsuleOverlayMismatch, got {other:?}"),
        }
    }

    #[test]
    fn validate_install_rejects_capsule_with_no_matching_overlay() {
        // Fixture is built for CMMC-L3 + FERPA. Re-narrow the
        // filter to HIPAA-only — every fixture listing is now
        // ineligible, so a previously visible Install must be
        // rejected.
        let mut v = MarketplaceView::fixture();
        v.filter.required_overlays = vec![Overlay::HipaaHitech];
        match v.validate_install("ferpa-redact@0.3.1") {
            Err(MarketplaceUiError::CapsuleOverlayMismatch { capsule }) => {
                assert_eq!(capsule, "ferpa-redact");
            }
            other => panic!("expected CapsuleOverlayMismatch, got {other:?}"),
        }
    }

    #[test]
    fn find_listing_searches_all_three_tabs() {
        let v = MarketplaceView::fixture();
        assert!(v.find_listing("ferpa-redact@0.3.1").is_some());
        assert!(v.find_listing("campus-grade-export@1.0.0").is_some());
        assert!(v.find_listing("0xcap0001").is_some());
        assert!(v.find_listing("missing").is_none());
    }

    #[test]
    fn source_serializes_kebab_case() {
        // Tab id is used as a stable wire form when persisting
        // operator's last-active-tab preference; pin it.
        assert_eq!(MarketplaceSource::OnChain.tab_id(), "on-chain");
        let json = serde_json::to_string(&MarketplaceSource::SiteMirror).unwrap();
        assert_eq!(json, "\"site-mirror\"");
    }

    #[test]
    fn action_round_trips_install() {
        let a = MarketplaceAction::Install {
            listing_id: "x".into(),
        };
        let s = serde_json::to_string(&a).unwrap();
        let r: MarketplaceAction = serde_json::from_str(&s).unwrap();
        assert_eq!(a, r);
    }
}
