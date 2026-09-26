//! C0b (`docs/vf-audit/design-c0-parameter-model.md` § 1a/§ 8/§ 10):
//! `Selection::params` moves from `BTreeMap<String, Id>` to
//! `BTreeMap<String, SelectionParamValue>` (`Single(Id) | Multi(BTreeSet<Id>)`),
//! producing only `Single` today. Wire-compatible: a `Single` serializes/
//! deserializes exactly as today's bare JSON string, so no `SCHEMA_VERSION`
//! bump is owed here (that lands with C5a's array *fold*).
//!
//! The golden round-trip test at the bottom is the regression net for the
//! *refactor* half of this slice: every real save under `examples/` and
//! `crates/arm-rules/tests/fixtures/book_templates/` must still round-trip
//! byte-identically through `load_entity_migrating` after the type change.

use arm_rules::DEFAULT_SAGA_YEAR;
use arm_rules::migration::load_entity_migrating;
use arm_rules::types::{Id, Selection, SelectionParamValue};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

/// C0b's own promise: this slice moves a value TYPE, not the save format, so
/// the bump stays with C5a's array fold.
#[test]
fn c0b_does_not_bump_schema_version() {
    assert_eq!(arm_rules::migration::SCHEMA_VERSION, 17);
}

#[test]
fn single_value_round_trips_as_a_bare_json_string() {
    let value: SelectionParamValue = serde_json::from_str("\"ability.brawl\"").unwrap();
    assert_eq!(value, SelectionParamValue::Single(Id::new("ability.brawl")));
    assert_eq!(serde_json::to_string(&value).unwrap(), "\"ability.brawl\"");
}

/// `Multi` is not yet produced by this engine (nothing constructs one), but the
/// untagged shape already parses a JSON array — this is the "hypothetical"
/// value the design note names, proved here so a future producer (C5a) is not
/// the first thing to exercise deserialization into this arm.
#[test]
fn a_hypothetical_multi_value_round_trips_as_a_json_array() {
    let json = r#"["ability.awareness","ability.brawl"]"#;
    let value: SelectionParamValue = serde_json::from_str(json).unwrap();
    assert_eq!(
        value,
        SelectionParamValue::Multi(BTreeSet::from([
            Id::new("ability.awareness"),
            Id::new("ability.brawl"),
        ]))
    );
    assert_eq!(serde_json::to_string(&value).unwrap(), json);
}

/// The wire-compatibility proof: a map of `Single` values serializes to
/// EXACTLY the shape `BTreeMap<String, Id>` produced before this slice — a
/// bare string per key, not a tagged `{"Single": …}` object.
#[test]
fn a_map_of_single_values_serializes_byte_identically_to_the_old_bare_string_shape() {
    let selection = Selection::with_params(
        Id::new("virtue.puissant_ability"),
        BTreeMap::from([("ability".to_string(), Id::new("ability.awareness"))]),
    );
    let json = serde_json::to_string(&selection).unwrap();
    assert_eq!(
        json,
        r#"{"ref":"virtue.puissant_ability","params":{"ability":"ability.awareness"}}"#
    );
}

/// An existing save's hand-written bare-string parameter value still resolves
/// as `Single` once `Selection::params` moves to the new value type.
#[test]
fn a_hand_written_bare_string_param_still_deserializes_as_single() {
    let json = r#"{"ref":"flaw.corrupted_abilities","params":{"targets":"ability.brawl"}}"#;
    let selection: Selection = serde_json::from_str(json).unwrap();
    assert_eq!(
        selection.params.get("targets"),
        Some(&SelectionParamValue::Single(Id::new("ability.brawl")))
    );
}

/// The not-yet-producible counterpart: a hand-crafted array value on a
/// `Selection` round-trips as `Multi`, byte-identically.
#[test]
fn a_hypothetical_array_valued_param_round_trips_as_multi() {
    let json = r#"{"ref":"flaw.corrupted_abilities","params":{"targets":["ability.awareness","ability.brawl"]}}"#;
    let selection: Selection = serde_json::from_str(json).unwrap();
    assert_eq!(
        selection.params.get("targets"),
        Some(&SelectionParamValue::Multi(BTreeSet::from([
            Id::new("ability.awareness"),
            Id::new("ability.brawl"),
        ])))
    );
    assert_eq!(serde_json::to_string(&selection).unwrap(), json);
}

fn all_json_files(dir: &Path) -> Vec<PathBuf> {
    let mut out: Vec<PathBuf> = fs::read_dir(dir)
        .unwrap_or_else(|e| panic!("{} readable: {e}", dir.display()))
        .map(|entry| entry.unwrap().path())
        .filter(|p| p.extension().is_some_and(|ext| ext == "json"))
        .collect();
    out.sort();
    out
}

/// Golden round-trip: every real save under `examples/` and
/// `tests/fixtures/book_templates/` loads (through the actual app load path,
/// `load_entity_migrating`) and re-serializes to a STABLE fixed point — a
/// second load/save cycle over the first output produces byte-identical
/// output. This is the property `migration.rs`'s own
/// `a_migrated_save_is_byte_stable_across_a_save_load_save_cycle` pins for one
/// hand-written fixture; here it runs over every real file in the repo, so the
/// `Selection::params` value-type refactor cannot silently break save/load
/// fidelity for a single one of them.
#[test]
fn every_example_and_book_template_save_round_trips_byte_identically() {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut files = all_json_files(&manifest.join("../../examples"));
    files.extend(all_json_files(
        &manifest.join("tests/fixtures/book_templates"),
    ));
    assert!(
        files.len() >= 27,
        "expected at least 27 example/fixture saves, found {}",
        files.len()
    );

    for path in files {
        let original = fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("{} readable: {e}", path.display()));

        let mut first = load_entity_migrating(&original, DEFAULT_SAGA_YEAR)
            .unwrap_or_else(|e| panic!("{} loads: {e}", path.display()))
            .entity;
        first.normalize();
        let first_bytes = serde_json::to_string_pretty(&first).unwrap();

        let mut second = load_entity_migrating(&first_bytes, DEFAULT_SAGA_YEAR)
            .unwrap_or_else(|e| panic!("{} reloads: {e}", path.display()))
            .entity;
        second.normalize();
        let second_bytes = serde_json::to_string_pretty(&second).unwrap();

        assert_eq!(
            first_bytes,
            second_bytes,
            "{} did not round-trip byte-identically",
            path.display()
        );
    }
}
