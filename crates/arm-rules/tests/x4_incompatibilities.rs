//! X4 (`tmp/x4-verdicts.md`; `docs/vf-audit/corrections.md` § 3.5;
//! `docs/vf-audit/decisions.md` D44, D68.4/.7/.8, D33, D23) — sourcing every
//! `incompatible_with` declaration, the Wealthy/Poor closed set (F-340
//! family), the Major/Minor twin sweep, and the three predicate cases
//! (F-526 University Dean, F-542 Weak Personality, Q-138 Flawed Powers).
//!
//! Phase 1 only — verdicts and failing tests, no data or engine changes.
//! Every test below is RED against the shipped ruleset unless its own doc
//! comment says SANITY (a verdict that already holds and must not regress
//! once Phase 2 reworks the magnitude-twin guard). See `tmp/x4-verdicts.md`
//! for the full per-pair citation table and `tmp/x4-handover.md` for exactly
//! what Phase 2 must change (data, `RULES.md` rows, and the two small engine
//! gaps: `ItemPredicate::GrantsReputation`'s missing `ItemKind::Flaw` check,
//! and the new `requires_hermetic_arts` predicate D68.4 asks for).

use arm_rules::ruleset::{Ruleset, RulesetSources};
use arm_rules::types::*;
use arm_rules::validation::{ValidationResult, validate};
use std::collections::BTreeMap;

const SHIPPED_HOUSES: &str = include_str!("../../../rules/core/houses.json");

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

/// Same shape as `x3_trained_gate.rs::entity` — duplicated since integration
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

fn issue_codes(result: &ValidationResult) -> Vec<&str> {
    result.issues.iter().map(|i| i.code.as_str()).collect()
}

// ---------------------------------------------------------------------------
// Group A — the Wealthy/Poor closed set (F-340 family; corrections.md § 3.5;
// tmp/x3-scope.md § 4). Eleven pairs across seven entries — none ships any
// `incompatible_with` today. Redcap's separate exclusion of The Gift (same
// passage, ArMDE:4850, a distinct finding, F-242) gets its own test.
// `virtue.covenfolk`/`virtue.custos` already carry both pairs (X1) and
// `virtue.guild_apprentice` is D47's suppression (X7a) — neither is here.
// Priest's ban is conditional text only (D68.7) and stays out of data.
// ---------------------------------------------------------------------------

/// (entry, excludes_wealthy, excludes_poor).
const WEALTHY_POOR_GAPS: &[(&str, bool, bool)] = &[
    ("virtue.almogavar", true, true),       // ArMDE:3406
    ("virtue.mendicant_friar", true, true), // ArMDE:4494
    ("virtue.redcap", true, true),          // ArMDE:4850
    ("virtue.turb_trained", true, true),    // ArMDE:5181
    ("virtue.perfectus", true, false),      // ArMDE:4634 (Wealthy only)
    ("flaw.branded_criminal", true, false), // ArMDE:5751 (Wealthy only)
    ("flaw.outcast", true, false),          // ArMDE:6540 (Wealthy only)
];

#[test]
fn wealthy_poor_closed_set_gaps_are_refused() {
    let rs = load_ruleset();
    let mut still_allowed = Vec::new();
    for (id, excludes_wealthy, excludes_poor) in WEALTHY_POOR_GAPS {
        rs.item(&Id::new(*id))
            .unwrap_or_else(|| panic!("{id} must exist in the shipped catalogue"));
        if *excludes_wealthy {
            let e = entity("companion", vec![sel(id), sel("virtue.wealthy")]);
            let result = validate(&e, &rs);
            if !issue_codes(&result).contains(&"incompatible") {
                still_allowed.push(format!("{id} + virtue.wealthy"));
            }
        }
        if *excludes_poor {
            let e = entity("companion", vec![sel(id), sel("flaw.poor")]);
            let result = validate(&e, &rs);
            if !issue_codes(&result).contains(&"incompatible") {
                still_allowed.push(format!("{id} + flaw.poor"));
            }
        }
    }
    assert!(
        still_allowed.is_empty(),
        "the Wealthy/Poor closed set (F-340 family) states these exclusions in so \
         many words, but validate() raises no `incompatible` issue for them yet: \
         {still_allowed:?}"
    );
}

