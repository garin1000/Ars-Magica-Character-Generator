//! Aging and its effect on characteristic and ability scores.
//!
//! Split out of `validation`; see `validation/mod.rs` for the public API and
//! the `ValidationIssue` issue-code contract.

use super::*;
use crate::aging::AgingError;

/// Validates a directly-entered aged character's aging state (advisory). Aging
/// drops are DERIVED from [`Entity::aging_points`] (Core Rules.md:16579); this
/// only surfaces informational notes, never blocking errors:
///
/// - `excessive_aging_reduction`: the derived drops would push a Characteristic's
///   effective score below the rules effective minimum (−5). The derived score is
///   clamped regardless; this only flags an implausible entry.
/// - `aging_rolls_pending`: the character is over 35 and no aging roll
///   is recorded, so the rolls the rules owe before play have not been made.
/// - `unknown_living_condition` and `living_conditions_conflict`: the character's
///   chosen Living Conditions do not resolve, or are mutually exclusive.
/// - `apparent_age_above_age`: the apparent age has outrun the actual one.
///
/// (An earlier `aging_points_force_drop` note announcing each auto-applied drop
/// was removed as validation noise — the drop is automatic and already reflected
/// in the effective score, so it is not an entry problem worth flagging.)
///
/// Three neighbouring findings were considered and **rejected as noise**, recorded
/// here so they are not re-litigated: a note for a Longevity Ritual carrying no
/// entered bonus (`LongevityBonus::entered` already surfaces that on the sheet, and
/// a ritual whose bonus the storyguide has not yet agreed is a legal state); a note
/// for accrued aging points with an empty log (that is
/// `aging_rolls_pending` restated, keyed off a weaker signal); and
/// widening that finding's args to `owed`/`recorded` (it would reword two locales
/// to say what the schedule read-out already says better).
///
/// Reads the un-aged bought score plus the derived drops; it never touches the
/// point-buy budget check (which is what keeps aging from perturbing creation
/// legality). Source: Core Rules.md:16579.
pub(crate) fn validate_aging(
    entity: &Entity,
    ruleset: &Ruleset,
    issues: &mut Vec<ValidationIssue>,
) {
    let bought = |c: &Characteristic| {
        entity
            .characteristics
            .get(c)
            .copied()
            .map_or(0i32, i32::from)
    };

    let effective_min = ruleset
        .characteristic_rules()
        .and_then(|r| r.effective_min_score())
        .map(i32::from);

    for (characteristic, points) in &entity.aging_points {
        if *points == 0 {
            continue;
        }
        let drops = crate::effective::aging_drops(entity, ruleset, *characteristic);
        if drops == 0 {
            continue;
        }
        let aged = bought(characteristic) - i32::try_from(drops).unwrap_or(i32::MAX);

        // The aged-down score would fall below the rules floor (clamped anyway).
        if let Some(min) = effective_min
            && aged < min
        {
            issues.push(ValidationIssue::warning(
                ValidationIssue::CODE_EXCESSIVE_AGING_REDUCTION,
                CreationPhase::Review,
                args([
                    ("characteristic", characteristic.to_string()),
                    ("reduction", drops.to_string()),
                    ("min", min.to_string()),
                ]),
                None,
            ));
        }
    }

    report_living_conditions(entity, ruleset, issues);
    report_apparent_age(entity, issues);
    report_pending_aging_rolls(entity, ruleset, issues);
}

