//! Conformance of every character template the core rulebook prints — the six
//! **Grog Templates** (ArMDE:1191-1404 `### Grog Templates`), the five
//! **Companion Templates** (ArMDE:1406-1597 `### Companion Templates`) and the
//! twelve **Magus Templates** (ArMDE:1599-2199 `### Magus Templates`), one
//! per House — against this engine.
//!
//! Each template is transcribed into a save fixture under
//! `fixtures/book_templates/` and run through the engine's real load path
//! ([`arm_rules::load_entity_migrating`]) and the shipped `rules/core/*.json`.
//! Two things are then asserted per template: the set of **error**-severity issue
//! codes validation reports, and the derived play stats the book prints beside
//! the statblock (Soak, Fatigue levels, Wound Penalties, Encumbrance, Combat).
//!
//! Every test here is GREEN. Where the engine and the book disagree, the expected
//! value written below is the **engine's actual output**, with a comment naming
//! the disagreement and pointing at its entry in
//! `docs/book-template-conformance.md`. Nothing is relaxed away and no fixture is
//! bent to suit the engine: a disagreement is an explicit, documented expectation
//! so that changing either side fails loudly here.
//!
//! Structural only — no assertion counts a catalogue.

use arm_rules::characteristics::Characteristic;
use arm_rules::derived::{
    CombatLine, WoundRange, casting_totals, combat_totals, encumbrance, fatigue_levels,
    penetration, soak, spell_casting_total, wound_ranges,
};
use arm_rules::ruleset::{Ruleset, RulesetSources};
use arm_rules::types::{Entity, Id, SelectionParamValue};
use arm_rules::validation::validate;
use std::collections::BTreeSet;

/// The shipped core ruleset, loaded exactly as `core_type_conformance.rs` does.
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

/// The shipped catalogue names — CV4's dependency for
/// [`arm_rules::load_entity_migrating`] (design § 5.6), both locales at once.
fn catalogue_names() -> std::collections::BTreeMap<Id, Vec<String>> {
    let ruleset = full_ruleset();
    let en = include_str!("../../../rules/i18n/en/parameter_catalogue.json");
    let de = include_str!("../../../rules/i18n/de/parameter_catalogue.json");
    arm_rules::load_catalogue_names(ruleset.parameter_catalogues(), en, de)
        .expect("catalogue names load")
}

/// Parses a fixture through the engine's real save-load path.
fn load(json: &str) -> Entity {
    arm_rules::load_entity_migrating(
        json,
        arm_rules::validation::DEFAULT_SAGA_YEAR,
        &full_ruleset(),
        &catalogue_names(),
    )
    .expect("book template fixture parses")
    .entity
}

/// The set of error-severity issue codes validation reports, deduplicated — two
/// Abilities tripping one rule are one disagreement, not two.
fn error_codes(entity: &Entity, ruleset: &Ruleset) -> BTreeSet<String> {
    validate(entity, ruleset)
        .errors()
        .map(|issue| issue.code.clone())
        .collect()
}

/// Builds an expected code set from a literal list, so a test reads as a set.
fn codes(list: &[&str]) -> BTreeSet<String> {
    list.iter().map(|c| (*c).to_string()).collect()
}

/// The set of warning-severity issue codes, deduplicated. Asserted alongside the
/// errors because two of the disagreements recorded in
/// `docs/book-template-conformance.md` are advisory rather than blocking, and a
/// documented disagreement nothing pins would rot silently.
fn warning_codes(entity: &Entity, ruleset: &Ruleset) -> BTreeSet<String> {
    validate(entity, ruleset)
        .warnings()
        .map(|issue| issue.code.clone())
        .collect()
}

/// One way of wielding a weapon: `with_shield` picks between the two lines the
/// engine emits for a one-handed weapon carried alongside a shield. Always
/// picks the UNMOUNTED line — mounted twins (K3) are appended after the loop
/// in `combat_totals`, so the on-foot line this helper has always found stays
/// the first match even once twins exist.
fn line<'a>(lines: &'a [CombatLine], weapon: &str, with_shield: bool) -> &'a CombatLine {
    lines
        .iter()
        .find(|l| l.weapon.as_str() == weapon && !l.shields.is_empty() == with_shield && !l.mounted)
        .unwrap_or_else(|| panic!("no {weapon} line (with_shield={with_shield}) in {lines:#?}"))
}

/// The mounted twin (K3) of the line [`line`] would find: same predicate, plus
/// `mounted: true`.
fn mounted_line<'a>(lines: &'a [CombatLine], weapon: &str, with_shield: bool) -> &'a CombatLine {
    lines
        .iter()
        .find(|l| l.weapon.as_str() == weapon && !l.shields.is_empty() == with_shield && l.mounted)
        .unwrap_or_else(|| {
            panic!("no mounted {weapon} line (with_shield={with_shield}) in {lines:#?}")
        })
}

/// Init / Attack / Defense / Damage, in the order the book's Combat rows print.
fn stats(line: &CombatLine) -> (i32, Option<i32>, i32, Option<i32>) {
    (line.initiative, line.attack, line.defense, line.damage)
}

/// The five Fatigue-level penalties, in order. Every grog and companion template
/// prints "OK, 0, -1, -3, -5, Unconscious" (e.g. ArMDE:1217 `#### The
/// Berserker`, :1435 `#### The Female Scholar`); the engine
/// omits Unconscious, which is a state rather than an action penalty.
const BOOK_FATIGUE_PENALTIES: [i32; 5] = [0, 0, -1, -3, -5];

/// The inclusive damage bounds of each wound band, for comparison with the
/// book's "Wound Penalties" row.
fn wound_bounds(ranges: &[WoundRange]) -> Vec<(i32, Option<i32>)> {
    ranges.iter().map(|r| (r.min, r.max)).collect()
}

/// Wound bands for a Size 0 character: -1 (1-5), -3 (6-10), -5 (11-15),
/// Incapacitated (16-20), Dead (21+). Source: ArMDE:1255 `#### The Grizzled
/// Veteran`.
fn size_zero_wound_bounds() -> Vec<(i32, Option<i32>)> {
    vec![
        (1, Some(5)),
        (6, Some(10)),
        (11, Some(15)),
        (16, Some(20)),
        (21, None),
    ]
}

/// Wound bands for a Size +1 character: -1 (1-6), -3 (7-12), -5 (13-18),
/// Incapacitated (19-24), Dead (25+). Source: ArMDE:1219 `#### The Berserker`.
fn size_one_wound_bounds() -> Vec<(i32, Option<i32>)> {
    vec![
        (1, Some(6)),
        (7, Some(12)),
        (13, Some(18)),
        (19, Some(24)),
        (25, None),
    ]
}

/// Wound bands for a Size +2 character: -1 (1-7), -3 (8-14), -5 (15-21),
/// Incapacitated (22-28), Dead (29+). Source: ArMDE:1780 `#### Ex
/// Miscellanea`.
fn size_two_wound_bounds() -> Vec<(i32, Option<i32>)> {
    vec![
        (1, Some(7)),
        (8, Some(14)),
        (15, Some(21)),
        (22, Some(28)),
        (29, None),
    ]
}

/// Wound bands for a Size -2 character: -1 (1-3), -3 (4-6), -5 (7-9),
/// Incapacitated (10-12), Dead (13+). Source: ArMDE:2178 `#### Verditius`.
fn size_minus_two_wound_bounds() -> Vec<(i32, Option<i32>)> {
    vec![
        (1, Some(3)),
        (4, Some(6)),
        (7, Some(9)),
        (10, Some(12)),
        (13, None),
    ]
}

/// The effective score of one Art — the single figure the book's Arts line
/// prints, with a Puissant Art bonus already folded in (`Te 12+3` → 15) and any
/// Elemental Magic redistribution applied. Source: ArMDE:1179 `### Format`.
fn art_score(entity: &Entity, ruleset: &Ruleset, art: &str) -> i32 {
    arm_rules::effective::effective_art_score(entity, ruleset, &Id::new(art))
}

/// The effective score of one Ability instance, `X+Y` likewise folded into one
/// figure. `parameter` names the instance of a parameterized Ability (Craft
/// (metalsmith)); `None` for a plain one. Source: ArMDE:1177 `### Format`.
fn ability_score(
    entity: &Entity,
    ruleset: &Ruleset,
    ability: &str,
    parameter: Option<&str>,
) -> i32 {
    arm_rules::effective::effective_ability_score(entity, ruleset, &Id::new(ability), parameter)
}

/// The Casting Total the book prints beside each spell — `Spell (TeFo level/+N)`,
/// ArMDE:1187 `### Format` — paired with its within-a-Magical-Focus counterpart.
///
/// The two come from different engine read-outs on purpose. `penetration` is
/// where the engine computes one Casting Total **per known spell**, which is the
/// shape the book prints; `casting_totals` computes the `(Technique, Form)` grid,
/// which is the only place the within-focus figure exists. Nothing in a save
/// records whether a given spell falls inside a free-text Magical Focus, so the
/// engine offers both figures and the book picks one per spell — see the Flambeau
/// and Mercere entries in `docs/book-template-conformance.md`.
///
/// The focused figure is `None` for a magus with no Magical Focus.
fn spell_casting(entity: &Entity, ruleset: &Ruleset, spell: &str) -> (i32, Option<i32>) {
    let id = Id::new(spell);
    let base = penetration(entity, ruleset)
        .into_iter()
        .find(|line| line.spell == id)
        .unwrap_or_else(|| panic!("{spell} is not a known spell of this fixture"))
        .casting_total;
    let definition = ruleset
        .spell(&id)
        .unwrap_or_else(|| panic!("{spell} is not in the spell catalogue"));
    let focused = casting_totals(entity, ruleset)
        .into_iter()
        .find(|cell| cell.technique == definition.technique && cell.form == definition.form)
        .and_then(|cell| cell.within_focus.map(|focus| focus.formulaic));
    (base, focused)
}

/// The ONE Casting Total the book prints beside a known spell — X10c's
/// per-spell selector (`SpellSelection::within_focus`), rather than the two
/// candidate figures [`spell_casting`] reports. Use this once a fixture's
/// spell carries an explicit `within_focus` marker; [`spell_casting`] stays
/// for fixtures where nothing adjudicates (no Focus, or MAG8's unresolved
/// capability gap).
fn printed_casting(entity: &Entity, ruleset: &Ruleset, spell: &str) -> i32 {
    let id = Id::new(spell);
    let chosen = entity
        .spells
        .iter()
        .find(|s| s.spell == id)
        .unwrap_or_else(|| panic!("{spell} is not a known spell of this fixture"));
    spell_casting_total(chosen, entity, ruleset)
        .unwrap_or_else(|| panic!("{spell} has no Casting Total"))
}

// --- Bjornaer (ArMDE:1603-1652 `#### Bjornaer`) -----------------------------

#[test]
fn the_bjornaer_matches_the_book() {
    let ruleset = full_ruleset();
    let bjornaer = load(include_str!("fixtures/book_templates/magus_bjornaer.json"));

    assert_eq!(error_codes(&bjornaer, &ruleset), codes(&[]));
    assert_eq!(warning_codes(&bjornaer, &ruleset), codes(&[]));

    // Arts: Mu 10, Pe 3, In 1, Re 1, An 8, Co 8, Ig 0 (ArMDE:1634 `#### Bjornaer`).
    assert_eq!(art_score(&bjornaer, &ruleset, "art.muto"), 10);
    assert_eq!(art_score(&bjornaer, &ruleset, "art.perdo"), 3);
    assert_eq!(art_score(&bjornaer, &ruleset, "art.intellego"), 1);
    assert_eq!(art_score(&bjornaer, &ruleset, "art.rego"), 1);
    assert_eq!(art_score(&bjornaer, &ruleset, "art.animal"), 8);
    assert_eq!(art_score(&bjornaer, &ruleset, "art.corpus"), 8);
    assert_eq!(art_score(&bjornaer, &ruleset, "art.ignem"), 0);

    // "Heartbeast 2" (ArMDE:1632 `#### Bjornaer`) — the House Virtue's free first point is a
    // floor, so the bought 2 stands.
    assert_eq!(
        ability_score(&bjornaer, &ruleset, "ability.heartbeast", None),
        2
    );

    // Soak: +1 (ArMDE:1626 `#### Bjornaer`).
    assert_eq!(soak(&bjornaer, &ruleset).total, 1);

    let fatigue: Vec<i32> = fatigue_levels(&bjornaer, &ruleset)
        .iter()
        .map(|l| l.penalty)
        .collect();
    assert_eq!(fatigue, BOOK_FATIGUE_PENALTIES);

    assert_eq!(
        wound_bounds(&wound_ranges(&bjornaer, &ruleset)),
        size_zero_wound_bounds()
    );

    // Encumbrance: 0 (0). Source: ArMDE:1640 `#### Bjornaer`.
    let enc = encumbrance(&bjornaer, &ruleset);
    assert_eq!((enc.burden, enc.total), (0, 0));

    // Dodging: Init +1, Atk n/a, Def +4, Dam n/a. Source: ArMDE:1624 `#### Bjornaer`.
    let lines = combat_totals(&bjornaer, &ruleset);
    assert_eq!(
        stats(line(&lines, "weapon.dodge", false)),
        (1, None, 4, None)
    );

    // F2 (design-f0-book-template-engine.md § 1a/§ 2c, D61) — RED CHECKPOINT:
    // DISAGREEMENT MAG1 is resolved by deleting Ways of the Land's
    // `casting_total_mod` and reclassifying it to `uncomputed_rule` — no
    // template ever prints the boosted figure. The book prints MuAn +19,
    // PeAn +12, ReAn +10 and the Corpus mirror of each (ArMDE:1643-1650
    // `#### Bjornaer`) —
    // Technique + Form + Stamina, with nothing else. No Magical Focus, so no
    // focused figure. `rules/core/virtues_flaws.json` has not been edited yet,
    // so the engine still adds the unconditional +3 until it is.
    let casting = |spell: &str| spell_casting(&bjornaer, &ruleset, spell);
    assert_eq!(
        casting("spell.transformation_of_the_ravenous_beast_to_the_torpid_toad"),
        (19, None)
    );
    assert_eq!(casting("spell.agony_of_the_beast"), (12, None));
    assert_eq!(casting("spell.circle_of_beast_warding"), (10, None));
    assert_eq!(casting("spell.vipers_gaze"), (10, None));
    assert_eq!(casting("spell.eyes_of_the_cat"), (19, None));
    assert_eq!(casting("spell.gift_of_the_bears_fortitude"), (19, None));
    assert_eq!(casting("spell.the_wound_that_weeps"), (12, None));
    assert_eq!(casting("spell.lifting_the_dangling_puppet"), (10, None));
}

