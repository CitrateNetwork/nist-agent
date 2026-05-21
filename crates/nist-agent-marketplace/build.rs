// Slint codegen gated by CARGO_FEATURE_FEAT_UI. Same pattern as
// `nist-agent-wizard` (S-10a) and `nist-agent-hitl` (S-10b).

fn main() {
    if std::env::var("CARGO_FEATURE_FEAT_UI").is_ok() {
        slint_build::compile("ui/marketplace.slint").expect("slint compile failed");
    }
}
