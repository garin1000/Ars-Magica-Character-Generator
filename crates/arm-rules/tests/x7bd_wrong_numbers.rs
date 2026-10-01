//! X7b-d (`docs/vf-audit/phase-2-plan.md` row X7b-d) — `docs/vf-audit/corrections.md`
//! § 3.9 "Wrong numbers and wrong arithmetic" and § 3.10 "Missing effects the
//! engine can already express", excluding every row-42 entry (owned by X7b-e,
//! running in parallel) and excluding § 3.17's two entries (F-428/F-439).
//!
//! Phase 1 only: these tests assert the BOOK's number/behaviour through the
//! engine's real compute functions (`validate`, `effective::*`, `derived::*`,
//! `aging::*`) against the shipped ruleset, and are expected to be RED until
//! Phase 2's fix lands. See `tmp/x7bd-verdicts.md` for the full per-finding
//! citation, rationale, and the entries that turned out to be already fixed,
//! excluded, or blocked on another slice — this file intentionally does not
//! re-derive that reasoning inline, to stay a plain behaviour check.
//!
//! Per CLAUDE.md's catalogue-size invariant, nothing here asserts a catalogue
//! total; every assertion is per-entry computed behaviour.

use arm_rules::ruleset::{Ruleset, RulesetSources};
use arm_rules::types::*;
use arm_rules::validation::{ValidationResult, validate};
use arm_rules::{
    FatigueTier, aging_total, combat_totals, confidence, entity_grants, fatigue_levels, lab_totals,
    power_levels_budget, resolve_outcome, true_faith,
};
use std::collections::BTreeMap;

const SHIPPED_HOUSES: &str = include_str!("../../../rules/core/houses.json");

fn load_ruleset() -> Ruleset {
    Ruleset::from_sources(RulesetSources {
        id: "arm5-core",
        version: "2024.1",
        point_items: include_str!("../../../rules/core/virtues_flaws.json"),
        type_profiles: include_str!("../../../rules/core/character_types.json"),
        abilities: Some(include_str!("../../../rules/core/abilities.json")),
        arts: Some(include_str!("../../../rules/core/arts.json")),
        houses: Some(SHIPPED_HOUSES),
        characteristics: Some(include_str!("../../../rules/core/characteristics.json")),
        life_stages: Some(include_str!("../../../rules/core/life_stages.json")),
        equipment: Some(include_str!("../../../rules/core/equipment.json")),
        aging: Some(include_str!("../../../rules/core/aging.json")),
        parameter_catalogues: Some(include_str!(
            "../../../rules/core/parameter_catalogues.json"
        )),
        ..RulesetSources::default()
    })
    .unwrap()
}

/// Builds a character entity of `type_id` with the given selections, at the
/// current schema version and empty trait data — same shape as
/// `x1_authorization_family.rs::entity`, duplicated here since integration
/// test binaries cannot share private helpers.
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

fn score(ability: &str, value: u8) -> AbilityScore {
    AbilityScore::new(Id::new(ability), value)
}

fn equipped(item: &str, specialization_applies: bool) -> EquipmentSlot {
    EquipmentSlot {
        item: Id::new(item),
        loadout: LoadoutState::Wielded,
        specialization_applies,
    }
}

fn issue_codes(result: &ValidationResult) -> Vec<&str> {
    result.issues.iter().map(|i| i.code.as_str()).collect()
}

// --- § 3.9 — wrong numbers and wrong arithmetic ----------------------------

