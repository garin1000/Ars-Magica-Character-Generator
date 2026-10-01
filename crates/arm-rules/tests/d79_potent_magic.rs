//! D79 (`docs/vf-audit/decisions.md`; design
//! `docs/vf-audit/design-x7-relic-and-ct-mirror.md` § 2) — GREEN, Phase 2
//! landed (Rust side; the Svelte UI port is a separate step, see
//! `tmp/d79-handover.md`).
//!
//! Potent Magic's +3/+6 (ArMDE:4744-4748) applies only "in her field of
//! magic ... much as in a Magical Focus" (ArMDE:4740-4742) — a free-text theme
//! the Arts cannot decide membership of, exactly like X10c's Magical Focus
//! marker and independent of it (a character's Potent Magic field and Magical
//! Focus descriptor need not be the same text). Before this slice:
//!
//! - `virtue.potent_magic_major`/`_minor`'s `casting_total_mod {scope: "all"}`
//!   reached every spell unconditionally — wrong rules output for anyone who
//!   takes Potent Magic and casts outside their field.
//! - The same two entries' `lab_total_mod` was gated on holding a **Magical
//!   Focus**, not on holding **Potent Magic**: a character with Potent Magic
//!   but no Focus never saw the bonus at all, and a character with both saw
//!   it leak into the Focus figure.
//!
//! This slice fixes both. `Effect::CastingTotalMod` gained
//! `potent_field_only: bool` (orthogonal to its existing cast-type `scope`);
//! `LabTotalModScope::WithinFocusOnly` was renamed `WithinPotentFieldOnly` and
//! re-gated on Potent Magic rather than Magical Focus
//! (`derived.rs::in_play_lab_total_mod_within_potent_field`,
//! `InPlayMods::has_potent_magic`/`casting_mods_within_potent_field`). Both
//! folds are MAX, not sum, across carriers — see the ArMDE:4742 note below.
//! `SpellSelection::within_potent_field` is wired into
//! `derived/casting.rs::spell_casting_total` via `formulaic_casting_score`'s
//! new `potent: bool` parameter, independently of `within_focus`, so a spell
//! marked under both markers gets the Magical-Focus doubling AND the Potent
//! Magic bonus together. `CastingTotal`/`LabTotal` each gained a
//! `within_potent_field` figure beside the existing `within_focus` one, gated
//! on `has_potent_magic` rather than `has_focus` — independently non-null, as
//! the grid-gate tests below pin.
//!
//! ArMDE:4742 also states the bound this file's `*_only_one_potent_magic_virtue_applies*`
//! tests pin: "a maga may have more than one area of Potent Magic, although
//! only one Potent Magic Virtue applies to any single activity" — two Potent
//! Magic Virtues must not stack additively on one spell/cell.
//!
//! See `tmp/d79-handover.md` for the Phase 1 verbatim RED output, the Phase 2
//! verbatim GREEN output, and the remaining Svelte/UI port work list
//! (`SpellTab`'s second toggle, the Lab grid's new column, Fluent keys in both
//! locales).

use arm_rules::derived::{CastingTotal, LabTotal, casting_totals, lab_totals, spell_casting_total};
use arm_rules::ruleset::{Ruleset, RulesetSources};
use arm_rules::types::*;
use std::collections::BTreeMap;

/// The shipped core ruleset — same set of files `book_templates.rs::full_ruleset`
/// loads; duplicated because integration test binaries cannot share private
/// helpers (convention already established by `x5b_ability_minimums.rs`,
/// `x10bc_banked_xp_and_within_focus.rs`).
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

fn magus() -> Entity {
    Entity::new(
        EntityKind::Character,
        Id::new("magus"),
        RulesetRef::new(Id::new("arm5-core"), "2024.1"),
    )
}

/// A magus with Creo 12 / Ignem 15 and nothing else — the same baseline
/// `x10bc_banked_xp_and_within_focus.rs` uses (base Casting Total 27, within a
/// Focus 39). Zero Stamina/Encumbrance/Aura, so `common == 27` exactly.
fn magus_with_arts() -> Entity {
    let mut e = magus();
    e.art_scores = vec![
        ArtScore::new(Id::new("art.creo"), 12),
        ArtScore::new(Id::new("art.ignem"), 15),
    ];
    e
}

