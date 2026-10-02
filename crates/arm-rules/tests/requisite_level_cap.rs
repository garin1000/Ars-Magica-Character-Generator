//! Phase 1 (RED tests only) of folding spell requisites into the creation-time
//! per-spell level cap — the follow-up to `b5ee82a` (Casting Totals fold spell
//! requisites), recorded as §5a of `tmp/requisites-handover.md`. See
//! `tmp/req-cap-handover.md` for this slice's own notes (book-template
//! conflict found, open questions).
//!
//! `effective/spell.rs::spell_level_cap` computes Technique + Form + Int +
//! Magic Theory + 3 (ArMDE:2465) from the spell's PRIMARY Technique/Form only
//! — it never reads `Spell::requisites`, even though :2465's own second
//! sentence says "If the spell has requisites (see page 311), they apply to
//! this total as well." `derived/casting.rs::fold_requisite` already
//! implements the identical lesser-of-primary-and-requisite fold for the
//! Casting Total (ArMDE:12309-12313); this slice folds the same rule into the
//! cap instead, reached only through `validation::validate`'s
//! `spell_level_exceeds_cap` issue (`validation/magus.rs::validate_spell_level_cap`),
//! since the public `spell_level_cap`/`spell_level_caps` functions take no
//! spell identity (only a bare Technique/Form/range triple — the same "no
//! specific spell at this grid cell" shape `derived/casting.rs::casting_totals`
//! has, which is exactly why `spell_casting_total` exists as a separate,
//! spell-aware function beside the grid). Every test below pins the
//! correct, rules-derived `cap` value validation should report and is RED
//! against today's engine unless its own comment says otherwise.
//!
//! Source: ArMDE:2465 (`If the spell has requisites ... they apply to this
//! total as well`), :12309-12311 (the base min-fold and several-requisites
//! rules, identical wording context to the Casting Total), :3737 (Elemental
//! Magic — "you use the primary Form to calculate totals, even if the
//! requisite is lower"; generic "totals" wording, not casting-specific, so it
//! governs this Lab-Total-shaped cap too, per :2465's own closing sentence
//! that the cap "is the appropriate Lab Total"), :4403 (Major Magical Focus —
//! "If a spell has requisites, the lowest applicable score may be one of the
//! requisites, rather than one of the primary Arts").
//!
//! D81.5 (`docs/vf-audit/decisions.md`) answers Phase 1's QUESTIONS 2-3: the
//! Magical Focus doubling is in scope for the cap (not just the Casting
//! Total), the player marks a spell `within_focus` the same way X10c already
//! does for casting, and the picker's "add within focus" action adds a spell
//! already so marked when only the focus cap admits its level.

use arm_rules::ValidationIssue;
use arm_rules::ruleset::{Ruleset, RulesetSources};
use arm_rules::types::*;
use arm_rules::validation::validate;
use std::collections::BTreeMap;

/// The shipped core ruleset — same set of files `book_templates.rs::full_ruleset`
/// and `requisite_casting_total.rs::full_ruleset` load; duplicated because
/// integration test binaries cannot share private helpers.
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