#[test]
fn redcap_excludes_the_gift() {
    let rs = load_ruleset();
    // ArMDE:4850, same passage as Redcap's Wealthy/Poor exclusion but a
    // separate stated prohibition ("You may not take The Gift"), F-242.
    let e = entity(
        "companion",
        vec![sel("virtue.redcap"), sel("virtue.the_gift")],
    );
    let result = validate(&e, &rs);
    assert!(
        issue_codes(&result).contains(&"incompatible"),
        "ArMDE:4850 'You may not take The Gift' — redcap + the_gift must be \
         incompatible, got: {:?}",
        issue_codes(&result)
    );
}

// ---------------------------------------------------------------------------
// Group B — D44 plain two-sided id pairs from corrections.md § 3.5's Findings
// list (F-51, F-79, F-291, F-352, F-366, F-371, F-385, F-435, F-465, F-527)
// plus F-23's seven flat exclusions (the category-wide and character-type
// clauses in the SAME passage are Group C / out of scope — see
// tmp/x4-verdicts.md § 3). F-02 and F-298 are NOT here: both are sourced but
// need a same-parameter-value constraint, not a flat pair (tmp/x4-verdicts.md
// § 5). F-459 is NOT here either: it turned out not to be a stated
// incompatibility at all (tmp/x4-verdicts.md § 5).
// ---------------------------------------------------------------------------

const D44_PLAIN_PAIRS: &[(&str, &str)] = &[
    ("virtue.demonic_blood", "virtue.unaging"), // ArMDE:3661 F-51
    ("virtue.demonic_blood", "flaw.age_quickly"), // ArMDE:3661 F-51
    ("virtue.forgettable_face", "virtue.venus_blessing"), // ArMDE:3931 F-79
    ("virtue.forgettable_face", "virtue.inspirational"), // ArMDE:3931 F-79
    ("virtue.strong_faerie_blood", "virtue.faerie_blood"), // ArMDE:5044 F-291
    ("virtue.withstand_casting", "flaw.vulnerable_casting"), // ArMDE:5269 F-352, creation-time
    ("flaw.blatant_gift", "flaw.blatant_magical_air"), // ArMDE:5717 F-366
    ("flaw.bound_magic", "virtue.harnessed_magic"), // ArMDE:5729 F-371
    (
        "flaw.ceremonial_spontaneous_magic",
        "flaw.difficult_spontaneous_magic",
    ), // ArMDE:5783 F-385
    (
        "flaw.ceremonial_spontaneous_magic",
        "flaw.weak_spontaneous_magic",
    ), // ArMDE:5783 F-385 — NOT difficult<->weak: the book explicitly
    // permits combining those two (ArMDE:5970/:7088), so that pair
    // must never be added.
    ("flaw.failed_student", "virtue.doctor_in_faculty"), // ArMDE:6074 F-435
    ("flaw.night_terrors", "flaw.sleep_disorder"),       // ArMDE:6492 F-465
    ("flaw.uncertain_faith", "virtue.true_faith"),       // ArMDE:6905 F-527
    ("virtue.blood_of_the_nephilim", "virtue.the_gift"), // ArMDE:3517 F-23
    ("virtue.blood_of_the_nephilim", "virtue.true_faith"), // ArMDE:3517 F-23
    ("virtue.blood_of_the_nephilim", "virtue.giant_blood"), // ArMDE:3517 F-23
    ("virtue.blood_of_the_nephilim", "virtue.mythic_blood"), // ArMDE:3517 F-23
    ("virtue.blood_of_the_nephilim", "virtue.faerie_blood"), // ArMDE:3517 F-23
    ("virtue.blood_of_the_nephilim", "flaw.age_quickly"), // ArMDE:3517 F-23
    ("virtue.blood_of_the_nephilim", "flaw.lycanthrope"), // ArMDE:3517 F-23
];

#[test]
fn d44_plain_pairs_are_refused() {
    let rs = load_ruleset();
    let mut still_allowed = Vec::new();
    for (a, b) in D44_PLAIN_PAIRS {
        rs.item(&Id::new(*a))
            .unwrap_or_else(|| panic!("{a} must exist in the shipped catalogue"));
        rs.item(&Id::new(*b))
            .unwrap_or_else(|| panic!("{b} must exist in the shipped catalogue"));
        let e = entity("companion", vec![sel(a), sel(b)]);
        let result = validate(&e, &rs);
        if !issue_codes(&result).contains(&"incompatible") {
            still_allowed.push(format!("{a} + {b}"));
        }
    }
    assert!(
        still_allowed.is_empty(),
        "these pairs are stated in the rulebook in so many words, but validate() \
         raises no `incompatible` issue for them yet: {still_allowed:?}"
    );
}