fn potent_magic_major(field: &str) -> Selection {
    Selection::with_params(
        Id::new("virtue.potent_magic_major"),
        BTreeMap::from([("field".to_string(), Id::new(field))]),
    )
}

fn potent_magic_minor(field: &str) -> Selection {
    Selection::with_params(
        Id::new("virtue.potent_magic_minor"),
        BTreeMap::from([("field".to_string(), Id::new(field))]),
    )
}

fn major_magical_focus(focus: &str) -> Selection {
    Selection::with_params(
        Id::new("virtue.major_magical_focus"),
        BTreeMap::from([("focus".to_string(), Id::new(focus))]),
    )
}

fn find_casting<'a>(totals: &'a [CastingTotal], te: &str, fo: &str) -> &'a CastingTotal {
    totals
        .iter()
        .find(|t| t.technique.as_str() == te && t.form.as_str() == fo)
        .expect("Creo/Ignem casting cell present")
}

fn find_lab<'a>(totals: &'a [LabTotal], te: &str, fo: &str) -> &'a LabTotal {
    totals
        .iter()
        .find(|t| t.technique.as_str() == te && t.form.as_str() == fo)
        .expect("Creo/Ignem lab cell present")
}

fn pilum_of_fire(within_focus: bool, within_potent_field: bool) -> SpellSelection {
    let mut s = SpellSelection::new(Id::new("spell.pilum_of_fire"));
    s.within_focus = within_focus;
    s.within_potent_field = within_potent_field;
    s
}

// --- Base Casting/Lab Totals must not include the bonus unconditionally ----

#[test]
fn casting_total_base_no_longer_includes_potent_magic_bonus() {
    // Potent Magic Major, no Focus: today's `casting_mod_for` fold is unscoped
    // by field, so the base (unmarked) Creo/Ignem cell already carries +6 —
    // wrong, since the maga is not (by this cell's own accounting) casting
    // within her field. ArMDE:4744: "Potent Magic provides the maga with a
    // bonus in her field of magic" — not outside it.
    let ruleset = full_ruleset();
    let mut e = magus_with_arts();
    e.selections = vec![potent_magic_major("fire")];

    let totals = casting_totals(&e, &ruleset);
    let cell = find_casting(&totals, "art.creo", "art.ignem");
    assert_eq!(
        cell.formulaic, 27,
        "the base Casting Total must not include Potent Magic's bonus unconditionally"
    );
}

#[test]
fn lab_total_base_is_unaffected_by_potent_magic_alone() {
    // Already correct today (D4 excludes `within_focus_only`-scoped
    // `LabTotalMod` carriers from the flat `lab_mod` fold entirely) — pinned
    // here as a regression guard alongside the genuinely red cases below, so
    // a Phase 2 refactor cannot quietly reintroduce the base-total leak while
    // fixing the within-focus one.
    let ruleset = full_ruleset();
    let mut e = magus_with_arts();
    e.selections = vec![potent_magic_major("fire")];

    let totals = lab_totals(&e, &ruleset);
    let cell = find_lab(&totals, "art.creo", "art.ignem");
    assert_eq!(
        cell.total, 27,
        "the base Lab Total must not include Potent Magic's bonus"
    );
}

// --- The per-spell `within_potent_field` marker must gate the bonus --------

#[test]
fn casting_total_marked_within_potent_field_gets_the_major_bonus() {
    // Diff-based rather than an absolute expected total: today
    // `spell_casting_total` ignores `within_potent_field` entirely, so an
    // absolute "marked spell reads 33" assertion would pass BY ACCIDENT
    // (today's bug already adds +6 to every spell, marked or not) without the
    // marker doing anything — not a valid red for "the marker gates the
    // bonus". Comparing a marked spell against its unmarked twin on the same
    // cell is red for the right reason: today the diff is 0 either way.
    let ruleset = full_ruleset();
    let mut e = magus_with_arts();
    e.selections = vec![potent_magic_major("fire")];

    let marked = spell_casting_total(&pilum_of_fire(false, true), &e, &ruleset)
        .expect("spell.pilum_of_fire is in the shipped catalogue");
    let unmarked = spell_casting_total(&pilum_of_fire(false, false), &e, &ruleset)
        .expect("spell.pilum_of_fire is in the shipped catalogue");
    assert_eq!(
        marked - unmarked,
        6,
        "within_potent_field: true must add Major Potent Magic's +6 over the unmarked figure \
         (marked {marked}, unmarked {unmarked})"
    );
}

