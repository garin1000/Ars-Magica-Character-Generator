//! D81.8/D81.15 (`docs/vf-audit/decisions.md`): Incompatible Arts records its
//! two Technique+Form combinations (ArMDE:6290-6292) as four selects per copy
//! — `technique_1`, `form_1`, `technique_2`, `form_2` — canonicalized as an
//! unordered pair of pairs (`PointItem::unordered_param_groups`) for
//! duplicate detection, flagged `unusable` in both grids
//! (`CastingTotal`/`LabTotal`), and a creation-time ERROR
//! (`CODE_SPELL_USES_INCOMPATIBLE_ARTS`) on any known spell that draws on a
//! barred combination, directly or through a requisite. See
//! `tmp/incompat-handover.md` for the Phase 1 design this implements. Every
//! test below runs against the REAL shipped ruleset.
//!
//! Verbatim (ArMDE:6290-6292): "For some reason you are completely unable to
//! use two combinations of Techniques and Forms. For example, you may be
//! unable to use Intellego Herbam and Intellego Animal. You may not use these
//! Arts together even if one or both are requisites. This Flaw may be taken
//! repeatedly with different combinations, but may not be combined with a
//! Deficiency (see page 125)."

use arm_rules::derived::{casting_totals, derived_totals, lab_totals};
use arm_rules::ruleset::{Ruleset, RulesetSources};
use arm_rules::types::*;
use arm_rules::validation::{IssueSeverity, ValidationIssue, validate};
use std::collections::BTreeMap;

/// The shipped core ruleset — same files `d81_devil_child.rs`/
/// `requisite_casting_total.rs::full_ruleset` load; duplicated per those
/// files' own precedent (integration test binaries cannot share private
/// helpers). No spell catalogue: tests 1-4 below need none.
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
        spells: None,
        spell_mastery_abilities: None,
        equipment: None,
        characteristics: Some(include_str!("../../../rules/core/characteristics.json")),
        life_stages: Some(include_str!("../../../rules/core/life_stages.json")),
        childhoods: None,
        aging: None,
        parameter_catalogues: Some(include_str!(
            "../../../rules/core/parameter_catalogues.json"
        )),
    })
    .expect("shipped core ruleset loads")
}

/// [`full_ruleset`] with its spell catalogue replaced by a single synthetic
/// spell (same `ruleset_with_only_spell` pattern `requisite_casting_total.rs`
/// uses) — needed only by the requisite-spell test, which needs a spell shape
/// (primary Arts elsewhere, one requisite landing on a flagged combination) no
/// shipped spell is guaranteed to have.
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
        equipment: None,
        characteristics: Some(include_str!("../../../rules/core/characteristics.json")),
        life_stages: Some(include_str!("../../../rules/core/life_stages.json")),
        childhoods: None,
        aging: None,
        parameter_catalogues: Some(include_str!(
            "../../../rules/core/parameter_catalogues.json"
        )),
    })
    .expect("single-synthetic-spell ruleset loads")
}

fn magus() -> Entity {
    Entity::new(
        EntityKind::Character,
        Id::new("magus"),
        RulesetRef::new(Id::new("arm5-core"), "2024.1"),
    )
}

/// One copy of Incompatible Arts naming Technique1+Form1 and Technique2+Form2.
fn incompatible_arts(t1: &str, f1: &str, t2: &str, f2: &str) -> Selection {
    Selection::with_params(
        Id::new("flaw.incompatible_arts"),
        BTreeMap::from([
            ("technique_1".to_string(), Id::new(t1)),
            ("form_1".to_string(), Id::new(f1)),
            ("technique_2".to_string(), Id::new(t2)),
            ("form_2".to_string(), Id::new(f2)),
        ]),
    )
}

fn missing_param_keys(issues: &[ValidationIssue], item: &str) -> Vec<String> {
    issues
        .iter()
        .filter(|i| {
            i.code == ValidationIssue::CODE_MISSING_PARAM
                && i.args.get("item").map(String::as_str) == Some(item)
        })
        .filter_map(|i| i.args.get("key").cloned())
        .collect()
}

