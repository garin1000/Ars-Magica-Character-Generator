//! I2 (try-out finding 7, Norbert C4): the banked-XP warning must NAME the
//! Ability or Art it is about, the Ability with its instance ("Craft
//! (Carpentry)"), and the top-score case must not claim "the next level needs
//! only 0".
//!
//! - **The instance.** An Ability finding carries `ability` (the id) plus
//!   `parameter` (the instance), emitted unconditionally and empty for a plain
//!   Ability — the convention `too_many_for_param_value` already follows, which
//!   the frontend's `ABILITY_INSTANCE_ARG` folds into one localized name. A
//!   Catalogued instance is its catalogue id, a free-text one its text, a
//!   Linked one the text it currently resolves to (never the `(item, param)`
//!   pair, which names nothing a player typed).
//! - **One code per subject**, as `unknown_ability`/`unknown_art` and
//!   `ability_score_out_of_range`/`art_score_out_of_range` already are: a Fluent
//!   message cannot branch on which of `$ability`/`$art` is present, and the
//!   issue catalogue carries no selectors at all. The Ability keeps the
//!   existing `banked_xp_at_or_above_next_level`; the Art gets
//!   `art_banked_xp_at_or_above_next_level`.
//! - **The top score** (the advancement table's last row) has no next level, so
//!   it gets its own code — `banked_xp_at_top_score` /
//!   `art_banked_xp_at_top_score` — carrying the `score` it is stuck at and no
//!   `needed`. Still a warning: the banked figure is charged against the pool
//!   (`effective/xp.rs::build_spends`) and can never buy anything.
//!
//! Source of the "X (Z)" notation the field records: ArMDE:1177-1179.

use arm_rules::ruleset::{Ruleset, RulesetSources};
use arm_rules::types::*;
use arm_rules::validation::{IssueSeverity, ValidationIssue, validate};
use std::collections::BTreeMap;

/// The shipped core ruleset — duplicated per integration-test binary, the
/// convention `x10bc_banked_xp_and_within_focus.rs` already follows.
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

fn entity(type_id: &str) -> Entity {
    let mut e = Entity::new(
        EntityKind::Character,
        Id::new(type_id),
        RulesetRef::new(Id::new("arm5-core"), "2024.1"),
    );
    e.xp_pool = u32::MAX; // this file is not about the XP budget
    e
}

/// The raw-table XP from `score` to `score + 1` on the Ability table.
fn ability_delta(ruleset: &Ruleset, score: u8) -> u32 {
    ruleset.advancement().xp_for_score(score + 1).unwrap()
        - ruleset.advancement().xp_for_score(score).unwrap()
}

fn ability_ceiling(ruleset: &Ruleset) -> u8 {
    ruleset
        .advancement()
        .rows()
        .iter()
        .map(|row| row.score)
        .max()
        .expect("shipped Ability table is non-empty")
}

fn art_ceiling(ruleset: &Ruleset) -> u8 {
    ruleset
        .art_advancement()
        .rows()
        .iter()
        .map(|row| row.score)
        .max()
        .expect("shipped Art table is non-empty")
}

/// The one issue with `code`, failing loudly (with every code that WAS
/// emitted) when there is none.
fn the_issue(issues: &[ValidationIssue], code: &str) -> ValidationIssue {
    let found: Vec<&ValidationIssue> = issues.iter().filter(|i| i.code == code).collect();
    let codes: Vec<&str> = issues.iter().map(|i| i.code.as_str()).collect();
    assert_eq!(
        found.len(),
        1,
        "expected exactly one `{code}`, got {codes:?}"
    );
    found[0].clone()
}

fn arg<'a>(issue: &'a ValidationIssue, key: &str) -> Option<&'a str> {
    issue.args.get(key).map(String::as_str)
}

/// One bought Ability row, banked exactly the delta to its next score.
fn banked_ability_issues(
    ability: &str,
    parameter: Option<AbilityParameterValue>,
) -> Vec<ValidationIssue> {
    let ruleset = full_ruleset();
    let mut e = entity("companion");
    let mut a = AbilityScore::new(Id::new(ability), 1);
    a.parameter = parameter;
    a.banked_xp = ability_delta(&ruleset, 1);
    e.ability_scores = vec![a];
    validate(&e, &ruleset).issues
}

// --- (1) the Ability's instance rides on the finding --------------------------

#[test]
fn a_plain_ability_names_itself_with_an_empty_instance() {
    let issues = banked_ability_issues("ability.awareness", None);
    let issue = the_issue(&issues, "banked_xp_at_or_above_next_level");
    assert_eq!(arg(&issue, "ability"), Some("ability.awareness"));
    assert_eq!(
        arg(&issue, "parameter"),
        Some(""),
        "emitted unconditionally, empty for a plain Ability (the `too_many_for_param_value` convention)"
    );
}

#[test]
fn a_free_text_instance_rides_on_the_finding() {
    let issues = banked_ability_issues(
        "ability.craft",
        Some(AbilityParameterValue::text("Carpentry".to_string())),
    );
    let issue = the_issue(&issues, "banked_xp_at_or_above_next_level");
    assert_eq!(arg(&issue, "ability"), Some("ability.craft"));
    assert_eq!(arg(&issue, "parameter"), Some("Carpentry"));
}

#[test]
fn a_catalogued_instance_rides_on_the_finding_as_its_catalogue_id() {
    let issues = banked_ability_issues(
        "ability.dead_language",
        Some(AbilityParameterValue::Catalogued {
            id: Id::new("language.latin"),
        }),
    );
    let issue = the_issue(&issues, "banked_xp_at_or_above_next_level");
    assert_eq!(arg(&issue, "ability"), Some("ability.dead_language"));
    assert_eq!(arg(&issue, "parameter"), Some("language.latin"));
}

