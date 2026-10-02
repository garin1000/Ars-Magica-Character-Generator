//! X5b (`tmp/x5b-verdicts.md`; `docs/vf-audit/corrections.md` § 3.6;
//! `docs/vf-audit/decisions.md` D16, D38, D56, D58, D61, D67, D68) — the
//! Ability-minimum / cross-Virtue prerequisite family carved out of X5a's
//! scope: F-30, F-56, F-145, F-169, F-172, F-183, F-223, F-252, F-261,
//! F-322, F-324 (11 entries; see `tmp/x5-verdicts.md`'s "Proposed split").
//!
//! Phase 1 only — verdicts and failing tests, no data or engine changes.
//! Every entry below is confirmed unmodified in the shipped
//! `rules/core/virtues_flaws.json` (re-checked against the working tree, not
//! assumed from the docs) and every referenced `ability.*`/`virtue.*` id is
//! confirmed present in the catalogue. See `tmp/x5b-verdicts.md` for the
//! full per-entry citation table and reasoning, and `tmp/x5b-handover.md`
//! for exactly what Phase 2 must change.
//!
//! All eleven gates are `Prereq::AbilityMin`/`Prereq::Has`/`Prereq::Any`/
//! `Prereq::Nor` — every variant already exists — so, unlike X5a, this file
//! needs zero new `Prereq` machinery. That confirms the previous X5 agent's
//! call: this family is pure data.
//!
//! A handful of the book's stated clauses are genuinely NOT expressible
//! (a per-entry age formula, a `faculty`-parameter-bound Ability, "must be
//! able to take Academic Abilities" as a category-authorization gate) and
//! stay `description` text under D3 — those are asserted as locale-text
//! checks against `rules/i18n/<lang>/virtues_flaws.json`, never through
//! `validate()`, exactly as the brief distinguishes.

use arm_rules::ruleset::{Ruleset, RulesetSources};
use arm_rules::types::*;
use arm_rules::validation::{ValidationResult, validate};
use serde_json::Value;

const SHIPPED_HOUSES: &str = include_str!("../../../rules/core/houses.json");
const EN_VF: &str = include_str!("../../../rules/i18n/en/virtues_flaws.json");
const DE_VF: &str = include_str!("../../../rules/i18n/de/virtues_flaws.json");

fn load_ruleset() -> Ruleset {
    Ruleset::from_sources(RulesetSources {
        id: "arm5-core",
        version: "2024.1",
        point_items: include_str!("../../../rules/core/virtues_flaws.json"),
        type_profiles: include_str!("../../../rules/core/character_types.json"),
        abilities: Some(include_str!("../../../rules/core/abilities.json")),
        houses: Some(SHIPPED_HOUSES),
        characteristics: Some(include_str!("../../../rules/core/characteristics.json")),
        life_stages: Some(include_str!("../../../rules/core/life_stages.json")),
        parameter_catalogues: Some(include_str!(
            "../../../rules/core/parameter_catalogues.json"
        )),
        ..RulesetSources::default()
    })
    .unwrap()
}

/// Same shape as `x5_prerequisites.rs::entity` / `x7bd_wrong_numbers.rs::entity`
/// — duplicated since integration test binaries cannot share private helpers.
fn entity(type_id: &str, selections: Vec<Selection>) -> Entity {
    let mut e = Entity::new(
        EntityKind::Character,
        Id::new(type_id),
        RulesetRef::new(Id::new("arm5-core"), "2024.1"),
    );
    e.selections = selections;
    e
}

fn sel(id: &str) -> Selection {
    Selection::new(Id::new(id))
}

fn sel_with_param(id: &str, key: &str, value: &str) -> Selection {
    let mut s = Selection::new(Id::new(id));
    s.params
        .insert(key.to_string(), SelectionParamValue::Single(Id::new(value)));
    s
}

fn score(ability: &str, value: u8) -> AbilityScore {
    AbilityScore::new(Id::new(ability), value)
}

fn issue_codes(result: &ValidationResult) -> Vec<&str> {
    result.issues.iter().map(|i| i.code.as_str()).collect()
}

fn has_prereq_not_met(result: &ValidationResult) -> bool {
    issue_codes(result).contains(&"prereq_not_met")
}