/// Emits `unknown_living_condition` for every chosen [`Entity::living_conditions`]
/// id the table does not carry, and `living_conditions_conflict` once when the
/// chosen rows include more than one *non*-cumulative row.
///
/// # Why an unknown id needs a finding
///
/// [`crate::aging::living_conditions_modifier`] deliberately skips an id it cannot
/// resolve — the engine has one evaluation path and always produces a number — so
/// a typo would otherwise contribute 0 and make the AGING TOTAL silently wrong.
/// This is the same referential check `unknown_ability`, `unknown_spell` and
/// `unknown_equipment` apply to their own catalogues.
///
/// # Why more than one non-cumulative row is a conflict
///
/// "Modifiers marked with an asterisk are cumulative with each other"
/// (`:16594`) — a sentence worth writing only because the unmarked rows are *not*.
/// They describe mutually exclusive situations: a character cannot be both
/// "Wealthy, or healthy location" (`:16583`) and an "Average peasant" (`:16587`),
/// and the four covenant rows (`:16584-16586`) are graded alternatives for the same
/// covenant. So at most one may be chosen.
///
/// ONE finding, naming the first two offenders in canonical id order, rather than
/// one per offending pair: three exclusive rows are a single mistake to fix, and a
/// pair explosion would report it three times over.
///
/// A ruleset shipping no aging rules stands the whole subsystem down (there is no
/// table for an id to resolve against), exactly as
/// [`report_pending_aging_rolls`] does with its threshold.
///
/// Source: Ars Magica - Definitive Edition (Core Rules).md:16581-16594.
fn report_living_conditions(entity: &Entity, ruleset: &Ruleset, issues: &mut Vec<ValidationIssue>) {
    let Some(table) = ruleset.aging().map(|rules| &rules.living_conditions) else {
        return;
    };

    // `living_conditions` is a `BTreeSet`, so both walks are in canonical id order.
    let mut exclusive: Vec<&Id> = Vec::new();
    for id in &entity.living_conditions {
        match table.iter().find(|row| row.id == *id) {
            None => issues.push(ValidationIssue::error(
                ValidationIssue::CODE_UNKNOWN_LIVING_CONDITION,
                CreationPhase::Review,
                args([("condition", id.to_string())]),
                Some(id.clone()),
            )),
            Some(row) if !row.cumulative => exclusive.push(id),
            Some(_) => {}
        }
    }

    if let [first, second, ..] = exclusive.as_slice() {
        issues.push(ValidationIssue::error(
            ValidationIssue::CODE_LIVING_CONDITIONS_CONFLICT,
            CreationPhase::Review,
            args([
                ("condition", first.to_string()),
                ("other", second.to_string()),
            ]),
            Some((*first).clone()),
        ));
    }
}

/// Emits `apparent_age_above_age` when the entered apparent age exceeds the actual
/// age.
///
/// "Otherwise, the character's apparent age increases by one year" (`:16577`) —
/// at most one year per year lived, so aging alone can never push the apparent age
/// past the actual one; a higher figure is a transposed entry.
///
/// A WARNING rather than an error: the closest the rules come to stating the bound
/// explicitly is Unaging's aside — "You may choose your apparent age freely,
/// although if you are basically human it should be less than or equal to your
/// actual age" (`:5189`). That is a *should*, and it carries its own escape for a
/// character who is not basically human, so the engine advises and never blocks.
///
/// Silent unless both figures are entered: with either missing there is nothing to
/// compare.
///
/// Source: Ars Magica - Definitive Edition (Core Rules).md:16577, :5189.
fn report_apparent_age(entity: &Entity, issues: &mut Vec<ValidationIssue>) {
    let (Some(apparent_age), Some(age)) = (entity.apparent_age, entity.age) else {
        return;
    };
    if apparent_age <= age {
        return;
    }

    issues.push(ValidationIssue::warning(
        ValidationIssue::CODE_APPARENT_AGE_ABOVE_AGE,
        CreationPhase::Review,
        args([
            ("apparent_age", apparent_age.to_string()),
            ("age", age.to_string()),
        ]),
        None,
    ));
}