fn has_duplicate_selection(issues: &[ValidationIssue], item: &str) -> bool {
    issues.iter().any(|i| {
        i.code == ValidationIssue::CODE_DUPLICATE_SELECTION
            && i.args.get("item").map(String::as_str) == Some(item)
    })
}

// --- 1. The four parameters are required ------------------------------------

#[test]
fn the_four_combination_parameters_are_required() {
    // A bare copy with no params at all. Today `flaw.incompatible_arts`
    // declares zero parameters, so this reports NONE of the four — red until
    // Phase 2 adds `technique_1`/`form_1`/`technique_2`/`form_2` to
    // `rules/core/virtues_flaws.json`.
    let ruleset = full_ruleset();
    let mut e = magus();
    e.selections = vec![Selection::new(Id::new("flaw.incompatible_arts"))];

    let result = validate(&e, &ruleset);
    let mut missing = missing_param_keys(&result.issues, "flaw.incompatible_arts");
    missing.sort();

    assert_eq!(
        missing,
        vec![
            "form_1".to_string(),
            "form_2".to_string(),
            "technique_1".to_string(),
            "technique_2".to_string(),
        ],
        "D81.8: all four combination slots must be required parameters; got {missing:?}"
    );
}

// --- 2. Two copies with different pairs are legal ---------------------------

#[test]
fn two_copies_with_different_pairs_are_legal() {
    // Combo A: Creo+Animal / Rego+Herbam. Combo B: Muto+Aquam / Perdo+Ignem —
    // no combination in common, so both copies must be legal together. This
    // already holds today (different `params` maps never collide in
    // `validate_duplicate_selections`'s key) — a guard against Phase 2
    // regressing it while it adds the swapped-pair canonicalization below.
    let ruleset = full_ruleset();
    let mut e = magus();
    e.selections = vec![
        incompatible_arts("art.creo", "art.animal", "art.rego", "art.herbam"),
        incompatible_arts("art.muto", "art.aquam", "art.perdo", "art.ignem"),
    ];

    let result = validate(&e, &ruleset);
    assert!(
        !has_duplicate_selection(&result.issues, "flaw.incompatible_arts"),
        "two copies naming disjoint combinations must not collide: {:?}",
        result.issues
    );
}

// --- 3. The same pair, swapped either way, is refused -----------------------

#[test]
fn the_same_pair_with_combinations_swapped_is_refused() {
    // Copy A: Creo+Animal paired with Rego+Herbam. Copy B restates the SAME
    // two combinations with WHICH ONE sits in slot 1 vs slot 2 flipped — the
    // ruling's own wording: "A copy that repeats another's pair, in either
    // order, is refused" (D81.8). The rulebook names an unordered PAIR of
    // combinations, not two ordered slots, so this is the same copy restated
    // and must be refused — red today, since the raw
    // `(technique_1, form_1, technique_2, form_2)` tuple differs between the
    // two selections and so collides in no key.
    let ruleset = full_ruleset();
    let mut e = magus();
    e.selections = vec![
        incompatible_arts("art.creo", "art.animal", "art.rego", "art.herbam"),
        incompatible_arts("art.rego", "art.herbam", "art.creo", "art.animal"),
    ];

    let result = validate(&e, &ruleset);
    assert!(
        has_duplicate_selection(&result.issues, "flaw.incompatible_arts"),
        "D81.8: restating the same two combinations with slots swapped must be \
         refused as a repeat, not accepted as a second distinct copy: {:?}",
        result.issues
    );
}

// --- 4. The flagged cells are unusable in both grids ------------------------

