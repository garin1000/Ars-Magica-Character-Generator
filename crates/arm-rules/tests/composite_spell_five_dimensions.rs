//! review-final.json finding #3 (MINOR): one spell combining all five
//! cross-cutting dimensions today's commits each tested individually (at most
//! two or three together) — the requisite fold (ArMDE:12309-12311), Magical
//! Focus doubling (ArMDE:4403), Potent Magic (D79), a Deficient Art
//! (ArMDE:5909-5915), and an Incompatible Arts barred combination reached only
//! through a requisite (ArMDE:6292, D81.8/D81.15) — with hand-computed
//! expected values, verified against the actual rule functions
//! (`effective/spell.rs::spell_level_cap`'s formula and
//! `derived/casting.rs::spell_casting_total`'s, both read directly rather than
//! recalled).
//!
//! A genuine rules wrinkle surfaces in building this: a Deficient Art flaw
//! (`flaw.deficient_form`/`flaw.deficient_technique`) is `incompatible_with`
//! an Incompatible Arts flaw (ArMDE:6292's own closing sentence, "may not be
//! combined with a Deficiency") — so the two cannot coexist LEGALLY. This test
//! still constructs the combination anyway (reachable via direct-unchecked
//! entry, a hand-edited save, or simply never running `validate`): CLAUDE.md's
//! robustness bar is "a reachable, crafted state must not panic or silently
//! misbehave," not "every reachable state is legal." The `incompatible` error
//! is asserted too, exactly because it is the HONEST outcome, not papered
//! over — see `entity_still_reports_the_flaw_level_incompatibility` below.
//!
//! Composition, Technique=Creo(Cr)/Form=Ignem(Ig), requisites=[Rego(Re),
//! Aquam(Aq)] (Re is Technique-class, Aq is Form-class):
//!
//! - Arts: Cr 15, Ig 12, Re 5 (LOWER than Cr — binds the Technique fold),
//!   Aq 20 (HIGHER than Ig — does NOT numerically bind the Form fold, but
//!   Deficiency still applies per ArMDE:12311's closing sentence).
//! - `flaw.deficient_form { form: art.aquam }` — Aq is Deficient.
//! - `virtue.major_magical_focus` + the spell marked `within_focus: true`.
//! - `virtue.potent_magic_major` (+6, ArMDE:4744-4748) + the spell marked
//!   `within_potent_field: true`.
//! - `flaw.incompatible_arts { technique_1: art.rego, form_1: art.ignem,
//!   technique_2: art.muto, form_2: art.herbam }` — combo 1 is (Re, Ig): NOT
//!   the spell's own primary pair (Cr, Ig), reached only because Re is a
//!   Technique-class requisite (ArMDE:6292, "even if one or both are
//!   requisites").
//!
//! **Level cap** (`effective/spell.rs::spell_level_cap`):
//! `tech = min(Cr 15, Re 5) = 5` (Aq is Form-class, skipped for the Technique
//! fold). `form = min(Ig 12, Aq 20) = 12` (Aq does not win the min, but see
//! below). `+6`: `virtue.potent_magic_major`'s own `lab_total_mod` effect
//! (scope `within_potent_field_only`) reaches the creation-time cap because
//! the spell is marked `within_potent_field` — R3 (after-deadline answer 3,
//! amending D1): Potent Magic is the one D1 carrier the cap gates on the
//! player's marker, exactly as the Casting Total does (D79); the other eight
//! stay flat. Unmarked, the +6 is absent — see
//! `level_cap_without_the_potent_field_mark_drops_the_potent_bonus` below.
//! `base = 5 + 12 + 0 (Int) + 0 (Magic Theory) + 3 + 6 = 26`.
//! `within_focus` adds `min(tech, form) = min(5, 12) = 5` -> `31`. Deficient
//! (Aq is a requisite in `deficient_arts`, regardless of whether it
//! numerically bound the fold) halves: `31.div_euclid(2) = 15`. No Range set,
//! so no Short-Ranged-Magic halving. **cap = 15.**
//!
//! **Casting Total** (`derived/casting.rs::spell_casting_total`, via
//! `formulaic_casting_score`): same `te = 5`, `fo = 12` fold (Potent Magic's
//! Lab-Total-only term does not reach this function at all — Casting and Lab
//! totals pull from disjoint `Effect` fields). `score = 5 + 12 + 0 (Stamina) -
//! 0 (Encumbrance) + 0 (Aura) + 0 (no unconditional CastingTotalMod) = 17`.
//! `within_focus` (focus held) adds `te.min(fo) = 5` -> `22`. `within_potent_
//! field` adds Potent Magic Major's own `casting_total_mod { amount: 6, scope:
//! "all", potent_field_only: true }`, the MAX (not sum, ArMDE:4742) over
//! matching carriers -> `28`. Deficient (same Aq-in-requisites test) halves:
//! `28 / 2 = 14`. **casting total = 14.**
//!
//! Both totals are computed by DIFFERENT functions reading DIFFERENT `Effect`
//! fields (`lab_total_mod` vs `casting_total_mod`/`potent_casting_mod_for`),
//! so the Potent Magic Virtue contributes to each through its own distinct,
//! independently-gated term — not a shared number that could silently leak
//! between the two totals. The barred-combination check
//! (`effective/spell.rs::spell_touches_barred_combination`) is a third,
//! independent function touching neither `Effect` field at all (only
//! `PointItem::unordered_param_groups`/`Spell::requisites`), so this test's
//! real claim is that none of these three independent code paths perturbs
//! another when all three are exercised on one entity/spell at once.

