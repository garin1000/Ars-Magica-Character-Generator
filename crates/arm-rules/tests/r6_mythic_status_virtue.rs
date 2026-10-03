//! R6 / D83.7 (Norbert's ruling, 2026-10-03): a Mythic Companion's status
//! Virtue (Devil Child, Faerie Doctor, Nephilim, Spirit Votary) is held ONLY as
//! the grant of the type that defines it — "should only be granted by selecting
//! the appropriate mythic companion type, not selectable by just anyone".
//!
//! Rulebook: ArMDE:2846 "You must take the Free Virtue defining which type of
//! Mythic Companion you are"; ArMDE:2637 "All Mythic Companions take a Free
//! Virtue which specifies their status."
//!
//! Run against the shipped ruleset. The status Virtues are found through the
//! data (the `mythic_companion` V/F category, which holds exactly them), never a
//! list of ids, and nothing here asserts a catalogue total.

use arm_rules::Grant;
use arm_rules::ruleset::{Ruleset, RulesetSources};
use arm_rules::types::*;
use arm_rules::validation::{IssueSeverity, validate};

/// The issue a bought status Virtue raises.
const BOUGHT: &str = "mythic_status_virtue_bought";

fn shipped_ruleset() -> Ruleset {
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
        characteristics: Some(include_str!("../../../rules/core/characteristics.json")),
        life_stages: Some(include_str!("../../../rules/core/life_stages.json")),
        parameter_catalogues: Some(include_str!(
            "../../../rules/core/parameter_catalogues.json"
        )),
        ..RulesetSources::default()
    })
    .expect("shipped core ruleset loads")
}

fn character(type_id: &str, selections: Vec<Selection>) -> Entity {
    let mut e = Entity::new(
        EntityKind::Character,
        Id::new(type_id),
        RulesetRef::new(Id::new("arm5-core"), "2024.1"),
    );
    e.selections = selections;
    e
}

/// The status Virtues, read off the data: every item in the `mythic_companion`
/// category (the book's "Mythic Companion, Free" index heading, ArMDE:3329).
fn status_virtues(rs: &Ruleset) -> Vec<Id> {
    let ids: Vec<Id> = rs
        .items()
        .filter(|item| item.categories.iter().any(|c| c == "mythic_companion"))
        .map(|item| item.id.clone())
        .collect();
    assert!(!ids.is_empty(), "the shipped catalogue has status Virtues");
    ids
}

/// The Virtues a mythic type grants fixed that are NOT its status Virtue —
/// Dowsing, Strong Angelic Heritage, Second Sight. Anyone may still buy these.
fn other_fixed_grants(rs: &Ruleset) -> Vec<Id> {
    let status = status_virtues(rs);
    let ids: Vec<Id> = rs
        .mythic_types()
        .flat_map(|t| t.grants.iter())
        .filter_map(|g| match g {
            Grant::Fixed { item, .. } if !status.contains(item) => Some(item.clone()),
            _ => None,
        })
        .collect();
    assert!(
        !ids.is_empty(),
        "some mythic type grants a second fixed Virtue"
    );
    ids
}

#[test]
fn a_bought_status_virtue_is_an_error_on_the_virtues_and_flaws_step() {
    let rs = shipped_ruleset();
    for status in status_virtues(&rs) {
        let e = character("mythic_companion", vec![Selection::new(status.clone())]);
        let result = validate(&e, &rs);
        let issue = result
            .issues
            .iter()
            .find(|i| i.code == BOUGHT)
            .unwrap_or_else(|| {
                panic!(
                    "buying {status} must report `{BOUGHT}`; got {:?}",
                    result.issues.iter().map(|i| &i.code).collect::<Vec<_>>()
                )
            });
        assert_eq!(issue.severity, IssueSeverity::Error, "{status}");
        assert_eq!(issue.phase, CreationPhase::VirtuesFlaws, "{status}");
        assert_eq!(
            issue.args.get("item").map(String::as_str),
            Some(status.as_str()),
            "the issue names the bought Virtue"
        );
    }
}

/// A bought copy is refused even beside the type that grants it: the grant is
/// the only legal copy.
#[test]
fn a_status_virtue_bought_beside_its_own_type_is_still_an_error() {
    let rs = shipped_ruleset();
    for mtype in rs.mythic_types() {
        let Some(status) = mtype.grants.iter().find_map(|g| match g {
            Grant::Fixed { item, .. } if status_virtues(&rs).contains(item) => Some(item.clone()),
            _ => None,
        }) else {
            panic!("{} grants no status Virtue", mtype.id);
        };
        let mut e = character("mythic_companion", vec![Selection::new(status.clone())]);
        e.mythic_type = Some(mtype.id.clone());
        let codes: Vec<String> = validate(&e, &rs).errors().map(|i| i.code.clone()).collect();
        assert!(
            codes.iter().any(|c| c == BOUGHT),
            "{}: a bought {status} beside the grant must report `{BOUGHT}`; got {codes:?}",
            mtype.id
        );
    }
}

#[test]
fn a_status_virtue_granted_by_its_type_raises_nothing() {
    let rs = shipped_ruleset();
    for mtype in rs.mythic_types() {
        let mut e = character("mythic_companion", vec![]);
        e.mythic_type = Some(mtype.id.clone());
        let codes: Vec<String> = validate(&e, &rs)
            .issues
            .iter()
            .map(|i| i.code.clone())
            .collect();
        assert!(
            !codes.iter().any(|c| c == BOUGHT),
            "{}: the granted status Virtue is the legal copy; got {codes:?}",
            mtype.id
        );
    }
}

/// Guards against reading "status Virtue" as "any fixed grant of a type": the
/// free Minor Virtues are ordinary Virtues anyone may buy.
#[test]
fn a_types_other_fixed_grants_stay_buyable() {
    let rs = shipped_ruleset();
    for id in other_fixed_grants(&rs) {
        let e = character("companion", vec![Selection::new(id.clone())]);
        let codes: Vec<String> = validate(&e, &rs)
            .issues
            .iter()
            .map(|i| i.code.clone())
            .collect();
        assert!(
            !codes.iter().any(|c| c == BOUGHT),
            "{id} is not a status Virtue; got {codes:?}"
        );
    }
}

/// Shipped-data guard: the `mythic_status` flag marks exactly the items the
/// `mythic_companion` category holds, and each shipped type grants exactly one
/// of them `fixed` — so a new type cannot ship with a buyable status Virtue.
/// (Load-time integrity checks only the converse: every flagged item is some
/// type's fixed grant.)
#[test]
fn every_shipped_type_grants_exactly_one_flagged_status_virtue() {
    let rs = shipped_ruleset();
    let mut flagged: Vec<Id> = rs
        .items()
        .filter(|i| i.mythic_status)
        .map(|i| i.id.clone())
        .collect();
    let mut by_category = status_virtues(&rs);
    flagged.sort();
    by_category.sort();
    assert_eq!(flagged, by_category, "flag and category disagree");

    for mtype in rs.mythic_types() {
        let count = mtype
            .grants
            .iter()
            .filter(|g| matches!(g, Grant::Fixed { item, .. } if flagged.contains(item)))
            .count();
        assert_eq!(
            count, 1,
            "{} must grant exactly one status Virtue",
            mtype.id
        );
    }
}
