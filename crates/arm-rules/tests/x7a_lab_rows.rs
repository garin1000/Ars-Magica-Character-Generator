//! X7a (`docs/vf-audit/corrections.md` § 3.8, `docs/vf-audit/decisions.md` D4,
//! D47, D52, D67) — the Lab Total side of D4's conditional-modifier table,
//! left open after F2 removed the unconditional Casting-Total clauses from
//! Berserk, Ways of the Land, Special Circumstances and both Cyclic Magic
//! entries (`f2_conditional_modifiers.rs`).
//!
//! D4 resolves each of the nine `lab_total_mod` carriers **statically**, from
//! its own passage, for the in-play Lab Total grid
//! (`derived::lab_totals`/`creo_corpus_lab_total`/familiar/masterpiece) —
//! **not** for `effective::spell_level_cap`, which D1 keeps deliberately flat
//! and condition-free for every carrier. Two of the nine keep applying flat
//! even under D4 (Inventive Genius, Creative Block — their condition is the
//! character-generation default) and are not tested here; this file is the
//! five that are NOT the character-generation default, plus the one X2a left
//! half-fixed, plus D47's unrelated Guild Apprentice suppression:
//!
//! | Entry | D4/D52 verdict | Status found here |
//! |---|---|---|
//! | `virtue.adept_laboratory_student` (F-04) | no | was live: unconditional +6 |
//! | `flaw.weak_scholar` | no | was live: unconditional -6 |
//! | `virtue.cyclic_magic_positive` (F-45) | no (unchanged by D52) | was live: unconditional +3 |
//! | `flaw.cyclic_magic_negative` (F-555) | yes, unless cycle is seasonal (D52) | was live: unconditional -3, no cycle parameter |
//! | `virtue.aristotelian_training` | must never apply (D4) | was live: N6/X2a reclassified but left `lab_total_mod: 1` active |
//!
//! X7a phase 2 (`derived.rs::in_play_lab_total_mod`, `life_stage.rs::later_life_rate`,
//! and the `cycle` parameter + Aristotelian Training's effect deletion in
//! `rules/core/virtues_flaws.json`) landed all six items; every test below is
//! GREEN. See `tmp/x7a-verdicts.md` and `tmp/x7a-handover.md` for the fix each
//! test drove.

use arm_rules::derived::{lab_totals, longevity_bonus};
use arm_rules::ruleset::{Ruleset, RulesetSources};
use arm_rules::types::{
    Entity, EntityKind, Id, LongevityRitual, LongevitySource, RulesetRef, Selection,
};
use arm_rules::{LifeStageRules, SpellMarks, spell_level_cap};
use std::collections::BTreeMap;

/// The shipped core ruleset, loaded exactly as `f2_conditional_modifiers.rs`
/// and `book_templates.rs` do.
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

/// A bare entity of the given type, holding exactly `selections` — every
/// Characteristic, Ability and Art left at 0 (the same "isolate the one
/// addend under test" fixture `derived.rs::lab_total_mod_adds_to_every_cell`
/// uses internally), so a Lab Total cell's `total` is exactly whatever the
/// entry under test contributes.
fn entity(type_id: &str, selections: Vec<Selection>) -> Entity {
    let mut e = Entity::new(
        EntityKind::Character,
        Id::new(type_id),
        RulesetRef::new(Id::new("arm5-core"), "2024.1"),
    );
    e.selections = selections;
    e
}

/// The (Creo, Corpus) Lab Total cell — an arbitrary but fixed Technique/Form
/// pick, since every cell shares the same `lab_mod` addend.
fn creo_corpus_total(e: &Entity, rs: &Ruleset) -> i32 {
    lab_totals(e, rs)
        .into_iter()
        .find(|t| t.technique.as_str() == "art.creo" && t.form.as_str() == "art.corpus")
        .expect("(Creo, Corpus) cell present")
        .total
}

// --- D4: `virtue.adept_laboratory_student` (F-04) --------------------------