/// `description` if present, else `summary`, else empty — the same "displayed
/// rules text" convention `uncomputed_clauses.rs` and D5 use, since that is
/// what a player actually reads. Panics if the id itself is missing from the
/// locale file (a real fixture bug, distinct from a missing description).
fn displayed_text(locale_json: &str, id: &str) -> String {
    let v: Value = serde_json::from_str(locale_json).expect("locale JSON parses");
    let entry = v
        .get(id)
        .unwrap_or_else(|| panic!("{id} is present in the locale file"));
    entry
        .get("description")
        .or_else(|| entry.get("summary"))
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string()
}

// ---------------------------------------------------------------------------
// F-30 — `virtue.cathedral_school_master`, ArMDE:3549-3554.
// "He is at least (30 – Intelligence) years old and must have scores of 5 in
// Latin and Artes Liberales, and a Teaching score of at least 3."
//
// Two of three minima are unparameterized and exactly expressible:
// `ability.artes_liberales` >= 5, `ability.teaching` >= 3. The third, Latin,
// is an instance of the parameterized `ability.dead_language` with no
// instance field on `Prereq::AbilityMin` (B6) — but
// `validation/prereq.rs::evaluate_prereq` already matches `AbilityMin` by
// ability id alone, ignoring `AbilityScore::parameter` entirely, which is
// EXACTLY the "id-level proxy" pattern `RULES.md`'s "Hermetic minimum
// Abilities — Parma Magica 1, Magic Theory 1, Latin 1 (M6/6b4)" section
// documents as a PERMANENT, deliberate, already-shipped design choice for
// this exact ability (a rulebook-stated instance of a parameterized,
// free-text Ability with no catalogue). Encoding
// `ability_min ability.dead_language 5` therefore costs
// no engine change and is not a fresh judgement call — it is that existing
// precedent applied to a second site. See `tmp/x5b-verdicts.md` for the full
// citation.
//
// The age formula and the male-only clause stay `description` text (D3/Q-05)
// and are out of this family's scope (F-29/F-32 own them).
// ---------------------------------------------------------------------------

#[test]
fn f30_cathedral_school_master_below_ability_minimums_is_refused() {
    let rs = load_ruleset();
    let mut e = entity("companion", vec![sel("virtue.cathedral_school_master")]);
    e.ability_scores = vec![
        score("ability.artes_liberales", 5),
        score("ability.teaching", 3),
        score("ability.dead_language", 4), // one below the Latin minimum
    ];

    let result = validate(&e, &rs);
    assert!(
        has_prereq_not_met(&result),
        "ArMDE:3551 requires Latin (dead_language) >= 5 as well; a score of 4 \
         must be refused, got issues {:?}",
        issue_codes(&result)
    );
}

#[test]
fn f30_cathedral_school_master_at_ability_minimums_is_legal() {
    let rs = load_ruleset();
    let mut e = entity("companion", vec![sel("virtue.cathedral_school_master")]);
    e.ability_scores = vec![
        score("ability.artes_liberales", 5),
        score("ability.teaching", 3),
        score("ability.dead_language", 5),
    ];

    let result = validate(&e, &rs);
    assert!(
        !has_prereq_not_met(&result),
        "ArMDE:3551's three minima exactly met must be legal, got issues {:?}",
        issue_codes(&result)
    );
}

// ---------------------------------------------------------------------------
// Wizard dead end (review-final.json finding #1, MAJOR): the same trap
// confirmed to pre-exist here as for the newer `AbilityCategoryScoreMin`/
// `AnyArtMin` variants (D81.3's Broken Vessel) — a V/F item whose hard
// `AbilityMin` prerequisite can only be satisfied by a later purchase (the
// Abilities phase, which every shipped type profile's `creation_phases`
// declares AFTER `virtues_flaws`: `rules/core/character_types.json`). With
// no Abilities bought yet, selecting this Virtue in the VirtuesFlaws phase
// must not report `prereq_not_met` on `CreationPhase::VirtuesFlaws` — the
// phase whose input surface (Abilities) actually owns the fix is the one
// `wizard-navigation.svelte.ts`'s `canAdvance` must gate on instead.
// ---------------------------------------------------------------------------

