//! Guard for the consolidated integration-test binaries.
//!
//! Each crate's integration tests are compiled into a few explicit `[[test]]`
//! targets instead of one binary per `tests/*.rs` file, because 131 separately
//! linked binaries made every engine edit cost minutes of relinking. The
//! manifests therefore set `autotests = false`, and the price of that switch is
//! a silent failure mode: a new `tests/*.rs` file that no target root declares
//! as a `mod` is never compiled, so its tests never run and nothing says so.
//!
//! This guard closes that gap for every crate under `crates/` that has a
//! `tests/` directory: each target root the manifest names must exist, and
//! every other top-level `tests/*.rs` file must be declared as a module in
//! exactly one of those roots (twice would compile and run it twice).

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

/// `crates/arm-rules` (this crate's manifest dir) -> the repository root.
fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// Every crate directory under `crates/` that has a `tests/` directory.
fn crates_with_tests() -> Vec<PathBuf> {
    let mut dirs: Vec<PathBuf> = fs::read_dir(repo_root().join("crates"))
        .expect("crates/ is readable")
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|dir| dir.join("tests").is_dir())
        .collect();
    dirs.sort();
    dirs
}

/// The `path = "..."` value of every `[[test]]` table in a manifest, as written
/// (relative to the crate directory).
fn test_target_paths(manifest: &str) -> Vec<String> {
    let mut paths = Vec::new();
    let mut in_test_table = false;
    for line in manifest.lines() {
        let line = line.trim();
        if line.starts_with('[') {
            in_test_table = line == "[[test]]";
            continue;
        }
        if !in_test_table {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        if key.trim() == "path" {
            paths.push(value.trim().trim_matches('"').to_string());
        }
    }
    paths
}

/// Whether the manifest switches cargo's test auto-discovery off.
fn autotests_disabled(manifest: &str) -> bool {
    manifest.lines().any(|line| {
        line.split_once('=')
            .is_some_and(|(key, value)| key.trim() == "autotests" && value.trim() == "false")
    })
}

/// The module names a target root declares with `mod name;` (optionally
/// `pub`), in source order. Inline `mod name { ... }` blocks are not file
/// modules and are not returned.
fn declared_modules(root: &str) -> Vec<String> {
    root.lines()
        .filter_map(|line| {
            let line = line.trim();
            let line = line.strip_prefix("pub ").unwrap_or(line);
            let name = line.strip_prefix("mod ")?.strip_suffix(';')?.trim();
            let is_identifier =
                !name.is_empty() && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_');
            is_identifier.then(|| name.to_string())
        })
        .collect()
}

/// The stems of the `.rs` files directly inside `tests_dir` (helper modules in
/// subdirectories are reached through a root's `mod` line, not listed here).
fn top_level_test_files(tests_dir: &Path) -> BTreeSet<String> {
    fs::read_dir(tests_dir)
        .expect("tests/ is readable")
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.is_file() && path.extension().is_some_and(|ext| ext == "rs"))
        .filter_map(|path| Some(path.file_stem()?.to_str()?.to_string()))
        .collect()
}

#[test]
fn declared_modules_reads_file_modules_only() {
    let root = "//! doc\nmod a_file;\npub mod helper;\nmod inline {\n}\n// mod commented;\n";
    assert_eq!(declared_modules(root), ["a_file", "helper"]);
}

#[test]
fn test_target_paths_reads_only_test_tables() {
    let manifest = "[package]\npath = \"no\"\n\n[[test]]\nname = \"all\"\npath = \"tests/all.rs\"\n\n[[bin]]\npath = \"src/main.rs\"\n";
    assert_eq!(test_target_paths(manifest), ["tests/all.rs"]);
}

#[test]
fn every_test_file_is_declared_in_exactly_one_test_target_root() {
    let mut errors = Vec::new();
    let mut checked = 0usize;

    for crate_dir in crates_with_tests() {
        let crate_name = crate_dir
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or_default()
            .to_string();
        let manifest =
            fs::read_to_string(crate_dir.join("Cargo.toml")).expect("the manifest is readable");
        let roots = test_target_paths(&manifest);

        let mut declared_in: BTreeMap<String, Vec<&str>> = BTreeMap::new();
        for root in &roots {
            let Ok(content) = fs::read_to_string(crate_dir.join(root)) else {
                errors.push(format!(
                    "crates/{crate_name}/Cargo.toml names the test target root {root}, which \
                     does not exist"
                ));
                continue;
            };
            for module in declared_modules(&content) {
                declared_in.entry(module).or_default().push(root);
            }
        }

        // With auto-discovery on, cargo builds every `tests/*.rs` as its own
        // binary, so no file can be left out.
        if !autotests_disabled(&manifest) {
            continue;
        }

        let root_stems: BTreeSet<&str> = roots
            .iter()
            .filter_map(|root| root.strip_prefix("tests/")?.strip_suffix(".rs"))
            .collect();
        for stem in top_level_test_files(&crate_dir.join("tests")) {
            if root_stems.contains(stem.as_str()) {
                continue;
            }
            checked += 1;
            match declared_in.get(&stem).map(Vec::as_slice) {
                None | Some([]) => errors.push(format!(
                    "crates/{crate_name}/tests/{stem}.rs is declared as `mod {stem};` in none of \
                     the crate's test target roots {roots:?}, so with `autotests = false` it is \
                     never compiled and its tests never run"
                )),
                Some([_]) => {}
                Some(several) => errors.push(format!(
                    "crates/{crate_name}/tests/{stem}.rs is declared as a module in {} roots \
                     {several:?}, so its tests are compiled and run more than once",
                    several.len()
                )),
            }
        }
    }

    // The floor that keeps this guard from passing vacuously: if the walk found
    // no test files behind `autotests = false`, it checked nothing.
    assert!(
        checked > 50,
        "expected many test files behind `autotests = false`, checked {checked}"
    );
    assert!(
        errors.is_empty(),
        "{} test file(s) would not run exactly once:\n{}",
        errors.len(),
        errors.join("\n")
    );
}