// --- Bonisagus (ArMDE:1654-1700 `#### Bonisagus`) ---------------------------

#[test]
fn the_bonisagus_matches_the_book() {
    let ruleset = full_ruleset();
    let bonisagus = load(include_str!("fixtures/book_templates/magus_bonisagus.json"));

    assert_eq!(error_codes(&bonisagus, &ruleset), codes(&[]));
    assert_eq!(warning_codes(&bonisagus, &ruleset), codes(&[]));

    // Int +5 (ArMDE:1656 `#### Bonisagus`) is a bought +3 lifted by Great Intelligence twice.
    assert_eq!(
        arm_rules::effective::effective_characteristic_score(
            &bonisagus,
            &ruleset,
            Characteristic::Int
        ),
        5
    );

    // Arts: Cr 12, Re 3, Au 12, Co 4 (ArMDE:1685 `#### Bonisagus`).
    assert_eq!(art_score(&bonisagus, &ruleset, "art.creo"), 12);
    assert_eq!(art_score(&bonisagus, &ruleset, "art.rego"), 3);
    assert_eq!(art_score(&bonisagus, &ruleset, "art.auram"), 12);
    assert_eq!(art_score(&bonisagus, &ruleset, "art.corpus"), 4);

    // "Magic Theory 4+2" (ArMDE:1683 `#### Bonisagus`) — the House's free Puissant Magic Theory.
    assert_eq!(
        ability_score(&bonisagus, &ruleset, "ability.magic_theory", None),
        6
    );

    // Soak: 0 (ArMDE:1677 `#### Bonisagus`).
    assert_eq!(soak(&bonisagus, &ruleset).total, 0);

    let fatigue: Vec<i32> = fatigue_levels(&bonisagus, &ruleset)
        .iter()
        .map(|l| l.penalty)
        .collect();
    assert_eq!(fatigue, BOOK_FATIGUE_PENALTIES);

    assert_eq!(
        wound_bounds(&wound_ranges(&bonisagus, &ruleset)),
        size_zero_wound_bounds()
    );

    // Encumbrance: 0 (0). Source: ArMDE:1691 `#### Bonisagus`.
    let enc = encumbrance(&bonisagus, &ruleset);
    assert_eq!((enc.burden, enc.total), (0, 0));

    // Dodging: Init +0, Atk n/a, Def +0, Dam n/a. Source: ArMDE:1675 `#### Bonisagus`.
    let lines = combat_totals(&bonisagus, &ruleset);
    assert_eq!(
        stats(line(&lines, "weapon.dodge", false)),
        (0, None, 0, None)
    );

    // Casting Totals, ArMDE:1694-1698 `#### Bonisagus`: CrAu +24 (12 + 12 + Sta 0) on all four
    // Auram spells, CrCo +16.
    let casting = |spell: &str| spell_casting(&bonisagus, &ruleset, spell);
    assert_eq!(casting("spell.charge_of_the_angry_winds"), (24, None));
    assert_eq!(casting("spell.clouds_of_rain_and_thunder"), (24, None));
    assert_eq!(casting("spell.clouds_of_summer_snow"), (24, None));
    assert_eq!(casting("spell.the_incantation_of_lightning"), (24, None));
    assert_eq!(
        casting("spell.purification_of_the_festering_wounds"),
        (16, None)
    );
}

// --- Criamon (ArMDE:1702-1750 `#### Criamon`) -------------------------------

#[test]
fn the_criamon_matches_the_book() {
    let ruleset = full_ruleset();
    let criamon = load(include_str!("fixtures/book_templates/magus_criamon.json"));

    assert_eq!(error_codes(&criamon, &ruleset), codes(&[]));
    // DISAGREEMENT MAG2 (docs/book-template-conformance.md). The book lists seven
    // spells totalling the magus budget of 120 levels, but one of them —
    // "Piercing the Magical Veil" (ArMDE:1745 `#### Criamon`) — is a spell `rules/core/spells.json`
    // does not contain: the template links it to `#piercing-the-faerie-veil` and
    // adds "(see Piercing the Faerie Veil)", so the extraction folded the two into
    // the single `spell.piercing_the_faerie_veil`. The fixture can therefore carry
    // only six spells, 100 of 120 levels, and the engine says so.
    assert_eq!(
        warning_codes(&criamon, &ruleset),
        codes(&["spell_levels_unspent"])
    );

    // Arts: Cr 4, In 6, Mu 4, Pe 4, Re 4, Im 2, Me 1, Vi 10 (ArMDE:1733 `#### Criamon`).
    assert_eq!(art_score(&criamon, &ruleset, "art.creo"), 4);
    assert_eq!(art_score(&criamon, &ruleset, "art.intellego"), 6);
    assert_eq!(art_score(&criamon, &ruleset, "art.muto"), 4);
    assert_eq!(art_score(&criamon, &ruleset, "art.perdo"), 4);
    assert_eq!(art_score(&criamon, &ruleset, "art.rego"), 4);
    assert_eq!(art_score(&criamon, &ruleset, "art.imaginem"), 2);
    assert_eq!(art_score(&criamon, &ruleset, "art.mentem"), 1);
    assert_eq!(art_score(&criamon, &ruleset, "art.vim"), 10);

    // "Enigmatic Wisdom 3+2" (ArMDE:1731 `#### Criamon`) — Puissant Enigmatic Wisdom on a
    // bought 3, whose own first point the House Virtue already pays for.
    assert_eq!(
        ability_score(&criamon, &ruleset, "ability.enigmatic_wisdom", None),
        5
    );

    // Soak: +2 (ArMDE:1725 `#### Criamon`).
    assert_eq!(soak(&criamon, &ruleset).total, 2);

    let fatigue: Vec<i32> = fatigue_levels(&criamon, &ruleset)
        .iter()
        .map(|l| l.penalty)
        .collect();
    assert_eq!(fatigue, BOOK_FATIGUE_PENALTIES);

    assert_eq!(
        wound_bounds(&wound_ranges(&criamon, &ruleset)),
        size_zero_wound_bounds()
    );

    // Encumbrance: 0 (0). Source: ArMDE:1739 `#### Criamon`.
    let enc = encumbrance(&criamon, &ruleset);
    assert_eq!((enc.burden, enc.total), (0, 0));

    // Dodging: Init +1, Atk n/a, Def +1, Dam n/a. Source: ArMDE:1723 `#### Criamon`.
    let lines = combat_totals(&criamon, &ruleset);
    assert_eq!(
        stats(line(&lines, "weapon.dodge", false)),
        (1, None, 1, None)
    );

    // Casting Totals, ArMDE:1742-1748 `#### Criamon`.
    let casting = |spell: &str| spell_casting(&criamon, &ruleset, spell);
    assert_eq!(casting("spell.phantasm_of_the_talking_head"), (8, None));
    assert_eq!(casting("spell.aura_of_ennobled_presence"), (8, None));
    assert_eq!(casting("spell.piercing_the_faerie_veil"), (18, None));
    assert_eq!(casting("spell.unravelling_the_fabric_of_form"), (16, None));
    assert_eq!(casting("spell.wind_of_mundane_silence"), (16, None));
    assert_eq!(casting("spell.circular_ward_against_demons"), (16, None));
}

/// X10(d) target shape (`docs/vf-audit/decisions.md` D65 row N5,
/// `tmp/x10-verdicts.md`). MAG2 (`docs/book-template-conformance.md`) is resolved
/// by adding `spell.piercing_the_magical_veil` to `rules/core/spells.json` — In Vi
/// 20, the Technique/Form/level *and* Casting Total the book itself prints for it,
/// "Piercing the Magical Veil (InVi 20/+18)" (`ArMDE:1745` `#### Criamon`),
/// distinct from its Faerie-veil sibling that same line links to and from
/// which the extraction folded it (see MAG2's full account and
/// `ArMDE:15709` `#### LEVEL 20`, which only *names* the
/// Magical/Divine/Infernal siblings without printing Range/Duration/Target for
/// any of them — a gap `tmp/x10-verdicts.md` records and does not invent a value
/// for). Once that entry ships, this fixture — the book's own seven spells,
/// unlike `magus_criamon.json`'s six — reaches the printed 120/120 exactly. RED
/// until the spell exists in the catalogue.
#[test]
fn the_criamon_reaches_120_spell_levels_once_piercing_the_magical_veil_exists() {
    let ruleset = full_ruleset();
    let criamon = load(include_str!(
        "fixtures/book_templates/magus_criamon_full.json"
    ));

    assert_eq!(error_codes(&criamon, &ruleset), codes(&[]));
    assert_eq!(warning_codes(&criamon, &ruleset), codes(&[]));

    // "Piercing the Magical Veil" (InVi 20/+18). Source: ArMDE:1745 `#### Criamon`.
    let casting = |spell: &str| spell_casting(&criamon, &ruleset, spell);
    assert_eq!(casting("spell.piercing_the_magical_veil"), (18, None));
}

// --- Ex Miscellanea (ArMDE:1752-1801 `#### Ex Miscellanea`) -----------------