#[test]
fn casting_total_marked_within_potent_field_gets_the_minor_bonus() {
    let ruleset = full_ruleset();
    let mut e = magus_with_arts();
    e.selections = vec![potent_magic_minor("fire")];

    let marked = spell_casting_total(&pilum_of_fire(false, true), &e, &ruleset)
        .expect("spell.pilum_of_fire is in the shipped catalogue");
    let unmarked = spell_casting_total(&pilum_of_fire(false, false), &e, &ruleset)
        .expect("spell.pilum_of_fire is in the shipped catalogue");
    assert_eq!(
        marked - unmarked,
        3,
        "within_potent_field: true must add Minor Potent Magic's +3 over the unmarked figure \
         (marked {marked}, unmarked {unmarked})"
    );
}

#[test]
fn casting_total_unmarked_spell_excludes_potent_magic_even_when_held() {
    // The direct complement of the diff tests above, stated as an absolute
    // figure: an unmarked spell on a character who holds Potent Magic must
    // read the plain base total, not base+6.
    let ruleset = full_ruleset();
    let mut e = magus_with_arts();
    e.selections = vec![potent_magic_major("fire")];

    let total = spell_casting_total(&pilum_of_fire(false, false), &e, &ruleset)
        .expect("spell.pilum_of_fire is in the shipped catalogue");
    assert_eq!(
        total, 27,
        "an unmarked spell must not receive Potent Magic's bonus"
    );
}

// --- Independence from X10c's Magical Focus marker -------------------------

#[test]
fn casting_total_marked_within_focus_only_excludes_the_potent_bonus() {
    // A magus with BOTH a Major Magical Focus and Major Potent Magic: a spell
    // marked `within_focus: true, within_potent_field: false` must read the
    // focus-doubled figure (27 + min(12, 15) = 39) and nothing more. Today,
    // `casting_mod_for(Formulaic)` (which still includes Potent Magic's +6,
    // unconditionally) feeds the within-focus variant exactly as it feeds the
    // base one, so the actual figure is 45 — the defect this test pins.
    let ruleset = full_ruleset();
    let mut e = magus_with_arts();
    e.selections = vec![major_magical_focus("fire"), potent_magic_major("fire")];

    let total = spell_casting_total(&pilum_of_fire(true, false), &e, &ruleset)
        .expect("spell.pilum_of_fire is in the shipped catalogue");
    assert_eq!(
        total, 39,
        "within_focus alone (within_potent_field: false) must not include Potent Magic's bonus"
    );
}

#[test]
fn casting_total_marked_both_gets_focus_doubling_and_the_potent_bonus_together() {
    // Diff against the within-focus-only figure computed the same way the
    // previous test does, rather than an absolute 45: today both reads come
    // out identical (45, 45 — the bug applies Potent Magic's bonus
    // regardless of either marker), so the diff is 0 where it must be 6. This
    // is genuinely red independently of whether the previous test is also
    // fixed, since it is self-contained.
    let ruleset = full_ruleset();
    let mut e = magus_with_arts();
    e.selections = vec![major_magical_focus("fire"), potent_magic_major("fire")];

    let focus_only = spell_casting_total(&pilum_of_fire(true, false), &e, &ruleset)
        .expect("spell.pilum_of_fire is in the shipped catalogue");
    let both = spell_casting_total(&pilum_of_fire(true, true), &e, &ruleset)
        .expect("spell.pilum_of_fire is in the shipped catalogue");
    assert_eq!(
        both - focus_only,
        6,
        "a spell marked under both must add the Potent Magic bonus on top of the focus-doubled \
         figure (focus_only {focus_only}, both {both})"
    );
}

// --- ArMDE:4742 — "only one Potent Magic Virtue applies to any single
// activity": two Potent Magic Virtues must not stack additively -------------

#[test]
fn casting_total_only_one_potent_magic_virtue_applies_two_majors() {
    // Two Major Potent Magic Virtues over different fields (legal:
    // `max_total: 255`, ArMDE:4742 "a maga may have more than one area of
    // Potent Magic"). A marked spell must gain +6 once, never +12.
    let ruleset = full_ruleset();
    let mut e = magus_with_arts();
    e.selections = vec![potent_magic_major("fire"), potent_magic_major("water")];

    let marked = spell_casting_total(&pilum_of_fire(false, true), &e, &ruleset)
        .expect("spell.pilum_of_fire is in the shipped catalogue");
    let unmarked = spell_casting_total(&pilum_of_fire(false, false), &e, &ruleset)
        .expect("spell.pilum_of_fire is in the shipped catalogue");
    assert_eq!(
        marked - unmarked,
        6,
        "two Major Potent Magic Virtues must not stack: the bonus is still +6, not +12 \
         (marked {marked}, unmarked {unmarked})"
    );
}

