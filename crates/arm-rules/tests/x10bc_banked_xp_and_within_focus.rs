//! X10bc (docs/vf-audit/design-x10bc-save-format.md;
//! `docs/vf-audit/decisions.md` D73) — GREEN, phase 2 landed: the two fields
//! `AbilityScore::banked_xp` / `ArtScore::banked_xp` (X10b) and
//! `SpellSelection::within_focus` (X10c) are fully wired, not just shipped.
//! The Ability/Art spend loops in `effective/xp.rs` fold `banked_xp` into the
//! charged cost, `validation/scores.rs` emits
//! `banked_xp_at_or_above_next_level`, and `spell_casting_total`
//! (`derived/casting.rs`) returns the focus-adjusted figure when
//! `within_focus` is set. See `tmp/x10bc-handover.md` for the phase 1/phase 2
//! split and the original verbatim RED output.

use arm_rules::export::{LABEL_KEYS, character_markdown};
use arm_rules::ruleset::{LocalizedRuleset, Ruleset, RulesetSources};
use arm_rules::types::*;
use arm_rules::validation::validate;
use std::collections::{BTreeMap, BTreeSet};

/// The shipped core ruleset — same set of files `book_templates.rs::full_ruleset`
/// loads; duplicated because integration test binaries cannot share private
/// helpers (convention already established by `x5b_ability_minimums.rs`,
/// `x7bd_wrong_numbers.rs`).
fn full_ruleset() -> Ruleset {
    Ruleset::from_sources(RulesetSources {
        id: "arm5-core",
        version: "2024.1",
        point_items: include_str!("../../../rules/core/virtues_flaws.json"),
        type_profiles: include_str!("../../../rules/core/character_types.json"),
        abilities: Some(include_str!("../../../rules/core/abilities.json")),
        arts: Some(include_str!("../../../rules/core/arts.json")),
        houses: Some(include_str!("../../../rules/core/houses.json")),
        mythic_types: Some(include_str!(
            "../../../rules/core/mythic_companion_types.json"
        )),
        spells: Some(include_str!("../../../rules/core/spells.json")),
        spell_mastery_abilities: None,
        equipment: Some(include_str!("../../../rules/core/equipment.json")),
        characteristics: Some(include_str!("../../../rules/core/characteristics.json")),
        life_stages: Some(include_str!("../../../rules/core/life_stages.json")),
        childhoods: None,
        aging: Some(include_str!("../../../rules/core/aging.json")),
        parameter_catalogues: Some(include_str!(
            "../../../rules/core/parameter_catalogues.json"
        )),
    })
    .expect("shipped core ruleset loads")
}

fn entity(type_id: &str) -> Entity {
    Entity::new(
        EntityKind::Character,
        Id::new(type_id),
        RulesetRef::new(Id::new("arm5-core"), "2024.1"),
    )
}

// --- X10b: the `banked_xp_at_or_above_next_level` warning -------------------
//
// Table-driven per design § 1 / D73.1 (warning, not error). Each case is its
// own #[test] rather than one function with several `assert!`s, so a failure
// names exactly which boundary broke and a later case's red is never hidden
// behind an earlier `panic!`.

/// Whether validating `entity` (with the given Ability score/banked_xp) reports
/// the new code. A bare string literal, not a `ValidationIssue::CODE_*`
/// constant: the constant does not exist yet (Phase 2 adds it alongside the
/// Fluent keys `every_validation_code_has_a_fluent_key_in_each_locale`,
/// `crates/arm-app/tests/commands.rs`, would otherwise demand immediately).
fn ability_warns_banked_xp_at_or_above_next_level(score: u8, banked_xp: u32) -> bool {
    let ruleset = full_ruleset();
    let table_n1 = ruleset.advancement().xp_for_score(score + 1).unwrap();
    let mut e = entity("grog");
    e.xp_pool = table_n1 + 100; // plenty — this test is not about the XP budget.
    let mut a = AbilityScore::new(Id::new("ability.awareness"), score);
    a.banked_xp = banked_xp;
    e.ability_scores = vec![a];
    validate(&e, &ruleset)
        .issues
        .iter()
        .any(|i| i.code == "banked_xp_at_or_above_next_level")
}

#[test]
fn ability_banked_xp_just_under_next_level_does_not_warn() {
    let ruleset = full_ruleset();
    let score = 2u8;
    let delta = ruleset.advancement().xp_for_score(score + 1).unwrap()
        - ruleset.advancement().xp_for_score(score).unwrap();
    assert!(!ability_warns_banked_xp_at_or_above_next_level(
        score,
        delta - 1
    ));
}