// ---------------------------------------------------------------------------
// Group C — F-23's category-wide clause: "Hermetic Virtues or Flaws" is not a
// flat id list (CLAUDE.md's catalogue-size invariant), it is the same shape
// as Group E's Weak Personality fix: Effect::ForbidsItemCategory. F-23's
// other two clauses (the Size-affecting cluster, and "Methods or Powers")
// are text-only or out of scope — see tmp/x4-verdicts.md § 4 (QUESTION) and
// § 5.
// ---------------------------------------------------------------------------

#[test]
fn blood_of_the_nephilim_excludes_hermetic_category() {
    let rs = load_ruleset();
    let e = entity(
        "companion",
        vec![
            sel("virtue.blood_of_the_nephilim"),
            sel("virtue.harnessed_magic"),
        ],
    );
    let result = validate(&e, &rs);
    assert!(
        issue_codes(&result).contains(&"category_forbidden_by_effect"),
        "ArMDE:3517 'Hermetic Virtues or Flaws' — blood_of_the_nephilim + any \
         hermetic-category item must raise category_forbidden_by_effect, got: {:?}",
        issue_codes(&result)
    );
}

// ---------------------------------------------------------------------------
// Group D — F-466: `flaw.no_sense_of_direction` is "incompatible with the
// Well Traveled Virtue" (ArMDE:6502), but `virtue.lone_redcap` GRANTS
// well_traveled for free — a plain `incompatible_with` would also block Lone
// Redcap, which the book never says. Needs `Prereq::Nor(Has(well_traveled))`
// on the flaw side instead (the shape `tmp/x3-scope.md` names).
// ---------------------------------------------------------------------------

#[test]
fn no_sense_of_direction_excludes_bought_well_traveled_but_not_granted() {
    let rs = load_ruleset();
    let bought = entity(
        "companion",
        vec![
            sel("flaw.no_sense_of_direction"),
            sel("virtue.well_traveled"),
        ],
    );
    let bought_result = validate(&bought, &rs);
    assert!(
        issue_codes(&bought_result).contains(&"prereq_not_met")
            || issue_codes(&bought_result).contains(&"incompatible"),
        "no_sense_of_direction + directly-bought well_traveled must be refused, \
         got: {:?}",
        issue_codes(&bought_result)
    );

    let granted = entity(
        "companion",
        vec![sel("flaw.no_sense_of_direction"), sel("virtue.lone_redcap")],
    );
    let granted_result = validate(&granted, &rs);
    let granted_codes = issue_codes(&granted_result);
    assert!(
        !granted_codes.contains(&"prereq_not_met") && !granted_codes.contains(&"incompatible"),
        "no_sense_of_direction + Lone Redcap (which GRANTS well_traveled for free) \
         must stay legal — a plain incompatible_with would wrongly block this, \
         got: {granted_codes:?}"
    );
}

// ---------------------------------------------------------------------------
// Group E — F-542 Weak Personality: "no other Personality Flaws" (ArMDE:7078)
// is a category-wide prohibition. `Effect::ForbidsItemCategory` and
// `validate_category_effect_prohibitions` already exist (the Effect variant's
// own doc comment cites this exact line — B1's D21 machinery), but
// `flaw.weak_personality` does not carry the effect yet: data-only gap.
// ---------------------------------------------------------------------------

#[test]
fn weak_personality_excludes_other_personality_flaws() {
    let rs = load_ruleset();
    let e = entity(
        "companion",
        vec![sel("flaw.weak_personality"), sel("flaw.ambitious_minor")],
    );
    let result = validate(&e, &rs);
    assert!(
        issue_codes(&result).contains(&"category_forbidden_by_effect"),
        "ArMDE:7078 'no other Personality Flaws' — weak_personality + any other \
         Personality Flaw must raise category_forbidden_by_effect, got: {:?}",
        issue_codes(&result)
    );
}