#[test]
fn casting_total_only_one_potent_magic_virtue_applies_major_and_minor() {
    // A Major (+6) and a Minor (+3) Potent Magic Virtue together: the one
    // that applies must be the larger, not the sum (+9) and not the smaller.
    let ruleset = full_ruleset();
    let mut e = magus_with_arts();
    e.selections = vec![potent_magic_major("fire"), potent_magic_minor("water")];

    let marked = spell_casting_total(&pilum_of_fire(false, true), &e, &ruleset)
        .expect("spell.pilum_of_fire is in the shipped catalogue");
    let unmarked = spell_casting_total(&pilum_of_fire(false, false), &e, &ruleset)
        .expect("spell.pilum_of_fire is in the shipped catalogue");
    assert_eq!(
        marked - unmarked,
        6,
        "a Major + a Minor Potent Magic Virtue must apply only the larger bonus (+6), not the \
         sum (+9) (marked {marked}, unmarked {unmarked})"
    );
}

// --- The Lab Totals grid's own within-potent-field figure ------------------

#[test]
fn lab_total_within_focus_no_longer_includes_the_potent_bonus() {
    // D4 originally folded Potent Magic's Lab Total bonus into the
    // Magical-Focus-gated `within_focus` figure (`lab_mod_within_focus`).
    // D79 separates them: with both Virtues held, `within_focus` must read
    // the focus-only figure (27 + min(12, 15) = 39), not 45.
    let ruleset = full_ruleset();
    let mut e = magus_with_arts();
    e.selections = vec![major_magical_focus("fire"), potent_magic_major("fire")];

    let totals = lab_totals(&e, &ruleset);
    let cell = find_lab(&totals, "art.creo", "art.ignem");
    assert_eq!(
        cell.within_focus,
        Some(39),
        "the Magical-Focus Lab Total figure must no longer include Potent Magic's bonus"
    );
}

#[test]
fn lab_total_within_potent_field_present_when_potent_magic_alone_is_held() {
    // No Magical Focus at all — today `within_focus` is correctly `None`
    // (Potent Magic carries no `Effect::MagicalFocus`), but there is also no
    // other figure to show the +6, since `within_potent_field` is a Phase 1
    // stub hardcoded to `None`. This is the gap: Potent Magic alone must
    // surface its own figure independent of holding a Focus.
    let ruleset = full_ruleset();
    let mut e = magus_with_arts();
    e.selections = vec![potent_magic_major("fire")];

    let totals = lab_totals(&e, &ruleset);
    let cell = find_lab(&totals, "art.creo", "art.ignem");
    assert_eq!(
        cell.within_focus, None,
        "no Magical Focus is held, so `within_focus` must stay None"
    );
    assert_eq!(
        cell.within_potent_field,
        Some(33),
        "within_potent_field must surface Potent Magic's +6 even without a Magical Focus"
    );
}

#[test]
fn lab_total_only_one_potent_magic_virtue_applies() {
    let ruleset = full_ruleset();
    let mut e = magus_with_arts();
    e.selections = vec![potent_magic_major("fire"), potent_magic_major("water")];

    let totals = lab_totals(&e, &ruleset);
    let cell = find_lab(&totals, "art.creo", "art.ignem");
    assert_eq!(
        cell.within_potent_field,
        Some(33),
        "two Major Potent Magic Virtues must not stack in the Lab Total either (expected base \
         27 + 6, not + 12)"
    );
}

// --- CastingTotal's grid-level gate: the UI's two independent toggles ------
//
// Norbert (mid-task): both per-spell markers must be offered only when the
// matching Virtue is held — the Focus toggle only with a Magical Focus, the
// Potent toggle only with Potent Magic — and the existing X10c gate
// (`CastingTotal.within_focus` non-null) must keep meaning "has a Magical
// Focus" and nothing broader. These four pin all of
// {focus, potent} x {held, not held} on the GRID cell the UI reads to decide
// which toggle(s) to show (`ui/src/lib/types.ts`'s mirror of `CastingTotal`,
// Phase 2). Two of the four are already correct today (the existing
// `has_focus` gate was never conflated with Potent Magic in the first place —
// Potent Magic carries no `Effect::MagicalFocus`); they are included anyway as
// regression guards, since a phase-2 edit that merges the two gates would
// silently break exactly this.