#[test]
fn the_ex_miscellanea_matches_the_book() {
    let ruleset = full_ruleset();
    let ex_misc = load(include_str!(
        "fixtures/book_templates/magus_ex_miscellanea.json"
    ));

    assert_eq!(error_codes(&ex_misc, &ruleset), codes(&[]));
    assert_eq!(warning_codes(&ex_misc, &ruleset), codes(&[]));

    // Str +4, Sta +4 (ArMDE:1754 `#### Ex Miscellanea`) are a bought +3 each plus Giant Blood's +1.
    assert_eq!(
        arm_rules::effective::effective_characteristic_score(
            &ex_misc,
            &ruleset,
            Characteristic::Str
        ),
        4
    );
    assert_eq!(
        arm_rules::effective::effective_characteristic_score(
            &ex_misc,
            &ruleset,
            Characteristic::Sta
        ),
        4
    );

    // Arts: Cr 8, Mu 4, Pe 3, Re 5, Co 1, Te 12+3 (ArMDE:1784 `#### Ex Miscellanea`).
    assert_eq!(art_score(&ex_misc, &ruleset, "art.creo"), 8);
    assert_eq!(art_score(&ex_misc, &ruleset, "art.muto"), 4);
    assert_eq!(art_score(&ex_misc, &ruleset, "art.perdo"), 3);
    assert_eq!(art_score(&ex_misc, &ruleset, "art.rego"), 5);
    assert_eq!(art_score(&ex_misc, &ruleset, "art.corpus"), 1);
    assert_eq!(art_score(&ex_misc, &ruleset, "art.terram"), 15);

    // Soak: +7 (Stamina +4, Tough +3). Source: ArMDE:1776 `#### Ex Miscellanea`.
    assert_eq!(soak(&ex_misc, &ruleset).total, 7);

    let fatigue: Vec<i32> = fatigue_levels(&ex_misc, &ruleset)
        .iter()
        .map(|l| l.penalty)
        .collect();
    assert_eq!(fatigue, BOOK_FATIGUE_PENALTIES);

    assert_eq!(
        wound_bounds(&wound_ranges(&ex_misc, &ruleset)),
        size_two_wound_bounds()
    );

    // Encumbrance: 0 (0). Source: ArMDE:1790 `#### Ex Miscellanea`.
    let enc = encumbrance(&ex_misc, &ruleset);
    assert_eq!((enc.burden, enc.total), (0, 0));

    // Dodging: Init -2, Atk n/a, Def +1, Dam n/a. Source: ArMDE:1773 `#### Ex Miscellanea`.
    // Grappling: Init -2, Attack +2, Defense +2, Damage n/a. Source: ArMDE:1774 `#### Ex Miscellanea`.
    // X10(d) target shape (docs/vf-audit/decisions.md D65 row N5,
    // tmp/x10-verdicts.md). MAG5 (docs/book-template-conformance.md) is resolved
    // by a zero-Load `weapon.grapple` in `rules/core/equipment.json` (Ability
    // Brawl, Init/Attack/Defense mods 0, no Damage) — the figures the bestiary's
    // own Natural Weapons Table gives for "Grapple" (ArMDE:18561 `### Natural
    // Weapons Table`), which the book itself says use "Combat Statistics ...
    // calculated as normal" (ArMDE:18551 `### 6. Combat Statistics`),
    // i.e. the same formula `derived/combat.rs::combat_totals` already applies to
    // a magus. Run through that formula with this magus's own Brawl 3
    // (grappling) — a specialty aligned to the fixture's grapple slot — and her
    // Dex/Qik -2, it reproduces her printed Grappling row exactly. RED until that
    // row ships.
    let lines = combat_totals(&ex_misc, &ruleset);
    let weapon_ids: BTreeSet<&str> = lines.iter().map(|l| l.weapon.as_str()).collect();
    assert_eq!(
        weapon_ids,
        BTreeSet::from(["weapon.dodge", "weapon.grapple"])
    );
    assert_eq!(
        stats(line(&lines, "weapon.dodge", false)),
        (-2, None, 1, None)
    );
    assert_eq!(
        stats(line(&lines, "weapon.grapple", false)),
        (-2, Some(2), 2, None)
    );

    // Casting Totals, ArMDE:1793-1799 `#### Ex Miscellanea`. Major Magical Focus (stone): the book
    // prints the within-focus figure for four spells (CrTe and MuTe cells) and
    // the base figure for one (PeTe) — `within_focus` on each spell in
    // `magus_ex_miscellanea.json` (X10c) now records exactly that. RED until
    // X10c's engine change wires the selector into `spell_casting_total`
    // (today it always returns the base/grid figure regardless of the flag).
    let casting = |spell: &str| printed_casting(&ex_misc, &ruleset, spell);
    assert_eq!(casting("spell.wall_of_protecting_stone"), 35);
    assert_eq!(casting("spell.the_crystal_dart"), 27);
    assert_eq!(casting("spell.rock_of_viscid_clay"), 27);
    assert_eq!(casting("spell.earth_that_breaks_no_more"), 27);
    assert_eq!(casting("spell.obliteration_of_the_metallic_barrier"), 22);
    // DISAGREEMENT MAG4 — RESOLVED (X11, requisite folding). Both spells are
    // Re(Mu)Te 15 with the SAME Muto requisite, on the same magus (Re 5, Te
    // 15, Mu 4, Sta +4). ArMDE:12309 ("You must use the lesser of your score
    // in the requisite and your score in the spell's main Technique"): Mu 4 <
    // Re 5, so the effective Technique folds to 4: 4 + 15 + 4 = 23 base. The
    // Earth's Carbuncle ("ground [becomes] jagged stone", ArMDE:15540) is
    // stone-themed and sits within the Major Magical Focus (stone), 23 + focus
    // (min(4, 15) = 4) = 27; Hands of the Grasping Earth ("Earthen hands...
    // rise out of the ground", ArMDE:15546) is earth/soil, not stone, same
    // distinction already drawn for `obliteration_of_the_metallic_barrier`
    // above ("metal, not stone"), so it stays at the base 23. Both now match
    // the book exactly (`ArMDE:1798-1799`): +27 and +23. The previous "(c) the
    // book" verdict was wrong — the engine was missing the fold, not the book.
    assert_eq!(casting("spell.the_earths_carbuncle"), 27);
    assert_eq!(casting("spell.hands_of_the_grasping_earth"), 23);
}

// --- Flambeau (ArMDE:1803-1849 `#### Flambeau`) -----------------------------

#[test]
fn the_flambeau_matches_the_book() {
    let ruleset = full_ruleset();
    let flambeau = load(include_str!("fixtures/book_templates/magus_flambeau.json"));

    assert_eq!(error_codes(&flambeau, &ruleset), codes(&[]));
    assert_eq!(warning_codes(&flambeau, &ruleset), codes(&[]));

    // Arts: Cr 12, Pe 4, Re 5, Ig 12+3, Te 1 (ArMDE:1834 `#### Flambeau`).
    assert_eq!(art_score(&flambeau, &ruleset, "art.creo"), 12);
    assert_eq!(art_score(&flambeau, &ruleset, "art.perdo"), 4);
    assert_eq!(art_score(&flambeau, &ruleset, "art.rego"), 5);
    assert_eq!(art_score(&flambeau, &ruleset, "art.ignem"), 15);
    assert_eq!(art_score(&flambeau, &ruleset, "art.terram"), 1);

    // Soak: +2 (ArMDE:1826 `#### Flambeau`).
    assert_eq!(soak(&flambeau, &ruleset).total, 2);

    let fatigue: Vec<i32> = fatigue_levels(&flambeau, &ruleset)
        .iter()
        .map(|l| l.penalty)
        .collect();
    assert_eq!(fatigue, BOOK_FATIGUE_PENALTIES);

    assert_eq!(
        wound_bounds(&wound_ranges(&flambeau, &ruleset)),
        size_zero_wound_bounds()
    );

    // Encumbrance: 0 (0). Source: ArMDE:1840 `#### Flambeau`.
    let enc = encumbrance(&flambeau, &ruleset);
    assert_eq!((enc.burden, enc.total), (0, 0));

    // Dodging: Init +1, Atk n/a, Def +4, Dam n/a. Source: ArMDE:1824 `#### Flambeau`.
    let lines = combat_totals(&flambeau, &ruleset);
    assert_eq!(
        stats(line(&lines, "weapon.dodge", false)),
        (1, None, 4, None)
    );

    // Casting Totals, ArMDE:1843-1847 `#### Flambeau`. Every spell is a flame and so inside the
    // Major Magical Focus: Cr 12 + Ig 15 + Sta +2 = 29 base, + min(12, 15) = 41.
    // The book prints the focused figure for all five — MAG8 in
    // docs/book-template-conformance.md is why this used to need reporting
    // both candidates; `within_focus: true` on every spell in
    // `magus_flambeau.json` (X10c) now picks the one the book prints. RED
    // until X10c's engine change wires the selector into
    // `spell_casting_total`.
    let casting = |spell: &str| printed_casting(&flambeau, &ruleset, spell);
    assert_eq!(casting("spell.palm_of_flame"), 41);
    assert_eq!(casting("spell.pilum_of_fire"), 41);
    assert_eq!(casting("spell.arc_of_fiery_ribbons"), 41);
    assert_eq!(casting("spell.ball_of_abysmal_flame"), 41);
    assert_eq!(casting("spell.circle_of_encompassing_flames"), 41);
}

// --- Guernicus (ArMDE:1851-1897 `#### Guernicus`) ---------------------------

#[test]
fn the_guernicus_matches_the_book() {
    let ruleset = full_ruleset();
    let guernicus = load(include_str!("fixtures/book_templates/magus_guernicus.json"));

    // DISAGREEMENT MAG12 (docs/book-template-conformance.md), TARGETED FOR
    // RESOLUTION by X10b (not yet resolved — see the RED note below):
    // the book prints "In 12+3 (5)" (ArMDE:1881 `#### Guernicus`) — score 12,
    // Puissant Art +3, and 5 XP already banked toward Intellego 13.
    // `art.intellego` now carries `"banked_xp": 5` and `xp_pool` is the age
    // formula's true 435 (not the old 432 that silently absorbed the missing 3
    // charged XP by underfunding the pool to match). RED until X10b's engine
    // change folds `banked_xp` into the raw table total before `charged_cost`
    // (`effective/xp.rs`'s Ability/Art spend loops) — today the 5 banked
    // points are still inert data, so the fixture's true 435-point pool reads
    // as 3 XP left unspent (`general_xp_unspent`) rather than exact.
    assert_eq!(error_codes(&guernicus, &ruleset), codes(&[]));
    assert_eq!(warning_codes(&guernicus, &ruleset), codes(&[]));

    // Reputation from Hermetic Prestige. DISAGREEMENT MAG11
    // (docs/book-template-conformance.md), and the FIRST witness to it: the
    // Virtue's own entry says level **4** (ArMDE:4071-4073 `#### Hermetic
    // Prestige`), but this very template prints "Quaesitor (Hermetic) 3"
    // (ArMDE:1869 `#### Guernicus`) — the same "3" the Darius worked example repeats
    // (ArMDE:2518 `### Reputations`, the SECOND witness, `darius_of_flambeau_at_gauntlet_matches_the_book`
    // below). D11/Q5 makes score enforced and exact by default, so a stored 3
    // under a grant of exactly 4 no longer validates (the empty error set above
    // already proves it): the fixture is corrected to 4, following the Virtue's
    // own definition, per MAG11's precedence ruling and matching Darius's
    // existing choice.
    assert_eq!(guernicus.reputations.len(), 1);
    assert_eq!(guernicus.reputations[0].score, 4);

    // Per +4 (ArMDE:1853 `#### Guernicus`) is a bought +3 lifted by Great Perception.
    assert_eq!(
        arm_rules::effective::effective_characteristic_score(
            &guernicus,
            &ruleset,
            Characteristic::Per
        ),
        4
    );

    // Arts: In 12+3, Pe 2, Co 5, Im 6, Me 6 (ArMDE:1881 `#### Guernicus`).
    assert_eq!(art_score(&guernicus, &ruleset, "art.intellego"), 15);
    assert_eq!(art_score(&guernicus, &ruleset, "art.perdo"), 2);
    assert_eq!(art_score(&guernicus, &ruleset, "art.corpus"), 5);
    assert_eq!(art_score(&guernicus, &ruleset, "art.imaginem"), 6);
    assert_eq!(art_score(&guernicus, &ruleset, "art.mentem"), 6);

    // Soak: +0 (ArMDE:1873 `#### Guernicus`).
    assert_eq!(soak(&guernicus, &ruleset).total, 0);

    let fatigue: Vec<i32> = fatigue_levels(&guernicus, &ruleset)
        .iter()
        .map(|l| l.penalty)
        .collect();
    assert_eq!(fatigue, BOOK_FATIGUE_PENALTIES);

    assert_eq!(
        wound_bounds(&wound_ranges(&guernicus, &ruleset)),
        size_zero_wound_bounds()
    );

    // Encumbrance: 0 (0). Source: ArMDE:1887 `#### Guernicus`.
    let enc = encumbrance(&guernicus, &ruleset);
    assert_eq!((enc.burden, enc.total), (0, 0));

    // Dodging: Init +0, Atk n/a, Def +2, Dam n/a. Source: ArMDE:1871 `#### Guernicus`.
    let lines = combat_totals(&guernicus, &ruleset);
    assert_eq!(
        stats(line(&lines, "weapon.dodge", false)),
        (0, None, 2, None)
    );

    // Casting Totals, ArMDE:1890-1895 `#### Guernicus`.
    let casting = |spell: &str| spell_casting(&guernicus, &ruleset, spell);
    assert_eq!(casting("spell.physicians_eye"), (20, None));
    assert_eq!(casting("spell.eyes_of_the_eagle"), (21, None));
    assert_eq!(casting("spell.summoning_the_distant_image"), (21, None));
    assert_eq!(
        casting("spell.invisibility_of_the_standing_wizard"),
        (8, None)
    );
    assert_eq!(casting("spell.frosty_breath_of_the_spoken_lie"), (21, None));
    assert_eq!(casting("spell.peering_into_the_mortal_mind"), (21, None));
}

// --- Jerbiton (ArMDE:1899-1950 `#### Jerbiton`) -----------------------------