/// A ruleset identical to [`full_ruleset`] except its spell catalogue is replaced
/// by a single synthetic spell, for tests that need a shape (two requisites of
/// the same Art class; an off-catalogue level) no shipped spell has — same
/// synthetic-spell technique `requisite_casting_total.rs` uses for the
/// identical reason on the Casting Total side.
fn ruleset_with_only_spell(spell_json: &str) -> Ruleset {
    let spells_file = format!(r#"{{"spells": [{spell_json}]}}"#);
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
        spells: Some(&spells_file),
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
    .expect("single-synthetic-spell ruleset loads")
}

/// A bare magus: `type_id: "magus"` alone makes `is_hermetically_trained` true
/// (`effective/hermetic_training.rs`'s own test confirms this), which is what
/// gates `validate_spell_level_cap` in `validate_spells`. No Int/Magic Theory
/// set (both default to 0), so every hand-computed cap below reads
/// `Technique + Form + 0 + 0 + 3`.
fn entity() -> Entity {
    Entity::new(
        EntityKind::Character,
        Id::new("magus"),
        RulesetRef::new(Id::new("arm5-core"), "2024.1"),
    )
}

fn art(id: &str, score: u8) -> ArtScore {
    ArtScore::new(Id::new(id), score)
}

fn deficient(flaw: &str, param: &str, art_id: &str) -> Selection {
    Selection::with_params(
        Id::new(flaw),
        BTreeMap::from([(param.to_string(), Id::new(art_id))]),
    )
}

/// Runs `validate`, and returns the `cap` arg of the `spell_level_exceeds_cap`
/// issue for `spell_id`, if any such issue was reported. `None` means no issue
/// — either because the level is legal, or (today) because the engine never
/// folds requisites into the cap at all.
fn cap_issue_arg(entity: &Entity, ruleset: &Ruleset, spell_id: &str) -> Option<String> {
    validate(entity, ruleset)
        .issues
        .into_iter()
        .find(|issue| {
            issue.code == ValidationIssue::CODE_SPELL_LEVEL_EXCEEDS_CAP
                && issue.context.as_ref() == Some(&Id::new(spell_id))
        })
        .and_then(|issue| issue.args.get("cap").cloned())
}

// --- Base min rule: ArMDE:12309, folded the same way as the Casting Total --

#[test]
fn technique_requisite_lower_than_the_technique_lowers_the_cap() {
    // spell.obliteration_of_the_metallic_barrier: Pe/Te, level 20, requisite
    // art.rego (Technique-class). Pe 10, Te 10, Re 3 (lower than Pe) -> fold
    // Pe to Re's 3: cap = 3 + 10 + 0 (Int) + 0 (Magic Theory) + 3 = 16. Level
    // 20 > 16, so the engine must report `spell_level_exceeds_cap` with
    // cap "16". Today's engine ignores the requisite, reaching cap
    // 10 + 10 + 3 = 23 (20 <= 23), so it reports no issue at all.
    let ruleset = full_ruleset();
    let mut e = entity();
    e.art_scores = vec![
        art("art.perdo", 10),
        art("art.terram", 10),
        art("art.rego", 3),
    ];
    e.spells = vec![SpellSelection::new(Id::new(
        "spell.obliteration_of_the_metallic_barrier",
    ))];
    assert_eq!(
        cap_issue_arg(&e, &ruleset, "spell.obliteration_of_the_metallic_barrier"),
        Some("16".to_string()),
        "ArMDE:2465/:12309: a lower Technique requisite must replace the main Technique score in the cap"
    );
}

#[test]
fn form_requisite_lower_than_the_form_lowers_the_cap() {
    // spell.rain_of_stones: Mu/Au, level 20, requisite art.terram (Form-class,
    // no Elemental Magic held here so no exception applies). Mu 10, Au 10,
    // Te 2 (lower than Au) -> fold Au to Te's 2: cap = 10 + 2 + 0 + 0 + 3 = 15.
    // Level 20 > 15, so the engine must report the issue with cap "15". Today
    // it reaches 10 + 10 + 3 = 23 (20 <= 23), no issue.
    let ruleset = full_ruleset();
    let mut e = entity();
    e.art_scores = vec![
        art("art.muto", 10),
        art("art.auram", 10),
        art("art.terram", 2),
    ];
    e.spells = vec![SpellSelection::new(Id::new("spell.rain_of_stones"))];
    assert_eq!(
        cap_issue_arg(&e, &ruleset, "spell.rain_of_stones"),
        Some("15".to_string()),
        "ArMDE:2465/:12309: a lower Form requisite must replace the main Form score in the cap"
    );
}

#[test]
fn requisite_higher_than_both_primary_arts_leaves_the_cap_unchanged() {
    // spell.the_crystal_dart: Mu/Te, level 10, requisite art.rego
    // (Technique-class). Mu 5, Te 8, Re 20 (higher) -> the lesser-of rule
    // picks Mu's own 5, so the cap is unaffected either way:
    // 5 + 8 + 0 + 0 + 3 = 16, and 10 <= 16. NOT currently red — this is a
    // forward confirmation that folding must leave an unbound requisite
    // alone, not a bug this file catches.
    let ruleset = full_ruleset();
    let mut e = entity();
    e.art_scores = vec![
        art("art.muto", 5),
        art("art.terram", 8),
        art("art.rego", 20),
    ];
    e.spells = vec![SpellSelection::new(Id::new("spell.the_crystal_dart"))];
    assert_eq!(
        cap_issue_arg(&e, &ruleset, "spell.the_crystal_dart"),
        None,
        "ArMDE:12309: a higher requisite must never lower the cap, so no issue is reported"
    );
}

// --- Several requisites: ArMDE:12311 ----------------------------------------

#[test]
fn two_form_requisites_in_the_same_category_fold_to_the_lowest_of_the_group() {
    // "if several requisites apply to the same primary Art ... your effective
    // score is the lowest of the group." No shipped spell carries two
    // same-class requisites, so this uses the same synthetic one-spell
    // ruleset `requisite_casting_total.rs` uses: Cr/Ig primary, requisites
    // Aquam(7) and Auram(4), both Form-class, level 20. Cr 10 (untouched,
    // no Technique requisite), Ig 10, Aq 7, Au 4 -> Form folds to the
    // group's lowest (4): cap = 10 + 4 + 0 + 0 + 3 = 17. Level 20 > 17, so
    // the engine must report the issue with cap "17". Today it reaches
    // 10 + 10 + 3 = 23 (20 <= 23), no issue.
    let ruleset = ruleset_with_only_spell(
        r#"{ "id": "spell.test_two_form_requisites", "technique": "art.creo",
             "form": "art.ignem", "level": 20,
             "requisites": ["art.aquam", "art.auram"] }"#,
    );
    let mut e = entity();
    e.art_scores = vec![
        art("art.creo", 10),
        art("art.ignem", 10),
        art("art.aquam", 7),
        art("art.auram", 4),
    ];
    e.spells = vec![SpellSelection::new(Id::new(
        "spell.test_two_form_requisites",
    ))];
    assert_eq!(
        cap_issue_arg(&e, &ruleset, "spell.test_two_form_requisites"),
        Some("17".to_string()),
        "ArMDE:12311: two requisites in the same class must fold to the lowest of the group"
    );
}