/// F-89 — `virtue.ferocity`'s `confidence_bonus` is a delta over the type
/// profile's own base, but the passage ("Like companion and magus characters,
/// this character has Confidence points") exists only to give an *animal* —
/// which has no base Confidence — what a companion/magus already has. On a
/// companion the delta used to double-count: Confidence Score 2 / Points 6,
/// where the book gives an animal Score 1 / Points 3 and a companion nothing
/// extra at all.
///
/// RE-SCOPED, not merely un-ignored (Q-11/D58 is now settled: animal
/// characters are a deliberate non-goal, so this Virtue's "animals only"
/// restriction is not a computable axis to build — it must instead become
/// unselectable by every buildable character type, exactly like
/// `virtue.domestic_animal`'s own F-556 remedy). X5a's gate
/// (`prerequisites: {kind: none, value: [is_grog, is_companion,
/// order_member]}`, ArMDE:3873-3876) covers grog, companion, magus AND
/// mythic_companion (`is_companion` is true for both), since none of the four
/// shipped types are animals. Once Ferocity can never be legally selected at
/// all, the ORIGINAL arithmetic assertion (compute `confidence()` directly on
/// an entity holding it) tests a state `validate()` now refuses outright —
/// asserting unselectability is the meaningful replacement, not the
/// now-moot arithmetic. The double-count "disappears with it" exactly as
/// `docs/vf-audit/batch-03.md`'s own F-89 analysis predicted for this branch.
#[test]
fn f89_ferocity_does_not_double_the_companion_profiles_own_confidence_base() {
    let rs = load_ruleset();
    for type_id in ["grog", "companion", "magus", "mythic_companion"] {
        let e = entity(type_id, vec![sel("virtue.ferocity")]);
        let result = validate(&e, &rs);
        assert!(
            issue_codes(&result).contains(&"prereq_not_met"),
            "ArMDE:3873-3876 — Ferocity is animals-only (Q-11/D58); a \
             {type_id} taking it must be refused, got: {:?}",
            issue_codes(&result)
        );
    }
}

/// F-256 — `virtue.relic`/`virtue.powerful_relic` ship `true_faith_grant`,
/// which `effective::true_faith` sums straight into the CHARACTER's own True
/// Faith Score. ArMDE:17607 is explicit: "Only by possessing the True Faith
/// Major Virtue may a character have a True Faith score." A relic's score
/// belongs to the relic (ArMDE:17623), not its bearer. This is no longer only
/// a display defect: `derived::magic_resistance`'s Q12 fix reads
/// `true_faith() * 10` as a floor, so a Relic-holder today gets a bogus Magic
/// Resistance floor too, on top of a bogus displayed True Faith Score.
#[test]
fn f256_relic_does_not_grant_the_character_a_true_faith_score() {
    let rs = load_ruleset();
    let e = entity("companion", vec![sel("virtue.relic")]);

    assert_eq!(
        true_faith(&e, &rs),
        0,
        "only virtue.true_faith may give a character a True Faith SCORE \
         (ArMDE:17607); virtue.relic states a score for the RELIC, not its \
         bearer"
    );
}

/// F-306 — `virtue.self_confident`'s `confidence_bonus` is a delta (+1/+2)
/// correct for companion/magus/mythic_companion, whose base is 1/3 (giving the
/// stated 2/5), but wrong for a grog: ArMDE:1161/:2522 flatly deny grogs
/// Confidence at all, and the grog profile's base is 0/0 by design. A grog who
/// takes this Minor Virtue is computed at Score 1 / Points 2 today, a state
/// the book forbids outright.
#[test]
fn f306_self_confident_does_not_give_a_grog_a_confidence_score() {
    let rs = load_ruleset();
    let e = entity("grog", vec![sel("virtue.self_confident")]);

    let c = confidence(0, 0, &e, &rs);

    assert_eq!(
        c.score, 0,
        "grogs don't have Confidence (ArMDE:1161); Self-Confident's delta \
         must not give one a Confidence Score, got {}",
        c.score
    );
    assert_eq!(
        c.points, 0,
        "grogs don't have Confidence Points (ArMDE:2522); got {}",
        c.points
    );
}