// ---------------------------------------------------------------------------
// Group F — F-526 University Dean (Q-137): "can not have the Poor Flaw or
// any other Flaw that grants a Bad Reputation" (ArMDE:6923-6926).
// `ItemPredicate::GrantsReputation` and `PointItem::excluded_if_holds` already
// exist, but two things are missing together: (1) university_dean does not
// carry `excluded_if_holds: [GrantsReputation]` yet (data gap), and (2) the
// predicate as built checks only `Effect::GrantsReputation`'s presence, not
// `ItemKind::Flaw` — so a naive data-only fix would ALSO exclude
// virtue.doctor_in_faculty, University Dean's own required Virtue (it grants
// an ACADEMIC, not Bad, Reputation). Both assertions live in one test on
// purpose: the first is today's live gap, the second is the regression a
// kind-blind fix would introduce.
// ---------------------------------------------------------------------------

#[test]
fn university_dean_excludes_bad_reputation_flaws_but_not_doctor_in_faculty() {
    let rs = load_ruleset();

    let with_bad_rep_flaw = entity(
        "companion",
        vec![sel("flaw.university_dean"), sel("flaw.outsider_major")],
    );
    let result = validate(&with_bad_rep_flaw, &rs);
    assert!(
        issue_codes(&result).contains(&"excluded_by_predicate"),
        "university_dean + a Flaw that grants a Reputation (Outsider) must be \
         excluded, got: {:?}",
        issue_codes(&result)
    );

    let with_doctor = entity(
        "companion",
        vec![sel("flaw.university_dean"), sel("virtue.doctor_in_faculty")],
    );
    let doctor_result = validate(&with_doctor, &rs);
    assert!(
        !issue_codes(&doctor_result).contains(&"excluded_by_predicate"),
        "university_dean + Doctor in (Faculty) (a VIRTUE, his own prerequisite, \
         which also grants a Reputation) must stay legal — the predicate must \
         check ItemKind::Flaw, not just GrantsReputation's presence — got: {:?}",
        issue_codes(&doctor_result)
    );
}

// ---------------------------------------------------------------------------
// Group G — Q-138/D33/D68.4: Flawed Powers' "Any Flaw that is only
// appropriate to Hermetic Magic ... cannot be taken with this Flaw"
// (ArMDE:6148) restricts what the Flaw IMPORTS, not what the holder may also
// hold (D33) — so this is NOT an incompatible_with or Prereq::Nor. It needs a
// new `parameters` entry naming the imported Flaw, with an `exclude_if`
// pointing at a new `requires_hermetic_arts` predicate (D68.4: Deficient
// Technique/Unstructured Caster excluded, Restriction/Necessary Condition
// importable). This only checks the DATA SHAPE (some parameter has an
// exclude_if at all) so it does not hardcode the predicate's name or the
// parameter's key before Phase 2 designs them — see tmp/x4-verdicts.md § 4.
// ---------------------------------------------------------------------------

#[test]
fn flawed_powers_has_no_import_filter_yet() {
    let rs = load_ruleset();
    let item = rs
        .item(&Id::new("flaw.flawed_powers"))
        .expect("flaw.flawed_powers must exist in the shipped catalogue");
    let has_import_filter = item.parameters.iter().any(|p| p.exclude_if.is_some());
    assert!(
        has_import_filter,
        "flaw.flawed_powers ships {} parameter(s) and none has an exclude_if — \
         the Hermetic-only import filter (D33/D68.4/Q-138) is not encoded yet",
        item.parameters.len()
    );
}

// ---------------------------------------------------------------------------
// Group H — D68.8: every Major/Minor twin exclusion needs ITS OWN passage;
// ArMDE:2814 is not a blanket source. These 26 personality-Flaw pairs carry
// only a generic "Major or Minor, Personality" header and NO passage stating
// the two magnitudes exclude each other — so `incompatible_with`/the guard
// must stop forcing them. They are still hard-blocked today (the load-time
// guard `validate_magnitude_variant_exclusivity` forces every detected pair
// symmetric, so the ruleset would fail to load otherwise).
// ---------------------------------------------------------------------------