#[test]
fn requisites_for_both_technique_and_form_fold_independently() {
    // "Sometimes a spell has a requisite for both its Technique and Form. You
    // must use the lowest in each case." spell.fog_of_confusion: Mu/Au,
    // level 45, requisites [art.imaginem (Form), art.rego (Technique)]. Mu 10,
    // Au 10, Im 3 (lower, Form side), Re 2 (lower, Technique side) -> folded
    // Technique 2 + folded Form 3 + 3 = 8. Level 45 is already above TODAY's
    // unfolded cap (10 + 10 + 3 = 23, cap arg "23"), so the issue already
    // fires either way — the assertion pins the CORRECT folded cap value
    // "8", which today's `cap` arg does not match (it reports "23").
    let ruleset = full_ruleset();
    let mut e = entity();
    e.art_scores = vec![
        art("art.muto", 10),
        art("art.auram", 10),
        art("art.imaginem", 3),
        art("art.rego", 2),
    ];
    e.spells = vec![SpellSelection::new(Id::new("spell.fog_of_confusion"))];
    assert_eq!(
        cap_issue_arg(&e, &ruleset, "spell.fog_of_confusion"),
        Some("8".to_string()),
        "ArMDE:12311: a Technique requisite and a Form requisite must fold independently"
    );
}

