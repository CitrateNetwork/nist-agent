//! Slint UI binding for the marketplace + chat surfaces.
//! Feature-gated behind `feat-ui`.
//!
//! Same pattern as `nist-agent-hitl` (S-10b): include the
//! `.slint`-generated module and provide `to_visual()` conversions
//! that translate from the headless render models to Slint visual
//! structs the integration site wraps in `slint::VecModel`.

slint::include_modules!();

use crate::{CapsuleListing, ChatSessionView, ChatStreamState, ChatTurn, MarketplaceView};

impl CapsuleListing {
    pub fn to_visual(&self) -> CapsuleListingVisual {
        let overlays = self
            .certified_overlays
            .iter()
            .map(|o| format!("{o:?}").to_lowercase())
            .collect::<Vec<_>>()
            .join(", ");
        CapsuleListingVisual {
            id: self.id.clone().into(),
            name: self.name.clone().into(),
            version: self.version.clone().into(),
            summary: self.summary.clone().into(),
            publisher: self.publisher_did.clone().into(),
            signing_tier_label: format!("{:?}", self.signing_tier).to_lowercase().into(),
            certified_overlays_summary: overlays.into(),
            risk_tier: self.risk_tier.clone().into(),
        }
    }
}

impl MarketplaceView {
    /// Translate each per-tab listing slice into the Slint visual
    /// form. The integration site wraps the returned vec in
    /// `slint::ModelRc::new(slint::VecModel::from(...))`.
    pub fn bundled_visual(&self) -> Vec<CapsuleListingVisual> {
        self.bundled.iter().map(CapsuleListing::to_visual).collect()
    }
    pub fn site_mirror_visual(&self) -> Vec<CapsuleListingVisual> {
        self.site_mirror
            .iter()
            .map(CapsuleListing::to_visual)
            .collect()
    }
    pub fn on_chain_visual(&self) -> Vec<CapsuleListingVisual> {
        self.on_chain
            .iter()
            .map(CapsuleListing::to_visual)
            .collect()
    }
}

impl ChatTurn {
    pub fn to_visual(&self) -> ChatTurnVisual {
        ChatTurnVisual {
            role_label: format!("{:?}", self.role).to_lowercase().into(),
            content: self.content.clone().into(),
            timestamp: self.timestamp_iso.clone().into(),
        }
    }
}

impl ChatSessionView {
    pub fn turns_visual(&self) -> Vec<ChatTurnVisual> {
        self.turns.iter().map(ChatTurn::to_visual).collect()
    }

    pub fn stream_visual(&self) -> ChatStreamStateVisual {
        match &self.stream_state {
            ChatStreamState::Idle => ChatStreamStateVisual {
                is_streaming: false,
                is_paused: false,
                partial_or_action: "".into(),
                checkpoint_id: "".into(),
            },
            ChatStreamState::Streaming { partial } => ChatStreamStateVisual {
                is_streaming: true,
                is_paused: false,
                partial_or_action: partial.clone().into(),
                checkpoint_id: "".into(),
            },
            ChatStreamState::PausedAtAction {
                checkpoint_id,
                action_summary,
            } => ChatStreamStateVisual {
                is_streaming: false,
                is_paused: true,
                partial_or_action: action_summary.clone().into(),
                checkpoint_id: checkpoint_id.clone().into(),
            },
        }
    }
}
