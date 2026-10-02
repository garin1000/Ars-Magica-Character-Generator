//! Review finding `tmp/review-d81.json` #1: `validate_spell_level_cap`
//! (`validation/magus.rs`) and `spell_casting_total`
//! (`derived/casting.rs`) both read `SpellSelection::within_focus` verbatim,
//! with no check that the entity still actually holds a Magical Focus. A
//! player can mark a spell "within focus" via the picker, then remove the
//! Magical Focus Virtue from Virtues/Flaws, and keep an over-cap spell with
//! no `spell_level_exceeds_cap` error — plus an inflated per-spell Casting
//! Total. `within_focus` must take effect only while the entity holds a
//! Magical Focus (reusing `effective/spell.rs::has_magical_focus`'s
//! predicate, not a hardcoded id list), without mutating the saved marker
//! (saves store choices, D-whatever).

use arm_rules::derived::spell_casting_total;
use arm_rules::ruleset::{Ruleset, RulesetSources};
use arm_rules::types::*;
use arm_rules::validation::validate;
use std::collections::BTreeMap;

/// The shipped core ruleset — duplicated per-binary per the established
/// convention (`x10bc_banked_xp_and_within_focus.rs`,
/// `x5b_ability_minimums.rs`): integration test binaries cannot share
/// private helpers.
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
    let mut e = Entity::new(
        EntityKind::Character,
        Id::new("magus"),
        RulesetRef::new(Id::new("arm5-core"), "2024.1"),
    );
    // spell.pilum_of_fire: Creo Ignem, level 20, range Voice (beyond Touch).
    // Plain cap with these Arts and no other modifiers: 8 + 8 + 0 (Int) +
    // 0 (Magic Theory) + 3 = 19 — one short of the spell's level 20.
    // Within-focus doubling adds min(te, fo) = 8, bringing the cap to 27.
    e.art_scores = vec![
        ArtScore::new(Id::new("art.creo"), 8),
        ArtScore::new(Id::new("art.ignem"), 8),
    ];
    e
}

fn pilum_marked_within_focus() -> SpellSelection {
    let mut s = SpellSelection::new(Id::new("spell.pilum_of_fire"));
    s.within_focus = true;
    s
}

fn has_cap_error(entity: &Entity, ruleset: &Ruleset) -> bool {
    validate(entity, ruleset)
        .issues
        .iter()
        .any(|i| i.code == "spell_level_exceeds_cap")
}

/// Guard case: marked within focus AND the entity actually holds a Major
/// Magical Focus — the doubled cap (27) admits the level-20 spell, so no
/// cap error. This must already pass before the fix; it pins the intended
/// behavior so the fix cannot regress the legitimate use of the marker.
#[test]
fn within_focus_marker_with_a_held_magical_focus_raises_no_cap_error() {
    let ruleset = full_ruleset();
    let mut e = magus();
    e.selections = vec![Selection::with_params(
        Id::new("virtue.major_magical_focus"),
        BTreeMap::from([("focus".to_string(), Id::new("fire"))]),
    )];
    e.spells = vec![pilum_marked_within_focus()];
    assert!(
        !has_cap_error(&e, &ruleset),
        "a held Magical Focus must still admit the within-focus-marked spell"
    );
}

/// The defect: marked within focus, but the entity holds NO Magical Focus at
/// all (e.g. it was removed after the marker was set). The plain cap (19) is
/// the character's TRUE cap, and the level-20 spell is illegal — the engine
/// must not let a stale marker silently double the cap.
#[test]
fn within_focus_marker_without_any_magical_focus_raises_the_cap_error() {
    let ruleset = full_ruleset();
    let mut e = magus();
    e.spells = vec![pilum_marked_within_focus()];
    assert!(
        has_cap_error(&e, &ruleset),
        "with no Magical Focus held, the stale within_focus marker must not \
         double the cap — spell_level_exceeds_cap must fire"
    );
}

/// The same defect also inflates the per-spell Casting Total: with no
/// Magical Focus held, a spell's `within_focus` marker must contribute
/// nothing, so its Casting Total must equal the unmarked spell's total.
#[test]
fn casting_total_without_any_magical_focus_ignores_the_stale_marker() {
    let ruleset = full_ruleset();
    let e = magus();

    let marked = spell_casting_total(&pilum_marked_within_focus(), &e, &ruleset)
        .expect("spell.pilum_of_fire is in the shipped catalogue");
    let unmarked = spell_casting_total(
        &SpellSelection::new(Id::new("spell.pilum_of_fire")),
        &e,
        &ruleset,
    )
    .expect("spell.pilum_of_fire is in the shipped catalogue");

    assert_eq!(
        marked, unmarked,
        "a within_focus marker with no Magical Focus held must not change the \
         Casting Total (marked: {marked}, unmarked: {unmarked})"
    );
}