/// Emits `aging_rolls_pending` for a character who has reached
/// [`AgingRules::first_roll_age`](crate::aging::AgingRules::first_roll_age) and
/// whose aging log is empty.
///
/// "The first thing to bear in mind is that a character over the age of 35 must
/// make aging rolls (see page 392) before the game begins."
/// Source: Ars Magica - Definitive Edition (Core Rules).md:2232.
///
/// The rule is about *any* character, however it was built, which is why this
/// lives here and not in `validate_life_stage_plan` — that one returns early for a
/// character with no plan, and a directly-entered magus of 60 owes the rolls just
/// as much as a guided one.
///
/// "Over the age of 35" is strict: aging begins "in the Winter after they turn
/// 35" (`:16565`), so 35 owes nothing and 36 owes the first roll. That derivation
/// lives once, in `first_roll_age()`, over the `start_age` the ruleset's
/// `rules/core/aging.json` carries — the threshold is a rules number, not an engine
/// one, so a ruleset shipping no aging rules emits nothing here rather than falling
/// back on a constant the engine invented.
///
/// The recorded [`Entity::aging_log`] — not [`Entity::aging_points`] — settles the
/// finding: a roll can legitimately produce no aging points, so a well-rolled
/// character would otherwise be nagged forever.
///
/// Filed under [`CreationPhase::Review`] because no aging phase exists yet; slice
/// 6b6 adds the `Aging` variant to [`CreationPhase`] and moves this finding onto
/// it.
fn report_pending_aging_rolls(
    entity: &Entity,
    ruleset: &Ruleset,
    issues: &mut Vec<ValidationIssue>,
) {
    let (Some(rules), Some(age)) = (ruleset.aging(), entity.age) else {
        return;
    };
    if age < rules.first_roll_age() || !entity.aging_log.is_empty() {
        return;
    }

    issues.push(ValidationIssue::warning(
        ValidationIssue::CODE_AGING_ROLLS_PENDING,
        CreationPhase::Review,
        args([("age", age.to_string())]),
        None,
    ));
}

/// Localizes the [`AgingError`] a refused aging roll produced.
///
/// [`AgingError`] is plain data with no `Display` and no user-facing prose, so
/// this is where a refusal becomes something the player can read — the same
/// division of labour as
/// [`childhood_rejection_issues`](super::childhood_rejection_issues), and for the
/// same reason: these are **command-input** findings, not entity state. Every
/// variant is a refusal to write, so no stored character can ever hold the
/// condition for [`validate`](super::validate) to find; the finding describes the
/// roll the player just submitted and is gone the moment it is corrected.
///
/// The emit site lives in `validation/` all the same, so the contract-table and
/// phase scanners keep seeing every `(code, phase)` pair the frontend must
/// localize.
///
/// No `characteristics` argument on
/// [`AgingError::DistributionNotOpen`]: they would reach the message as slugs,
/// and a slug is never shown to a user. The count says what the player needs —
/// that the table fixed this row's Characteristics itself.
///
/// Source: Ars Magica - Definitive Edition (Core Rules).md:16597-16617.
pub fn aging_error_issue(error: &AgingError) -> ValidationIssue {
    match error {
        AgingError::NoAgingRules => ValidationIssue::error(
            ValidationIssue::CODE_AGING_RULES_MISSING,
            CreationPhase::Review,
            args([]),
            None,
        ),
        AgingError::YearAlreadyRecorded { age } => ValidationIssue::error(
            ValidationIssue::CODE_AGING_YEAR_ALREADY_RECORDED,
            CreationPhase::Review,
            args([("age", age.to_string())]),
            None,
        ),
        AgingError::DistributionMismatch { owed, distributed } => ValidationIssue::error(
            ValidationIssue::CODE_AGING_DISTRIBUTION_MISMATCH,
            CreationPhase::Review,
            args([
                ("distributed", distributed.to_string()),
                ("owed", owed.to_string()),
            ]),
            None,
        ),
        AgingError::DistributionNotOpen { characteristics } => ValidationIssue::error(
            ValidationIssue::CODE_AGING_DISTRIBUTION_NOT_OPEN,
            CreationPhase::Review,
            args([("count", characteristics.len().to_string())]),
            None,
        ),
        AgingError::AwardUnpriceable => ValidationIssue::error(
            ValidationIssue::CODE_AGING_AWARD_UNPRICEABLE,
            CreationPhase::Review,
            args([]),
            None,
        ),
        AgingError::YearNotRecorded { age } => ValidationIssue::error(
            ValidationIssue::CODE_AGING_YEAR_NOT_RECORDED,
            CreationPhase::Review,
            args([("age", age.to_string())]),
            None,
        ),
    }
}