const UNSOURCED_TWIN_PAIRS: &[(&str, &str)] = &[
    ("flaw.ambitious_major", "flaw.ambitious_minor"), // ArMDE:5663
    ("flaw.avaricious_major", "flaw.avaricious_minor"), // ArMDE:5683
    ("flaw.compassionate_major", "flaw.compassionate_minor"), // ArMDE:5809
    ("flaw.compulsion_major", "flaw.compulsion_minor"), // ArMDE:5813
    ("flaw.compulsive_lying_major", "flaw.compulsive_lying_minor"), // ArMDE:5817
    ("flaw.depraved_major", "flaw.depraved_minor"),   // ArMDE:5936
    ("flaw.driven_major", "flaw.driven_minor"),       // ArMDE:5988
    ("flaw.envious_major", "flaw.envious_minor"),     // ArMDE:6016
    (
        "flaw.gender_nonconforming_major",
        "flaw.gender_nonconforming_minor",
    ), // ArMDE:6202
    ("flaw.generous_major", "flaw.generous_minor"),   // ArMDE:6206
    ("flaw.greedy_major", "flaw.greedy_minor"),       // ArMDE:6214
    ("flaw.hatred_major", "flaw.hatred_minor"),       // ArMDE:6236
    ("flaw.higher_purpose_major", "flaw.higher_purpose_minor"), // ArMDE:6256
    ("flaw.lecherous_major", "flaw.lecherous_minor"), // ArMDE:6334
    ("flaw.meddler_major", "flaw.meddler_minor"),     // ArMDE:6422
    ("flaw.obsessed_major", "flaw.obsessed_minor"),   // ArMDE:6520
    ("flaw.optimistic_major", "flaw.optimistic_minor"), // ArMDE:6534
    ("flaw.overconfident_major", "flaw.overconfident_minor"), // ArMDE:6562
    ("flaw.oversensitive_major", "flaw.oversensitive_minor"), // ArMDE:6566
    ("flaw.pious_major", "flaw.pious_minor"),         // ArMDE:6586
    ("flaw.proud_major", "flaw.proud_minor"),         // ArMDE:6642
    ("flaw.rebellious_major", "flaw.rebellious_minor"), // ArMDE:6659
    ("flaw.reckless_major", "flaw.reckless_minor"),   // ArMDE:6663
    ("flaw.vow_major", "flaw.vow_minor"),             // ArMDE:6989
    ("flaw.weakness_major", "flaw.weakness_minor"),   // ArMDE:7090
    ("flaw.wrathful_major", "flaw.wrathful_minor"),   // ArMDE:7106
];

#[test]
fn unsourced_personality_twin_pairs_should_be_allowed_together() {
    let rs = load_ruleset();
    let mut still_blocked = Vec::new();
    for (major, minor) in UNSOURCED_TWIN_PAIRS {
        let e = entity("companion", vec![sel(major), sel(minor)]);
        let result = validate(&e, &rs);
        if issue_codes(&result).contains(&"incompatible") {
            still_blocked.push(format!("{major} + {minor}"));
        }
    }
    assert!(
        still_blocked.is_empty(),
        "D68.8: no book passage states these twin pairs exclude each other (a \
         generic 'Major or Minor, Personality' header is not a passage) — \
         validate_magnitude_variant_exclusivity must stop forcing them, but \
         these are still blocked: {still_blocked:?}"
    );
}

/// D68.8/D44: ArMDE:4742 explicitly permits holding more than one area of
/// Potent Magic — an unconditional permission, unlike Beloved Rival's hedge.
#[test]
fn potent_magic_twins_allowed() {
    let rs = load_ruleset();
    // "magus" so both entries' `hermetically_trained` prerequisite is
    // satisfied by the profile and does not mask the assertion.
    let e = entity(
        "magus",
        vec![
            sel("virtue.potent_magic_major"),
            sel("virtue.potent_magic_minor"),
        ],
    );
    let result = validate(&e, &rs);
    assert!(
        !issue_codes(&result).contains(&"incompatible"),
        "Potent Magic Major + Minor must be allowed together (ArMDE:4742), got: {:?}",
        issue_codes(&result)
    );
}

/// D68.8/D16/Q-X4-3: ArMDE:5697's "the troupe MAY allow... both" is HEDGED,
/// unlike Potent Magic's unconditional permission — D16's rule is that a
/// hedged restriction becomes a WARNING, never a hard block, and the
/// machinery for that (`advisory_prerequisites` /
/// `CODE_ADVISORY_PREREQ_NOT_MET`) already exists and needs no engine change.
#[test]
fn beloved_rival_twins_become_advisory_not_a_hard_block() {
    let rs = load_ruleset();
    let e = entity(
        "companion",
        vec![
            sel("flaw.beloved_rival_major"),
            sel("flaw.beloved_rival_minor"),
        ],
    );
    let result = validate(&e, &rs);
    let codes = issue_codes(&result);
    assert!(
        !codes.contains(&"incompatible"),
        "Beloved Rival Major + Minor must not be a hard block (ArMDE:5697), got: {codes:?}"
    );
    assert!(
        codes.contains(&"advisory_prereq_not_met"),
        "Beloved Rival Major + Minor together should raise the D16 hedged- \
         restriction warning instead, got: {codes:?}"
    );
}