/// ArMDE:3368-3371: "+6 bonus to Lab Totals **when working from the lab texts
/// of others**." D4: that condition does not hold at character generation (a
/// beginning magus writes his own texts), so the bonus must not appear in the
/// in-play Lab Total grid. Today's data folds it in unconditionally.
#[test]
fn adept_laboratory_student_lab_bonus_does_not_apply_at_creation() {
    let rs = full_ruleset();
    let e = entity(
        "magus",
        vec![Selection::new(Id::new("virtue.adept_laboratory_student"))],
    );
    assert_eq!(
        creo_corpus_total(&e, &rs),
        0,
        "the +6 is conditional on working from ANOTHER magus's lab texts (ArMDE:3368-3371); \
         nothing about character generation makes that true, so the Lab Total grid must not \
         include it (D4)"
    );
}

// --- D4: `flaw.weak_scholar` (mirror of the above) --------------------------

/// ArMDE:7080-7083: "-6 penalty to Lab Totals **when working from the Lab
/// Texts of others**." Same D4 shape as Adept Laboratory Student, opposite
/// sign.
#[test]
fn weak_scholar_lab_penalty_does_not_apply_at_creation() {
    let rs = full_ruleset();
    let e = entity("magus", vec![Selection::new(Id::new("flaw.weak_scholar"))]);
    assert_eq!(
        creo_corpus_total(&e, &rs),
        0,
        "the -6 is conditional on working from ANOTHER magus's Lab Texts (ArMDE:7080-7083); \
         character generation does not make that true, so it must not appear in the grid (D4)"
    );
}

// --- D4/D52: `virtue.cyclic_magic_positive` (F-45), unchanged by D52 -------

/// ArMDE:3635-3638: the Lab Total bonus applies "**if** the positive part of
/// the cycle covers the whole season" — a fact character generation cannot
/// know (D4: "creation fixes no season"). D52 re-examined this row for the
/// Flaw and left the Virtue's "no" standing. Today's data folds the +3 in
/// unconditionally.
#[test]
fn cyclic_magic_positive_lab_bonus_never_applies_at_creation() {
    let rs = full_ruleset();
    let e = entity(
        "magus",
        vec![Selection::new(Id::new("virtue.cyclic_magic_positive"))],
    );
    assert_eq!(
        creo_corpus_total(&e, &rs),
        0,
        "the Lab bonus is conditional on which part of an unspecified cycle the character is \
         in; creation fixes no season, so the grid must never include it (D4, confirmed \
         unchanged by D52)"
    );
}

/// D4's preamble: this defect "propagates into every figure computed from the
/// grid" — not just the displayed Technique×Form totals. The Creo/Corpus Lab
/// Total that feeds the Longevity Ritual hint is one of the named examples.
#[test]
fn cyclic_magic_positive_bonus_does_not_inflate_the_longevity_hint() {
    let rs = full_ruleset();
    let mut e = entity(
        "magus",
        vec![Selection::new(Id::new("virtue.cyclic_magic_positive"))],
    );
    e.longevity_ritual = Some(LongevityRitual {
        source: LongevitySource::SelfMade,
        bonus: None,
        focus: String::new(),
    });
    let hint = longevity_bonus(&e, &rs)
        .expect("a self-made ritual is present")
        .hint
        .expect("self-made ritual carries a live hint");
    assert_eq!(
        hint.lab_total, 0,
        "the same unconditional +3 the Lab-Total grid must not show also must not inflate the \
         Creo Corpus Lab Total the Longevity Ritual hint reads (D4)"
    );
}

// --- D52/F-555: `flaw.cyclic_magic_negative` --------------------------------

/// ArMDE:5893-5896: "The penalty applies to Lab Totals **even if** the
/// negative period does not cover the whole of the season" — the opposite
/// wording from the Virtue. D52: on any cycle shorter than a season (the
/// book's own solar/lunar examples), every season contains negative time, so
/// the -3 always applies; only a **seasonal** cycle makes the two behave
/// alike, in which case the penalty is exactly as uncertain as the Virtue's
/// bonus and must not apply either. The cycle type is a D9 choice, carried by
/// `flaw.cyclic_magic_negative`'s `cycle` selection parameter
/// (`rules/core/virtues_flaws.json`, values `cycle.solar`/`cycle.lunar`/
/// `cycle.seasonal`, matching the enumerated-parameter naming convention used
/// by every other such parameter in the catalogue, e.g. `bloodline.magic_human`).
#[test]
fn cyclic_magic_negative_lab_penalty_is_cycle_gated() {
    let rs = full_ruleset();

    let default_cycle = entity(
        "magus",
        vec![Selection::new(Id::new("flaw.cyclic_magic_negative"))],
    );
    assert_eq!(
        creo_corpus_total(&default_cycle, &rs),
        -3,
        "a solar/lunar (non-seasonal) cycle makes the -3 certain regardless of which unspecified \
         season creation lands in (D52) — already correct by default today"
    );

    let mut params = BTreeMap::new();
    params.insert("cycle".to_string(), Id::new("cycle.seasonal"));
    let seasonal_cycle = entity(
        "magus",
        vec![Selection::with_params(
            Id::new("flaw.cyclic_magic_negative"),
            params,
        )],
    );
    assert_eq!(
        creo_corpus_total(&seasonal_cycle, &rs),
        0,
        "a seasonal cycle makes the penalty exactly as uncertain as the Virtue's bonus \
         (D52) — creation fixes no season, so it must not apply — but nothing reads the \
         `cycle` parameter yet"
    );
}

