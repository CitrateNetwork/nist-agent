// Slint codegen gated by CARGO_FEATURE_FEAT_UI. Same pattern as
// `nist-agent-wizard`'s build.rs.

fn main() {
    if std::env::var("CARGO_FEATURE_FEAT_UI").is_ok() {
        slint_build::compile("ui/hitl.slint").expect("slint compile failed");
    }
}
