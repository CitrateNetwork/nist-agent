// Slint compile-time codegen. Runs only when the `feat-ui`
// feature is enabled (Cargo's optional build-dependencies aren't
// gated automatically; we detect by checking the env var Cargo
// sets for active features).

fn main() {
    // The CARGO_FEATURE_* env var is set per active feature.
    // Convert the kebab-case feature name to UPPER_SNAKE_CASE
    // for the env-var lookup.
    if std::env::var("CARGO_FEATURE_FEAT_UI").is_ok() {
        slint_build::compile("ui/wizard.slint").expect("slint compile failed");
    }
}