/// F-449 — `flaw.low_tolerance`'s `fatigue_penalty` delta (-1) is folded
/// uniformly over all five Fatigue tiers by `derived::fatigue_levels`, but
/// ArMDE:17129 states only Weary/Tired/Dazed carry a penalty at all — Fresh
/// and Winded are not "reduced Fatigue levels" in any reading. Today's engine
/// invents a standing -1 on Fresh and Winded that no passage grants.
#[test]
fn f449_low_tolerance_does_not_penalize_fresh_or_winded() {
    let rs = load_ruleset();
    let e = entity("companion", vec![sel("flaw.low_tolerance")]);

    let levels = fatigue_levels(&e, &rs);
    let fresh = levels
        .iter()
        .find(|l| l.level == FatigueTier::Fresh)
        .unwrap();
    let winded = levels
        .iter()
        .find(|l| l.level == FatigueTier::Winded)
        .unwrap();

    assert_eq!(
        fresh.penalty, 0,
        "ArMDE:17129 gives Fresh no penalty at all; Low Tolerance must not \
         invent one, got {}",
        fresh.penalty
    );
    assert_eq!(
        winded.penalty, 0,
        "ArMDE:17129 gives Winded no penalty at all; Low Tolerance must not \
         invent one, got {}",
        winded.penalty
    );
}

/// F-462 — `flaw.missing_eye` ships one unscoped `combat_mod { amount: -1,
/// target: attack }`, so `derived::combat_totals` folds it onto EVERY
/// equipped weapon's Attack line. ArMDE:6436 states two different amounts for
/// two different weapon classes: -3 on missile/spell-targeting rolls, -1 in
/// melee. `Effect::CombatMod`'s `weapon` field (already used by `flaw.lame`,
/// per `corrections.md` F-462) is exactly the scoping mechanism this needs.
/// Today a character with Missing Eye and a bow is shown an Attack total only
/// one point down, not three.
#[test]
fn f462_missing_eye_applies_the_stated_ranged_penalty_not_the_melee_one() {
    let rs = load_ruleset();
    let mut e = entity("companion", vec![sel("flaw.missing_eye")]);
    e.ability_scores = vec![score("ability.bows", 5)];
    e.equipment = vec![equipped("weapon.bow_short", false)];

    let lines = combat_totals(&e, &rs);
    let bow = lines
        .iter()
        .find(|l| l.weapon == Id::new("weapon.bow_short"))
        .expect("the bow's combat line is present");

    // Baseline without the Flaw, to isolate exactly what the Flaw changed.
    let mut baseline_entity = entity("companion", vec![]);
    baseline_entity.ability_scores = e.ability_scores.clone();
    baseline_entity.equipment = e.equipment.clone();
    let baseline_lines = combat_totals(&baseline_entity, &rs);
    let baseline = baseline_lines
        .iter()
        .find(|l| l.weapon == Id::new("weapon.bow_short"))
        .expect("the bow's combat line is present without the Flaw too");

    let delta = bow.attack.unwrap() - baseline.attack.unwrap();
    assert_eq!(
        delta, -3,
        "ArMDE:6436 gives Missing Eye a -3 on missile Attack rolls; the \
         shipped unscoped -1 effect yields a delta of {delta} instead"
    );
}

// --- § 3.10 — missing effects the engine can already express --------------

/// F-21 — `virtue.blood_of_the_nephilim`'s -5 Aging Roll modifier
/// (ArMDE:3517) is `AgingEffect::AgingRoll`'s exact shape (Faerie Blood's -1
/// is the shipped precedent), and the entry carries no `aging_mod` at all
/// today.
#[test]
fn f21_blood_of_the_nephilim_applies_its_stated_aging_roll_penalty() {
    let rs = load_ruleset();
    let e = entity("companion", vec![sel("virtue.blood_of_the_nephilim")]);

    let total = aging_total(&e, &rs, 40, 5).expect("the shipped ruleset carries aging rules");
    assert_eq!(
        total.trait_modifier, -5,
        "ArMDE:3517 gives Blood of the Nephilim a -5 Aging Roll modifier; \
         got a trait_modifier of {}",
        total.trait_modifier
    );
}