#[test]
fn the_flagged_combinations_are_unusable_in_the_casting_and_lab_grids() {
    // The rulebook's own example pair (ArMDE:6291): Intellego+Herbam and
    // Intellego+Animal.
    let ruleset = full_ruleset();
    let mut e = magus();
    e.selections = vec![incompatible_arts(
        "art.intellego",
        "art.herbam",
        "art.intellego",
        "art.animal",
    )];

    let casting = casting_totals(&e, &ruleset);
    let lab = lab_totals(&e, &ruleset);

    let casting_cell = |t: &str, f: &str| {
        casting
            .iter()
            .find(|c| c.technique == Id::new(t) && c.form == Id::new(f))
            .unwrap_or_else(|| panic!("({t}, {f}) must be a grid cell"))
    };
    let lab_cell = |t: &str, f: &str| {
        lab.iter()
            .find(|c| c.technique == Id::new(t) && c.form == Id::new(f))
            .unwrap_or_else(|| panic!("({t}, {f}) must be a grid cell"))
    };

    for (t, f) in [
        ("art.intellego", "art.herbam"),
        ("art.intellego", "art.animal"),
    ] {
        assert!(
            casting_cell(t, f).unusable,
            "D81.8: the Casting Total cell ({t}, {f}) must be unusable"
        );
        assert!(
            lab_cell(t, f).unusable,
            "D81.8: the Lab Total cell ({t}, {f}) must be unusable"
        );
    }

    // A neighbouring cell sharing the Technique with one flagged pair, but not
    // itself one of the two DECLARED combinations, must stay usable — the Flaw
    // forbids two SPECIFIC combinations, not every cell touching Intellego.
    assert!(
        !casting_cell("art.intellego", "art.corpus").unusable,
        "an undeclared combination must not be caught by the flag"
    );
    assert!(
        !lab_cell("art.intellego", "art.corpus").unusable,
        "an undeclared combination must not be caught by the flag"
    );
}

// --- 5. A requisite spell touching a flagged pair is flagged ----------------

#[test]
fn a_spell_using_a_flagged_combination_only_through_a_requisite_is_flagged() {
    // Primary Rego+Animal (NOT itself a declared combination) with an
    // Intellego Technique requisite. Folding the requisite Technique onto the
    // primary Form reconstructs Intellego+Animal — one of the two declared
    // combinations below — so ArMDE:6292's "even if one or both are
    // requisites" must flag this spell even though its own primary pair is
    // clean.
    let ruleset = ruleset_with_only_spell(
        r#"{ "id": "spell.test_touches_incompatible_pair", "technique": "art.rego",
             "form": "art.animal", "level": 10, "requisites": ["art.intellego"] }"#,
    );
    let mut e = magus();
    e.selections = vec![incompatible_arts(
        "art.intellego",
        "art.herbam",
        "art.intellego",
        "art.animal",
    )];
    e.spells = vec![SpellSelection::new(Id::new(
        "spell.test_touches_incompatible_pair",
    ))];

    let totals = derived_totals(&e, &ruleset);
    assert_eq!(
        totals.spell_casting_unusable,
        vec![true],
        "D81.8: a spell reaching a flagged combination through a requisite \
         must be flagged unusable even though its own primary Technique+Form \
         is not itself one of the two declared combinations"
    );
}

// --- 6. A spell using a barred combination is a creation-time ERROR (D81.15) -

fn has_spell_uses_incompatible_arts_error(issues: &[ValidationIssue], spell: &str) -> bool {
    issues.iter().any(|i| {
        i.code == ValidationIssue::CODE_SPELL_USES_INCOMPATIBLE_ARTS
            && i.severity == IssueSeverity::Error
            && i.args.get("spell").map(String::as_str) == Some(spell)
    })
}

#[test]
fn a_spell_directly_using_a_barred_combination_is_a_creation_time_error() {
    // Primary Intellego+Animal is itself one of the two declared combinations.
    let ruleset = ruleset_with_only_spell(
        r#"{ "id": "spell.test_direct_hit", "technique": "art.intellego",
             "form": "art.animal", "level": 10 }"#,
    );
    let mut e = magus();
    e.selections = vec![incompatible_arts(
        "art.intellego",
        "art.herbam",
        "art.intellego",
        "art.animal",
    )];
    e.spells = vec![SpellSelection::new(Id::new("spell.test_direct_hit"))];

    let result = validate(&e, &ruleset);
    assert!(
        has_spell_uses_incompatible_arts_error(&result.issues, "spell.test_direct_hit"),
        "D81.15: a spell whose own primary Technique+Form is a barred \
         combination must be a creation-time error: {:?}",
        result.issues
    );
}

