//! Tauri desktop shell for the Ars Magica character generator.
//!
//! All filesystem IO and Tauri commands live here; the rules engine
//! (`arm-rules`) stays pure. The command logic is split so the bulk lives in
//! [`ruleset_io`] as webview-free functions that integration tests call
//! directly, while [`commands`] holds only the thin Tauri shims. The Tauri
//! `Builder` itself lives in `main.rs` so this library — and its tests —
//! compile without the bundled frontend.

#![deny(clippy::all)]

pub mod commands;
pub mod error;
pub mod ruleset_io;
pub mod settings;