#[test]
fn the_jerbiton_matches_the_book() {
    let ruleset = full_ruleset();
    let jerbiton = load(include_str!("fixtures/book_templates/magus_jerbiton.json"));

    assert_eq!(error_codes(&jerbiton, &ruleset), codes(&[]));
    assert_eq!(warning_codes(&jerbiton, &ruleset), codes(&[]));

    // Arts: Cr 6, In 1, Mu 6, Pe 1, Re 6, Co 5, Im 10 (ArMDE:1930 `#### Jerbiton`).
    assert_eq!(art_score(&jerbiton, &ruleset, "art.creo"), 6);
    assert_eq!(art_score(&jerbiton, &ruleset, "art.intellego"), 1);
    assert_eq!(art_score(&jerbiton, &ruleset, "art.muto"), 6);
    assert_eq!(art_score(&jerbiton, &ruleset, "art.perdo"), 1);
    assert_eq!(art_score(&jerbiton, &ruleset, "art.rego"), 6);
    assert_eq!(art_score(&jerbiton, &ruleset, "art.corpus"), 5);
    assert_eq!(art_score(&jerbiton, &ruleset, "art.imaginem"), 10);

    // "Music 4+2" (ArMDE:1928 `#### Jerbiton`) — the House's free Minor Virtue, spent on
    // Puissant Music (ArMDE:1950 `#### Jerbiton`).
    assert_eq!(ability_score(&jerbiton, &ruleset, "ability.music", None), 6);

    // Soak: +0 (ArMDE:1922 `#### Jerbiton`).
    assert_eq!(soak(&jerbiton, &ruleset).total, 0);

    let fatigue: Vec<i32> = fatigue_levels(&jerbiton, &ruleset)
        .iter()
        .map(|l| l.penalty)
        .collect();
    assert_eq!(fatigue, BOOK_FATIGUE_PENALTIES);

    assert_eq!(
        wound_bounds(&wound_ranges(&jerbiton, &ruleset)),
        size_zero_wound_bounds()
    );

    // Encumbrance: 0 (0). Source: ArMDE:1936 `#### Jerbiton`.
    let enc = encumbrance(&jerbiton, &ruleset);
    assert_eq!((enc.burden, enc.total), (0, 0));

    // Dodging: Init +0, Atk n/a, Def +0, Dam n/a. Source: ArMDE:1920 `#### Jerbiton`.
    let lines = combat_totals(&jerbiton, &ruleset);
    assert_eq!(
        stats(line(&lines, "weapon.dodge", false)),
        (0, None, 0, None)
    );

    // Casting Totals, ArMDE:1939-1948 `#### Jerbiton`.
    let casting = |spell: &str| spell_casting(&jerbiton, &ruleset, spell);
    assert_eq!(casting("spell.phantasm_of_the_talking_head"), (16, None));
    assert_eq!(casting("spell.phantasm_of_the_human_form"), (16, None));
    assert_eq!(casting("spell.discern_own_illusions"), (11, None));
    assert_eq!(casting("spell.taste_of_the_spices_and_herbs"), (16, None));
    assert_eq!(casting("spell.aura_of_ennobled_presence"), (16, None));
    assert_eq!(casting("spell.notes_of_a_delightful_sound"), (16, None));
    assert_eq!(
        casting("spell.disguise_of_the_transformed_image"),
        (16, None)
    );
    // DISAGREEMENT MAG6 (docs/book-template-conformance.md). Deficient Technique
    // (Perdo) halves every total the Technique is added to (ArMDE:5915 `####
    // Deficient Technique`), and the Perdo Imaginem Casting Score is Pe 1 +
    // Im 10 + Sta 0 = 11. The book prints +6, i.e. 11 rounded **up**;
    // ArMDE:547 `### Ease Factor` says that where a rule does not say which
    // way to round, "round down", and ArMDE:5915 `#### Deficient Technique`
    // does not say. The engine rounds down and returns 5.
    assert_eq!(casting("spell.illusion_of_cool_flames"), (5, None));
    assert_eq!(casting("spell.illusion_of_the_shifted_image"), (16, None));
    assert_eq!(casting("spell.wizards_sidestep"), (16, None));
}

// --- Mercere (ArMDE:1952-1998 `#### Mercere`) -------------------------------

#[test]
fn the_mercere_matches_the_book() {
    let ruleset = full_ruleset();
    let mercere = load(include_str!("fixtures/book_templates/magus_mercere.json"));

    assert_eq!(error_codes(&mercere, &ruleset), codes(&[]));
    assert_eq!(warning_codes(&mercere, &ruleset), codes(&[]));

    // Arts: Cr 6+3, In 4, Mu 4, Pe 3, Re 5, Au 12+3, Co 2, Me 2 (ArMDE:1983 `#### Mercere`).
    assert_eq!(art_score(&mercere, &ruleset, "art.creo"), 9);
    assert_eq!(art_score(&mercere, &ruleset, "art.intellego"), 4);
    assert_eq!(art_score(&mercere, &ruleset, "art.muto"), 4);
    assert_eq!(art_score(&mercere, &ruleset, "art.perdo"), 3);
    assert_eq!(art_score(&mercere, &ruleset, "art.rego"), 5);
    assert_eq!(art_score(&mercere, &ruleset, "art.auram"), 15);
    assert_eq!(art_score(&mercere, &ruleset, "art.corpus"), 2);
    assert_eq!(art_score(&mercere, &ruleset, "art.mentem"), 2);

    // Soak: +2 (ArMDE:1975 `#### Mercere`).
    assert_eq!(soak(&mercere, &ruleset).total, 2);

    let fatigue: Vec<i32> = fatigue_levels(&mercere, &ruleset)
        .iter()
        .map(|l| l.penalty)
        .collect();
    assert_eq!(fatigue, BOOK_FATIGUE_PENALTIES);

    assert_eq!(
        wound_bounds(&wound_ranges(&mercere, &ruleset)),
        size_zero_wound_bounds()
    );

    // Encumbrance: 0 (0). Source: ArMDE:1989 `#### Mercere`.
    let enc = encumbrance(&mercere, &ruleset);
    assert_eq!((enc.burden, enc.total), (0, 0));

    // Dodging: Init +1, Atk n/a, Def +1, Dam n/a. Source: ArMDE:1973 `#### Mercere`.
    let lines = combat_totals(&mercere, &ruleset);
    assert_eq!(
        stats(line(&lines, "weapon.dodge", false)),
        (1, None, 1, None)
    );

    // F2 (design-f0-book-template-engine.md § 1a/§ 2c, D61): DISAGREEMENT MAG1
    // is resolved — the Casting-Total clause is gone from all three
    // conditional carriers (Cyclic Magic Positive/Negative, Special
    // Circumstances) in `rules/core/virtues_flaws.json`, leaving only each
    // entry's other, untouched effect (Lab Total / aura_bonus — X7a's
    // separate problem), so this fixture (which selects none of them anyway)
    // sees no +3 offset. The book's Cr 9 + Au 15 + Sta +2 = 26 base, 35 within
    // the Major Magical Focus (Weather) — ArMDE:1992-1996 `#### Mercere`
    // prints +35 for the two weather spells (Clouds of Rain and Thunder,
    // Clouds of Summer Snow — `within_focus: true` in `magus_mercere.json`,
    // X10c) and +26 for the rest, now selected by `spell_casting_total`
    // (X10bc phase 2).
    let casting = |spell: &str| printed_casting(&mercere, &ruleset, spell);
    assert_eq!(casting("spell.jupiters_resounding_blow"), 26);
    assert_eq!(casting("spell.clouds_of_rain_and_thunder"), 35);
    assert_eq!(casting("spell.clouds_of_summer_snow"), 35);
    assert_eq!(casting("spell.pull_of_the_skybound_winds"), 26);
    // MAG7, RESOLVED by the requisite fold: the book prints +27 here
    // (ArMDE:1996 `#### Mercere`). Wings of the Soaring Wind is Cr(Re)Au,
    // requisite Rego (ArMDE:13140), so the effective Technique folds to the
    // lesser of Creo 9 and Rego 5 (ArMDE:12309). Within the Weather focus,
    // "the lowest applicable score may be one of the requisites" (ArMDE:4403),
    // so Rego's 5 is the score doubled: 5 + 5 + Au 15 + Sta 2 = 27, exactly the
    // printed figure. Hence `within_focus: true` in `magus_mercere.json`.
    assert_eq!(casting("spell.wings_of_the_soaring_wind"), 27);
}

// --- Merinita (ArMDE:2000-2048 `#### Merinita`) -----------------------------

#[test]
fn the_merinita_matches_the_book() {
    let ruleset = full_ruleset();
    // The fixture's `virtue.strong_faerie_blood` selection carries
    // `"quirk": "webbed fingers"` — a placeholder value. D70 makes `quirk` a
    // required parameter, but the book's statblock (ArMDE:2000-2048) never
    // states what the character's faerie quirk actually is, so this fixture
    // invents one purely to satisfy the parameter requirement. Plain JSON
    // carries no comment syntax, hence the note lives here instead.
    let merinita = load(include_str!("fixtures/book_templates/magus_merinita.json"));

    assert_eq!(error_codes(&merinita, &ruleset), codes(&[]));
    assert_eq!(warning_codes(&merinita, &ruleset), codes(&[]));

    // Arts: Cr 5, In 1, Mu 5, Pe 2, Re 5, Co 1, Im 10+3, Me 5 (ArMDE:2031 `#### Merinita`).
    assert_eq!(art_score(&merinita, &ruleset, "art.creo"), 5);
    assert_eq!(art_score(&merinita, &ruleset, "art.intellego"), 1);
    assert_eq!(art_score(&merinita, &ruleset, "art.muto"), 5);
    assert_eq!(art_score(&merinita, &ruleset, "art.perdo"), 2);
    assert_eq!(art_score(&merinita, &ruleset, "art.rego"), 5);
    assert_eq!(art_score(&merinita, &ruleset, "art.corpus"), 1);
    assert_eq!(art_score(&merinita, &ruleset, "art.imaginem"), 13);
    assert_eq!(art_score(&merinita, &ruleset, "art.mentem"), 5);

    // DISAGREEMENT P3, RESOLVED (docs/book-template-conformance.md): the book
    // prints "Faerie Lore 3+2" (ArMDE:2029 `#### Merinita`) for Student of Faerie's "+2 bonus
    // on all uses of the appropriate Lore" (ArMDE:5054 `#### Student of (Realm)`); `virtue.student_of_realm`
    // now carries a gated `ability_bonus_gated`, so the engine matches.
    assert_eq!(
        ability_score(&merinita, &ruleset, "ability.faerie_lore", None),
        5
    );

    // Soak: -1 (ArMDE:2023 `#### Merinita`).
    assert_eq!(soak(&merinita, &ruleset).total, -1);

    let fatigue: Vec<i32> = fatigue_levels(&merinita, &ruleset)
        .iter()
        .map(|l| l.penalty)
        .collect();
    assert_eq!(fatigue, BOOK_FATIGUE_PENALTIES);

    assert_eq!(
        wound_bounds(&wound_ranges(&merinita, &ruleset)),
        size_zero_wound_bounds()
    );

    // Encumbrance: 0 (0). Source: ArMDE:2037 `#### Merinita`.
    let enc = encumbrance(&merinita, &ruleset);
    assert_eq!((enc.burden, enc.total), (0, 0));

    // Dodging: Init -1, Atk n/a, Def -1, Dam n/a. Source: ArMDE:2021 `#### Merinita`.
    let lines = combat_totals(&merinita, &ruleset);
    assert_eq!(
        stats(line(&lines, "weapon.dodge", false)),
        (-1, None, -1, None)
    );

    // Casting Totals, ArMDE:2040-2046 `#### Merinita`.
    let casting = |spell: &str| spell_casting(&merinita, &ruleset, spell);
    assert_eq!(casting("spell.phantasmal_animal"), (17, None));
    assert_eq!(casting("spell.phantasm_of_the_human_form"), (17, None));
    assert_eq!(casting("spell.image_phantom"), (17, None));
    assert_eq!(casting("spell.veil_of_invisibility"), (14, None));
    assert_eq!(casting("spell.wizards_sidestep"), (17, None));
    assert_eq!(casting("spell.panic_of_the_trembling_heart"), (9, None));
    assert_eq!(casting("spell.the_call_to_slumber"), (9, None));
}

// --- Tremere (ArMDE:2050-2101 `#### Tremere`) -------------------------------