#[test]
fn ability_banked_xp_at_next_level_warns() {
    let ruleset = full_ruleset();
    let score = 2u8;
    let delta = ruleset.advancement().xp_for_score(score + 1).unwrap()
        - ruleset.advancement().xp_for_score(score).unwrap();
    assert!(
        ability_warns_banked_xp_at_or_above_next_level(score, delta),
        "banked_xp exactly at the next level's raw-table delta must warn"
    );
}

#[test]
fn ability_banked_xp_over_next_level_warns() {
    let ruleset = full_ruleset();
    let score = 2u8;
    let delta = ruleset.advancement().xp_for_score(score + 1).unwrap()
        - ruleset.advancement().xp_for_score(score).unwrap();
    assert!(
        ability_warns_banked_xp_at_or_above_next_level(score, delta + 5),
        "banked_xp comfortably over the next level must warn"
    );
}

#[test]
fn ability_banked_xp_at_u32_max_warns_without_panicking() {
    // The point of this case: `u32::MAX` must not panic anywhere on the way to
    // the warning — see `validate_ability_banked_xp`'s own doc comment
    // (`validation/scores.rs`) for the overflow-safety shape this locks in.
    assert!(
        ability_warns_banked_xp_at_or_above_next_level(2, u32::MAX),
        "banked_xp at u32::MAX must warn (and must not panic reaching that assertion)"
    );
}

fn art_warns_banked_xp_at_or_above_next_level(score: u8, banked_xp: u32) -> bool {
    let ruleset = full_ruleset();
    let table_n1 = ruleset.art_advancement().xp_for_score(score + 1).unwrap();
    let mut e = entity("magus");
    e.xp_pool = table_n1 + 100;
    let mut a = ArtScore::new(Id::new("art.creo"), score);
    a.banked_xp = banked_xp;
    e.art_scores = vec![a];
    validate(&e, &ruleset)
        .issues
        .iter()
        .any(|i| i.code == "banked_xp_at_or_above_next_level")
}

#[test]
fn art_banked_xp_just_under_next_level_does_not_warn() {
    let ruleset = full_ruleset();
    let score = 5u8;
    let delta = ruleset.art_advancement().xp_for_score(score + 1).unwrap()
        - ruleset.art_advancement().xp_for_score(score).unwrap();
    assert!(!art_warns_banked_xp_at_or_above_next_level(
        score,
        delta - 1
    ));
}

#[test]
fn art_banked_xp_at_next_level_warns() {
    let ruleset = full_ruleset();
    let score = 5u8;
    let delta = ruleset.art_advancement().xp_for_score(score + 1).unwrap()
        - ruleset.art_advancement().xp_for_score(score).unwrap();
    assert!(
        art_warns_banked_xp_at_or_above_next_level(score, delta),
        "an Art's banked_xp exactly at the next level's delta must warn"
    );
}

#[test]
fn art_banked_xp_over_next_level_warns() {
    let ruleset = full_ruleset();
    let score = 5u8;
    let delta = ruleset.art_advancement().xp_for_score(score + 1).unwrap()
        - ruleset.art_advancement().xp_for_score(score).unwrap();
    assert!(
        art_warns_banked_xp_at_or_above_next_level(score, delta + 5),
        "an Art's banked_xp comfortably over the next level must warn"
    );
}

#[test]
fn art_banked_xp_at_u32_max_warns_without_panicking() {
    // Art-side mirror of `ability_banked_xp_at_u32_max_warns_without_panicking`:
    // `validate_art_banked_xp` (`validation/scores.rs`) duplicates the same
    // checked_add/saturating_sub shape as the Ability-side validator, so a
    // hostile save file's Art `banked_xp: u32::MAX` must warn without panicking
    // here too.
    assert!(
        art_warns_banked_xp_at_or_above_next_level(5, u32::MAX),
        "an Art's banked_xp at u32::MAX must warn (and must not panic reaching that assertion)"
    );
}

// --- The ceiling-score branch (review E-1): the advancement table's own last
// row has no score+1 entry to bank toward, so `validate_ability_banked_xp` /
// `validate_art_banked_xp` fall to their `None` arm (`needed = 0`) rather than
// comparing against a next-level delta. Any `banked_xp > 0` there must still
// warn — there is nowhere left for it to go. The behavior is already
// implemented (commit 6918b0c); this locks it with the dedicated test that
// commit's own review found missing.

#[test]
fn ability_banked_xp_at_the_ceiling_score_warns() {
    let ruleset = full_ruleset();
    let ceiling = ruleset
        .advancement()
        .rows()
        .iter()
        .map(|row| row.score)
        .max()
        .expect("shipped table is non-empty");
    assert!(
        ruleset.advancement().xp_for_score(ceiling + 1).is_none(),
        "the shipped table's own last row must have no score+1 entry"
    );

    let mut e = entity("grog");
    e.xp_pool = u32::MAX;
    let mut a = AbilityScore::new(Id::new("ability.awareness"), ceiling);
    a.banked_xp = 1;
    e.ability_scores = vec![a];
    let issues = validate(&e, &ruleset).issues;
    let issue = issues
        .iter()
        .find(|i| i.code == "banked_xp_at_or_above_next_level")
        .expect("banked_xp above 0 at the ceiling score must still warn");
    assert_eq!(issue.args.get("needed").map(String::as_str), Some("0"));
}

