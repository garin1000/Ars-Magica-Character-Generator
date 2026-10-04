//! I3 (try-out finding 15): the per-source breakdown of the Arts' and
//! Abilities' effective delta rides the `EffectiveScores` DTO additively, as
//! `art_bonus_sources` and `ability_bonus_sources`, next to the existing summed
//! `art_bonuses` / `ability_bonuses`. The JSON field names are the contract the
//! TS `EffectiveScores` type (`ui/src/lib/types.ts`) reads, so they are pinned
//! on the serialized value, not only on the Rust struct.
//!
//! Duplicated `repo_root`/`rules_dir` helpers: integration test binaries cannot
//! share private items across files.

use std::collections::BTreeMap;
use std::path::PathBuf;

use arm_app::effective_dto::effective_scores_loaded;
use arm_app::ruleset_io::load_ruleset_from_dir;
use arm_rules::{ArtScore, Entity, EntityKind, Id, RulesetRef, Selection};
use serde_json::json;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn rules_dir() -> PathBuf {
    repo_root().join("rules")
}

/// A magus with Puissant (Ignem) (+3, ArMDE:4820) and Second Sight (grants
/// Second Sight 1, ArMDE:4890) plus Puissant (Second Sight) (+2, ArMDE:4816).
fn magus() -> Entity {
    let mut e = Entity::new(
        EntityKind::Character,
        Id::new("magus"),
        RulesetRef::new(Id::new("arm5-core"), "2024.1"),
    );
    e.selections = vec![
        Selection::with_params(
            Id::new("virtue.puissant_art"),
            BTreeMap::from([("art".into(), Id::new("art.ignem"))]),
        ),
        Selection::new(Id::new("virtue.second_sight")),
        Selection::with_params(
            Id::new("virtue.puissant_ability"),
            BTreeMap::from([("ability".into(), Id::new("ability.second_sight"))]),
        ),
    ];
    e.art_scores = vec![ArtScore::new(Id::new("art.ignem"), 5)];
    e
}

#[test]
fn effective_scores_serialize_art_and_ability_bonus_sources() {
    let ruleset = load_ruleset_from_dir(&rules_dir(), "en").unwrap().ruleset;
    let dto = serde_json::to_value(effective_scores_loaded(&magus(), &ruleset)).unwrap();

    let ignem = dto["art_bonus_sources"]
        .as_array()
        .expect("art_bonus_sources is an array")
        .iter()
        .find(|entry| entry["art"] == "art.ignem")
        .expect("Ignem has a breakdown");
    assert_eq!(
        ignem["sources"],
        json!([{ "source": "virtue.puissant_art", "amount": 3 }])
    );

    let second_sight = dto["ability_bonus_sources"]
        .as_array()
        .expect("ability_bonus_sources is an array")
        .iter()
        .find(|entry| entry["ability"] == "ability.second_sight")
        .expect("Second Sight has a breakdown");
    assert!(
        second_sight.get("parameter").is_none(),
        "an unparameterized instance omits `parameter` (never null), like AbilityBonus"
    );
    let mut sources = second_sight["sources"]
        .as_array()
        .expect("sources is an array")
        .clone();
    sources.sort_by(|a, b| a["source"].as_str().cmp(&b["source"].as_str()));
    assert_eq!(
        sources,
        vec![
            json!({ "source": "virtue.puissant_ability", "amount": 2 }),
            json!({ "source": "virtue.second_sight", "amount": 1 }),
        ]
    );
}