#[test]
fn f30_cathedral_school_master_prereq_issue_is_reported_on_a_reachable_phase() {
    let rs = load_ruleset();
    let e = entity("companion", vec![sel("virtue.cathedral_school_master")]);

    let result = validate(&e, &rs);
    let issue = result
        .issues
        .iter()
        .find(|i| {
            i.code == "prereq_not_met"
                && i.context.as_ref() == Some(&Id::new("virtue.cathedral_school_master"))
        })
        .expect(
            "cathedral_school_master must carry a prereq_not_met issue with no Abilities bought",
        );
    assert_eq!(
        issue.phase,
        CreationPhase::Abilities,
        "the issue must be attributed to Abilities — the phase whose input surface can \
         satisfy all three AbilityMin minima — not to VirtuesFlaws, where the player is \
         stuck with no way to reach Abilities and fix it, got {:?}",
        issue.phase
    );
}

// ---------------------------------------------------------------------------
// F-56 — `virtue.doctor_in_faculty`, ArMDE:3683-3698.
// "He must have a score of 5 in Latin. Artes Liberales, and the Ability that
// correlates to his faculty degree." (German resolves the OCR's missing
// comma: three Abilities at 5, not "Latin at 5" plus two unquantified.)
//
// Two of three are expressible today: `ability.artes_liberales` >= 5 and
// `ability.dead_language` >= 5 (Latin, same precedent as F-30). The third —
// "the Ability that correlates to his faculty degree" — depends on this
// entry's own open `faculty` parameter, which no `Prereq` variant can read
// (confirmed: `Prereq::AbilityMin` names a fixed Ability id, never a
// parameter binding). That is a genuine, structural D3 gap distinct from the
// Latin case, and stays `description` text (F-58/F-172 note the same for the
// fixed-faculty sibling). The age clause "(27 – Intelligence)" is likewise
// text-only (F-58), out of this family's scope.
// ---------------------------------------------------------------------------

#[test]
fn f56_doctor_in_faculty_below_ability_minimums_is_refused() {
    let rs = load_ruleset();
    let mut e = entity(
        "companion",
        vec![sel_with_param(
            "virtue.doctor_in_faculty",
            "faculty",
            "medicine",
        )],
    );
    e.ability_scores = vec![
        score("ability.artes_liberales", 4), // one below the minimum
        score("ability.dead_language", 5),
    ];

    let result = validate(&e, &rs);
    assert!(
        has_prereq_not_met(&result),
        "ArMDE:3687 requires Artes Liberales >= 5; a score of 4 must be \
         refused, got issues {:?}",
        issue_codes(&result)
    );
}

#[test]
fn f56_doctor_in_faculty_at_ability_minimums_is_legal() {
    let rs = load_ruleset();
    let mut e = entity(
        "companion",
        vec![sel_with_param(
            "virtue.doctor_in_faculty",
            "faculty",
            "medicine",
        )],
    );
    e.ability_scores = vec![
        score("ability.artes_liberales", 5),
        score("ability.dead_language", 5),
    ];

    let result = validate(&e, &rs);
    assert!(
        !has_prereq_not_met(&result),
        "ArMDE:3687's two expressible minima exactly met must be legal, got \
         issues {:?}",
        issue_codes(&result)
    );
}

// ---------------------------------------------------------------------------
// F-145 — `virtue.license_of_absence`, ArMDE:4291-4294.
// "A license of absence may only be taken by a character with the Priest
// Social Status. It may not be taken by Senior Clergy."
//
// Fully expressible: `all[has(virtue.priest), none[has(virtue.senior_clergy)]]`
// (`tmp/x5b-verdicts.md`; both target ids confirmed in the catalogue, neither
// carries a prerequisite of its own). Both directions of the gate are live
// gaps today (no `prerequisites` key at all).
// ---------------------------------------------------------------------------

#[test]
fn f145_license_of_absence_without_priest_status_is_refused() {
    let rs = load_ruleset();
    let e = entity("companion", vec![sel("virtue.license_of_absence")]);

    let result = validate(&e, &rs);
    assert!(
        has_prereq_not_met(&result),
        "ArMDE:4293 restricts this Virtue to characters with the Priest \
         Social Status; holding neither must be refused, got issues {:?}",
        issue_codes(&result)
    );
}

