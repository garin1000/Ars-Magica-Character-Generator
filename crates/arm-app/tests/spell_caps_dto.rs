//! X11b/D81.5: the per-spell creation-time level cap (requisites folded, plus
//! the Magical Focus doubling) surfaced through the `EffectiveScores` DTO as
//! `spell_caps: Vec<SpellCap>` — one row per catalogue spell, so the picker can
//! grey a requisite-bearing spell by its own folded cap rather than the
//! Te/Fo-keyed grid's (`spell_level_caps`) unfolded figure. See
//! `tmp/cap-handover.md` for the design note and the UI hand-off this field is
//! built for.
//!
//! Duplicated `repo_root`/`rules_dir`/`RULESET_ID`/`RULESET_VERSION` helpers:
//! integration test binaries cannot share private items across files (same
//! reason `crates/arm-rules/tests/requisite_level_cap.rs` duplicates
//! `full_ruleset`).

use std::collections::BTreeMap;
use std::path::PathBuf;

use arm_app::effective_dto::effective_scores_loaded;
use arm_app::ruleset_io::load_ruleset_from_dir;
use arm_rules::{ArtScore, Entity, EntityKind, Id, RulesetRef, Selection};

const RULESET_ID: &str = "arm5-core";
const RULESET_VERSION: &str = "2024.1";

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn rules_dir() -> PathBuf {
    repo_root().join("rules")
}

fn magus() -> Entity {
    Entity::new(
        EntityKind::Character,
        Id::new("magus"),
        RulesetRef::new(Id::new(RULESET_ID), RULESET_VERSION),
    )
}

/// Against the **real shipped ruleset**: `spell.obliteration_of_the_metallic_barrier`
/// is Pe/Te level 20 with a Technique-class requisite, `art.rego` (same spell
/// `requisite_level_cap.rs`'s own first test uses). Pe 10, Te 10, Re 3 folds
/// Perdo down to 3 (ArMDE:2465/:12309): plain cap = 3 + 10 + 0 (Int) + 0
/// (Magic Theory) + 3 = 16 — not the unfolded 10 + 10 + 3 = 23 the Te/Fo grid
/// (`spell_level_caps`) still reports for this cell, since the grid has no
/// specific spell to fold against. With no Magical Focus held,
/// `within_focus_cap` must be absent entirely (the picker has no "add within
/// focus" action to offer). Holding a Major Magical Focus then doubles the
/// lowest FOLDED score (3, from the requisite) — ArMDE:4403: "the lowest
/// applicable score may be one of the requisites" — giving
/// `within_focus_cap = Some(16 + 3) = Some(19)`, while the plain `cap` stays
/// 16 regardless of the Focus.
#[test]
fn effective_scores_surface_a_per_spell_level_cap_that_folds_requisites() {
    let ruleset = load_ruleset_from_dir(&rules_dir(), "en").unwrap().ruleset;
    let mut m = magus();
    m.art_scores = vec![
        ArtScore::new(Id::new("art.perdo"), 10),
        ArtScore::new(Id::new("art.terram"), 10),
        ArtScore::new(Id::new("art.rego"), 3),
    ];

    let caps = effective_scores_loaded(&m, &ruleset).spell_caps;
    let row = caps
        .iter()
        .find(|c| c.spell == Id::new("spell.obliteration_of_the_metallic_barrier"))
        .expect("every catalogue spell gets a row, including requisite-bearing ones");
    assert_eq!(
        row.cap, 16,
        "ArMDE:2465/:12309: the Rego requisite (3) must fold against Perdo (10), not \
         report the unfolded 10 + 10 + 3 = 23 the Te/Fo grid still carries"
    );
    assert_eq!(
        row.within_focus_cap, None,
        "no Magical Focus held, so the picker has no within-focus figure to offer"
    );

    m.selections = vec![Selection::with_params(
        Id::new("virtue.major_magical_focus"),
        BTreeMap::from([("focus".to_string(), Id::new("stone"))]),
    )];
    let caps = effective_scores_loaded(&m, &ruleset).spell_caps;
    let row = caps
        .iter()
        .find(|c| c.spell == Id::new("spell.obliteration_of_the_metallic_barrier"))
        .expect("the row is still present once a Focus is held");
    assert_eq!(row.cap, 16, "holding a Focus must not change the plain cap");
    assert_eq!(
        row.within_focus_cap,
        Some(19),
        "ArMDE:4403: the focus doubling must use the lowest FOLDED score (the requisite's \
         3), giving 16 + 3 = 19 — not the unfolded Perdo (10), which would wrongly reach 26"
    );
}