#[test]
fn the_tremere_matches_the_book() {
    let ruleset = full_ruleset();
    let tremere = load(include_str!("fixtures/book_templates/magus_tremere.json"));

    assert_eq!(error_codes(&tremere, &ruleset), codes(&[]));
    assert_eq!(warning_codes(&tremere, &ruleset), codes(&[]));

    // X10(a) target shape (docs/vf-audit/decisions.md D65 row N5,
    // tmp/x10-verdicts.md). MAG10 (docs/book-template-conformance.md) is resolved
    // by a `params: { "focus": "certamen" } }` on House Tremere's fixed grant in
    // `rules/core/houses.json` — "Minor Magical Focus(certamen)\*" (ArMDE:2064
    // `#### Tremere`), the asterisk marking it the free House Virtue, and the
    // House table's "Minor Magical Focus (certamen)." (ArMDE:2281 `####
    // Hermetic Houses Summary`). RED until that param ships.
    let tremere_grants = arm_rules::house::granted_selections(&tremere, &ruleset);
    let focus_grant = tremere_grants
        .iter()
        .find(|s| s.item_ref == Id::new("virtue.minor_magical_focus"))
        .unwrap_or_else(|| panic!("House Tremere must grant Minor Magical Focus"));
    assert_eq!(
        focus_grant
            .params
            .get("focus")
            .and_then(SelectionParamValue::as_single),
        Some(&Id::new("certamen"))
    );

    // Arts: Cr 5, In 5, Mu 5, Pe 5, Re 5, Aq 8 (3), Au 9 (1), Ig 9 (1), Me 1,
    // Te 9 (1) — ArMDE:2081 `#### Tremere`. The four elemental Forms are bought at Aq 3, Au 6,
    // Ig 6, Te 6 and lifted by Elemental Magic, whose arithmetic the template's
    // own Customization Notes spell out at ArMDE:2099-2101 `#### Tremere`.
    assert_eq!(art_score(&tremere, &ruleset, "art.creo"), 5);
    assert_eq!(art_score(&tremere, &ruleset, "art.intellego"), 5);
    assert_eq!(art_score(&tremere, &ruleset, "art.muto"), 5);
    assert_eq!(art_score(&tremere, &ruleset, "art.perdo"), 5);
    assert_eq!(art_score(&tremere, &ruleset, "art.rego"), 5);
    assert_eq!(art_score(&tremere, &ruleset, "art.aquam"), 8);
    assert_eq!(art_score(&tremere, &ruleset, "art.auram"), 9);
    assert_eq!(art_score(&tremere, &ruleset, "art.ignem"), 9);
    assert_eq!(art_score(&tremere, &ruleset, "art.mentem"), 1);
    assert_eq!(art_score(&tremere, &ruleset, "art.terram"), 9);

    // Soak: +2 (ArMDE:2073 `#### Tremere`).
    assert_eq!(soak(&tremere, &ruleset).total, 2);

    let fatigue: Vec<i32> = fatigue_levels(&tremere, &ruleset)
        .iter()
        .map(|l| l.penalty)
        .collect();
    assert_eq!(fatigue, BOOK_FATIGUE_PENALTIES);

    assert_eq!(
        wound_bounds(&wound_ranges(&tremere, &ruleset)),
        size_zero_wound_bounds()
    );

    // Encumbrance: 0 (0). Source: ArMDE:2087 `#### Tremere`.
    let enc = encumbrance(&tremere, &ruleset);
    assert_eq!((enc.burden, enc.total), (0, 0));

    // Dodging: Init +1, Atk n/a, Def +1, Dam n/a. Source: ArMDE:2071 `#### Tremere`.
    let lines = combat_totals(&tremere, &ruleset);
    assert_eq!(
        stats(line(&lines, "weapon.dodge", false)),
        (1, None, 1, None)
    );

    // Casting Totals, ArMDE:2090-2097 `#### Tremere`: every row +16, which is Technique 5 +
    // Form 9 + Sta +2. The Minor Magical Focus (certamen) covers no spell on the
    // list (none is certamen), so the book prints the base figure throughout —
    // no spell in `magus_tremere.json` carries `within_focus: true` (X10c),
    // matching that. This block stays green even under the phase-1 stub
    // (`spell_casting_total` always returns the base figure today), since
    // `false` is what every one of these spells should resolve to anyway.
    let casting = |spell: &str| printed_casting(&tremere, &ruleset, spell);
    assert_eq!(casting("spell.circling_winds_of_protection"), 16);
    assert_eq!(casting("spell.rain_of_stones"), 16);
    assert_eq!(casting("spell.pilum_of_fire"), 16);
    assert_eq!(casting("spell.soothe_the_raging_flames"), 16);
    assert_eq!(casting("spell.seal_the_earth"), 16);
    assert_eq!(casting("spell.the_miners_keen_eye"), 16);
    assert_eq!(casting("spell.earth_that_breaks_no_more"), 16);
    assert_eq!(casting("spell.pit_of_the_gaping_earth"), 16);
}

// --- Tytalus (ArMDE:2103-2149 `#### Tytalus`) -------------------------------

#[test]
fn the_tytalus_matches_the_book() {
    let ruleset = full_ruleset();
    let tytalus = load(include_str!("fixtures/book_templates/magus_tytalus.json"));

    assert_eq!(error_codes(&tytalus, &ruleset), codes(&[]));
    assert_eq!(warning_codes(&tytalus, &ruleset), codes(&[]));

    // Int +4 (ArMDE:2105 `#### Tytalus`) is a bought +3 lifted by Great Intelligence.
    assert_eq!(
        arm_rules::effective::effective_characteristic_score(
            &tytalus,
            &ruleset,
            Characteristic::Int
        ),
        4
    );

    // Confidence Score: 2 (5) — ArMDE:2115 `#### Tytalus` — against the magus profile's 1 (3),
    // because House Tytalus's free Self-Confident adds +1 score and +2 points.
    let profile = ruleset
        .profile(&tytalus.type_id)
        .expect("the magus profile is in the shipped ruleset");
    assert_eq!(
        arm_rules::effective::confidence(
            profile.confidence_score,
            profile.confidence_points,
            &tytalus,
            &ruleset
        ),
        arm_rules::effective::Confidence {
            score: 2,
            points: 5
        }
    );

    // Arts: Cr 5, In 5, Re 5, Me 9 (ArMDE:2134 `#### Tytalus`).
    assert_eq!(art_score(&tytalus, &ruleset, "art.creo"), 5);
    assert_eq!(art_score(&tytalus, &ruleset, "art.intellego"), 5);
    assert_eq!(art_score(&tytalus, &ruleset, "art.rego"), 5);
    assert_eq!(art_score(&tytalus, &ruleset, "art.mentem"), 9);

    // Soak: +2 (ArMDE:2126 `#### Tytalus`).
    assert_eq!(soak(&tytalus, &ruleset).total, 2);

    let fatigue: Vec<i32> = fatigue_levels(&tytalus, &ruleset)
        .iter()
        .map(|l| l.penalty)
        .collect();
    assert_eq!(fatigue, BOOK_FATIGUE_PENALTIES);

    assert_eq!(
        wound_bounds(&wound_ranges(&tytalus, &ruleset)),
        size_zero_wound_bounds()
    );

    // Encumbrance: 0 (0). Source: ArMDE:2140 `#### Tytalus`.
    let enc = encumbrance(&tytalus, &ruleset);
    assert_eq!((enc.burden, enc.total), (0, 0));

    // Dodging: Init +1, Atk n/a, Def +4, Dam n/a. Source: ArMDE:2124 `#### Tytalus`.
    let lines = combat_totals(&tytalus, &ruleset);
    assert_eq!(
        stats(line(&lines, "weapon.dodge", false)),
        (1, None, 4, None)
    );

    // Casting Totals, ArMDE:2143-2147 `#### Tytalus`.
    let casting = |spell: &str| spell_casting(&tytalus, &ruleset, spell);
    assert_eq!(casting("spell.pains_of_the_perpetual_worry"), (16, None));
    assert_eq!(casting("spell.posing_the_silent_question"), (16, None));
    assert_eq!(casting("spell.trust_of_childlike_faith"), (11, None));
    assert_eq!(casting("spell.aura_of_rightful_authority"), (16, None));
    assert_eq!(casting("spell.scent_of_peaceful_slumber"), (16, None));
}

// --- Verditius (ArMDE:2151-2199 `#### Verditius`) ---------------------------

#[test]
fn the_verditius_matches_the_book() {
    let ruleset = full_ruleset();
    let verditius = load(include_str!("fixtures/book_templates/magus_verditius.json"));

    // Affinity-bought Craft (stonemason) 4 costs 33 experience points (row
    // 47/F-547, formerly S2/MAG13 — `charged_cost` used to overcharge this to 34),
    // so the fixture's `xp_pool: 435` matches the age formula's grant exactly.
    assert_eq!(error_codes(&verditius, &ruleset), codes(&[]));
    assert_eq!(warning_codes(&verditius, &ruleset), codes(&[]));

    // Str -3, Sta +1 (ArMDE:2153 `#### Verditius`) are a bought -2 / +2 shifted by Dwarf's -1 each.
    assert_eq!(
        arm_rules::effective::effective_characteristic_score(
            &verditius,
            &ruleset,
            Characteristic::Str
        ),
        -3
    );
    assert_eq!(
        arm_rules::effective::effective_characteristic_score(
            &verditius,
            &ruleset,
            Characteristic::Sta
        ),
        1
    );

    // Arts: Cr 7, In 3, Mu 5, Pe 3, Re 5, Te 12+3 (ArMDE:2182 `#### Verditius`).
    assert_eq!(art_score(&verditius, &ruleset, "art.creo"), 7);
    assert_eq!(art_score(&verditius, &ruleset, "art.intellego"), 3);
    assert_eq!(art_score(&verditius, &ruleset, "art.muto"), 5);
    assert_eq!(art_score(&verditius, &ruleset, "art.perdo"), 3);
    assert_eq!(art_score(&verditius, &ruleset, "art.rego"), 5);
    assert_eq!(art_score(&verditius, &ruleset, "art.terram"), 15);

    // Two instances of one parameterized Ability, each with its own Affinity and
    // its own Puissant, and the engine keeps them apart: the bonus lands on the
    // instance the selection names and on no other.
    //
    // DISAGREEMENT MAG14 (docs/book-template-conformance.md). The book prints
    // "Craft (metalsmith) 5+3" and "Craft (stonemason) 4+3" (ArMDE:2180 `####
    // Verditius`), but Puissant *Ability* adds 2 — "add 2 to its value
    // whenever you use it" (ArMDE:4816 `#### Puissant Ability`) — and 3 is
    // Puissant *Art*'s figure (ArMDE:4820 `#### Puissant Art`). Every other
    // template prints the +2 correctly (the Knight's "Single Weapon 5+2"
    // ArMDE:1480 `#### The Knight`, Bonisagus's "Magic Theory 4+2" ArMDE:1683
    // `#### Bonisagus`, Criamon's "Enigmatic Wisdom 3+2" ArMDE:1731 `####
    // Criamon`, Jerbiton's "Music 4+2" ArMDE:1928 `#### Jerbiton`), so
    // the engine's 7 and 6 are right and this statblock is alone in saying 3.
    assert_eq!(
        ability_score(&verditius, &ruleset, "ability.craft", Some("metalsmith")),
        7
    );
    assert_eq!(
        ability_score(&verditius, &ruleset, "ability.craft", Some("stonemason")),
        6
    );

    // Soak: +1 (ArMDE:2174 `#### Verditius`).
    assert_eq!(soak(&verditius, &ruleset).total, 1);

    let fatigue: Vec<i32> = fatigue_levels(&verditius, &ruleset)
        .iter()
        .map(|l| l.penalty)
        .collect();
    assert_eq!(fatigue, BOOK_FATIGUE_PENALTIES);

    assert_eq!(
        wound_bounds(&wound_ranges(&verditius, &ruleset)),
        size_minus_two_wound_bounds()
    );

    // Encumbrance: 0 (0). Source: ArMDE:2188 `#### Verditius`.
    let enc = encumbrance(&verditius, &ruleset);
    assert_eq!((enc.burden, enc.total), (0, 0));

    // Dodging: Init +0, Atk n/a, Def +0, Dam n/a. Source: ArMDE:2172 `#### Verditius`.
    let lines = combat_totals(&verditius, &ruleset);
    assert_eq!(
        stats(line(&lines, "weapon.dodge", false)),
        (0, None, 0, None)
    );

    // Casting Totals, ArMDE:2191-2197 `#### Verditius`: Technique + Te 15 + Sta +1.
    let casting = |spell: &str| spell_casting(&verditius, &ruleset, spell);
    assert_eq!(casting("spell.seal_the_earth"), (23, None));
    // DISAGREEMENT MAG9 (docs/book-template-conformance.md). The book prints +25
    // for Touch of Midas (ArMDE:2192 `#### Verditius`) where its two Creo Terram neighbours on the
    // same statblock read +23, and Cr 7 + Te 15 + Sta +1 = 23. This magus holds
    // no Magical Focus, so there is no second reading to reach 25 by.
    assert_eq!(casting("spell.touch_of_midas"), (23, None));
    assert_eq!(casting("spell.wall_of_protecting_stone"), (23, None));
    assert_eq!(casting("spell.the_crystal_dart"), (21, None));
    assert_eq!(casting("spell.edge_of_the_razor"), (21, None));
    assert_eq!(casting("spell.pit_of_the_gaping_earth"), (19, None));
    assert_eq!(casting("spell.hands_of_the_grasping_earth"), (21, None));
}