#[test]
fn f145_license_of_absence_for_senior_clergy_is_refused() {
    let rs = load_ruleset();
    let e = entity(
        "companion",
        vec![
            sel("virtue.license_of_absence"),
            sel("virtue.priest"),
            sel("virtue.senior_clergy"),
        ],
    );

    let result = validate(&e, &rs);
    assert!(
        has_prereq_not_met(&result),
        "ArMDE:4293 explicitly excludes Senior Clergy; a Senior Clergy \
         character must be refused even while also holding Priest, got \
         issues {:?}",
        issue_codes(&result)
    );
}

#[test]
fn f145_license_of_absence_for_a_priest_is_legal() {
    let rs = load_ruleset();
    let e = entity(
        "companion",
        vec![sel("virtue.license_of_absence"), sel("virtue.priest")],
    );

    let result = validate(&e, &rs);
    assert!(
        !has_prereq_not_met(&result),
        "a Priest with no Senior Clergy status must be legal, got issues {:?}",
        issue_codes(&result)
    );
}

// ---------------------------------------------------------------------------
// F-169 — `virtue.magister_in_artibus`, ArMDE:4385-4394.
// "You are at least (25 – Int) years old, and must have scores of at least 5
// in Latin and Artes Liberales."
//
// Same shape as F-30/F-56: `ability.dead_language` >= 5 (Latin, id-level
// proxy precedent) and `ability.artes_liberales` >= 5. The age formula stays
// text (F-171), out of scope here.
// ---------------------------------------------------------------------------

#[test]
fn f169_magister_in_artibus_below_ability_minimums_is_refused() {
    let rs = load_ruleset();
    let mut e = entity("companion", vec![sel("virtue.magister_in_artibus")]);
    e.ability_scores = vec![
        score("ability.dead_language", 5),
        score("ability.artes_liberales", 4), // one below the minimum
    ];

    let result = validate(&e, &rs);
    assert!(
        has_prereq_not_met(&result),
        "ArMDE:4389 requires Artes Liberales >= 5; a score of 4 must be \
         refused, got issues {:?}",
        issue_codes(&result)
    );
}

#[test]
fn f169_magister_in_artibus_at_ability_minimums_is_legal() {
    let rs = load_ruleset();
    let mut e = entity("companion", vec![sel("virtue.magister_in_artibus")]);
    e.ability_scores = vec![
        score("ability.dead_language", 5),
        score("ability.artes_liberales", 5),
    ];

    let result = validate(&e, &rs);
    assert!(
        !has_prereq_not_met(&result),
        "ArMDE:4389's two minima exactly met must be legal, got issues {:?}",
        issue_codes(&result)
    );
}

// ---------------------------------------------------------------------------
// F-172 — `virtue.magister_in_medicina`, ArMDE:4395-4398, importing
// ArMDE:3687's "must have a score of 5 in Latin, Artes Liberales, and the
// Ability that correlates to his faculty degree" — but here the faculty is
// FIXED (medicine), so all three resolve to concrete ids:
// `ability.dead_language` >= 5, `ability.artes_liberales` >= 5,
// `ability.medicine` >= 5. Unlike F-56, nothing here is parameter-bound, so
// all three are fully expressible. The age clause stays text (F-173).
// ---------------------------------------------------------------------------

#[test]
fn f172_magister_in_medicina_below_ability_minimums_is_refused() {
    let rs = load_ruleset();
    let mut e = entity("companion", vec![sel("virtue.magister_in_medicina")]);
    e.ability_scores = vec![
        score("ability.dead_language", 5),
        score("ability.artes_liberales", 5),
        score("ability.medicine", 4), // one below the minimum
    ];

    let result = validate(&e, &rs);
    assert!(
        has_prereq_not_met(&result),
        "ArMDE:3687 (imported) requires Medicine >= 5; a score of 4 must be \
         refused, got issues {:?}",
        issue_codes(&result)
    );
}