use arm_rules::derived::spell_casting_total;
use arm_rules::ruleset::{Ruleset, RulesetSources};
use arm_rules::types::*;
use arm_rules::validation::{ValidationIssue, validate};
use std::collections::BTreeMap;

/// The shipped core ruleset, point items + a single synthetic spell (same
/// `ruleset_with_only_spell` pattern `requisite_level_cap.rs`/
/// `d81_incompatible_arts.rs` use — no shipped spell is guaranteed to have
/// this exact Technique/Form/requisite shape). Duplicated per those files'
/// own precedent: integration test binaries cannot share private helpers.
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

fn art(id: &str, score: u8) -> ArtScore {
    ArtScore::new(Id::new(id), score)
}

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

fn deficient_form(form: &str) -> Selection {
    Selection::with_params(
        Id::new("flaw.deficient_form"),
        BTreeMap::from([("form".to_string(), Id::new(form))]),
    )
}

fn major_magical_focus(focus: &str) -> Selection {
    Selection::with_params(
        Id::new("virtue.major_magical_focus"),
        BTreeMap::from([("focus".to_string(), Id::new(focus))]),
    )
}

fn potent_magic_major(field: &str) -> Selection {
    Selection::with_params(
        Id::new("virtue.potent_magic_major"),
        BTreeMap::from([("field".to_string(), Id::new(field))]),
    )
}

/// The fully-loaded entity this whole file exercises: a magus holding all
/// four of the above selections, Cr 15 / Ig 12 / Re 5 / Aq 20, and the
/// synthetic spell (level 16 — one above the hand-computed cap of 15) marked
/// both `within_focus` and `within_potent_field`.
fn composite_entity() -> Entity {
    let mut e = Entity::new(
        EntityKind::Character,
        Id::new("magus"),
        RulesetRef::new(Id::new("arm5-core"), "2024.1"),
    );
    e.art_scores = vec![
        art("art.creo", 15),
        art("art.ignem", 12),
        art("art.rego", 5),
        art("art.aquam", 20),
    ];
    e.selections = vec![
        incompatible_arts("art.rego", "art.ignem", "art.muto", "art.herbam"),
        deficient_form("art.aquam"),
        major_magical_focus("test"),
        potent_magic_major("test"),
    ];
    let mut sel = SpellSelection::new(Id::new("spell.test_composite"));
    sel.within_focus = true;
    sel.within_potent_field = true;
    e.spells = vec![sel];
    e
}

const SPELL_JSON: &str = r#"{ "id": "spell.test_composite", "technique": "art.creo",
     "form": "art.ignem", "level": 16, "requisites": ["art.rego", "art.aquam"] }"#;

#[test]
fn level_cap_folds_requisite_focus_potent_lab_mod_and_deficiency_together() {
    let ruleset = ruleset_with_only_spell(SPELL_JSON);
    let e = composite_entity();

    let result = validate(&e, &ruleset);
    let cap_arg = result
        .issues
        .iter()
        .find(|i| {
            i.code == ValidationIssue::CODE_SPELL_LEVEL_EXCEEDS_CAP
                && i.context.as_ref() == Some(&Id::new("spell.test_composite"))
        })
        .map(|i| i.args.get("cap").cloned().expect("cap arg present"));

    assert_eq!(
        cap_arg,
        Some("15".to_string()),
        "hand-computed cap: min(15,5)+min(12,20)+0+0+3+6(Potent Magic lab_total_mod, \
         spell marked within_potent_field, R3) = 26, +5 within_focus = 31, halved for \
         the Deficient Aq requisite -> 15; got issues {:?}",
        result.issues
    );
}