// --- The Female Scholar (ArMDE:1410-1445 `#### The Female Scholar`) --------

#[test]
fn the_female_scholar_matches_the_book() {
    let ruleset = full_ruleset();
    let scholar = load(include_str!(
        "fixtures/book_templates/companion_female_scholar.json"
    ));

    // DISAGREEMENT F1 (docs/book-template-conformance.md) — RESOLVED (X1/D43).
    // Clerk's own description grants Academic access — "Due to your training,
    // you may take Academic Abilities during character generation" (ArMDE:3573 `#### Clerk`)
    // — and `virtue.clerk` now carries `ability_authorization{categories:[academic]}`,
    // so her six Academic Abilities are legal, matching the book.
    assert_eq!(error_codes(&scholar, &ruleset), codes(&[]));
    assert_eq!(warning_codes(&scholar, &ruleset), codes(&[]));

    // Soak: -1 (Stamina). Source: ArMDE:1433 `#### The Female Scholar`.
    assert_eq!(soak(&scholar, &ruleset).total, -1);

    let fatigue: Vec<i32> = fatigue_levels(&scholar, &ruleset)
        .iter()
        .map(|l| l.penalty)
        .collect();
    assert_eq!(fatigue, BOOK_FATIGUE_PENALTIES);

    assert_eq!(
        wound_bounds(&wound_ranges(&scholar, &ruleset)),
        size_zero_wound_bounds()
    );

    // Encumbrance: 0 (0). Source: ArMDE:1443 `#### The Female Scholar`.
    let enc = encumbrance(&scholar, &ruleset);
    assert_eq!((enc.burden, enc.total), (0, 0));

    // Dodging: Init +1, Atk n/a, Def +1, Dam n/a. Source: ArMDE:1431 `#### The Female Scholar`.
    let lines = combat_totals(&scholar, &ruleset);
    assert_eq!(
        stats(line(&lines, "weapon.dodge", false)),
        (1, None, 1, None)
    );
}

// --- The Knight (ArMDE:1447-1486 `#### The Knight`) -------------------------

#[test]
fn the_knight_matches_the_book() {
    let ruleset = full_ruleset();
    let knight = load(include_str!(
        "fixtures/book_templates/companion_knight.json"
    ));

    // DISAGREEMENT K1 (docs/book-template-conformance.md) — RESOLVED (X1/D43),
    // the same shape as F1 and as the Berserker's B1. The Knight Virtue says
    // "You may take Martial Abilities during character generation" (ArMDE:4197 `#### Knight`),
    // and `virtue.knight` now carries `ability_authorization{categories:[martial]}`,
    // so Great Weapon 5 and Single Weapon 5 are both legal, matching the book.
    assert_eq!(error_codes(&knight, &ruleset), codes(&[]));
    assert_eq!(warning_codes(&knight, &ruleset), codes(&[]));

    // "Single Weapon 5+2 (heater shield)" (ArMDE:1480 `#### The Knight`) — Puissant Single Weapon's
    // fixed +2 on a bought 5, and the reason the sword rows below read +14 rather
    // than +12.
    assert_eq!(
        arm_rules::effective::effective_ability_score(
            &knight,
            &ruleset,
            &arm_rules::types::Id::new("ability.single_weapon"),
            None
        ),
        7
    );

    // Soak: +10 (chain mail, Stamina). Source: ArMDE:1474 `#### The Knight`.
    assert_eq!(soak(&knight, &ruleset).total, 10);

    let fatigue: Vec<i32> = fatigue_levels(&knight, &ruleset)
        .iter()
        .map(|l| l.penalty)
        .collect();
    assert_eq!(fatigue, BOOK_FATIGUE_PENALTIES);

    assert_eq!(
        wound_bounds(&wound_ranges(&knight, &ruleset)),
        size_zero_wound_bounds()
    );

    // Encumbrance: 2 (3). Source: ArMDE:1484 `#### The Knight`. Reproduced exactly, and it is the
    // reason the fixture leaves the great sword UNequipped: only equipped gear
    // contributes Load (`combat.rs::encumbrance`), so the wielded set is chain
    // mail 6 + long sword 1 + heater shield 2 = 9 → Burden 3, less Strength +1.
    // Counting the stowed great sword too would give Load 11 → Burden 4, which is
    // what the engine returned until 2026-09-23; see K2 in
    // docs/book-template-conformance.md for why the book's reading won.
    let enc = encumbrance(&knight, &ruleset);
    assert_eq!((enc.burden, enc.total), (3, 2));

    let lines = combat_totals(&knight, &ruleset);
    // Long sword and heater shield (on foot): Init +2, Atk +14, Def +14, Dam +7
    // (ArMDE:1469 `#### The Knight`). Every figure exact.
    assert_eq!(
        stats(line(&lines, "weapon.sword_long", true)),
        (2, Some(14), 14, Some(7))
    );
    // Fist: Init +0, Atk +5, Def +5, Dam +1 (ArMDE:1472 `#### The Knight`). Every figure exact.
    assert_eq!(
        stats(line(&lines, "weapon.fist", false)),
        (0, Some(5), 5, Some(1))
    );
    // Great sword (on foot): Init +2, Atk +13, Def +10, Dam +10 (ArMDE:1471 `#### The Knight`).
    // Already exact today — F1's K5 fix (the `loadout: "carried"` row) needs no
    // F2 change to be right; this is a control assertion, not a new red.
    assert_eq!(
        stats(line(&lines, "weapon.sword_great", false)),
        (2, Some(13), 10, Some(10))
    );

    // K3 (RED CHECKPOINT — design-f0-book-template-engine.md § 2b, D66): the
    // book prints TWO mounted rows (ArMDE:1468 `#### The Knight`, :1470), each
    // the on-foot figure plus min(Ride, 3) = +3 (Ride 5, ArMDE:1480 `#### The
    // Knight`) on Attack and Defense only.
    // Long sword and heater shield (mounted): Init +2, Atk +17, Def +17, Dam +7
    // (ArMDE:1468 `#### The Knight`).
    assert_eq!(
        stats(mounted_line(&lines, "weapon.sword_long", true)),
        (2, Some(17), 17, Some(7))
    );
    // Great sword (mounted): Init +2, Atk +16, Def +13, Dam +10 (ArMDE:1470 `#### The Knight`).
    assert_eq!(
        stats(mounted_line(&lines, "weapon.sword_great", false)),
        (2, Some(16), 13, Some(10))
    );
    // Fist gets NO mounted twin (D66: `weapon.fist` is a body attack) — the
    // book's own template prints no mounted Fist row (ArMDE:1472 `#### The Knight`).
    assert!(
        !lines
            .iter()
            .any(|l| l.weapon.as_str() == "weapon.fist" && l.mounted),
        "Fist must never gain a mounted twin"
    );

    // The book prints five Combat rows; the engine's own model additionally
    // computes a "dropped shield" bare line the book does not print (K5), so
    // five-becomes-eight once K3's mounted twins land: Fist stays singular (2
    // lines); the great sword's bare line and the long sword's with-shield/bare
    // pair each gain a mounted twin, appended after the on-foot set — per
    // design-f0-book-template-engine.md § 2b's sketch, the twin pass filters
    // and maps over the already-built on-foot lines and extends the vector, so
    // every twin lands after all five on-foot lines, in the same relative
    // order as the line it doubles. Confirms the fixture's `mounted: true` is
    // required (§ 9 of the design note): with it absent, `combat_totals` still
    // returns the pre-F2 five and every element below reads `mounted: false`.
    let emitted: Vec<(&str, bool, bool)> = lines
        .iter()
        .map(|l| (l.weapon.as_str(), !l.shields.is_empty(), l.mounted))
        .collect();
    assert_eq!(
        emitted,
        vec![
            ("weapon.fist", true, false),
            ("weapon.fist", false, false),
            ("weapon.sword_great", false, false),
            ("weapon.sword_long", true, false),
            ("weapon.sword_long", false, false),
            ("weapon.sword_great", false, true),
            ("weapon.sword_long", true, true),
            ("weapon.sword_long", false, true),
        ]
    );
}

// --- The Priest (ArMDE:1488-1523 `#### The Priest`) -------------------------

#[test]
fn the_priest_matches_the_book() {
    let ruleset = full_ruleset();
    let priest = load(include_str!(
        "fixtures/book_templates/companion_priest.json"
    ));

    // DISAGREEMENT P1 (docs/book-template-conformance.md) — RESOLVED (X1/D43),
    // a third instance of F1/K1: "You may purchase Academic Abilities during
    // character generation" (ArMDE:4804 `#### Priest`) is part of the Priest Virtue, and
    // `virtue.priest` now carries `ability_authorization{categories:[academic]}`,
    // so his four Academic Abilities are legal. His *Arcane* one, Dominion Lore,
    // is accepted — `virtue.student_of_realm`'s gated `ability_bonus_gated`
    // target (realm.divine) authorizes it (P3/P4, RESOLVED).
    //
    // `xp_pool` moved 590 → 540 in the same change (X1/D43). Well-Traveled now
    // carries its own 50-point restricted pool (ArMDE:5239-5242 `#### Well-Traveled`) covering
    // exactly six of the Abilities on this sheet (Area Lore, Charm, Etiquette,
    // Folk Ken, Living Language, Organization Lore). `two_phase_max_flow`
    // (`effective/xp.rs`) fills restricted pools before General, so those 50
    // points now come from Well-Traveled rather than General — the fixture's
    // total demand (590) is unchanged, but General's own share of it is 540;
    // leaving it at 590 left 50 General points genuinely unspent
    // (`general_xp_unspent`).
    assert_eq!(error_codes(&priest, &ruleset), codes(&[]));
    assert_eq!(warning_codes(&priest, &ruleset), codes(&[]));

    // DISAGREEMENT P3, RESOLVED (docs/book-template-conformance.md). The book
    // prints "Dominion Lore 3+2 (angels)" (ArMDE:1517 `#### The Priest`) — a
    // bought 3 plus a fixed Virtue bonus of 2 (the `X+Y` format, ArMDE:1177
    // `### Format`), here Student of the Divine: "you have a +2 bonus on all
    // uses of the appropriate Lore" (ArMDE:5054 `#### Student of (Realm)`).
    // `virtue.student_of_realm` now carries a gated
    // `ability_bonus_gated`, so the engine matches.
    assert_eq!(
        arm_rules::effective::effective_ability_score(
            &priest,
            &ruleset,
            &arm_rules::types::Id::new("ability.dominion_lore"),
            None
        ),
        5
    );
    // "Sense Holiness and Unholiness 4" (ArMDE:1517 `#### The Priest`) is a Supernatural Ability the
    // character may hold only because a Virtue confers it (ArMDE:4928 `#### Sense Holiness and Unholiness`); that grant is
    // a free floor of 1, so the bought 4 stands as the effective score and the
    // first point costs nothing (S-note in the doc).
    assert_eq!(
        arm_rules::effective::effective_ability_score(
            &priest,
            &ruleset,
            &arm_rules::types::Id::new("ability.sense_holiness_and_unholiness"),
            None
        ),
        4
    );

    // Soak: +0 (Stamina). Source: ArMDE:1511 `#### The Priest`.
    assert_eq!(soak(&priest, &ruleset).total, 0);

    let fatigue: Vec<i32> = fatigue_levels(&priest, &ruleset)
        .iter()
        .map(|l| l.penalty)
        .collect();
    assert_eq!(fatigue, BOOK_FATIGUE_PENALTIES);

    assert_eq!(
        wound_bounds(&wound_ranges(&priest, &ruleset)),
        size_zero_wound_bounds()
    );

    // Encumbrance: 0 (0). Source: ArMDE:1521 `#### The Priest`.
    let enc = encumbrance(&priest, &ruleset);
    assert_eq!((enc.burden, enc.total), (0, 0));

    // Dodging: Init +0, Atk n/a, Def +2, Dam n/a. Source: ArMDE:1509 `#### The Priest`.
    let lines = combat_totals(&priest, &ruleset);
    assert_eq!(
        stats(line(&lines, "weapon.dodge", false)),
        (0, None, 2, None)
    );
}

// --- The Rogue (ArMDE:1525-1560 `#### The Rogue`) ---------------------------

