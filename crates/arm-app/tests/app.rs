//! The root of this crate's main integration-test binary.
//!
//! Every `tests/*.rs` file except `commands.rs` is a module of this one crate
//! rather than a binary of its own, so an edit relinks one tauri-linked test
//! binary instead of one per file. `commands.rs` stays its own target because
//! it mutates process environment variables, which a shared process would
//! expose to every other test running beside it. `Cargo.toml` sets
//! `autotests = false`, so a new test file must be declared here (or get a
//! target of its own); the `test_targets.rs` guard in arm-rules fails until it
//! is.
//!
//! Run one file's tests with its module path as the filter:
//! `cargo test --workspace --test app menu::`.

mod a2_shipped_examples;
mod a3_mock_app_commands;
mod atomic_write;
mod i3_bonus_sources_dto;
mod l2_language_requirements;
mod menu;
mod settings;
mod spell_caps_dto;
mod update_check;
