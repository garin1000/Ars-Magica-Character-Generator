//! B2 (`docs/vf-audit/design-b0-ranging-and-predicates.md` § 1 "B2", § 8's B2
//! row; D41; `docs/vf-audit/decisions.md`) — RED-checkpoint phase 1 tests.
//!
//! B2 is pure **data + validator wiring** on top of B1's already-landed
//! `CategoryCap.min`/`min_hard` and `Prereq::HasCategory` mechanism (commit
//! e5e3126). These tests run against the REAL shipped ruleset
//! (`rules/core/*.json`), not a hand-authored fixture, because B2 owes no new
//! type — only data (the `social_status` cap row on every profile, D41; the
//! `virtue.male_guild_sponsor` prerequisite, ArMDE:4439-4442) and a small
//! wiring change (naming the paired entries on the ceiling warning).
//!
//! Every test here MUST fail today, for the reason its own doc comment
//! states, and go green only once phase 2 lands the data + wiring.

use arm_rules::ruleset::{Ruleset, RulesetSources};
use arm_rules::types::*;
use arm_rules::validation::{IssueSeverity, ValidationIssue, validate};

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
        aging: None,
        parameter_catalogues: Some(include_str!(
            "../../../rules/core/parameter_catalogues.json"
        )),
    })
    .expect("shipped core ruleset loads")
}

fn entity(type_id: &str, selections: Vec<Selection>) -> Entity {
    let mut e = Entity::new(
        EntityKind::Character,
        Id::new(type_id),
        RulesetRef::new(Id::new("arm5-core"), "2024.1"),
    );
    e.age = Some(25);
    e.selections = selections;
    e
}

fn sel(id: &str) -> Selection {
    Selection::new(Id::new(id))
}

fn has_code(issues: &[ValidationIssue], code: &str) -> bool {
    issues.iter().any(|i| i.code == code)
}

// --- D41's floor: every profile must take one Social Status -----------------
//
// ArMDE:2816: "All characters must take one Social Status..." — a profile-wide
// floor, not a per-item prereq. `CategoryCap.min`/`min_hard` is the landed B1
// mechanism (`validation/caps.rs::push_category_cap_issues`); this is its
// first data user. Fails today because NO shipped profile's
// `virtue_category_caps` carries a `social_status` row at all — the mechanism
// is wired, but nothing feeds it.

/// All four character types permit `social_status` in their catalogue
/// (`permitted_categories`), so ArMDE:2816's "All characters" floor applies
/// uniformly across companion, grog, magus, and mythic companion alike
/// (mythic companions are companions — `is_companion: true` in the shipped
/// profile — so any reading of "characters" that includes companions admits
/// them too).
#[test]
fn every_profile_requires_at_least_one_social_status_selection() {
    let ruleset = full_ruleset();
    for type_id in ["companion", "grog", "magus", "mythic_companion"] {
        let e = entity(type_id, vec![]);
        let result = validate(&e, &ruleset);
        let floor_errors: Vec<_> = result
            .issues
            .iter()
            .filter(|i| i.code.starts_with("too_few_") && i.severity == IssueSeverity::Error)
            .collect();
        assert!(
            !floor_errors.is_empty(),
            "'{type_id}' with zero Social Status selections must raise a hard \
             floor error (D41/ArMDE:2816; B2 data not yet landed) — issues: {:?}",
            result.issues
        );
    }
}

/// The positive control: a companion holding exactly one Social Status
/// selection must clear the floor cleanly (the floor is `min: 1`, not
/// "exactly one"). Passes vacuously today (no cap row exists to trip either
/// way) and must keep passing, meaningfully, once phase 2 lands the data.
#[test]
fn one_social_status_selection_clears_the_floor() {
    let ruleset = full_ruleset();
    let e = entity("companion", vec![sel("virtue.gentleman")]);
    let result = validate(&e, &ruleset);
    let floor_errors: Vec<_> = result
        .issues
        .iter()
        .filter(|i| i.code.starts_with("too_few_") && i.severity == IssueSeverity::Error)
        .collect();
    assert!(
        floor_errors.is_empty(),
        "one Social Status selection must clear the D41 floor — issues: {:?}",
        result.issues
    );
}