// --- Deficient Art as a requisite: ArMDE:12311 (closing sentence) ----------

#[test]
fn deficient_requisite_still_lowers_the_cap_even_when_it_does_not_numerically_bind() {
    // "any Deficiencies you have with an Art apply when you use that Art as a
    // requisite." spell.obliteration_of_the_metallic_barrier: Pe/Te, level 20,
    // requisite art.rego. Pe 10, Te 10, Re 15 (HIGHER than Pe, so the
    // min-rule alone would leave the cap unaffected: 10 + 10 + 3 = 23) — but
    // Rego is Deficient (flaw.deficient_technique), and using a Deficient Art
    // as a requisite still halves the whole cap (the cap is a Lab Total,
    // ArMDE:2465, and ArMDE:5911/:5915's halving reaches a requisite Art, not
    // only a primary one): halve(23) = 11 (floor). Level 20 > 11, so the
    // engine must report the issue with cap "11". Today `spell_level_cap`'s
    // deficiency check only reads the spell's own primary Technique/Form,
    // never its requisites, so it is not halved: 20 <= 23, no issue.
    let ruleset = full_ruleset();
    let mut e = entity();
    e.art_scores = vec![
        art("art.perdo", 10),
        art("art.terram", 10),
        art("art.rego", 15),
    ];
    e.selections = vec![deficient(
        "flaw.deficient_technique",
        "technique",
        "art.rego",
    )];
    e.spells = vec![SpellSelection::new(Id::new(
        "spell.obliteration_of_the_metallic_barrier",
    ))];
    assert_eq!(
        cap_issue_arg(&e, &ruleset, "spell.obliteration_of_the_metallic_barrier"),
        Some("11".to_string()),
        "ArMDE:12311: a Deficient requisite Art halves the cap even when it does not numerically bind"
    );
}

// --- Elemental Magic exception: ArMDE:3737 ----------------------------------

#[test]
fn elemental_magic_uses_the_primary_form_for_the_cap_even_when_an_elemental_requisite_is_lower() {
    // "if a spell with one of these Forms as its primary Form has another
    // element as a requisite, you use the primary Form to calculate totals,
    // even if the requisite is lower" — generic "totals" wording (not
    // casting-specific), and ArMDE:2465 calls this cap itself a Lab Total, so
    // the exception governs it too. spell.rain_of_stones: Mu/Au, level 20,
    // requisite art.terram — both Auram and Terram are elemental Forms
    // (virtue.elemental_magic's own `forms` list). Mu 9, Au 10 (bought),
    // Te 2 (bought). Elemental Magic's own XP-space redistribution
    // (`effective/art.rs::elemental_form_bonus`) lifts Terram's EFFECTIVE
    // score to 7 here (half of Auram's 275 XP for score 10, rounded up,
    // added to Terram's own 15 XP for score 2, re-resolved against the
    // table) — still lower than Auram's 10, so the exception is the only
    // thing standing between the correct cap (9 + 10 + 0 + 0 + 3 = 22, no
    // issue, level 20 <= 22) and what a PLAIN fold (no exception) would
    // wrongly compute: 9 + min(10, 7) + 3 = 19, where level 20 > 19 WOULD
    // wrongly report an issue. NOT currently red: today's engine folds no
    // requisite at all (Elemental Magic or not), reaching the same 9 + 10 +
    // 3 = 22 the exception requires, for an unrelated reason. This is a
    // forward guard: Phase 2 must special-case Elemental Magic so this stays
    // issue-free once the plain fold (which alone would wrongly flag it) is
    // implemented.
    let ruleset = full_ruleset();
    let mut e = entity();
    e.art_scores = vec![
        art("art.muto", 9),
        art("art.auram", 10),
        art("art.terram", 2),
    ];
    e.selections = vec![Selection::new(Id::new("virtue.elemental_magic"))];
    e.spells = vec![SpellSelection::new(Id::new("spell.rain_of_stones"))];
    assert_eq!(
        cap_issue_arg(&e, &ruleset, "spell.rain_of_stones"),
        None,
        "ArMDE:3737: Elemental Magic must use the primary elemental Form for the cap, never a lower elemental requisite"
    );
}