#[test]
fn a_spell_using_a_barred_combination_only_through_a_requisite_is_a_creation_time_error() {
    // Same fixture as the derived_totals test above (test 5): primary
    // Rego+Animal, clean on its own, touches Intellego+Animal only via its
    // Intellego Technique requisite.
    let ruleset = ruleset_with_only_spell(
        r#"{ "id": "spell.test_requisite_hit", "technique": "art.rego",
             "form": "art.animal", "level": 10, "requisites": ["art.intellego"] }"#,
    );
    let mut e = magus();
    e.selections = vec![incompatible_arts(
        "art.intellego",
        "art.herbam",
        "art.intellego",
        "art.animal",
    )];
    e.spells = vec![SpellSelection::new(Id::new("spell.test_requisite_hit"))];

    let result = validate(&e, &ruleset);
    assert!(
        has_spell_uses_incompatible_arts_error(&result.issues, "spell.test_requisite_hit"),
        "D81.15: 'even if one or both are requisites' (ArMDE:6292) — a spell \
         reaching a barred combination only through a requisite must still be \
         a creation-time error, not merely a quiet grid marker: {:?}",
        result.issues
    );
}

#[test]
fn a_spell_reaching_a_barred_combination_through_a_form_class_requisite_is_an_error() {
    // Same shape as the Technique-requisite test above, but the requisite Art
    // (art.herbam) is FORM-class rather than Technique-class —
    // `effective/spell.rs::spell_touches_barred_combination`'s
    // `Some(ArtType::Form) => forms.push(req)` arm, which the Technique-requisite
    // test above never reaches (its own requisite, art.intellego, only ever
    // exercises the sibling Technique arm). Primary Rego+Animal is clean on its
    // own; the barred pair (Rego, Herbam) is reached only via the Form
    // requisite.
    let ruleset = ruleset_with_only_spell(
        r#"{ "id": "spell.test_form_requisite_hit", "technique": "art.rego",
             "form": "art.animal", "level": 10, "requisites": ["art.herbam"] }"#,
    );
    let mut e = magus();
    e.selections = vec![incompatible_arts(
        "art.rego",
        "art.herbam",
        "art.perdo",
        "art.terram",
    )];
    e.spells = vec![SpellSelection::new(Id::new(
        "spell.test_form_requisite_hit",
    ))];

    let result = validate(&e, &ruleset);
    assert!(
        has_spell_uses_incompatible_arts_error(&result.issues, "spell.test_form_requisite_hit"),
        "a spell reaching a barred combination through a FORM-class requisite \
         (distinct from its own primary Form) must still be a creation-time \
         error: {:?}",
        result.issues
    );
}

#[test]
fn a_spell_using_no_barred_combination_is_not_flagged() {
    let ruleset = ruleset_with_only_spell(
        r#"{ "id": "spell.test_clean", "technique": "art.creo",
             "form": "art.corpus", "level": 10 }"#,
    );
    let mut e = magus();
    e.selections = vec![incompatible_arts(
        "art.intellego",
        "art.herbam",
        "art.intellego",
        "art.animal",
    )];
    e.spells = vec![SpellSelection::new(Id::new("spell.test_clean"))];

    let result = validate(&e, &ruleset);
    assert!(
        !has_spell_uses_incompatible_arts_error(&result.issues, "spell.test_clean"),
        "a spell touching neither declared combination must not be flagged: {:?}",
        result.issues
    );
}

// --- 7. An incomplete copy never counts as a duplicate (finding 1) ---------
//
// tmp/review-incompat.json #1: an OLD SAVE may legally hold 2+ copies of this
// Flaw from before this schema change, each with all four new params absent
// (the old schema had none). `duplicate_key()` used to skip a group's
// role-map entry whenever a key was missing ("missing_param already
// reported"), so two such copies collapsed to the IDENTICAL empty
// `DuplicateKey` and `max_per_target`'s new default of 1 raised a spurious
// `duplicate_selection` on top of the expected `missing_param`s.