#[test]
fn casting_total_grid_focus_only_character_has_focus_figure_and_no_potent_figure() {
    let ruleset = full_ruleset();
    let mut e = magus_with_arts();
    e.selections = vec![major_magical_focus("fire")];

    let totals = casting_totals(&e, &ruleset);
    let cell = find_casting(&totals, "art.creo", "art.ignem");
    assert!(
        cell.within_focus.is_some(),
        "a Focus-holding character must get the focus figure"
    );
    assert!(
        cell.within_potent_field.is_none(),
        "a character with no Potent Magic must get no potent-field figure"
    );
}

#[test]
fn casting_total_grid_potent_only_character_has_potent_figure_and_no_focus_figure() {
    let ruleset = full_ruleset();
    let mut e = magus_with_arts();
    e.selections = vec![potent_magic_major("fire")];

    let totals = casting_totals(&e, &ruleset);
    let cell = find_casting(&totals, "art.creo", "art.ignem");
    assert!(
        cell.within_potent_field.is_some(),
        "a Potent-Magic-holding character must get the potent-field figure"
    );
    assert!(
        cell.within_focus.is_none(),
        "a character with no Magical Focus must get no focus figure"
    );
}

#[test]
fn casting_total_grid_both_character_has_both_figures() {
    let ruleset = full_ruleset();
    let mut e = magus_with_arts();
    e.selections = vec![major_magical_focus("fire"), potent_magic_major("fire")];

    let totals = casting_totals(&e, &ruleset);
    let cell = find_casting(&totals, "art.creo", "art.ignem");
    assert!(
        cell.within_focus.is_some(),
        "a character with both must get the focus figure"
    );
    assert!(
        cell.within_potent_field.is_some(),
        "a character with both must get the potent-field figure"
    );
}

#[test]
fn casting_total_grid_neither_character_has_neither_figure() {
    let ruleset = full_ruleset();
    let e = magus_with_arts();

    let totals = casting_totals(&e, &ruleset);
    let cell = find_casting(&totals, "art.creo", "art.ignem");
    assert!(
        cell.within_focus.is_none(),
        "a character with neither must get no focus figure"
    );
    assert!(
        cell.within_potent_field.is_none(),
        "a character with neither must get no potent-field figure"
    );
}

// --- Round trip: `SpellSelection::within_potent_field` ---------------------

#[test]
fn spell_selection_within_potent_field_round_trips() {
    let mut original = SpellSelection::new(Id::new("spell.pilum_of_fire"));
    original.within_potent_field = true;
    let json = serde_json::to_string(&original).expect("serializes");
    assert!(json.contains("\"within_potent_field\":true"), "{json}");
    let back: SpellSelection = serde_json::from_str(&json).expect("deserializes");
    assert_eq!(back, original);
}

#[test]
fn a_save_without_within_potent_field_defaults_to_false_and_omits_it_on_write() {
    // An old (pre-D79) save never wrote `within_potent_field` at all.
    let old_spell = r#"{"spell":"spell.pilum_of_fire"}"#;
    let spell: SpellSelection = serde_json::from_str(old_spell).expect("deserializes");
    assert!(!spell.within_potent_field);
    let rewritten = serde_json::to_string(&spell).expect("serializes");
    assert!(
        !rewritten.contains("within_potent_field"),
        "a false within_potent_field must not appear on write: {rewritten}"
    );
}

#[test]
fn spell_selection_within_focus_and_within_potent_field_round_trip_independently() {
    // Both markers true at once, to pin that they are two independent fields
    // rather than accidentally aliased.
    let mut original = SpellSelection::new(Id::new("spell.pilum_of_fire"));
    original.within_focus = true;
    original.within_potent_field = true;
    let json = serde_json::to_string(&original).expect("serializes");
    assert!(json.contains("\"within_focus\":true"), "{json}");
    assert!(json.contains("\"within_potent_field\":true"), "{json}");
    let back: SpellSelection = serde_json::from_str(&json).expect("deserializes");
    assert_eq!(back, original);
}