#[test]
fn art_banked_xp_at_the_ceiling_score_warns() {
    let ruleset = full_ruleset();
    let ceiling = ruleset
        .art_advancement()
        .rows()
        .iter()
        .map(|row| row.score)
        .max()
        .expect("shipped table is non-empty");
    assert!(
        ruleset
            .art_advancement()
            .xp_for_score(ceiling + 1)
            .is_none(),
        "the shipped table's own last row must have no score+1 entry"
    );

    let mut e = entity("magus");
    e.xp_pool = u32::MAX;
    let mut a = ArtScore::new(Id::new("art.creo"), ceiling);
    a.banked_xp = 1;
    e.art_scores = vec![a];
    let issues = validate(&e, &ruleset).issues;
    let issue = issues
        .iter()
        .find(|i| i.code == "banked_xp_at_or_above_next_level")
        .expect("an Art's banked_xp above 0 at the ceiling score must still warn");
    assert_eq!(issue.args.get("needed").map(String::as_str), Some("0"));
}

// --- Round trip (X10b + X10c): plain serde mechanics, expected GREEN today --

#[test]
fn ability_score_banked_xp_round_trips() {
    let mut original = AbilityScore::new(Id::new("ability.awareness"), 3);
    original.banked_xp = 7;
    let json = serde_json::to_string(&original).expect("serializes");
    assert!(json.contains("\"banked_xp\":7"), "{json}");
    let back: AbilityScore = serde_json::from_str(&json).expect("deserializes");
    assert_eq!(back, original);
}

#[test]
fn art_score_banked_xp_round_trips() {
    let mut original = ArtScore::new(Id::new("art.creo"), 5);
    original.banked_xp = 4;
    let json = serde_json::to_string(&original).expect("serializes");
    assert!(json.contains("\"banked_xp\":4"), "{json}");
    let back: ArtScore = serde_json::from_str(&json).expect("deserializes");
    assert_eq!(back, original);
}

#[test]
fn spell_selection_within_focus_round_trips() {
    let mut original = SpellSelection::new(Id::new("spell.pilum_of_fire"));
    original.within_focus = true;
    let json = serde_json::to_string(&original).expect("serializes");
    assert!(json.contains("\"within_focus\":true"), "{json}");
    let back: SpellSelection = serde_json::from_str(&json).expect("deserializes");
    assert_eq!(back, original);
}

#[test]
fn a_save_without_the_new_keys_defaults_to_zero_and_false_and_omits_them_on_write() {
    // An old (pre-X10bc) save never wrote `banked_xp`/`within_focus` at all.
    let old_ability = r#"{"ability":"ability.awareness","score":3}"#;
    let ability: AbilityScore = serde_json::from_str(old_ability).expect("deserializes");
    assert_eq!(ability.banked_xp, 0);
    let rewritten = serde_json::to_string(&ability).expect("serializes");
    assert!(
        !rewritten.contains("banked_xp"),
        "a zero banked_xp must not appear on write: {rewritten}"
    );

    let old_art = r#"{"art":"art.creo","score":5}"#;
    let art: ArtScore = serde_json::from_str(old_art).expect("deserializes");
    assert_eq!(art.banked_xp, 0);
    let rewritten = serde_json::to_string(&art).expect("serializes");
    assert!(
        !rewritten.contains("banked_xp"),
        "a zero banked_xp must not appear on write: {rewritten}"
    );

    let old_spell = r#"{"spell":"spell.pilum_of_fire"}"#;
    let spell: SpellSelection = serde_json::from_str(old_spell).expect("deserializes");
    assert!(!spell.within_focus);
    let rewritten = serde_json::to_string(&spell).expect("serializes");
    assert!(
        !rewritten.contains("within_focus"),
        "a false within_focus must not appear on write: {rewritten}"
    );
}

// --- X10c: `spell_casting_total` picks per `within_focus` -------------------

