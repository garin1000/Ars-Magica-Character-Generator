//! X7c (`docs/vf-audit/phase-2-plan.md` row X7c) — row 46 residue
//! (`docs/open-todos.md` row 46; `corrections.md` § 8 row 12e): consumer-tracing
//! the computed (`creation_effect`/`in_play_effect`) entries in the V/F audit's
//! batches **B01–B11** (`ArMDE:3362-5931`), the span the sign-check instruction
//! did NOT yet cover (it entered the briefs only at B12). Excludes every entry
//! X7b-d (`tmp/x7bd-verdicts.md`) and X7b-e (`tmp/x7be-verdicts.md`) already
//! traced, and anything the D4/row-49 conditional-Lab-Total-modifier class or
//! X3's training gates already own.
//!
//! Phase 1 only: these tests assert the BOOK's number/scope through the
//! engine's real compute/data surface against the shipped ruleset, and are
//! expected to be RED until Phase 2's fix lands. See `tmp/x7c-verdicts.md` for
//! the full per-entry table (120 entries checked across parts 1-2, line-ordered,
//! B01 through the middle of B07) and `tmp/x7c-handover.md` for the remaining
//! range.
//!
//! Per CLAUDE.md's catalogue-size invariant, nothing here asserts a catalogue
//! total; every assertion is per-entry computed behaviour.
//!
//! Cross-checked against the concurrently in-flight `x3_trained_gate.rs`,
//! `x4_incompatibilities.rs`, `x5_prerequisites.rs`, `x5b_ability_minimums.rs`
//! and `x7a_lab_rows.rs` before each Part-2 finding was written, to avoid
//! duplicating a defect another slice already owns and tests (e.g.
//! `virtue.mendicant_friar`/`virtue.perfectus`'s Wealthy/Poor incompatibility
//! is X4's; `virtue.magister_in_artibus`/`_medicina`/`master_bard`'s ability-
//! and age-minimum prerequisites are X5b's).

use arm_rules::ruleset::{Ruleset, RulesetSources};
use arm_rules::types::*;
use serde_json::Value;

const EN_VF: &str = include_str!("../../../rules/i18n/en/virtues_flaws.json");
const DE_VF: &str = include_str!("../../../rules/i18n/de/virtues_flaws.json");

/// The text actually shown to the player for `id`: `description` if present,
/// else `summary` (mirrors `x5b_ability_minimums.rs::displayed_text` and
/// `x7bd_wrong_numbers.rs`'s inline equivalent — same fallback everywhere in
/// this audit).
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