/// F-49 — `virtue.demonic_blood` states, clause for clause, the engine's own
/// `no_apparent_aging`/`no_aging` pair (ArMDE:3659; `virtue.unaging` already
/// ships exactly this pair). Today the entry carries neither, so a Demonic
/// Blooded character's apparent age advances exactly like an unmodified
/// character's — which the book flatly denies.
#[test]
fn f49_demonic_blood_suppresses_apparent_aging() {
    let rs = load_ruleset();
    let e = entity("companion", vec![sel("virtue.demonic_blood")]);

    let outcome = resolve_outcome(&e, &rs, 20).expect("the shipped ruleset carries aging rules");
    assert!(
        !outcome.apparent_age_increases,
        "ArMDE:3659 ('she does not show the effects of aging') must suppress \
         the apparent-age increase at every total; a total of 20 still \
         reports apparent_age_increases = true"
    );
}

// F-196 — routed to X5 (D70): `virtue.mercurian_magic`'s Ceremonial
// Spontaneous Magic prerequisite is a `Prereq`/character-creation-order
// concern owned by X5's slice, not this one. Test removed here.

/// F-463 — `flaw.magical_air` (ArMDE:6384) states outright "You may not take
/// this Flaw if you actually do have The Gift", but no `incompatible_with`
/// or `prerequisites` names it (or is named by it), and no profile blocks
/// the pairing — so today a magus (who always holds The Gift as a bought
/// selection via `virtue.hermetic_magus`'s own prereq) can hold Magical Air
/// too, which the passage forbids outright.
#[test]
fn f463_magical_air_is_incompatible_with_the_gift() {
    let rs = load_ruleset();
    let e = entity(
        "magus",
        vec![
            sel("virtue.the_gift"),
            sel("virtue.hermetic_magus"),
            sel("flaw.magical_air"),
        ],
    );

    let result = validate(&e, &rs);
    assert!(
        issue_codes(&result).contains(&"incompatible"),
        "ArMDE:6384 forbids Magical Air on a Gifted character outright; \
         validate() issues were: {:?}",
        issue_codes(&result)
    );
}

/// F-208 — `virtue.nephilim`'s free Strong Angelic Heritage grant
/// (ArMDE:4596) is a fixed, single-item, free grant — `Effect::GrantsSelection`'s
/// exact shape, already carried by the sibling Mythic Companion Virtues
/// `virtue.faerie_doctor`/`virtue.spirit_votary`. `virtue.nephilim` carries no
/// effects at all, so a Nephilim built by picking the Virtue directly from the
/// V/F list (legal: `mythic_companion`'s profile permits the `mythic_companion`
/// category) never receives it.
#[test]
fn f208_nephilim_grants_strong_angelic_heritage() {
    let rs = load_ruleset();
    let e = entity("mythic_companion", vec![sel("virtue.nephilim")]);

    let grants = entity_grants(&e, &rs);
    assert!(
        grants
            .iter()
            .any(|g| g.item_ref == Id::new("virtue.strong_angelic_heritage")),
        "ArMDE:4596 grants Strong Angelic Heritage free with Nephilim; \
         entity_grants() returned: {:?}",
        grants
            .iter()
            .map(|g| g.item_ref.as_str())
            .collect::<Vec<_>>()
    );
}