// ---------------------------------------------------------------------------
// Sanity — verdicts that already hold and must NOT regress once Phase 2
// reworks the magnitude-twin guard. Not red today.
// ---------------------------------------------------------------------------

/// D68.8: ArMDE:4405 is an explicit blanket source ("either major or
/// minor... regardless of the source") — the one twin pair the ruling names
/// as staying sourced.
#[test]
fn magical_focus_twins_stay_hard_blocked() {
    let rs = load_ruleset();
    let e = entity(
        "magus",
        vec![
            sel("virtue.major_magical_focus"),
            sel("virtue.minor_magical_focus"),
        ],
    );
    let result = validate(&e, &rs);
    assert!(
        issue_codes(&result).contains(&"incompatible"),
        "ArMDE:4405 is a blanket source — this exclusion must stay, got: {:?}",
        issue_codes(&result)
    );
}

/// D44 entailment applied to the four twin pairs whose OWN text contradicts
/// itself across the two magnitudes, even with no passage naming the OTHER
/// side explicitly (orchestrator judgment applying D44 to D68.8's per-pair
/// sweep — Norbert may override, see tmp/x4-verdicts.md § 2):
/// - Outsider: ArMDE:6554/:6556 state alternate, mutually exclusive living
///   circumstances for the SAME Social Status.
/// - True Love (Flaw): ArMDE:6877 states alternate, mutually exclusive
///   competence levels for the SAME named NPC ("the one person meant for
///   you", singular).
/// - Amorphous: ArMDE:3412 describes two mutually exclusive trigger
///   mechanics for the SAME shapeshifting ability.
/// - Magian Lineage: ArMDE:4345's Major explicitly states it already
///   includes Minor's benefit "in addition to" — holding both double-counts
///   the same bonus.
const ENTAILED_TWIN_PAIRS: &[(&str, &str)] = &[
    ("flaw.outsider_major", "flaw.outsider_minor"),
    ("flaw.true_love_major", "flaw.true_love_minor"),
    // X2t (D60.3): True Friend copies True Love's own passage (ArMDE:6877)
    // verbatim, including the same alternate, mutually exclusive competence
    // levels for the same named NPC — same entailment, same reasoning.
    ("flaw.true_friend_major", "flaw.true_friend_minor"),
    ("virtue.amorphous_major", "virtue.amorphous_minor"),
    ("virtue.magian_lineage_major", "virtue.magian_lineage_minor"),
];

#[test]
fn entailed_twin_pairs_stay_hard_blocked() {
    let rs = load_ruleset();
    let mut wrongly_allowed = Vec::new();
    for (major, minor) in ENTAILED_TWIN_PAIRS {
        let e = entity("companion", vec![sel(major), sel(minor)]);
        let result = validate(&e, &rs);
        if !issue_codes(&result).contains(&"incompatible") {
            wrongly_allowed.push(format!("{major} + {minor}"));
        }
    }
    assert!(
        wrongly_allowed.is_empty(),
        "D44 entailment still supports excluding these pairs — must stay \
         blocked: {wrongly_allowed:?}"
    );
}

/// F-385's negative control: the book explicitly says these two MAY be
/// combined (ArMDE:5970, ArMDE:7088) — must never gain an `incompatible_with`
/// entry.
#[test]
fn difficult_and_weak_spontaneous_magic_stay_compatible() {
    let rs = load_ruleset();
    let e = entity(
        "companion",
        vec![
            sel("flaw.difficult_spontaneous_magic"),
            sel("flaw.weak_spontaneous_magic"),
        ],
    );
    let result = validate(&e, &rs);
    assert!(
        !issue_codes(&result).contains(&"incompatible"),
        "must stay legal — ArMDE:5970/:7088, got: {:?}",
        issue_codes(&result)
    );
}

// ---------------------------------------------------------------------------
// Group I — D69.6: the same-choice constraint (new engine machinery,
// `PointItem::same_choice_exclusions`). Student of (Realm) may not name the
// same Lore Ability Puissant Ability targets (ArMDE:5054); Academic
// Concentration (Subject) always means Artes Liberales regardless of its own
// `subject` parameter, so it conflicts with Puissant Ability only when THAT
// targets Artes Liberales specifically (ArMDE:3364).
// ---------------------------------------------------------------------------