#[test]
fn level_cap_without_the_potent_field_mark_drops_the_potent_bonus() {
    // The same composite with only the `within_potent_field` marker cleared:
    // R3 (after-deadline answer 3, amending D1) — Potent Magic's +6 counts
    // toward a spell's cap only when the spell is marked within its field.
    // min(15,5) + min(12,20) + 0 + 0 + 3 = 20, +5 within_focus = 25, halved
    // for the Deficient Aq requisite -> 12.
    let ruleset = ruleset_with_only_spell(SPELL_JSON);
    let mut e = composite_entity();
    e.spells[0].within_potent_field = false;

    let result = validate(&e, &ruleset);
    let cap_arg = result
        .issues
        .iter()
        .find(|i| {
            i.code == ValidationIssue::CODE_SPELL_LEVEL_EXCEEDS_CAP
                && i.context.as_ref() == Some(&Id::new("spell.test_composite"))
        })
        .map(|i| i.args.get("cap").cloned().expect("cap arg present"));

    assert_eq!(
        cap_arg,
        Some("12".to_string()),
        "hand-computed cap without the Potent mark: 5+12+0+0+3 = 20, +5 within_focus = 25, \
         halved -> 12 (no Potent Magic +6); got issues {:?}",
        result.issues
    );
}

#[test]
fn casting_total_folds_requisite_focus_potent_bonus_and_deficiency_together() {
    let ruleset = ruleset_with_only_spell(SPELL_JSON);
    let e = composite_entity();
    let sel = &e.spells[0];

    let total = spell_casting_total(sel, &e, &ruleset);

    assert_eq!(
        total,
        Some(14),
        "hand-computed casting total: min(15,5)+min(12,20)+0+0+0+0 = 17, +5 \
         within_focus (min of the two folded scores) = 22, +6 within_potent_field \
         (Major Potent Magic, MAX not sum per ArMDE:4742) = 28, halved for the \
         Deficient Aq requisite -> 14; got {total:?}"
    );
}

#[test]
fn incompatible_arts_barred_combination_still_fires_alongside_everything_else() {
    // The barred pair (Re, Ig) is reached ONLY through the spell's Rego
    // requisite — its own primary pair (Cr, Ig) is not one of the two
    // declared combinations — exactly ArMDE:6292's "even if one or both are
    // requisites" (D81.15), and this must still fire with every other marker
    // (focus, potent field, deficiency) also in play.
    let ruleset = ruleset_with_only_spell(SPELL_JSON);
    let e = composite_entity();

    let result = validate(&e, &ruleset);

    assert!(
        result.issues.iter().any(|i| {
            i.code == ValidationIssue::CODE_SPELL_USES_INCOMPATIBLE_ARTS
                && i.context.as_ref() == Some(&Id::new("spell.test_composite"))
        }),
        "D81.15: the barred (Rego, Ignem) combination, reached only through the \
         spell's own Rego requisite, must still be a creation-time error even with \
         Focus/Potent-Magic/Deficiency markers all set on the same spell: {:?}",
        result.issues
    );
}

#[test]
fn entity_still_reports_the_flaw_level_incompatibility() {
    // Honest, not papered over (task instruction): `flaw.incompatible_arts`
    // and `flaw.deficient_form` are mutually `incompatible_with` each other
    // (ArMDE:6292, "may not be combined with a Deficiency") — this combination
    // is reachable (direct-unchecked entry, a hand-edited save) but is a rules
    // VIOLATION the engine must still flag, independently of whichever of the
    // two numeric totals above compose correctly regardless.
    let ruleset = ruleset_with_only_spell(SPELL_JSON);
    let e = composite_entity();

    let result = validate(&e, &ruleset);

    assert!(
        result
            .issues
            .iter()
            .any(|i| i.code == ValidationIssue::CODE_INCOMPATIBLE),
        "ArMDE:6292: Incompatible Arts + a Deficiency on the same character must \
         still be reported as mutually incompatible: {:?}",
        result.issues
    );
}