// --- Validation consequence: a spell at or below a REAL folded cap is legal -

#[test]
fn a_spell_whose_level_remains_at_or_below_the_folded_cap_is_not_reported() {
    // spell.rope_of_bronze: Mu/Herbam, level 15, requisite art.terram
    // (Form-class). Mu 10 (untouched), Herbam 10, Terram 6 (lower) -> folded
    // cap = 10 + 6 + 0 + 0 + 3 = 19, genuinely lower than the unfolded
    // 10 + 10 + 3 = 23 — but the known level (15) still sits at or below
    // BOTH readings, so no issue is reported either before or after folding.
    // Already green today; pinned so Phase 2's fold cannot regress a
    // legitimately-learnable spell into a false positive.
    let ruleset = full_ruleset();
    let mut e = entity();
    e.art_scores = vec![
        art("art.muto", 10),
        art("art.herbam", 10),
        art("art.terram", 6),
    ];
    e.spells = vec![SpellSelection::new(Id::new("spell.rope_of_bronze"))];
    assert_eq!(
        cap_issue_arg(&e, &ruleset, "spell.rope_of_bronze"),
        None,
        "ArMDE:2465/:12309: a spell at or below the folded cap must not be reported"
    );
}

// --- Magical Focus doubling (ArMDE:4403, D81.5) -----------------------------

#[test]
fn within_focus_marker_doubles_the_lowest_folded_score_and_can_admit_an_otherwise_illegal_level() {
    // A synthetic Cr/Au spell at level 40 with a Rego requisite (same shape
    // as spell.wings_of_the_soaring_wind, Mercere's Cr(Re)Au level-30 spell,
    // but pushed higher so even the FOCUSED cap still falls short — this
    // isolates "does the engine read `sel.within_focus` and double the
    // correct, already-folded score" from "does the fold happen at all",
    // which the tests above already cover).
    //
    // Cr 25, Au 20, Re 5 (Technique-class requisite, lower than Cr) ->
    // folded Technique = min(25, 5) = 5. cap0 (no focus) = 5 + 20 + 0 + 0 + 3
    // = 28. Marked within_focus: focus doubles the lower of the two FOLDED
    // scores, min(5, 20) = 5, added before any halving (ArMDE:4403, "the
    // lowest applicable score may be one of the requisites") ->
    // cap_focus = 28 + 5 = 33. Level 40 > 33, so the issue still fires, but
    // with the correct focused cap "33" — not "28" (the fold alone, ignoring
    // the marker) and not today's engine's unfolded, unfocused "48"
    // (Cr25 + Au20 + 3, no fold, no focus logic at all: 40 <= 48, no issue —
    // today reports None here, not Some("33")).
    let ruleset = ruleset_with_only_spell(
        r#"{ "id": "spell.test_focus_requisite", "technique": "art.creo",
             "form": "art.auram", "level": 40,
             "requisites": ["art.rego"] }"#,
    );
    let mut e = entity();
    e.art_scores = vec![
        art("art.creo", 25),
        art("art.auram", 20),
        art("art.rego", 5),
    ];
    let mut sel = SpellSelection::new(Id::new("spell.test_focus_requisite"));
    sel.within_focus = true;
    e.spells = vec![sel];
    assert_eq!(
        cap_issue_arg(&e, &ruleset, "spell.test_focus_requisite"),
        Some("33".to_string()),
        "ArMDE:4403: a spell marked within_focus must double the lowest FOLDED score \
         (including a requisite that won the fold) before the cap is compared to the level"
    );
}