#[test]
fn spell_casting_total_picks_the_within_focus_figure_when_marked() {
    // A minimal magus with a Major Magical Focus and one known Creo Ignem
    // spell: base Casting Total 27 (Cr 12 + Ig 15), within-focus 39
    // (+ min(12, 15)).
    let ruleset = full_ruleset();
    let mut e = entity("magus");
    e.art_scores = vec![
        ArtScore::new(Id::new("art.creo"), 12),
        ArtScore::new(Id::new("art.ignem"), 15),
    ];
    e.selections = vec![Selection::with_params(
        Id::new("virtue.major_magical_focus"),
        BTreeMap::from([("focus".to_string(), Id::new("fire"))]),
    )];
    e.spells = vec![{
        let mut s = SpellSelection::new(Id::new("spell.pilum_of_fire"));
        s.within_focus = true;
        s
    }];

    let total = arm_rules::derived::spell_casting_total(&e.spells[0], &e, &ruleset)
        .expect("spell.pilum_of_fire is in the shipped catalogue");
    assert_eq!(total, 39, "within_focus: true must pick the focused figure");
}

// --- Export: the score cell prints " (Z)" only when banked_xp > 0 ----------

/// Every declared chrome key resolved to itself, plus the catalogue-derived
/// families `export.rs` deliberately excludes from [`LABEL_KEYS`] — same
/// construction as `export_golden.rs::synthetic_labels`, duplicated per the
/// same per-binary-helper convention.
fn synthetic_labels(rs: &LocalizedRuleset) -> BTreeMap<String, String> {
    let mut keys: BTreeSet<String> = LABEL_KEYS.iter().map(|k| k.to_string()).collect();
    for profile in rs.ruleset.profiles() {
        keys.insert(format!("type-{}", profile.id));
    }
    for item in rs.ruleset.items() {
        for category in &item.categories {
            keys.insert(format!("category-{category}"));
        }
        for param in &item.parameters {
            keys.insert(format!("param-label-{}", param.key));
        }
    }
    for ability in rs.ruleset.abilities() {
        if let Some(parameter) = &ability.parameter {
            keys.insert(format!("param-label-{parameter}"));
        }
    }
    for spell in rs.ruleset.spells() {
        for param in &spell.parameters {
            keys.insert(format!("param-label-{}", param.key));
        }
    }
    keys.into_iter().map(|k| (k.clone(), k)).collect()
}

fn localized_ruleset() -> LocalizedRuleset {
    LocalizedRuleset::from_merged(
        full_ruleset(),
        &[
            include_str!("../../../rules/i18n/en/virtues_flaws.json"),
            include_str!("../../../rules/i18n/en/abilities.json"),
            include_str!("../../../rules/i18n/en/arts.json"),
            include_str!("../../../rules/i18n/en/houses.json"),
            include_str!("../../../rules/i18n/en/mythic_companion_types.json"),
            include_str!("../../../rules/i18n/en/spells.json"),
            include_str!("../../../rules/i18n/en/equipment.json"),
            include_str!("../../../rules/i18n/en/aging.json"),
        ],
    )
    .expect("the shipped English rules text loads")
}

#[test]
fn export_shows_banked_xp_in_parentheses_beside_the_art_score() {
    let ruleset = localized_ruleset();
    let mut e = entity("magus");
    let mut a = ArtScore::new(Id::new("art.creo"), 5);
    a.banked_xp = 3;
    e.art_scores = vec![a];
    let labels = synthetic_labels(&ruleset);
    let md = character_markdown(&e, &ruleset, &labels).expect("exports");
    assert!(
        md.contains("| 5 (3) |") || md.contains("5 (3)"),
        "expected the Arts score cell to show the banked XP in parentheses (design § 5):\n{md}"
    );
}

#[test]
fn export_shows_banked_xp_in_parentheses_beside_the_ability_score() {
    let ruleset = localized_ruleset();
    let mut e = entity("magus");
    let mut a = AbilityScore::new(Id::new("ability.awareness"), 2);
    a.banked_xp = 4;
    e.ability_scores = vec![a];
    let labels = synthetic_labels(&ruleset);
    let md = character_markdown(&e, &ruleset, &labels).expect("exports");
    assert!(
        md.contains("2 (4)"),
        "expected the Abilities score cell to show the banked XP in parentheses (design § 5):\n{md}"
    );
}

#[test]
fn export_never_prints_a_casting_total_column_for_spells() {
    // D73.2: no Casting Total column in the Markdown export, now or ever in
    // this slice — X10c covers the marker and the in-app totals only. This is
    // a negative guard against Phase 2 accidentally bundling one in.
    let ruleset = localized_ruleset();
    let mut e = entity("magus");
    e.art_scores = vec![
        ArtScore::new(Id::new("art.creo"), 5),
        ArtScore::new(Id::new("art.ignem"), 5),
    ];
    e.spells = vec![SpellSelection::new(Id::new("spell.pilum_of_fire"))];
    let labels = synthetic_labels(&ruleset);
    let md = character_markdown(&e, &ruleset, &labels).expect("exports");
    assert!(
        !md.to_lowercase().contains("casting total"),
        "the Markdown export must not grow a Casting Total column (D73.2):\n{md}"
    );
}