/// ArMDE:2816 says "Virtues **or** Flaws" — `flaw.outlaw` is a real,
/// unrestricted `social_status`-category FLAW (`rules/core/virtues_flaws.json`).
/// A character represented ONLY by such a Flaw (an outlaw/criminal concept
/// with no separate Virtue-based status) must ALSO clear the floor: the cap
/// row must count matching Flaws, not only Virtues, even though it lives in
/// `virtue_category_caps`.
#[test]
fn a_flaw_based_social_status_alone_also_clears_the_floor() {
    let ruleset = full_ruleset();
    let e = entity("companion", vec![sel("flaw.outlaw")]);
    let result = validate(&e, &ruleset);
    let floor_errors: Vec<_> = result
        .issues
        .iter()
        .filter(|i| i.code.starts_with("too_few_") && i.severity == IssueSeverity::Error)
        .collect();
    assert!(
        floor_errors.is_empty(),
        "a Flaw-based Social Status alone must clear the D41 floor (ArMDE:2816 \
         says 'Virtues or Flaws') — issues: {:?}",
        result.issues
    );
}

// --- D41's ceiling: a second Social Status is a warning, naming both -------
//
// D41: "A second Social Status raises a warning, never an error... carrying
// the entry names so the player can see what was paired." The ceiling
// machinery itself is unchanged (`max: 1, hard: false`); what B2 owes is (a)
// the data row and (b) the warning naming the two paired entries, which
// `push_category_cap_issues` does not do for ANY category today (only
// `count`/`max` args).

/// `virtue.gentleman` and `virtue.craftsman` are both real, unrestricted
/// `social_status` Virtues (`rules/core/virtues_flaws.json`) usable by any
/// character type. Fails today for two independent reasons: no
/// `social_status` cap row exists yet (so no warning fires at all), and even
/// once one does, nothing names the paired entries in `args` yet.
#[test]
fn a_second_social_status_selection_raises_exactly_one_warning_naming_both_entries() {
    let ruleset = full_ruleset();
    let e = entity(
        "companion",
        vec![sel("virtue.craftsman"), sel("virtue.gentleman")],
    );
    let result = validate(&e, &ruleset);

    let ceiling_warnings: Vec<_> = result
        .issues
        .iter()
        .filter(|i| i.code == "too_many_social_status_virtues")
        .collect();
    assert_eq!(
        ceiling_warnings.len(),
        1,
        "a second Social Status must raise EXACTLY ONE warning (D41; B2 data \
         not yet landed) — issues: {:?}",
        result.issues
    );
    let issue = ceiling_warnings[0];
    assert_eq!(
        issue.severity,
        IssueSeverity::Warning,
        "never an error (D41)"
    );

    let named: std::collections::BTreeSet<&str> = issue.args.values().map(String::as_str).collect();
    assert!(
        named.contains("virtue.craftsman") && named.contains("virtue.gentleman"),
        "the warning must name BOTH paired entries so the player can see what \
         was paired (D41) — args: {:?}",
        issue.args
    );
}