// --- D4 (residual): `virtue.aristotelian_training` --------------------------

/// ArMDE:3440-3443: the +1 Lab Total is conditional on "attempting to
/// synthesize the New Aristotle with Magic Theory" (Art and Academe, p.11) —
/// content this app does not model at all. D4's explicit verdict: "the
/// bonus can never legitimately fire in any Lab Total" — not the D4 grid,
/// and not D1's otherwise-generous flat `spell_level_cap` fold either, since
/// the condition is unsatisfiable in principle, not merely undecided per
/// character. N6/X2a (`5b53259`) reclassified the entry to `uncomputed_rule`
/// but left `effects: [{ "type": "lab_total_mod", "amount": 1 }]` in place,
/// so both consumers still add the +1 today. This is IN SCOPE for X7a
/// despite the brief's assumption that N6 was complete — verification found
/// otherwise; see `tmp/x7a-verdicts.md`.
#[test]
fn aristotelian_trainings_lab_bonus_must_never_apply_anywhere() {
    let rs = full_ruleset();
    let e = entity(
        "magus",
        vec![Selection::new(Id::new("virtue.aristotelian_training"))],
    );
    assert_eq!(
        creo_corpus_total(&e, &rs),
        0,
        "D4: the condition can never be satisfied by anything this app models, so the bonus \
         must never fire in the in-play Lab Total grid — N6/X2a left the effect active"
    );
    assert_eq!(
        spell_level_cap(
            &e,
            &rs,
            &Id::new("art.creo"),
            &Id::new("art.ignem"),
            &[],
            false,
            SpellMarks::default()
        ),
        // Int(0) + MagicTheory(0) + Creo(0) + Ignem(0) + 3 (ArMDE:2465) + 0 (no lab_mod)
        3,
        "D1's normally-generous flat fold must ALSO exclude this carrier — D4 says the \
         condition is unsatisfiable in principle, which is why D4 flags this entry as \
         distinct from the other eight — but the cap still reads the +1 today"
    );
}

// --- D47: Guild Apprentice suppresses Wealthy/Poor's later-life XP rate ----

/// ArMDE:4043: a Guild Apprentice is "not able to benefit from either the
/// Poor Flaw or the Wealthy Virtue … until he moves to the journeyman
/// stage" — D47: keyed on holding `virtue.guild_apprentice`, since D41 makes
/// the guild stages mutually exclusive Social Status Virtues. Nothing
/// suppresses `later_life_xp_rate` today, so a companion with both is shown
/// Wealthy's 20 XP/year that the book denies him; the correct figure is the
/// ruleset's base rate (15, `rules/core/life_stages.json`).
#[test]
fn guild_apprentice_suppresses_wealthys_later_life_rate() {
    let rs = full_ruleset();
    let rules: &LifeStageRules = rs.life_stages().expect("shipped life-stage rules loaded");
    let e = entity(
        "companion",
        vec![
            Selection::new(Id::new("virtue.guild_apprentice")),
            Selection::new(Id::new("virtue.wealthy")),
        ],
    );
    assert_eq!(
        rules.later_life_rate(&e, &rs),
        15,
        "Guild Apprentice denies Wealthy's benefit until the journeyman stage (ArMDE:4043); a \
         companion holding both is shown the ruleset's base rate, not Wealthy's 20 (D47)"
    );
}