fn sel_param(id: &str, key: &str, value: &str) -> Selection {
    Selection::with_params(
        Id::new(id),
        BTreeMap::from([(key.to_string(), Id::new(value))]),
    )
}

#[test]
fn student_of_realm_and_puissant_ability_conflict_for_the_same_lore() {
    let rs = load_ruleset();
    let e = entity(
        "companion",
        vec![
            sel_param("virtue.student_of_realm", "realm", "realm.divine"),
            sel_param(
                "virtue.puissant_ability",
                "ability",
                "ability.dominion_lore",
            ),
        ],
    );
    let result = validate(&e, &rs);
    assert!(
        issue_codes(&result).contains(&"same_choice_conflict"),
        "ArMDE:5054 'You may not take Student of (Realm) and Puissant Ability for \
         the same Lore' — got: {:?}",
        issue_codes(&result)
    );
}

#[test]
fn student_of_realm_and_puissant_ability_stay_legal_for_different_lores() {
    let rs = load_ruleset();
    let e = entity(
        "companion",
        vec![
            sel_param("virtue.student_of_realm", "realm", "realm.divine"),
            sel_param("virtue.puissant_ability", "ability", "ability.faerie_lore"),
        ],
    );
    let result = validate(&e, &rs);
    assert!(
        !issue_codes(&result).contains(&"same_choice_conflict"),
        "different Lores must stay legal, got: {:?}",
        issue_codes(&result)
    );
}

#[test]
fn academic_concentration_and_puissant_artes_liberales_conflict() {
    let rs = load_ruleset();
    let e = entity(
        "companion",
        vec![
            sel_param(
                "virtue.academic_concentration_subject",
                "subject",
                "Rhetoric",
            ),
            sel_param(
                "virtue.puissant_ability",
                "ability",
                "ability.artes_liberales",
            ),
        ],
    );
    let result = validate(&e, &rs);
    assert!(
        issue_codes(&result).contains(&"same_choice_conflict"),
        "ArMDE:3364 'This Virtue is incompatible with the Virtue Puissant Artes \
         Liberales' — got: {:?}",
        issue_codes(&result)
    );
}

#[test]
fn academic_concentration_and_puissant_ability_stay_legal_for_other_abilities() {
    let rs = load_ruleset();
    let e = entity(
        "companion",
        vec![
            sel_param(
                "virtue.academic_concentration_subject",
                "subject",
                "Rhetoric",
            ),
            sel_param("virtue.puissant_ability", "ability", "ability.awareness"),
        ],
    );
    let result = validate(&e, &rs);
    assert!(
        !issue_codes(&result).contains(&"same_choice_conflict"),
        "Puissant Ability on an unrelated Ability must stay legal, got: {:?}",
        issue_codes(&result)
    );
}

// ---------------------------------------------------------------------------
// Group J — D69.5: the "affects Size" tag (`ItemPredicate::AffectsSize`).
// Blood of the Nephilim excludes every OTHER Size-affecting Virtue/Flaw
// (ArMDE:3517, "such as Giant..."). Giant Blood is already a flat D44 pair
// (Group B); Dwarf/Small Frame/Large are not, so they are this tag's actual
// reach.
// ---------------------------------------------------------------------------

#[test]
fn blood_of_the_nephilim_excludes_other_size_affecting_entries() {
    let rs = load_ruleset();
    for other in ["flaw.dwarf", "flaw.small_frame", "virtue.large"] {
        let e = entity(
            "companion",
            vec![sel("virtue.blood_of_the_nephilim"), sel(other)],
        );
        let result = validate(&e, &rs);
        assert!(
            issue_codes(&result).contains(&"excluded_by_predicate"),
            "ArMDE:3517 'Virtues or Flaws that affect your Size' — \
             blood_of_the_nephilim + {other} must raise excluded_by_predicate, \
             got: {:?}",
            issue_codes(&result)
        );
    }
}

#[test]
fn blood_of_the_nephilim_stays_legal_with_a_non_size_affecting_entry() {
    let rs = load_ruleset();
    let e = entity(
        "companion",
        vec![sel("virtue.blood_of_the_nephilim"), sel("virtue.tough")],
    );
    let result = validate(&e, &rs);
    assert!(
        !issue_codes(&result).contains(&"excluded_by_predicate"),
        "virtue.tough affects Soak, not Size, and must stay legal, got: {:?}",
        issue_codes(&result)
    );
}