/// F-243/F-342 (second half) — `virtue.redcap`'s passage (ArMDE:4848) states
/// "In addition, you have the Well-Traveled Virtue (page 116) at no cost",
/// the identical fixed-single-item-free-grant shape `virtue.lone_redcap`
/// already carries for the same Virtue. `virtue.redcap` carries no
/// `grants_selection` at all, so a Redcap must buy Well-Traveled with a point
/// the book says is free, or go without its 50 XP.
#[test]
fn f243_redcap_grants_well_traveled() {
    let rs = load_ruleset();
    let e = entity("companion", vec![sel("virtue.redcap")]);

    let grants = entity_grants(&e, &rs);
    assert!(
        grants
            .iter()
            .any(|g| g.item_ref == Id::new("virtue.well_traveled")),
        "ArMDE:4848 grants Well-Traveled free with Redcap; entity_grants() \
         returned: {:?}",
        grants
            .iter()
            .map(|g| g.item_ref.as_str())
            .collect::<Vec<_>>()
    );
}

/// F-249 — `virtue.ripper`'s two named powers (PeAn(He) 25, PeAn 45 —
/// ArMDE:4868) total 70 levels, exactly `Effect::PowerLevels`'s shape
/// (`virtue.personal_power`/`virtue.ritual_power` already carry it for
/// player-designed powers). The entry grants no `power_levels`, so
/// `validation::might::validate_powers` charges the entity's 70 recorded
/// levels against a budget of 0 and refuses a character the book describes
/// as legal.
#[test]
fn f249_ripper_funds_its_own_seventy_levels_of_power() {
    let rs = load_ruleset();
    let mut e = entity("companion", vec![sel("virtue.ripper")]);
    e.powers = vec![
        SupernaturalPower {
            name: "Destroy Cloth".into(),
            level: 25,
            penetration: 0,
        },
        SupernaturalPower {
            name: "Disembowel Animal".into(),
            level: 45,
            penetration: 0,
        },
    ];

    let result = validate(&e, &rs);
    assert!(
        !issue_codes(&result).contains(&"over_power_levels"),
        "ArMDE:4868 describes a Ripper's two powers (70 levels total) as a \
         legal, book-defined build; budget is {}, validate() issues were: \
         {:?}",
        power_levels_budget(&e, &rs),
        issue_codes(&result)
    );
}

/// F-406 — `flaw.a_deal_with_the_devil` (ArMDE:5905-5908) states "This Flaw
/// includes the effects of Plagued By Supernatural Entity", the same
/// "includes the effects of the X Virtue/Flaw" idiom the catalogue encodes
/// four other times (`virtue.rosh_beth_din`, `virtue.templar_commander`,
/// `flaw.bound_to_role_role`, `virtue.spirit_votary`) — every one of them via
/// `grants_selection` or an equivalent effect. This entry carries none.
#[test]
fn f406_a_deal_with_the_devil_includes_plagued_by_supernatural_entity() {
    let rs = load_ruleset();
    let e = entity("companion", vec![sel("flaw.a_deal_with_the_devil")]);

    let grants = entity_grants(&e, &rs);
    assert!(
        grants
            .iter()
            .any(|g| g.item_ref == Id::new("flaw.plagued_by_supernatural_entity")),
        "ArMDE:5905-5908 states A Deal with the Devil includes Plagued By \
         Supernatural Entity's effects; entity_grants() returned: {:?}",
        grants
            .iter()
            .map(|g| g.item_ref.as_str())
            .collect::<Vec<_>>()
    );
}

// F-494 — routed out (D69): `flaw.raised_from_the_dead`'s warping_grant_param
// + grants_reputation effects already shipped via X7b-e; this test is already
// green and redundant. Test removed here.