#[test]
fn two_param_less_copies_report_missing_param_but_not_duplicate_selection() {
    let ruleset = full_ruleset();
    let mut e = magus();
    e.selections = vec![
        Selection::new(Id::new("flaw.incompatible_arts")),
        Selection::new(Id::new("flaw.incompatible_arts")),
    ];

    let result = validate(&e, &ruleset);
    let mut missing = missing_param_keys(&result.issues, "flaw.incompatible_arts");
    missing.sort();
    assert_eq!(
        missing,
        vec![
            "form_1".to_string(),
            "form_1".to_string(),
            "form_2".to_string(),
            "form_2".to_string(),
            "technique_1".to_string(),
            "technique_1".to_string(),
            "technique_2".to_string(),
            "technique_2".to_string(),
        ],
        "both copies must each still report all four missing params; got {missing:?}"
    );
    assert!(
        !has_duplicate_selection(&result.issues, "flaw.incompatible_arts"),
        "two param-less copies must never collide as a duplicate — an \
         incomplete group is not comparable to another incomplete group: {:?}",
        result.issues
    );
}

/// Items WITHOUT `unordered_param_groups` must behave exactly as before the
/// fix above: the early-return for an incomplete GROUP must never touch an
/// ungrouped item's own duplicate detection. A real shipped ungrouped,
/// parameterized Virtue naming the same target twice must still collide.
#[test]
fn an_ungrouped_item_still_collides_on_an_identical_repeat() {
    let ruleset = full_ruleset();
    let mut e = magus();
    e.selections = vec![
        Selection::with_params(
            Id::new("virtue.puissant_ability"),
            BTreeMap::from([("ability".to_string(), Id::new("ability.awareness"))]),
        ),
        Selection::with_params(
            Id::new("virtue.puissant_ability"),
            BTreeMap::from([("ability".to_string(), Id::new("ability.awareness"))]),
        ),
    ];

    let result = validate(&e, &ruleset);
    assert!(
        has_duplicate_selection(&result.issues, "virtue.puissant_ability"),
        "an ungrouped item naming the same target twice must still collide: {:?}",
        result.issues
    );
}

// --- 8. One copy repeating its own pair in both groups is refused (D81.16) -

fn has_param_groups_not_distinct(issues: &[ValidationIssue], item: &str) -> bool {
    issues.iter().any(|i| {
        i.code == ValidationIssue::CODE_PARAM_GROUPS_NOT_DISTINCT
            && i.args.get("item").map(String::as_str) == Some(item)
    })
}

#[test]
fn one_copy_naming_the_same_combination_in_both_groups_is_an_error() {
    // D81.16: "One Incompatible Arts copy naming the same combination twice is
    // an ERROR: its 'two combinations' (ArMDE:6292) must differ." Both groups
    // resolve to the SAME (Technique, Form) pair.
    let ruleset = full_ruleset();
    let mut e = magus();
    e.selections = vec![incompatible_arts(
        "art.intellego",
        "art.herbam",
        "art.intellego",
        "art.herbam",
    )];

    let result = validate(&e, &ruleset);
    assert!(
        has_param_groups_not_distinct(&result.issues, "flaw.incompatible_arts"),
        "D81.16: a copy whose two groups name the identical combination must \
         be refused: {:?}",
        result.issues
    );
}

#[test]
fn one_copy_naming_two_different_combinations_is_not_flagged() {
    let ruleset = full_ruleset();
    let mut e = magus();
    e.selections = vec![incompatible_arts(
        "art.intellego",
        "art.herbam",
        "art.intellego",
        "art.animal",
    )];

    let result = validate(&e, &ruleset);
    assert!(
        !has_param_groups_not_distinct(&result.issues, "flaw.incompatible_arts"),
        "two genuinely different combinations must not be flagged: {:?}",
        result.issues
    );
}

#[test]
fn an_incomplete_copy_is_not_flagged_as_repeating_itself() {
    // A copy missing params for both groups must not ALSO draw
    // `param_groups_not_distinct` on top of `missing_param` — an incomplete
    // group is not comparable to anything, including another incomplete group
    // within the SAME selection.
    let ruleset = full_ruleset();
    let mut e = magus();
    e.selections = vec![Selection::new(Id::new("flaw.incompatible_arts"))];

    let result = validate(&e, &ruleset);
    assert!(
        !has_param_groups_not_distinct(&result.issues, "flaw.incompatible_arts"),
        "an incomplete copy must not be flagged for repeating itself: {:?}",
        result.issues
    );
}
