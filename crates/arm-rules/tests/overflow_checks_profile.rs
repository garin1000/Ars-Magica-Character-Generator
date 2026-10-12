//! K9: locks in `[profile.release] overflow-checks = true` (root `Cargo.toml`).
//!
//! Rust's `dev`/`test` profiles enable overflow checks by default regardless
//! of this setting, so this test is only meaningful run against the
//! **release** profile: `cargo test --workspace --release
//! --test all overflow_checks_profile::`. Under `cargo test --workspace` (the
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

// --- E3: a static guard, exercised under every profile ----------------------
//
// `overflow_checks_are_enabled_in_this_profile` above only proves anything
// under `cargo test --workspace --release`, which neither CLAUDE.md's
// Required gate nor CI ever run (see its own doc comment). A future removal
// or weakening of `[profile.release] overflow-checks = true` would therefore
// ship silently: every gate stays green, including the one test written to
// catch it. The check below reads the root `Cargo.toml` as text and runs
// under the one command CLAUDE.md's Required gate and CI actually use
// (`cargo test --workspace`, any profile), closing that gap without adding a
// `--release` step anywhere (left alone, per the fix brief).

/// Root `Cargo.toml`, included at compile time — three levels up from
/// `crates/arm-rules/tests/`, the same relative depth
/// `tests/robust_overflow.rs` already uses for `rules/core/*.json`.
const ROOT_CARGO_TOML: &str = include_str!("../../../Cargo.toml");

/// Whether `cargo_toml`'s `[profile.release]` table sets `overflow-checks =
/// true`. Checked as text rather than through a TOML parser — this workspace
/// has no `toml` crate dependency, and one line's presence inside one named
/// section does not need one (the same reasoning already governs the other
/// text-based source guards in this crate, e.g. `rulebook_citations.rs`).
///
/// Finds the `[profile.release]` header, takes everything up to the next `[`
/// header (or end of file), and looks for an `overflow-checks = true` line
/// inside that slice — so a `true` sitting under a DIFFERENT profile is
/// correctly not counted.
fn release_profile_has_overflow_checks(cargo_toml: &str) -> bool {
    let Some(after_header) = cargo_toml.split("[profile.release]").nth(1) else {
        return false;
    };
    let block = after_header.split("\n[").next().unwrap_or(after_header);
    block
        .lines()
        .any(|line| line.trim() == "overflow-checks = true")
}

/// A `[profile.release]` table missing the key — the shape a future edit that
/// drops or weakens the line would leave behind. The checker above must
/// report absence here rather than always answering `true`.
const RELEASE_PROFILE_WITHOUT_OVERFLOW_CHECKS: &str = r#"
[profile.dev]
debug = "line-tables-only"

[profile.release]
lto = true
"#;

#[test]
fn a_release_profile_missing_the_key_is_detected_as_missing() {
    assert!(
        !release_profile_has_overflow_checks(RELEASE_PROFILE_WITHOUT_OVERFLOW_CHECKS),
        "a [profile.release] table with no overflow-checks line must not read as locked in"
    );
}

/// A `true` sitting under a DIFFERENT profile must not count — otherwise the
/// checker would pass even after `[profile.release]`'s own line was deleted,
/// as long as some other profile still mentioned the key.
const OVERFLOW_CHECKS_TRUE_UNDER_A_DIFFERENT_PROFILE: &str = r#"
[profile.dev]
overflow-checks = true

[profile.release]
lto = true
"#;

#[test]
fn overflow_checks_under_a_different_profile_does_not_count() {
    assert!(
        !release_profile_has_overflow_checks(OVERFLOW_CHECKS_TRUE_UNDER_A_DIFFERENT_PROFILE),
        "overflow-checks under [profile.dev] must not satisfy the [profile.release] check"
    );
}

/// E3's actual regression guard: run under every `cargo test --workspace`
/// invocation, not only `--release`, so a future removal of the line is
/// caught by the one command CLAUDE.md's Required gate and CI actually run.
#[test]
fn the_shipped_cargo_toml_locks_overflow_checks_in_the_release_profile() {
    assert!(
        release_profile_has_overflow_checks(ROOT_CARGO_TOML),
        "root Cargo.toml's [profile.release] must set overflow-checks = true (K9); see \
         overflow_checks_are_enabled_in_this_profile for why dev-profile `cargo test` \
         alone cannot catch its removal"
    );
}