#[test]
fn f172_magister_in_medicina_at_ability_minimums_is_legal() {
    let rs = load_ruleset();
    let mut e = entity("companion", vec![sel("virtue.magister_in_medicina")]);
    e.ability_scores = vec![
        score("ability.dead_language", 5),
        score("ability.artes_liberales", 5),
        score("ability.medicine", 5),
    ];

    let result = validate(&e, &rs);
    assert!(
        !has_prereq_not_met(&result),
        "the three imported minima exactly met must be legal, got issues {:?}",
        issue_codes(&result)
    );
}

// ---------------------------------------------------------------------------
// F-183 — `virtue.master_bard`, ArMDE:4457-4462.
// "must have a Profession: Storyteller or Poet of at least 5, and also a 5 in
// at least one of Area Lore, Organization Lore, Faerie Lore, or Magic Lore."
//
// The four-way Lore clause is fully expressible as
// `any[ability_min(area_lore,5), ability_min(organization_lore,5),
// ability_min(faerie_lore,5), ability_min(magic_lore,5)]` (B6: `any` short-
// circuits True and `ability_min` is never `Unknown`, so this is exact).
//
// The Profession clause IS tested here — D70 (Norbert, 2026-09-29,
// `tmp/x6-scope.md`) resolved QUESTION 1 as option (a): "Master Bard's
// Profession clause is enforced as 'some Profession at 5', the id-level check
// already used for the Latin minimums." So the gate is
// `all[ability_min(profession,5), any[the four Lores at 5]]`.
// ---------------------------------------------------------------------------

#[test]
fn f183_master_bard_below_ability_minimums_is_refused() {
    let rs = load_ruleset();
    let mut e = entity("companion", vec![sel("virtue.master_bard")]);
    e.ability_scores = vec![
        score("ability.profession", 5),
        score("ability.area_lore", 4),
        score("ability.organization_lore", 4),
        score("ability.faerie_lore", 4),
        score("ability.magic_lore", 4),
    ];

    let result = validate(&e, &rs);
    assert!(
        has_prereq_not_met(&result),
        "ArMDE:4461 requires at least one of the four Lores at 5; all four \
         at 4 must be refused even with Profession >= 5, got issues {:?}",
        issue_codes(&result)
    );
}

#[test]
fn f183_master_bard_at_one_lore_minimum_is_legal() {
    let rs = load_ruleset();
    let mut e = entity("companion", vec![sel("virtue.master_bard")]);
    e.ability_scores = vec![
        score("ability.profession", 5),
        score("ability.area_lore", 5),
    ];

    let result = validate(&e, &rs);
    assert!(
        !has_prereq_not_met(&result),
        "any ONE of the four Lores at 5 (ArMDE:4461's \"at least one of\") \
         plus Profession >= 5 (D70) must be legal, got issues {:?}",
        issue_codes(&result)
    );
}

#[test]
fn f183_master_bard_without_profession_minimum_is_refused() {
    let rs = load_ruleset();
    let mut e = entity("companion", vec![sel("virtue.master_bard")]);
    e.ability_scores = vec![score("ability.area_lore", 5)];

    let result = validate(&e, &rs);
    assert!(
        has_prereq_not_met(&result),
        "D70 enforces Profession >= 5 alongside the Lore clause; a Lore \
         minimum alone with no Profession score must be refused, got issues \
         {:?}",
        issue_codes(&result)
    );
}

// ---------------------------------------------------------------------------
// F-223 — `virtue.physician_of_salerno`, ArMDE:4732-4735.
// "To take this Virtue, you must be able to take Academic Abilities."
//
// NOT expressible as a `Prereq`: this gates on whether a *category* is
// authorized, and no `Prereq` variant tests that (`Prereq::HasCategory`
// tests whether an item of the category is HELD, not whether the category
// is AUTHORIZED for purchase — a different fact). Worse, the entry's own
// `restricted_ability_xp` effect grants exactly the permission the sentence
// presupposes, so encoding it as a same-entry gate would be circular. D3
// governs: `description` text in both locales, verbatim. Currently absent
// from both (confirmed: only `summary`, no `description` key).
// ---------------------------------------------------------------------------