/// F-205 — `virtue.mythic_blood` (ArMDE:4588) states a free hereditary Minor
/// Personality Flaw "at no extra cost" alongside the Minor Magical Focus the
/// entry already delivers. The grant is *open* (any Minor Personality Flaw),
/// which `Effect::GrantsSelection`'s fixed-id set structurally cannot express
/// (D3/D67), so the correct fix is text — but today neither locale's
/// `description` states the clause at all.
#[test]
fn f205_mythic_blood_states_its_free_personality_flaw_in_both_locales() {
    let en: serde_json::Value =
        serde_json::from_str(include_str!("../../../rules/i18n/en/virtues_flaws.json")).unwrap();
    let de: serde_json::Value =
        serde_json::from_str(include_str!("../../../rules/i18n/de/virtues_flaws.json")).unwrap();

    for (lang, doc) in [("en", &en), ("de", &de)] {
        let text = doc["virtue.mythic_blood"]["description"]
            .as_str()
            .or_else(|| doc["virtue.mythic_blood"]["summary"].as_str())
            .unwrap_or("");
        let mentions_personality_flaw =
            text.to_lowercase().contains("personality") || text.contains("Persönlichkeits");
        assert!(
            mentions_personality_flaw,
            "{lang}/virtue.mythic_blood: ArMDE:4588's free hereditary Minor \
             Personality Flaw must reach the player as text (D3/D67); \
             displayed text was: {text:?}"
        );
    }
}

/// F-77 — `virtue.faerie_raised_magic` (ArMDE:3839) states "This Virtue also
/// includes the Virtue Spell Improvisation", but the shipped entry carries
/// only its own `special_casting_mod { kind: faerie_raised }` — never the
/// second kind (`spell_improvisation`) the cross-reference entitles it to.
#[test]
fn f77_faerie_raised_magic_also_carries_spell_improvisation() {
    let rs = load_ruleset();
    let item = rs
        .item(&Id::new("virtue.faerie_raised_magic"))
        .expect("virtue.faerie_raised_magic is in the shipped catalogue");

    let carries_spell_improvisation = item.effects.iter().any(|e| {
        matches!(
            e,
            Effect::SpecialCastingMod {
                kind: SpecialCasting::SpellImprovisation,
                ..
            }
        )
    });
    let grants_it = item
        .effects
        .iter()
        .any(|e| matches!(e, Effect::GrantsSelection { items } if items.contains(&Id::new("virtue.spell_improvisation"))));

    assert!(
        carries_spell_improvisation || grants_it,
        "ArMDE:3839 entitles Faerie-Raised Magic to Spell Improvisation's \
         effect as well as its own; shipped effects were: {:?}",
        item.effects
    );
}