#[test]
fn the_rogue_matches_the_book() {
    let ruleset = full_ruleset();
    let rogue = load(include_str!("fixtures/book_templates/companion_rogue.json"));

    assert_eq!(error_codes(&rogue, &ruleset), codes(&[]));
    assert_eq!(warning_codes(&rogue, &ruleset), codes(&[]));

    // The book prints Dex +4 and Qik +4 (ArMDE:1527 `#### The Rogue`) although the buy floor is +3:
    // Great Dexterity and Great Quickness each add their +1 after purchase
    // (ArMDE:3987-3989 `#### Great (Characteristic)`). The fixture buys +3 twice and the engine derives +4.
    assert_eq!(
        arm_rules::effective::effective_characteristic_score(&rogue, &ruleset, Characteristic::Dex),
        4
    );
    assert_eq!(
        arm_rules::effective::effective_characteristic_score(&rogue, &ruleset, Characteristic::Qik),
        4
    );

    // Soak: 0 (Stamina). Source: ArMDE:1548 `#### The Rogue`.
    assert_eq!(soak(&rogue, &ruleset).total, 0);

    let fatigue: Vec<i32> = fatigue_levels(&rogue, &ruleset)
        .iter()
        .map(|l| l.penalty)
        .collect();
    assert_eq!(fatigue, BOOK_FATIGUE_PENALTIES);

    assert_eq!(
        wound_bounds(&wound_ranges(&rogue, &ruleset)),
        size_zero_wound_bounds()
    );

    // Encumbrance: 0 (0). Source: ArMDE:1558 `#### The Rogue`.
    let enc = encumbrance(&rogue, &ruleset);
    assert_eq!((enc.burden, enc.total), (0, 0));

    // Fist: Init +4, Atk +7, Def +7, Dam -1. Source: ArMDE:1546 `#### The Rogue`.
    let lines = combat_totals(&rogue, &ruleset);
    assert_eq!(
        stats(line(&lines, "weapon.fist", false)),
        (4, Some(7), 7, Some(-1))
    );
}

// --- The Witch (ArMDE:1562-1597 `#### The Witch`) ---------------------------

#[test]
fn the_witch_matches_the_book() {
    let ruleset = full_ruleset();
    let witch = load(include_str!("fixtures/book_templates/companion_witch.json"));

    // DISAGREEMENT W1 (docs/book-template-conformance.md) — RESOLVED (X1/D43).
    // Educated is "You may purchase Academic Abilities during character
    // generation" plus 50 experience points "which must be spent on Latin and
    // Artes Liberales" (ArMDE:3713 `#### Educated`); `virtue.educated` now carries
    // `ability_authorization{categories:[academic]}` in addition to its
    // earmarked pool, so Medicine — Academic, and not on the XP list — is
    // legal too, matching the book.
    assert_eq!(error_codes(&witch, &ruleset), codes(&[]));
    assert_eq!(warning_codes(&witch, &ruleset), codes(&[]));

    // DISAGREEMENT P3, RESOLVED, again on the other side of the same Virtue:
    // the book prints "Magic Lore 3+2 (regiones)" (ArMDE:1591 `#### The Witch`) for Student of
    // Magic, and the engine now matches.
    assert_eq!(
        arm_rules::effective::effective_ability_score(
            &witch,
            &ruleset,
            &arm_rules::types::Id::new("ability.magic_lore"),
            None
        ),
        5
    );

    // Soak: +0 (Stamina). Source: ArMDE:1585 `#### The Witch`.
    assert_eq!(soak(&witch, &ruleset).total, 0);

    let fatigue: Vec<i32> = fatigue_levels(&witch, &ruleset)
        .iter()
        .map(|l| l.penalty)
        .collect();
    assert_eq!(fatigue, BOOK_FATIGUE_PENALTIES);

    assert_eq!(
        wound_bounds(&wound_ranges(&witch, &ruleset)),
        size_zero_wound_bounds()
    );

    // Encumbrance: 0 (0). Source: ArMDE:1595 `#### The Witch`.
    let enc = encumbrance(&witch, &ruleset);
    assert_eq!((enc.burden, enc.total), (0, 0));

    // Dodging: Init +0, Atk n/a, Def +0, Dam n/a. Source: ArMDE:1583 `#### The Witch`.
    let lines = combat_totals(&witch, &ruleset);
    assert_eq!(
        stats(line(&lines, "weapon.dodge", false)),
        (0, None, 0, None)
    );
}

// --- The Berserker (ArMDE:1195-1227 `#### The Berserker`) -------------------

#[test]
fn the_berserker_matches_the_book() {
    let ruleset = full_ruleset();
    let berserker = load(include_str!("fixtures/book_templates/grog_berserker.json"));

    // DISAGREEMENT B1 (docs/book-template-conformance.md) — RESOLVED (X1/D43).
    // The Berserker buys Great Weapon 5 and Single Weapon 1 with no Warrior,
    // and the engine gates Martial Abilities behind a Virtue (ArMDE:2392 `#### Later Life`).
    // Berserk IS such a Virtue — "You may learn Martial Abilities at character
    // creation" (ArMDE:3502 `#### Berserk`) — and `virtue.berserk` now carries
    // `ability_authorization{categories:[martial]}`, so both weapons are legal,
    // matching the book.
    assert_eq!(error_codes(&berserker, &ruleset), codes(&[]));
    // DISAGREEMENT B3 (docs/book-template-conformance.md). Short Attention Span
    // and Wrathful (Minor) are both Personality Flaws, and the grog rule is "You
    // should not take more than one Personality Flaw" (ArMDE:2827 `#### Grogs`). The engine is
    // right and the template breaks the book's own grog checklist; it is a
    // guideline ("should"), hence a warning rather than an error.
    assert_eq!(
        warning_codes(&berserker, &ruleset),
        codes(&["too_many_personality_flaws"])
    );

    // F2 (design-f0-book-template-engine.md § 1a/§ 2c, D61/D15) — RED
    // CHECKPOINT: DISAGREEMENT B2 is resolved by deleting Berserk's
    // Soak/Attack/Defense effects and reclassifying it to `uncomputed_rule`
    // (no template anywhere shows the "while berserk" figure). The book
    // prints Soak: +9 (Stamina +2, full metal scale armor +7) — ArMDE:1215 `#### The Berserker`.
    // `rules/core/virtues_flaws.json` has not been edited yet, so the engine
    // still returns 11 (the old unconditional +2 Soak) until it is.
    assert_eq!(soak(&berserker, &ruleset).total, 9);

    let fatigue: Vec<i32> = fatigue_levels(&berserker, &ruleset)
        .iter()
        .map(|l| l.penalty)
        .collect();
    assert_eq!(fatigue, BOOK_FATIGUE_PENALTIES);

    assert_eq!(
        wound_bounds(&wound_ranges(&berserker, &ruleset)),
        size_one_wound_bounds()
    );

    // Encumbrance: 0 (3). Source: ArMDE:1225 `#### The Berserker`.
    let enc = encumbrance(&berserker, &ruleset);
    assert_eq!((enc.burden, enc.total), (3, 0));

    // F2 — RED CHECKPOINT: same fix, on both weapon lines. The book prints
    // Pole Axe: Init +2, Attack +13, Defense +7, Damage +14 (ArMDE:1212 `#### The Berserker`) and
    // Kick: Init +0, Attack +6, Defense +4, Damage +6 (ArMDE:1213 `#### The Berserker`) — the
    // figures once Berserk's unconditional +2 Attack / -2 Defense is deleted.
    // Initiative and Damage, which Berserk never touched, already match.
    let lines = combat_totals(&berserker, &ruleset);
    assert_eq!(
        stats(line(&lines, "weapon.pole_axe", false)),
        (2, Some(13), 7, Some(14))
    );
    assert_eq!(
        stats(line(&lines, "weapon.kick", false)),
        (0, Some(6), 4, Some(6))
    );
}

// --- The Grizzled Veteran (ArMDE:1229-1263 `#### The Grizzled Veteran`) ----

#[test]
fn the_grizzled_veteran_matches_the_book() {
    let ruleset = full_ruleset();
    let veteran = load(include_str!(
        "fixtures/book_templates/grog_grizzled_veteran.json"
    ));

    assert_eq!(error_codes(&veteran, &ruleset), codes(&[]));
    // Not a rules disagreement (docs/book-template-conformance.md, V3): at 45 the
    // character owes aging rolls, and the template shows their *outcome*
    // (Decrepitude 1 (2), ArMDE:1237 `#### The Grizzled Veteran`) without the per-year log that recorded
    // them. The save format has nowhere to put an unrolled history, so the
    // advisory stands.
    assert_eq!(
        warning_codes(&veteran, &ruleset),
        codes(&["aging_rolls_pending"])
    );

    // The book prints Pre -1 and Com -1 although the point-buy bought them at 0:
    // "the years have already reduced his Presence and Communication to -1 each"
    // (ArMDE:1263 `#### The Grizzled Veteran`). The fixture records the aging points; the engine derives the
    // drop.
    assert_eq!(
        arm_rules::effective::effective_characteristic_after_aging(
            &veteran,
            &ruleset,
            Characteristic::Pre
        ),
        -1
    );
    assert_eq!(
        arm_rules::effective::effective_characteristic_after_aging(
            &veteran,
            &ruleset,
            Characteristic::Com
        ),
        -1
    );

    // Soak: +8 (full metal scale armor). Source: ArMDE:1251 `#### The Grizzled Veteran`.
    assert_eq!(soak(&veteran, &ruleset).total, 8);

    let fatigue: Vec<i32> = fatigue_levels(&veteran, &ruleset)
        .iter()
        .map(|l| l.penalty)
        .collect();
    assert_eq!(fatigue, BOOK_FATIGUE_PENALTIES);

    assert_eq!(
        wound_bounds(&wound_ranges(&veteran, &ruleset)),
        size_zero_wound_bounds()
    );

    // Encumbrance: 4 (4). Source: ArMDE:1261 `#### The Grizzled Veteran`.
    let enc = encumbrance(&veteran, &ruleset);
    assert_eq!((enc.burden, enc.total), (4, 4));

    // Axe & Heater Shield: Init -1, Attack +15, Defense +14, Damage +6.
    // Source: ArMDE:1248 `#### The Grizzled Veteran`.
    let lines = combat_totals(&veteran, &ruleset);
    assert_eq!(
        stats(line(&lines, "weapon.axe", true)),
        (-1, Some(15), 14, Some(6))
    );
    // Kick: Init -3, Attack +6, Defense +5, Damage +3. Source: ArMDE:1249 `#### The Grizzled Veteran`. The
    // book prints the bare line; the engine also offers a with-shield one.
    assert_eq!(
        stats(line(&lines, "weapon.kick", false)),
        (-3, Some(6), 5, Some(3))
    );
}

// --- The Hunter (ArMDE:1265-1296 `#### The Hunter`) -------------------------

#[test]
fn the_hunter_matches_the_book() {
    let ruleset = full_ruleset();
    let hunter = load(include_str!("fixtures/book_templates/grog_hunter.json"));

    assert_eq!(error_codes(&hunter, &ruleset), codes(&[]));
    assert_eq!(warning_codes(&hunter, &ruleset), codes(&[]));

    // Soak: +3 (partial leather armor, Stamina). Source: ArMDE:1284 `#### The Hunter`.
    assert_eq!(soak(&hunter, &ruleset).total, 3);

    let fatigue: Vec<i32> = fatigue_levels(&hunter, &ruleset)
        .iter()
        .map(|l| l.penalty)
        .collect();
    assert_eq!(fatigue, BOOK_FATIGUE_PENALTIES);

    assert_eq!(
        wound_bounds(&wound_ranges(&hunter, &ruleset)),
        size_zero_wound_bounds()
    );

    // Encumbrance: 2 (2). Source: ArMDE:1294 `#### The Hunter`.
    let enc = encumbrance(&hunter, &ruleset);
    assert_eq!((enc.burden, enc.total), (2, 2));

    // Short Bow: Init -1, Attack +9, Defense +6, Damage +6. Source: ArMDE:1282 `#### The Hunter`.
    let lines = combat_totals(&hunter, &ruleset);
    assert_eq!(
        stats(line(&lines, "weapon.bow_short", false)),
        (-1, Some(9), 6, Some(6))
    );
}

// --- The Specialist (ArMDE:1298-1332 `#### The Specialist`) ----------------