#[test]
fn f223_physician_of_salerno_academic_gate_is_stated_in_english() {
    let text = displayed_text(EN_VF, "virtue.physician_of_salerno");
    assert!(
        text.to_lowercase().contains("academic abilit"),
        "ArMDE:4734 requires being able to take Academic Abilities; the \
         English displayed text does not state it: {text:?}"
    );
}

#[test]
fn f223_physician_of_salerno_academic_gate_is_stated_in_german() {
    let text = displayed_text(DE_VF, "virtue.physician_of_salerno");
    assert!(
        text.contains("Akademische Fertigkeiten") || text.contains("akademische Fertigkeiten"),
        "ArMDE:4734 (DE) requires being able to acquire Academic Abilities; \
         the German displayed text does not state it: {text:?}"
    );
}

// ---------------------------------------------------------------------------
// F-252 — `virtue.rosh_beth_din`, ArMDE:4878-4883.
// "Your character must be (30 – Int) years old to take this Virtue and have
// scores of at least 5 in Hebrew, Rabbinic Law, and Theology: Judaism."
//
// Two of three Ability minima are unparameterized and directly expressible:
// `ability.rabbinic_law` >= 5, `ability.theology_judaism` >= 5. Hebrew is the
// same dead-language-instance shape as F-30/F-56/F-169's Latin (id-level
// proxy precedent, `ability.dead_language` >= 5). The age formula
// "(30 – Int)" is NOT expressible (no `Prereq` reads age minus a
// Characteristic) and stays `description` text — this is the entry the
// brief tags "age → text".
// ---------------------------------------------------------------------------

#[test]
fn f252_rosh_beth_din_below_ability_minimums_is_refused() {
    let rs = load_ruleset();
    let mut e = entity("companion", vec![sel("virtue.rosh_beth_din")]);
    e.ability_scores = vec![
        score("ability.rabbinic_law", 5),
        score("ability.theology_judaism", 4), // one below the minimum
        score("ability.dead_language", 5),
    ];

    let result = validate(&e, &rs);
    assert!(
        has_prereq_not_met(&result),
        "ArMDE:4880 requires Theology: Judaism >= 5; a score of 4 must be \
         refused, got issues {:?}",
        issue_codes(&result)
    );
}

#[test]
fn f252_rosh_beth_din_at_ability_minimums_is_legal() {
    let rs = load_ruleset();
    let mut e = entity("companion", vec![sel("virtue.rosh_beth_din")]);
    e.ability_scores = vec![
        score("ability.rabbinic_law", 5),
        score("ability.theology_judaism", 5),
        score("ability.dead_language", 5),
    ];

    let result = validate(&e, &rs);
    assert!(
        !has_prereq_not_met(&result),
        "ArMDE:4880's three minima exactly met must be legal, got issues {:?}",
        issue_codes(&result)
    );
}

#[test]
fn f252_rosh_beth_din_age_formula_is_stated_in_both_locales() {
    let en = displayed_text(EN_VF, "virtue.rosh_beth_din");
    let de = displayed_text(DE_VF, "virtue.rosh_beth_din");
    assert!(
        en.contains("30") && en.to_lowercase().contains("int"),
        "ArMDE:4880 gives a (30 - Int) age floor; English displayed text \
         does not state it: {en:?}"
    );
    assert!(
        de.contains("30") && de.to_lowercase().contains("int"),
        "ArMDE:4880 (DE) gives a (30 - Int) age floor; German displayed text \
         does not state it: {de:?}"
    );
}

// ---------------------------------------------------------------------------
// F-261 — `virtue.senior_bard`, ArMDE:4904-4909.
// "The character has a minimum age of 22."
//
// NOT expressible: B6 lists the `Prereq` variants and none reads
// `Entity::age` (unlike the newer `Prereq::AgeMin`, which does not exist at
// this slice's HEAD — see the note below). D3 governs: `description` text
// in both locales. Currently absent from both (only `summary`).
//
// NOTE FOR THE ORCHESTRATOR: `crates/arm-rules/src/types.rs` currently
// carries an UNCOMMITTED, in-flight `Prereq::AgeMin(u32)` (added by a
// parallel slice for University Dean, D69/X7b-e row 42) that reads
// `Entity::age` directly and is NOT hedged to a Characteristic formula. If
// that lands on `main` first, Senior Bard's FLAT "minimum age of 22" becomes
// directly expressible as `{"kind":"age_min","value":22}` — unlike Rosh Beth
// Din's `(30 – Int)`, which is a formula and stays out of `AgeMin`'s reach
// even then. This slice does not consume `AgeMin` (it is not on `main` at
// HEAD and the brief scopes this entry to text), but a later data pass on
// Senior Bard should reconsider text-only once `AgeMin` ships.
// ---------------------------------------------------------------------------