/// D4 (`decisions.md`) — `virtue.potent_magic_major`/`_minor`'s `lab_total_mod`
/// applies only "within the chosen [Magical] focus" (ArMDE:4740-4781), which
/// `derived::lab_totals` already has a field for (`LabTotal::within_focus`,
/// gated on `mods.has_focus`) — but the ruling is that the flat bonus must
/// land THERE ONLY, never in the ordinary `total`. Today `lab_mod` folds every
/// `lab_total_mod` effect into `base` unconditionally, so `total` and
/// `within_focus` both carry it: a magus with Potent Magic gets the +6 on
/// every Lab Total, including a Longevity Ritual or an enchanted item outside
/// his focus (flagged by X2c as an engine defect for whichever slice
/// implements D4 — not itself an `F-`numbered corrections.md finding, but
/// explicitly in this slice's collection brief).
///
/// **D79 correction**: "within the chosen [Magical] focus" was D4's own
/// reading of ArMDE:4742's "much as in a Magical Focus" — which turned out to
/// describe the SHAPE of Potent Magic's field (narrow/wide, like a Minor/Major
/// Focus), not its gate. D79 re-reads ArMDE:4740-4748 and separates the two:
/// Potent Magic's bonus now lands in its OWN `within_potent_field` figure,
/// gated on holding Potent Magic itself, never in `within_focus` (gated on
/// holding a Magical Focus) — a character could hold either Virtue without
/// the other, and D4's original reading made that case silently lose the
/// bonus entirely (Potent Magic with no Focus) while another case leaked it
/// into the wrong figure (both held). This test is renamed and rewritten to
/// pin the corrected behavior; `crates/arm-rules/tests/d79_potent_magic.rs`
/// covers the rest of D79 (the Casting Total mirror, the per-spell marker, the
/// ArMDE:4742 one-Virtue-applies bound).
#[test]
fn d79_potent_magic_lab_bonus_applies_only_within_its_own_field_not_the_focus() {
    let rs = load_ruleset();
    let focus_params: BTreeMap<String, Id> = [("focus".to_string(), Id::new("fire"))]
        .into_iter()
        .collect();

    let baseline = entity(
        "companion",
        vec![Selection::with_params(
            Id::new("virtue.major_magical_focus"),
            focus_params.clone(),
        )],
    );
    let with_potent_magic = entity(
        "companion",
        vec![
            Selection::with_params(Id::new("virtue.major_magical_focus"), focus_params),
            sel("virtue.potent_magic_major"),
        ],
    );

    let baseline_cell = &lab_totals(&baseline, &rs)[0];
    let potent_cell = &lab_totals(&with_potent_magic, &rs)[0];

    assert_eq!(
        potent_cell.total, baseline_cell.total,
        "ArMDE:4740-4781 gives Potent Magic's +6 only within its own field; \
         the ordinary Lab Total (outside that field) must be unaffected. \
         baseline total {}, with Potent Magic {}",
        baseline_cell.total, potent_cell.total
    );
    assert_eq!(
        potent_cell.within_focus, baseline_cell.within_focus,
        "D79: the Magical-Focus figure must NOT include Potent Magic's bonus \
         — the two free-text themes are independent. baseline within_focus \
         {:?}, with Potent Magic {:?}",
        baseline_cell.within_focus, potent_cell.within_focus
    );
    assert_eq!(
        potent_cell.within_potent_field,
        Some(baseline_cell.total + 6),
        "D79: Potent Magic gets its OWN within-field figure, independent of \
         whether a Magical Focus is also held: {:?}",
        potent_cell.within_potent_field
    );
    assert_eq!(
        baseline_cell.within_potent_field, None,
        "no Potent Magic is held in the baseline, so this figure must stay None"
    );
}

/// F-351 — `types.rs::HealthTrack::CastingFatigue`'s own doc comment and the
/// Fluent label `derived-detail-casting_fatigue` both describe the track as a
/// *cost*, but Q-81 settled that the DATA convention (positive = better for
/// the character) is the one to keep: the three shipped amounts are correct
/// and must NOT be flipped. The label is what is backwards — a Withstand
/// Casting **Virtue** prints "Casting fatigue: +1", read naturally as bad.
/// This pins the two textual defects the ruling actually calls for (comment +
/// label), independent of the (correct, untouched) data.
#[test]
fn f351_casting_fatigue_label_and_comment_no_longer_read_as_a_cost() {
    let types_src = include_str!("../src/types.rs");
    let doc_comment_line = types_src
        .lines()
        .find(|l| l.contains("Fatigue levels lost per spell cast"))
        .expect("the CastingFatigue doc comment is present");
    assert!(
        !doc_comment_line.contains("Vulnerable Casting +1"),
        "Q-81 settled that positive = better for the character; the doc \
         comment must not describe Vulnerable Casting (a Flaw) as +1, which \
         is backwards from what the shipped data carries. Line: {doc_comment_line:?}"
    );

    let en_ftl = include_str!("../../../locales/en/main.ftl");
    let de_ftl = include_str!("../../../locales/de/main.ftl");
    for (lang, ftl) in [("en", en_ftl), ("de", de_ftl)] {
        let line = ftl
            .lines()
            .find(|l| l.trim_start().starts_with("derived-detail-casting_fatigue"))
            .unwrap_or_else(|| panic!("{lang}/main.ftl carries the casting_fatigue label"));
        assert!(
            !line.trim_end().ends_with("Casting fatigue")
                && !line.trim_end().ends_with("Zauber-Erschöpfung"),
            "{lang}: Q-81 requires the label renamed so a positive value \
             (Withstand Casting, a Virtue) does not read as a cost. Line: \
             {line:?}"
        );
    }
}