#[test]
fn a_linked_instance_rides_on_the_finding_as_the_text_it_resolves_to() {
    let ruleset = full_ruleset();
    let mut e = entity("companion");
    e.selections = vec![Selection::with_params(
        Id::new("virtue.craft_guild_training"),
        BTreeMap::from([("guild".into(), Id::new("Smiths' Guild of Verdi"))]),
    )];
    let mut a = AbilityScore::new(Id::new("ability.organization_lore"), 1);
    a.parameter = Some(AbilityParameterValue::Linked {
        item: Id::new("virtue.craft_guild_training"),
        param: "guild".into(),
    });
    a.banked_xp = ability_delta(&ruleset, 1);
    e.ability_scores = vec![a];
    let issues = validate(&e, &ruleset).issues;
    let issue = the_issue(&issues, "banked_xp_at_or_above_next_level");
    assert_eq!(arg(&issue, "parameter"), Some("Smiths' Guild of Verdi"));
}

#[test]
fn two_instances_of_one_ability_are_told_apart() {
    // Craft (Carpentry) is over its next level, Craft (Smithing) is not: the
    // one finding must say which of the two rows it is about.
    let ruleset = full_ruleset();
    let mut e = entity("companion");
    let mut carpentry = AbilityScore::new(Id::new("ability.craft"), 1);
    carpentry.parameter = Some(AbilityParameterValue::text("Carpentry".to_string()));
    carpentry.banked_xp = ability_delta(&ruleset, 1);
    let mut smithing = AbilityScore::new(Id::new("ability.craft"), 1);
    smithing.parameter = Some(AbilityParameterValue::text("Smithing".to_string()));
    smithing.banked_xp = 1;
    e.ability_scores = vec![carpentry, smithing];
    let issues = validate(&e, &ruleset).issues;
    let issue = the_issue(&issues, "banked_xp_at_or_above_next_level");
    assert_eq!(arg(&issue, "parameter"), Some("Carpentry"));
    assert_eq!(issue.context, Some(Id::new("ability.craft")));
}

// --- one code per subject: the Art's own finding ------------------------------

#[test]
fn an_art_reports_its_own_code_naming_the_art() {
    let ruleset = full_ruleset();
    let mut e = entity("magus");
    let score = 5u8;
    let delta = ruleset.art_advancement().xp_for_score(score + 1).unwrap()
        - ruleset.art_advancement().xp_for_score(score).unwrap();
    let mut a = ArtScore::new(Id::new("art.creo"), score);
    a.banked_xp = delta;
    e.art_scores = vec![a];
    let issues = validate(&e, &ruleset).issues;
    let issue = the_issue(&issues, "art_banked_xp_at_or_above_next_level");
    assert_eq!(arg(&issue, "art"), Some("art.creo"));
    let delta_text = delta.to_string();
    assert_eq!(arg(&issue, "needed"), Some(delta_text.as_str()));
    assert_eq!(issue.severity, IssueSeverity::Warning);
    assert_eq!(issue.phase, CreationPhase::Arts);
    assert_eq!(issue.context, Some(Id::new("art.creo")));
    assert!(
        !issues
            .iter()
            .any(|i| i.code == "banked_xp_at_or_above_next_level"),
        "the Ability code is the Ability's alone now"
    );
}

// --- (2) the top score: its own code, no "needs only 0" -----------------------

#[test]
fn an_ability_at_the_top_score_reports_the_top_score_code() {
    let ruleset = full_ruleset();
    let ceiling = ability_ceiling(&ruleset);
    let mut e = entity("grog");
    let mut a = AbilityScore::new(Id::new("ability.craft"), ceiling);
    a.parameter = Some(AbilityParameterValue::text("Carpentry".to_string()));
    a.banked_xp = 3;
    e.ability_scores = vec![a];
    let issues = validate(&e, &ruleset).issues;
    let issue = the_issue(&issues, "banked_xp_at_top_score");
    assert_eq!(issue.severity, IssueSeverity::Warning);
    assert_eq!(issue.phase, CreationPhase::Abilities);
    assert_eq!(arg(&issue, "ability"), Some("ability.craft"));
    assert_eq!(arg(&issue, "parameter"), Some("Carpentry"));
    assert_eq!(arg(&issue, "banked"), Some("3"));
    let ceiling_text = ceiling.to_string();
    assert_eq!(arg(&issue, "score"), Some(ceiling_text.as_str()));
    assert_eq!(
        arg(&issue, "needed"),
        None,
        "there is no next level to need anything"
    );
    assert!(
        !issues
            .iter()
            .any(|i| i.code == "banked_xp_at_or_above_next_level"),
        "the next-level finding must not also fire at the top score"
    );
}

#[test]
fn an_art_at_the_top_score_reports_the_art_top_score_code() {
    let ruleset = full_ruleset();
    let ceiling = art_ceiling(&ruleset);
    let mut e = entity("magus");
    let mut a = ArtScore::new(Id::new("art.creo"), ceiling);
    a.banked_xp = 1;
    e.art_scores = vec![a];
    let issues = validate(&e, &ruleset).issues;
    let issue = the_issue(&issues, "art_banked_xp_at_top_score");
    assert_eq!(issue.severity, IssueSeverity::Warning);
    assert_eq!(issue.phase, CreationPhase::Arts);
    assert_eq!(arg(&issue, "art"), Some("art.creo"));
    assert_eq!(arg(&issue, "banked"), Some("1"));
    let ceiling_text = ceiling.to_string();
    assert_eq!(arg(&issue, "score"), Some(ceiling_text.as_str()));
    assert_eq!(arg(&issue, "needed"), None);
}