fn load_ruleset() -> Ruleset {
    Ruleset::from_sources(RulesetSources {
        id: "arm5-core",
        version: "2024.1",
        point_items: include_str!("../../../rules/core/virtues_flaws.json"),
        type_profiles: include_str!("../../../rules/core/character_types.json"),
        abilities: Some(include_str!("../../../rules/core/abilities.json")),
        arts: Some(include_str!("../../../rules/core/arts.json")),
        houses: Some(include_str!("../../../rules/core/houses.json")),
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

/// `virtue.educated_vernacular` (ArMDE:3727-3729): "gains 50 additional
/// experience points, which must be spent on Academic Abilities, Bargain, the
/// Organization Lore **of the character's company**, Profession Merchant, or
/// the language of trade in the company's region." The Organization Lore
/// target is the character's OWN company only — the same "link the Ability to
/// a parameter on the granting Virtue" shape `virtue.craft_guild_training`
/// (ArMDE:3613-3616) already implements correctly for its own "guild" param
/// (`{"ability":"ability.organization_lore","instance":{"param":"guild"}}` in
/// `instances`, never in the unscoped `abilities` list).
///
/// `virtue.educated_vernacular` ships `ability.organization_lore` in the
/// UNSCOPED `abilities` list instead — `Effect::RestrictedAbilityXp::abilities`
/// is documented as "any instance" (`types.rs`) — so today the pool funds
/// Organization Lore for ANY organization (Order of Hermes, a rival guild, a
/// noble house — anything), not only the character's own company. Fixing this
/// needs a new parameter on the Virtue (there are none today) and moving the
/// target from `abilities` to `instances`, mirroring Craft Guild Training's
/// shape exactly — a data-only fix, no new engine capability.
#[test]
fn educated_vernacular_restricts_organization_lore_to_the_characters_own_company() {
    let rs = load_ruleset();
    let item = rs
        .item(&Id::new("virtue.educated_vernacular"))
        .expect("virtue.educated_vernacular is in the shipped catalogue");

    let unscoped_organization_lore = item.effects.iter().any(|e| {
        matches!(
            e,
            Effect::RestrictedAbilityXp { abilities, .. }
                if abilities.contains(&Id::new("ability.organization_lore"))
        )
    });

    assert!(
        !unscoped_organization_lore,
        "ArMDE:3727-3729 restricts the Organization Lore target to \
         'the character's company' only, not any organization; Educated \
         (Vernacular) must not list ability.organization_lore in the \
         UNSCOPED `abilities` list (which funds any instance) — it must be a \
         parameter-bound `instances` entry, like virtue.craft_guild_training's \
         own `guild` param. Shipped effects were: {:?}",
        item.effects
    );
}

// ---------------------------------------------------------------------------
// Part 2 (`ArMDE:3929-4820`, entries 61-120 of the 180-entry post-exclusion
// list) — see `tmp/x7c-verdicts.md` for the full per-entry table.
// ---------------------------------------------------------------------------

/// `virtue.inventive_genius` (ArMDE:4151-4154): "You get +3 to your Lab Total
/// if you are not using a Laboratory Text or being taught. **If you
/// experiment, you get +6.**" D4/X7a already rules the +3's "not using a Lab
/// Text/not being taught" condition holds by default in a creation Lab
/// Total, so `effective/spell.rs::lab_total_mod`'s unconditional +3 fold is
/// correct and not this test's concern.
///
/// The SECOND clause — the +6 alternative when the character experiments —
/// is a wholly separate stated rule that reaches the player NOWHERE: the
/// shipped `effects` carry only the flat `lab_total_mod: 3` (no
/// experimentation concept exists anywhere in the engine, `grep experiment
/// crates/arm-rules/src` is empty), and neither locale's `description`/
/// `summary` mentions it either (both are just "Invention comes naturally to
/// you."/"Erfinden liegt dir im Blut."). This is the exact shape D46/D67
/// already rule on `virtue.mythic_blood` (F-205, `x7bd_wrong_numbers.rs`):
/// an entry that correctly computes ONE clause while a second stated rule is
/// computed nowhere and described nowhere must carry that second rule as
/// `description` text in both locales (and, per D67, reclassify away from
/// `in_play_effect` while it does).
#[test]
fn inventive_genius_states_its_experimentation_bonus_in_both_locales() {
    let en = displayed_text(EN_VF, "virtue.inventive_genius");
    let de = displayed_text(DE_VF, "virtue.inventive_genius");
    assert!(
        en.contains('6') && en.to_lowercase().contains("experiment"),
        "ArMDE:4153 gives +6 (instead of +3) if the magus experiments; \
         English displayed text does not state it: {en:?}"
    );
    assert!(
        de.contains('6') && de.to_lowercase().contains("experiment"),
        "ArMDE:4153 (DE) gives +6 if the magus experiments; German \
         displayed text does not state it: {de:?}"
    );
}

/// `virtue.magian_lineage_minor`/`_major` (ArMDE:4339-4346): Minor grants "a
/// -1 bonus to Aging rolls **and a +3 bonus to resist the effects of
/// disease**"; Major includes the Minor's benefits in addition to its own
/// three-Ability connected-XP clause. Both shipped entries carry only
/// `aging_mod{kind: aging_roll, amount: -1}` — the aging half is correct —
/// but the disease-resistance bonus reaches the player NOWHERE: no disease
/// mechanic exists anywhere in the engine to compute it against (`grep
/// disease crates/arm-rules/src` is empty), and neither locale's
/// `description`/`summary` mentions disease at all (both are generic lineage
/// flavor text).
///
/// Major's OTHER gap — the three connected Arcane/Supernatural Abilities —
/// is already tracked (`tmp/x6-scope.md` § 1 "M-group", F-157/F-158) as
/// needing a new parameter X6 owns; this test does not touch that clause, only
/// the disease-resistance bonus both Minor and Major state.
#[test]
fn magian_lineage_states_its_disease_resistance_bonus_in_both_locales() {
    for id in ["virtue.magian_lineage_minor", "virtue.magian_lineage_major"] {
        let en = displayed_text(EN_VF, id);
        let de = displayed_text(DE_VF, id);
        assert!(
            en.to_lowercase().contains("disease"),
            "{id}: ArMDE:4343 gives a +3 bonus to resist disease; English \
             displayed text does not state it: {en:?}"
        );
        assert!(
            de.to_lowercase().contains("krankheit"),
            "{id}: ArMDE:4343 (DE) gives a +3 bonus to resist disease; \
             German displayed text does not state it: {de:?}"
        );
    }
}

/// `virtue.physician_of_salerno` (ArMDE:4732-4735): "he carries the
/// reputation of the school **with him** (granting a Reputation of Physician
/// of Salerno 2)" — a reputation that travels WITH the character is
/// precisely the one shape `ReputationType::Local` ("known to those who live
/// near the character") excludes, per its own doc comment. This question is
/// already SETTLED (`docs/vf-audit/corrections.md` Q-55, 2026-09-22): "local
/// is positively wrong… the pin is dropped and the grant ships the
/// player-chosen wildcard (`kind: None`)" — explicitly rejecting `academic`
/// too, as "a guess dressed as data". `flaw.infamous` got the matching fix
/// (F-450); this entry did not.
///
/// The shipped effect still carries `kind: Some(Local)`, contradicting the
/// settled ruling.
#[test]
fn physician_of_salerno_reputation_kind_is_player_chosen_not_pinned_local() {
    let rs = load_ruleset();
    let item = rs
        .item(&Id::new("virtue.physician_of_salerno"))
        .expect("virtue.physician_of_salerno is in the shipped catalogue");

    let pinned_local = item.effects.iter().any(|e| {
        matches!(
            e,
            Effect::GrantsReputation {
                kind: Some(ReputationType::Local),
                ..
            }
        )
    });

    assert!(
        !pinned_local,
        "ArMDE:4732-4735, Q-55 (settled 2026-09-22): the granted Reputation \
         travels with the character, so `local` is positively wrong and the \
         kind must be the player-chosen wildcard (`None`), not pinned to any \
         `ReputationType` — least of all `Local`. Shipped effects were: {:?}",
        item.effects
    );
}

// ---------------------------------------------------------------------------
// Part 3 (`ArMDE:4822-5931`, entries 121-180 of the 180-entry post-exclusion
// list) — see `tmp/x7c-verdicts.md` for the full per-entry table.
// ---------------------------------------------------------------------------

/// `flaw.age_quickly` (ArMDE:5659-5662): "you age twice as fast as normal
/// people... your effective age... increases two years for every year that
/// passes, and you make two aging rolls each year." `flaw.baneful_circumstances`
/// (ArMDE:5687-5690): "the character cannot recover Fatigue, heal wounds, or
/// recover Might [under stated conditions], and if... the character has spent
/// more than half of his time subject to these conditions, he must make an
/// additional Aging roll."
///
/// Both ship only `aging_mod { kind: aging_roll, amount: 0 }` — a modifier of
/// literally zero, i.e. a no-op — and no other effect. Neither locale's text
/// is a `description`; both fall back to a one-sentence flavor `summary`
/// ("you age twice as fast as normal people" / "weakens him in relatively
/// common circumstances") that states neither the doubled cadence, nor the
/// Fatigue/healing/Might block, nor the conditional extra roll. No engine
/// concept for "rolls twice a year" or "an additional roll under a condition"
/// exists anywhere (`AgingEffect` is `AgingRoll`/`LongevityBonus`/`NoAging`/
/// `NoApparentAging` only — `grep AgingEffect types.rs`). Same D46/D67 shape
/// as `inventive_genius`/`magian_lineage`: a computed entry whose stated rule
/// reaches the player nowhere.
#[test]
fn age_quickly_and_baneful_circumstances_state_their_mechanic_in_both_locales() {
    for id in ["flaw.age_quickly", "flaw.baneful_circumstances"] {
        let en = displayed_text(EN_VF, id);
        let de = displayed_text(DE_VF, id);
        assert!(
            en.to_lowercase().contains("aging roll") || en.to_lowercase().contains("two years"),
            "{id}: the aging-roll mechanic (ArMDE:5661 / :5689) is not stated \
             in the English displayed text: {en:?}"
        );
        assert!(
            de.to_lowercase().contains("alterungswurf") || de.to_lowercase().contains("zwei jahre"),
            "{id}: the aging-roll mechanic is not stated in the German \
             displayed text: {de:?}"
        );
    }
}

/// `virtue.strong_faerie_blood` (ArMDE:5032-5047): "First, you have natural
/// longevity. You start making aging rolls at the age of fifty, rather than
/// the normal 35, and get -3 to Aging Rolls". The `-3` half ships correctly
/// (`aging_mod kind:aging_roll amount:-3`), but the "starts rolling at 50, not
/// 35" clause reaches the player NOWHERE: no such concept exists in
/// `AgingEffect` (checked above), and both locales' text is flavor-only
/// ("The blood of the fay is strong in you." / German equivalent) with no
/// mention of the age-50 threshold. Same shape as `magian_lineage`'s
/// disease-resistance gap — a second stated rule computed and described
/// nowhere.
#[test]
fn strong_faerie_blood_states_its_delayed_aging_onset_in_both_locales() {
    let en = displayed_text(EN_VF, "virtue.strong_faerie_blood");
    let de = displayed_text(DE_VF, "virtue.strong_faerie_blood");
    assert!(
        en.contains('5') && en.to_lowercase().contains("aging"),
        "ArMDE:5036: aging rolls start at 50 instead of 35; English displayed \
         text does not state it: {en:?}"
    );
    assert!(
        de.contains('5') && de.to_lowercase().contains("alter"),
        "ArMDE:5036 (DE): aging rolls start at 50 instead of 35; German \
         displayed text does not state it: {de:?}"
    );
}

/// `flaw.creative_block` (ArMDE:5873-5876): "You take a -3 penalty to all Lab
/// Totals unless you are using a Laboratory Text or being taught. **If you
/// experiment, roll twice as many dice on the experimentation table.**" The
/// first clause ships correctly (`lab_total_mod amount:-3`, D4-consistent —
/// the "unless Lab Text/taught" condition holds by default in a creation Lab
/// Total). The SECOND clause reaches the player NOWHERE: no experimentation
/// concept exists anywhere in the engine (`grep experiment
/// crates/arm-rules/src` is empty, same root cause as `virtue.inventive_genius`'s
/// already-RED test above), and — unlike `flaw.clumsy_magic`, whose
/// auto-botch clause is at least stated in its own `description` — this
/// entry has no `description` key in either locale at all, only a one-sentence
/// flavor `summary` ("You have problems creating new things in the lab."/
/// German equivalent). So unlike clumsy_magic, this fails D20's RAW-fidelity
/// test outright: the rule reaches the player neither as a computed number
/// nor as written text. Same D46/D67 shape as `inventive_genius`/
/// `magian_lineage`, but a distinct shipped entry needing its own text fix —
/// fixing `inventive_genius`'s experimentation concept does not by itself add
/// text to this entry's locale files.
#[test]
fn creative_block_states_its_experimentation_dice_penalty_in_both_locales() {
    let en = displayed_text(EN_VF, "flaw.creative_block");
    let de = displayed_text(DE_VF, "flaw.creative_block");
    assert!(
        en.to_lowercase().contains("experiment") && en.to_lowercase().contains("twice"),
        "ArMDE:5875-5876 doubles the experimentation dice; English displayed \
         text does not state it: {en:?}"
    );
    assert!(
        de.to_lowercase().contains("experiment"),
        "ArMDE:5875-5876 (DE) doubles the experimentation dice; German \
         displayed text does not state it: {de:?}"
    );
}