#[test]
fn f261_senior_bard_minimum_age_is_stated_in_both_locales() {
    let en = displayed_text(EN_VF, "virtue.senior_bard");
    let de = displayed_text(DE_VF, "virtue.senior_bard");
    assert!(
        en.contains("22"),
        "ArMDE:4908 gives a minimum age of 22; English displayed text does \
         not state it: {en:?}"
    );
    assert!(
        de.contains("22"),
        "ArMDE:4908 (DE) gives a minimum age of 22; German displayed text \
         does not state it: {de:?}"
    );
}

// ---------------------------------------------------------------------------
// F-322 — `virtue.town_magistrate`, ArMDE:5149-5152.
// "have a score of at least 3 in the Civil and Canon Law (or Common Law)
// Ability."
//
// Fully expressible, no approximation at all:
// `any[ability_min(civil_and_canon_law,3), ability_min(common_law,3)]`. Both
// ids are unparameterized.
// ---------------------------------------------------------------------------

#[test]
fn f322_town_magistrate_below_ability_minimums_is_refused() {
    let rs = load_ruleset();
    let mut e = entity("companion", vec![sel("virtue.town_magistrate")]);
    e.ability_scores = vec![
        score("ability.civil_and_canon_law", 2),
        score("ability.common_law", 2),
    ];

    let result = validate(&e, &rs);
    assert!(
        has_prereq_not_met(&result),
        "ArMDE:5151 requires Civil and Canon Law OR Common Law >= 3; both at \
         2 must be refused, got issues {:?}",
        issue_codes(&result)
    );
}

#[test]
fn f322_town_magistrate_at_one_ability_minimum_is_legal() {
    let rs = load_ruleset();
    let mut e = entity("companion", vec![sel("virtue.town_magistrate")]);
    e.ability_scores = vec![score("ability.civil_and_canon_law", 3)];

    let result = validate(&e, &rs);
    assert!(
        !has_prereq_not_met(&result),
        "Civil and Canon Law alone at 3 must be legal (the book's \"or\"), \
         got issues {:?}",
        issue_codes(&result)
    );
}

// ---------------------------------------------------------------------------
// F-324 — `virtue.trained_assassin`, ArMDE:5153-5156.
// "This Virtue is only available to characters with one of the Social
// Status Virtues of the Nizaris."
//
// The passage names no Virtue directly; the referent is `virtue.fidai`
// (ArMDE:3877, "an assassin of the Nizari Isma'ilis") and `virtue.lasiq`
// (ArMDE:4233, "an experienced assassin of the Nizari Isma'ilis") — both
// confirmed in the catalogue, both `social_status`, neither carrying a
// prerequisite of its own. Fully expressible:
// `any[has(virtue.fidai), has(virtue.lasiq)]`.
// ---------------------------------------------------------------------------

#[test]
fn f324_trained_assassin_without_a_nizari_status_is_refused() {
    let rs = load_ruleset();
    let e = entity("companion", vec![sel("virtue.trained_assassin")]);

    let result = validate(&e, &rs);
    assert!(
        has_prereq_not_met(&result),
        "ArMDE:5155 restricts this Virtue to a Nizari Social Status; holding \
         neither Fida'i nor Lasiq must be refused, got issues {:?}",
        issue_codes(&result)
    );
}

#[test]
fn f324_trained_assassin_with_fidai_is_legal() {
    let rs = load_ruleset();
    let e = entity(
        "companion",
        vec![sel("virtue.trained_assassin"), sel("virtue.fidai")],
    );

    let result = validate(&e, &rs);
    assert!(
        !has_prereq_not_met(&result),
        "holding Fida'i must satisfy the Nizari-status gate, got issues {:?}",
        issue_codes(&result)
    );
}