#[test]
fn the_specialist_matches_the_book() {
    let ruleset = full_ruleset();
    let specialist = load(include_str!("fixtures/book_templates/grog_specialist.json"));

    assert_eq!(error_codes(&specialist, &ruleset), codes(&[]));
    // Affinity now charges Single Weapon 7 at 93 XP, not the old bugged 94
    // (row 47/F-547, formerly S2). At the book's own general-XP grant of 330
    // (age formula: 75 + 45 + 15x14), that leaves exactly the 2 XP the book
    // itself banks toward the next Bows increase — the printed "Bows 1 (2)"
    // (ArMDE:1326 `#### The Specialist`). TARGETED FOR RESOLUTION by X10b (not
    // yet resolved): `ability.bows` now carries `"banked_xp": 2`, and once
    // X10b's engine change folds `banked_xp` into the raw table total before
    // `charged_cost`, the fixture becomes fully exact — no more
    // `general_xp_unspent`, since those 2 XP are no longer unspent, they are
    // banked. RED today: the fixture still reports the warning.
    assert_eq!(warning_codes(&specialist, &ruleset), codes(&[]));

    // Soak: +9 (full metal scale armor). Source: ArMDE:1320 `#### The Specialist`.
    assert_eq!(soak(&specialist, &ruleset).total, 9);

    let fatigue: Vec<i32> = fatigue_levels(&specialist, &ruleset)
        .iter()
        .map(|l| l.penalty)
        .collect();
    assert_eq!(fatigue, BOOK_FATIGUE_PENALTIES);

    assert_eq!(
        wound_bounds(&wound_ranges(&specialist, &ruleset)),
        size_zero_wound_bounds()
    );

    // Encumbrance: 2 (4). Source: ArMDE:1330 `#### The Specialist`.
    let enc = encumbrance(&specialist, &ruleset);
    assert_eq!((enc.burden, enc.total), (4, 2));

    // Axe & Heater Shield: Init +1, Attack +17, Defense +15, Damage +8.
    // Source: ArMDE:1317 `#### The Specialist`.
    let lines = combat_totals(&specialist, &ruleset);
    assert_eq!(
        stats(line(&lines, "weapon.axe", true)),
        (1, Some(17), 15, Some(8))
    );
    // Fist: Init 0, Attack +8, Defense +7, Damage +2. Source: ArMDE:1318 `#### The Specialist`.
    assert_eq!(
        stats(line(&lines, "weapon.fist", false)),
        (0, Some(8), 7, Some(2))
    );
}

// --- The Standard Soldier (ArMDE:1334-1368 `#### The Standard Soldier`) ----

#[test]
fn the_standard_soldier_matches_the_book() {
    let ruleset = full_ruleset();
    let soldier = load(include_str!(
        "fixtures/book_templates/grog_standard_soldier.json"
    ));

    assert_eq!(error_codes(&soldier, &ruleset), codes(&[]));
    assert_eq!(warning_codes(&soldier, &ruleset), codes(&[]));

    // Soak: +8 (full metal scale armor). Source: ArMDE:1356 `#### The Standard Soldier`.
    assert_eq!(soak(&soldier, &ruleset).total, 8);

    let fatigue: Vec<i32> = fatigue_levels(&soldier, &ruleset)
        .iter()
        .map(|l| l.penalty)
        .collect();
    assert_eq!(fatigue, BOOK_FATIGUE_PENALTIES);

    assert_eq!(
        wound_bounds(&wound_ranges(&soldier, &ruleset)),
        size_zero_wound_bounds()
    );

    // Encumbrance: 3 (4). Source: ArMDE:1366 `#### The Standard Soldier`.
    let enc = encumbrance(&soldier, &ruleset);
    assert_eq!((enc.burden, enc.total), (4, 3));

    // Axe & Heater Shield: Init +0, Attack +12, Defense +11, Damage +7.
    // Source: ArMDE:1353 `#### The Standard Soldier`.
    let lines = combat_totals(&soldier, &ruleset);
    assert_eq!(
        stats(line(&lines, "weapon.axe", true)),
        (0, Some(12), 11, Some(7))
    );
    // Fist: Init -1, Attack +7, Defense +7, Damage +1. Source: ArMDE:1354 `#### The Standard Soldier`.
    assert_eq!(
        stats(line(&lines, "weapon.fist", false)),
        (-1, Some(7), 7, Some(1))
    );
}

// --- The Tough Guy (ArMDE:1370-1404 `#### The Tough Guy`) -------------------

#[test]
fn the_tough_guy_matches_the_book() {
    let ruleset = full_ruleset();
    let tough_guy = load(include_str!("fixtures/book_templates/grog_tough_guy.json"));

    assert_eq!(error_codes(&tough_guy, &ruleset), codes(&[]));
    // DISAGREEMENT T1 (docs/book-template-conformance.md), the same shape as B3:
    // Overconfident and Weakness are both Personality Flaws, against the grog
    // rule "You should not take more than one Personality Flaw" (ArMDE:2827 `#### Grogs`).
    assert_eq!(
        warning_codes(&tough_guy, &ruleset),
        codes(&["too_many_personality_flaws"])
    );

    // Soak: +13 (full metal scale armor + Tough). Source: ArMDE:1392 `#### The Tough Guy`.
    assert_eq!(soak(&tough_guy, &ruleset).total, 13);

    let fatigue: Vec<i32> = fatigue_levels(&tough_guy, &ruleset)
        .iter()
        .map(|l| l.penalty)
        .collect();
    assert_eq!(fatigue, BOOK_FATIGUE_PENALTIES);

    assert_eq!(
        wound_bounds(&wound_ranges(&tough_guy, &ruleset)),
        size_one_wound_bounds()
    );

    // Encumbrance: 3 (4). Source: ArMDE:1402 `#### The Tough Guy`.
    let enc = encumbrance(&tough_guy, &ruleset);
    assert_eq!((enc.burden, enc.total), (4, 3));

    // Axe & Heater Shield: Init -1, Attack +10, Defense +10, Damage +7.
    // Source: ArMDE:1389 `#### The Tough Guy`.
    let lines = combat_totals(&tough_guy, &ruleset);
    assert_eq!(
        stats(line(&lines, "weapon.axe", true)),
        (-1, Some(10), 10, Some(7))
    );
    // Fist: Init -2, Attack +3, Defense +4, Damage +1. Source: ArMDE:1390 `#### The Tough Guy`.
    assert_eq!(
        stats(line(&lines, "weapon.fist", false)),
        (-2, Some(3), 4, Some(1))
    );
}

// --- Darius of Flambeau, at Gauntlet (ArMDE:2285-2449 `## Detailed Character
// Creation`) ----------------------------------------------------------------

/// The core rulebook's one **worked** character, as he stands the day he passes
/// his Gauntlet — the state the Detailed Character Creation chapter builds him
/// to, before the post-apprenticeship advancement at ArMDE:2484-2492 `####
/// Magus Only — After Apprenticeship` carries him to 87.
///
/// He is a sharper instrument than the 23 statblock templates, because the book
/// shows its arithmetic at every step instead of only printing totals: the Flaw
/// points as they are spent (ArMDE:2336 `#### Quick and Easy Virtues and
/// Flaws`), the Virtue points (ArMDE:2338 `#### Quick and Easy Virtues and
/// Flaws`), the Characteristic buy narrated point by point (ArMDE:2358-2360
/// `### Characteristics`), the childhood and later-life experience
/// (ArMDE:2400, :2402 `#### Later Life`), the apprenticeship split
/// (ArMDE:2441, :2443 `#### Magus Only — Apprenticeship`) and the final 5
/// points (ArMDE:2449 `#### Magus Only — Apprenticeship`). Every figure below
/// is therefore checkable against a stated intermediate, not inferred from a
/// total.
///
/// Deliberately **not** asserted here: the Soak, Combat, Fatigue and Wound rows
/// printed at ArMDE:2556-2565 `### Equipment`. Those belong to the
/// 87-year-old, after 62 years of advancement, aging and Twilight that this
/// entity does not represent.
#[test]
fn darius_of_flambeau_at_gauntlet_matches_the_book() {
    let ruleset = full_ruleset();
    let darius = load(include_str!(
        "fixtures/book_templates/magus_darius_gauntlet.json"
    ));

    // Clean, and the empty error set is load-bearing: it is what proves the
    // fixture's `xp_pool` of 435 is **exact**. The book's own total is
    // 75 (native language) + 45 (childhood) + 15 x 5 (ages 5-10) + 240
    // (apprenticeship) = 435 (ArMDE:2378 `#### Early Childhood`, :2392 `####
    // Later Life`, :2400, :2402 `#### Later Life`, :2435 `#### Magus Only —
    // Apprenticeship`). Probed in
    // both directions while writing this test: at 436 the engine reports
    // `general_xp_unspent`, at 434 `not_enough_xp`. So every figure the book
    // narrates spending — 180 on apprenticeship Abilities, 55 on Arts, the last 5
    // on Parma — reproduces to the point.
    assert_eq!(error_codes(&darius, &ruleset), codes(&[]));
    assert_eq!(warning_codes(&darius, &ruleset), codes(&[]));

    // Arts (ArMDE:2443 `#### Magus Only — Apprenticeship`): "37 points on
    // Perdo, which his affinity turns into 56
    // points, so that he has Perdo 10 (1), and a bonus of +3 from Puissant", then
    // "15 exp on Creo 5, 3 exp on Corpus 2". Perdo reads 13 with the House
    // Virtue's +3 folded in (ArMDE:4820 `#### Puissant Art` — Puissant *Art*
    // is +3, unlike Puissant Ability's +2; see MAG14).
    assert_eq!(art_score(&darius, &ruleset, "art.perdo"), 13);
    assert_eq!(art_score(&darius, &ruleset, "art.creo"), 5);
    assert_eq!(art_score(&darius, &ruleset, "art.corpus"), 2);

    // The two Abilities his Virtues hand him outright, before any experience is
    // spent: "Darius has a number of free Abilities from his Virtues, so Niall
    // notes them first: Premonitions 1, Second Sight 1" (ArMDE:2398 `#### Later Life`). Neither is
    // bought in the fixture; both must still read 1.
    assert_eq!(
        ability_score(&darius, &ruleset, "ability.premonitions", None),
        1
    );
    assert_eq!(
        ability_score(&darius, &ruleset, "ability.second_sight", None),
        1
    );

    // Apprenticeship Abilities, the four the book names a cost for (ArMDE:2441
    // `#### Magus Only — Apprenticeship`):
    // "50 exp for Latin 4, 50 exp on Magic Theory 4, 30 exp on Artes Liberales 3
    // … 15 exp on Penetration 2".
    // The fixture types "Latin" (ArMDE:2441's own word); the catalogue-matching
    // fold (design § 5.3) resolves it to `Catalogued{id: language.latin}`, so the
    // instance query below names the resolved id, not the player-typed word.
    assert_eq!(
        ability_score(
            &darius,
            &ruleset,
            "ability.dead_language",
            Some("language.latin")
        ),
        4
    );
    assert_eq!(
        ability_score(&darius, &ruleset, "ability.magic_theory", None),
        4
    );
    assert_eq!(
        ability_score(&darius, &ruleset, "ability.artes_liberales", None),
        3
    );
    assert_eq!(
        ability_score(&darius, &ruleset, "ability.penetration", None),
        2
    );

    // "Finally, just before Gauntlet, he spends his last 5 exp on Parma Magica 1"
    // (ArMDE:2449 `#### Magus Only — Apprenticeship`) — the spend that makes the 240 apprenticeship points exact.
    assert_eq!(
        ability_score(&darius, &ruleset, "ability.parma_magica", None),
        1
    );

    // Reputation from Hermetic Prestige. The Virtue says level **4**
    // (ArMDE:4071-4073 `#### Hermetic Prestige`); the worked example says 3
    // ("it has a level of 3", ArMDE:2518 `### Reputations`), exactly as the Guernicus template
    // prints 3 (ArMDE:1869 `#### Guernicus`). The data follows the Virtue, so Darius is the
    // SECOND witness to MAG11 in docs/book-template-conformance.md and the
    // fixture records the Virtue's figure rather than the example's.
    let reputations = &darius.reputations;
    assert_eq!(reputations.len(), 1);
    assert_eq!(reputations[0].score, 4);

    // Spells (ArMDE:2445 `#### Magus Only — Apprenticeship`): seven, all
    // Perdo, 15+15+20+20+20+15+15 = 120 — exactly
    // the magus spell-levels budget (ArMDE:2435 `#### Magus Only —
    // Apprenticeship`), which an empty error set above
    // already proves is neither over- nor under-spent.
    assert_eq!(darius.spells.len(), 7);

    // Casting Totals. Pe 13 + Sta 0, plus the Form: PeCo 13 + 2 = 15, and PeAq /
    // PeIg / PeIm / PeMe 13 + 0 = 13. No aura is recorded and no Magical Focus is
    // held, so there is no within-focus figure.
    let casting = |spell: &str| spell_casting(&darius, &ruleset, spell);
    assert_eq!(casting("spell.the_wound_that_weeps"), (15, None));
    assert_eq!(casting("spell.dust_to_dust"), (15, None));
    assert_eq!(casting("spell.parching_wind"), (13, None));
    assert_eq!(casting("spell.soothe_the_raging_flames"), (13, None));
    assert_eq!(casting("spell.veil_of_invisibility"), (13, None));
    assert_eq!(casting("spell.calm_the_motion_of_the_heart"), (13, None));
    assert_eq!(casting("spell.loss_of_but_a_moments_memory"), (13, None));
}
