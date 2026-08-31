//! K9: locks in `[profile.release] overflow-checks = true` (root `Cargo.toml`).
//!
//! Rust's `dev`/`test` profiles enable overflow checks by default regardless
//! of this setting, so this test is only meaningful run against the
//! **release** profile: `cargo test --workspace --release
//! --test overflow_checks_profile`. Under `cargo test --workspace` (the
//! default dev profile) it passes trivially either way, because dev already
//! checks overflow — that is precisely the gap K9 closed: dev panics, release
//! silently wrapped. If a future change reverts or weakens the
//! `[profile.release]` block, this is the test that would go from "panics as
//! expected" to "test did not panic" the next time someone runs the suite
//! with `--release`.
#[test]
#[should_panic = "attempt to add with overflow"]
fn overflow_checks_are_enabled_in_this_profile() {
    let max = u8::MAX;
    let one = 1u8;
    let _ = std::hint::black_box(max) + std::hint::black_box(one);
}