/// ArMDE:2816 says "Virtues **or** Flaws" (the reason `both_kinds` exists at
/// all — see `a_flaw_based_social_status_alone_also_clears_the_floor` above).
/// The pairing D41's ceiling warns about is not Virtue-only either: a Virtue
/// Social Status (`virtue.gentleman`) plus a Flaw one (`flaw.outlaw`) must
/// raise the SAME single warning, naming both, mixed kind or not — the
/// ceiling and floor share one `matched` count in `caps.rs`, so this is a
/// confirmation that the shared computation carries `both_kinds` to both
/// halves, not a second, independent mechanism.
#[test]
fn a_second_social_status_selection_warns_even_when_the_two_are_different_kinds() {
    let ruleset = full_ruleset();
    let e = entity(
        "companion",
        vec![sel("virtue.gentleman"), sel("flaw.outlaw")],
    );
    let result = validate(&e, &ruleset);

    let ceiling_warnings: Vec<_> = result
        .issues
        .iter()
        .filter(|i| i.code == "too_many_social_status_virtues")
        .collect();
    assert_eq!(
        ceiling_warnings.len(),
        1,
        "a Virtue-and-Flaw Social Status pair must raise EXACTLY ONE warning \
         (D41/both_kinds) — issues: {:?}",
        result.issues
    );
    let issue = ceiling_warnings[0];
    assert_eq!(
        issue.severity,
        IssueSeverity::Warning,
        "never an error (D41)"
    );
    let named: std::collections::BTreeSet<&str> = issue.args.values().map(String::as_str).collect();
    assert!(
        named.contains("virtue.gentleman") && named.contains("flaw.outlaw"),
        "the warning must name BOTH paired entries regardless of kind — args: {:?}",
        issue.args
    );
}

// --- ArMDE:4441: Male Guild Sponsor needs a SEPARATE guild Social Status ---
//
// "The character must select a separate guild Social Status Virtue as well
// as this free Virtue to represent her status in the guild system"
// (ArMDE:4439-4442). `virtue.male_guild_sponsor` itself carries
// `"categories": ["social_status"]` (`rules/core/virtues_flaws.json:5198`) —
// this is exactly the self-referential trap the design note's § 1 flags
// ("HasCategory alone cannot distinguish 'holds one' from 'holds a second,
// distinct one' without also excluding the entry's own row") and leaves as
// B2's "remaining data call". See the phase-1 report for the finding this
// surfaces.

/// The carrier ALONE, with no other Social Status held, must fail
/// `prereq_not_met` — ArMDE:4441 requires a SEPARATE guild status. Fails
/// today because `virtue.male_guild_sponsor` carries no `prerequisites` field
/// at all yet.
#[test]
fn male_guild_sponsor_alone_fails_prereq_not_met() {
    let ruleset = full_ruleset();
    let e = entity("companion", vec![sel("virtue.male_guild_sponsor")]);
    let result = validate(&e, &ruleset);
    assert!(
        has_code(&result.issues, ValidationIssue::CODE_PREREQ_NOT_MET),
        "virtue.male_guild_sponsor with no OTHER Social Status held must fail \
         prereq_not_met (ArMDE:4441; B2 data not yet landed) — issues: {:?}",
        result.issues
    );
}

/// Control: held alongside a genuinely separate GUILD Social Status, the
/// prerequisite must be satisfied. X5a (D68.10/Q-X5-2) narrowed the
/// prerequisite from a bare `has_category: social_status` leaf (vacuous — ANY
/// Social Status satisfied it, since D41 already mandates one) to a closed
/// list of the five guild-rank Social Status Virtues, so `virtue.gentleman`
/// (a non-guild status) no longer qualifies here; `virtue.guild_apprentice`
/// does.
#[test]
fn male_guild_sponsor_with_a_separate_social_status_is_satisfied() {
    let ruleset = full_ruleset();
    let e = entity(
        "companion",
        vec![
            sel("virtue.male_guild_sponsor"),
            sel("virtue.guild_apprentice"),
        ],
    );
    let result = validate(&e, &ruleset);
    let named_male_guild_sponsor = result.issues.iter().any(|i| {
        i.code == ValidationIssue::CODE_PREREQ_NOT_MET
            && i.context
                .as_ref()
                .is_some_and(|c| c.as_str() == "virtue.male_guild_sponsor")
    });
    assert!(
        !named_male_guild_sponsor,
        "virtue.male_guild_sponsor WITH a separate Social Status must not fail \
         prereq_not_met — issues: {:?}",
        result.issues
    );
}