/// R3 (after-deadline answer 3, amends D1): Potent Magic's Lab Total bonus
/// counts toward a spell's cap only for a spell marked within the Potent
/// field, so the DTO surfaces it as its own `within_potent_field_cap` (and,
/// with a Magical Focus too, `within_focus_and_potent_field_cap`) beside the
/// plain `cap`, which no longer includes it. Same spell and Arts as above:
/// plain cap 16; Major Potent Magic +6 (ArMDE:4748) -> 22; with a Major
/// Magical Focus as well, 16 + 3 (doubling the folded requisite) + 6 = 25.
#[test]
fn effective_scores_surface_the_within_potent_field_caps() {
    let ruleset = load_ruleset_from_dir(&rules_dir(), "en").unwrap().ruleset;
    let mut m = magus();
    m.art_scores = vec![
        ArtScore::new(Id::new("art.perdo"), 10),
        ArtScore::new(Id::new("art.terram"), 10),
        ArtScore::new(Id::new("art.rego"), 3),
    ];
    let potent = Selection::with_params(
        Id::new("virtue.potent_magic_major"),
        BTreeMap::from([("field".to_string(), Id::new("metal"))]),
    );
    let focus = Selection::with_params(
        Id::new("virtue.major_magical_focus"),
        BTreeMap::from([("focus".to_string(), Id::new("stone"))]),
    );
    let spell = Id::new("spell.obliteration_of_the_metallic_barrier");

    m.selections = vec![potent.clone()];
    let caps = effective_scores_loaded(&m, &ruleset).spell_caps;
    let row = caps.iter().find(|c| c.spell == spell).expect("row present");
    assert_eq!(row.cap, 16, "the plain cap excludes Potent Magic");
    assert_eq!(row.within_potent_field_cap, Some(22));
    assert_eq!(row.within_focus_and_potent_field_cap, None, "no Focus held");

    m.selections = vec![focus, potent];
    let caps = effective_scores_loaded(&m, &ruleset).spell_caps;
    let row = caps.iter().find(|c| c.spell == spell).expect("row present");
    assert_eq!(row.cap, 16);
    assert_eq!(row.within_focus_cap, Some(19));
    assert_eq!(row.within_potent_field_cap, Some(22));
    assert_eq!(row.within_focus_and_potent_field_cap, Some(25));
}

/// N1 (try-out 2026-10-04): the Te/Fo grid (`spell_level_caps`) carries the
/// same marked figures for the Spells tab's group-header tooltip. Pe 10 /
/// Te 10, no requisites folded at grid level: plain 23; a Major Magical
/// Focus doubles the lower Art (+10) -> 33; Major Potent Magic +6 -> 29;
/// both -> 39. Absent while the Virtue is not held.
#[test]
fn effective_scores_surface_the_marked_caps_on_the_te_fo_grid() {
    let ruleset = load_ruleset_from_dir(&rules_dir(), "en").unwrap().ruleset;
    let mut m = magus();
    m.art_scores = vec![
        ArtScore::new(Id::new("art.perdo"), 10),
        ArtScore::new(Id::new("art.terram"), 10),
    ];
    let grid_row = |m: &Entity| {
        effective_scores_loaded(m, &ruleset)
            .spell_level_caps
            .into_iter()
            .find(|c| {
                c.technique == Id::new("art.perdo")
                    && c.form == Id::new("art.terram")
                    && !c.range_beyond_touch
            })
            .expect("the grid has a Perdo/Terram Touch row")
    };

    let row = grid_row(&m);
    assert_eq!(row.cap, 23);
    assert_eq!(row.within_focus_cap, None, "no Magical Focus held");
    assert_eq!(row.within_potent_field_cap, None, "no Potent Magic held");
    assert_eq!(row.within_focus_and_potent_field_cap, None);

    m.selections = vec![
        Selection::with_params(
            Id::new("virtue.major_magical_focus"),
            BTreeMap::from([("focus".to_string(), Id::new("stone"))]),
        ),
        Selection::with_params(
            Id::new("virtue.potent_magic_major"),
            BTreeMap::from([("field".to_string(), Id::new("metal"))]),
        ),
    ];
    let row = grid_row(&m);
    assert_eq!(
        row.cap, 23,
        "holding the Virtues must not change the plain cap"
    );
    assert_eq!(row.within_focus_cap, Some(33));
    assert_eq!(row.within_potent_field_cap, Some(29));
    assert_eq!(row.within_focus_and_potent_field_cap, Some(39));
}
