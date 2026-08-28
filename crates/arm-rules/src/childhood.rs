//! Sample Childhood packages: ready-made Ability spreads a player may take
//! instead of dividing the early-childhood experience by hand.
//!
//! The packages are an optional shortcut, never a restriction: "The following
//! Ability packages can be taken to speed up character generation. Each
//! represents a particular sort of childhood. Note that you can spend the 45
//! experience points for yourself, as well."
//! (Source: Ars Magica - Definitive Edition (Core Rules).md:2382.) So a
//! character that takes no package is not incomplete — a package is only ever a
//! pre-filled answer to the question
//! [`ChildhoodRules`](crate::life_stage::ChildhoodRules) already poses.
//!
//! Each package spends exactly the childhood block it is a shortcut for: 75
//! experience points in the native language plus 45 across the closed spread
//! list (Ars Magica - Definitive Edition (Core Rules).md:2378). This module therefore *prices* a package but never
//! charges anything — the restricted 45/75 pools in
//! [`crate::effective::xp_allocation`] already do the funding.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use crate::ability::AdvancementTable;
use crate::ruleset::Ruleset;
use crate::types::{AbilityScore, Entity, Id, SourceRef, is_false};

/// One Ability score a Sample Childhood package grants.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChildhoodEntry {
    /// The Ability this entry buys, e.g. `ability.athletics`.
    pub ability: Id,
    /// The whole Ability score the package brings this Ability to.
    pub score: u8,
    /// For a parameterized Ability, the key under which the player supplies the
    /// parameter — `area_a`, `area_b`, `language`. Two entries naming the same
    /// Ability are told apart by this key alone, which is how the Traveling
    /// package grants two different Area Lores ("Area A Lore 1, Area B Lore 1",
    /// Ars Magica - Definitive Edition (Core Rules).md:2388). `None` for an entry that needs nothing asked.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub slot: Option<String>,
    /// Whether this entry is the package's native language — the one the
    /// childhood's 75-point block funds, as opposed to the 45-point spread.
    /// Which language it is is not recorded here: that is the player's choice,
    /// held in [`LifeStagePlan::native_language`](crate::life_stage::LifeStagePlan).
    #[serde(default, skip_serializing_if = "is_false")]
    pub native: bool,
}

/// A Sample Childhood package: one line of the rulebook's list, as data.
///
/// Source: Ars Magica - Definitive Edition (Core Rules).md:2380-2388 (the five
/// packages at `:2384-2388`, one per line).
///
/// # Entry order
///
/// Entries keep their **file order**; they are deliberately not re-sorted on
/// load, unlike [`AdvancementTable`](crate::ability::AdvancementTable) whose
/// order is not observable. Here it is observable — the order decides the
/// sequence in which a UI asks for the parameterized entries — so the shipped
/// file's authored `(ability, slot)` order is the order callers see. Keeping the
/// file canonical is thus a data concern, checked where the file is loaded,
/// rather than papered over by a sort here.
///
/// # Application (later slices)
///
/// Applying a package is a **monotone raise** — `score = max(existing, entry.score)`
/// keyed by `(ability, parameter)` — and therefore idempotent: applying the same
/// package twice, or applying one over hand-spent points, never lowers a score
/// and never charges twice. Funding is not checked at application time either;
/// the existing restricted 45/75 pools report an overspend.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChildhoodPackage {
    /// Slug-style id, e.g. `childhood.athletic`. Its display name lives in
    /// `rules/i18n`, keyed by this id.
    pub id: Id,
    /// The Ability scores the package grants, in file order.
    pub entries: Vec<ChildhoodEntry>,
    /// Provenance into the authoritative Markdown rules source.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<SourceRef>,
}

impl ChildhoodPackage {
    /// The package's native-language entry — the one the childhood's 75-point
    /// block funds. `None` for a package that grants no native language.
    ///
    /// The first flagged entry wins. That a package carries at most one is a
    /// load-time data-integrity guarantee, so it is not re-litigated here.
    pub fn native_entry(&self) -> Option<&ChildhoodEntry> {
        self.entries.iter().find(|entry| entry.native)
    }

    /// Every entry the 45-point spread funds: all but the native language, in
    /// file order. The spread may itself include a *second* Living Language —
    /// "Living Language (other than the character's native language)"
    /// (Ars Magica - Definitive Edition (Core Rules).md:2378), which the Traveling package takes up — so
    /// membership is decided by the `native` flag, never by the Ability id.
    pub fn spread_entries(&self) -> impl Iterator<Item = &ChildhoodEntry> {
        self.entries.iter().filter(|entry| !entry.native)
    }

    /// What the package's spread costs off the Ability advancement table
    /// ("ABILITY To Buy", Ars Magica - Definitive Edition (Core Rules).md:2406-2427). Every package in the
    /// rulebook prices to exactly the 45 points early childhood grants
    /// (Ars Magica - Definitive Edition (Core Rules).md:2378), so taking one can never smuggle in experience the
    /// block does not fund — which is what makes this figure worth computing
    /// rather than assuming.
    ///
    /// `None` when any entry's score has no row in the table: an unpriceable
    /// package is reported as such, never silently costed at 0.
    pub fn spread_xp(&self, advancement: &AdvancementTable) -> Option<u32> {
        self.spread_entries().try_fold(0u32, |total, entry| {
            Some(total.saturating_add(advancement.xp_for_score(entry.score)?))
        })
    }

    /// What the package's native language costs off the same table: 75 points
    /// for the score of 5 every package grants (Ars Magica - Definitive Edition (Core Rules).md:2378, :2384-2388).
    ///
    /// `None` when the package has no native entry, or its score is off-table.
    pub fn native_xp(&self, advancement: &AdvancementTable) -> Option<u32> {
        advancement.xp_for_score(self.native_entry()?.score)
    }

    /// The parameters the player must supply, as `(slot key, Ability)` pairs in
    /// file order — what a UI has to ask for before the package can be applied.
    /// The native language is never among them: it is chosen once for the
    /// character as a whole, not per package.
    pub fn slots(&self) -> impl Iterator<Item = (&str, &Id)> {
        self.entries
            .iter()
            .filter_map(|entry| Some((entry.slot.as_deref()?, &entry.ability)))
    }
}

/// Why a Sample Childhood package could not be applied to a character.
///
/// Plain data: no issue codes, no Fluent keys, no user-facing prose. Turning a
/// rejection into a localized validation issue happens in `validation/`, where
/// the emit sites stay visible to the contract-table and phase scanners.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChildhoodRejection {
    /// No ruleset package carries the requested id, so there is nothing to
    /// apply. A package is taken by id, so an unknown one is reported rather
    /// than silently doing nothing.
    UnknownPackage {
        /// The id that resolved to no package.
        package: Id,
    },
    /// The character has no life-stage plan, or its plan names no native
    /// language, so the package's native-language entry has no language to be
    /// written under.
    NativeLanguageUnset,
    /// A parameterized entry's slot was left unanswered — no value at all, or
    /// one that is blank once trimmed — so the row it would write has no
    /// parameter to be told apart by.
    SlotUnfilled {
        /// The entry's slot key, for targeting the offending field.
        slot: String,
        /// The Ability the unanswered entry buys.
        ability: Id,
    },
    /// A language slot was answered with the character's own native language. The
    /// childhood spread buys "Living Language (other than the character's native
    /// language)" (Ars Magica - Definitive Edition (Core Rules).md:2378), so the
    /// second language has to be a different one — and merging it into the native
    /// row would silently discard the entry besides.
    SlotIsNativeLanguage {
        /// The entry's slot key, for targeting the offending field.
        slot: String,
        /// The Ability the entry buys — the childhood's native-language Ability.
        ability: Id,
        /// The language the slot and the plan agree on.
        language: String,
    },
    /// Two slots on the same Ability were answered with the same value. Rows
    /// merge by `(ability, parameter)`, so the two entries would collapse into
    /// one — the character would silently lose everything the second entry
    /// bought.
    DuplicateSlotValue {
        /// The slot reported, for targeting the offending field.
        slot: String,
        /// The earlier slot on the same Ability carrying the same value.
        other_slot: String,
        /// The Ability both slots buy.
        ability: Id,
        /// The value they share.
        value: String,
    },
}

/// Applies the Sample Childhood package `package` names, looked up in the
/// ruleset's catalogue.
///
/// A thin wrapper over [`apply_package`]: it resolves the id — an unknown one is
/// [`ChildhoodRejection::UnknownPackage`], never a silent no-op — and delegates.
/// See [`apply_package`] for what applying one does.
pub fn apply_childhood_package(
    entity: &Entity,
    package: &Id,
    slot_values: &BTreeMap<String, String>,
    ruleset: &Ruleset,
) -> Result<Entity, Vec<ChildhoodRejection>> {
    let Some(found) = ruleset.childhood(package) else {
        return Err(vec![ChildhoodRejection::UnknownPackage {
            package: package.clone(),
        }]);
    };
    apply_package(entity, found, slot_values, ruleset)
}

/// Applies a Sample Childhood package to a character, returning the character it
/// becomes.
///
/// The entity is never mutated in place: a rejected application therefore leaves
/// the caller's own character exactly as it was.
///
/// # What it writes
///
/// Each of the package's entries becomes an ordinary bought
/// [`AbilityScore`] keyed by `(ability, parameter)` — the same rows a
/// hand-divided childhood produces, which is why nothing downstream needs to
/// know a package was involved. The parameter comes from the entry: the
/// native-language entry takes the plan's
/// [`native_language`](crate::life_stage::LifeStagePlan::native_language), a
/// parameterized entry takes `slot_values[slot]` trimmed (so Traveling's two Area
/// Lore slots become two distinct rows), and a plain entry takes none. A slot
/// that is missing or blank once trimmed is unanswered, not answered with an
/// empty string.
///
/// The write is a **monotone raise**: an existing row is brought to
/// `max(existing, entry.score)` and keeps its specialty, and a row the package
/// does not name is left alone. Two consequences are the reason for that choice:
/// a score bought from later life is never lowered by taking a package, and the
/// application is **idempotent** — applying the same package with the same slot
/// values twice yields an identical entity, so a double click cannot charge
/// twice or drift the character.
///
/// The result is [`Entity::normalize`]d, so the rows come back in canonical
/// order, and the package's id is recorded in
/// [`childhood_package`](crate::life_stage::LifeStagePlan::childhood_package) as
/// the for-the-record annotation it is.
///
/// # What it does not check
///
/// **Funding.** The restricted 45/75 childhood pools in
/// [`crate::effective::xp_allocation`] already price what the rows cost against
/// what the childhood blocks grant, so an overspend surfaces as `not_enough_xp`
/// exactly as a hand-typed one would. Charging here as well would double-count.
///
/// # Rejections
///
/// Every [`ChildhoodRejection`] is **collected, not short-circuited**, so a UI
/// can flag every bad field in one pass rather than one per attempt. A rejected
/// application writes nothing at all.
///
/// Source: Ars Magica - Definitive Edition (Core Rules).md:2380-2388.
pub fn apply_package(
    entity: &Entity,
    package: &ChildhoodPackage,
    slot_values: &BTreeMap<String, String>,
    ruleset: &Ruleset,
) -> Result<Entity, Vec<ChildhoodRejection>> {
    let mut rejections = Vec::new();

    // The plan is what this function reads and writes — the native language the
    // package's native entry needs, and the slot the taken package is recorded in —
    // so its absence, not the funding mode, is what leaves nothing to apply to. This
    // is deliberately a plan-presence check and NOT a
    // [`crate::AbilityFunding`] check: the caller decides when to offer a package
    // (the guided flow does so only in life-stage mode), and a package applied to a
    // plan the character is not currently funded by still writes exactly the Ability
    // rows the player asked for, funded from whichever pool is active.
    // A blank language is no language, exactly as a blank slot value is no value.
    let native_language = entity
        .life_stages
        .as_ref()
        .and_then(|plan| plan.native_language.as_deref())
        .map(str::trim)
        .filter(|language| !language.is_empty());
    if native_language.is_none() {
        rejections.push(ChildhoodRejection::NativeLanguageUnset);
    }

    // Only the childhood's own language Ability may not repeat the native
    // language; an Area Lore named after it is perfectly ordinary. Absent
    // life-stage rules the question cannot be asked, and the load-time integrity
    // check has already rejected packages shipped without them.
    let native_language_ability = ruleset
        .life_stages()
        .map(|rules| &rules.childhood.native_language_ability);
    // The slot each `(ability, value)` pair was first answered under, so a repeat
    // can name the slot it collides with.
    let mut answered: BTreeMap<(&Id, &str), &str> = BTreeMap::new();

    let mut scores = entity.ability_scores.clone();
    for entry in &package.entries {
        let parameter = if entry.native {
            match native_language {
                Some(language) => Some(language.to_string()),
                // Already reported above; there is nothing to write it under.
                None => continue,
            }
        } else if let Some(slot) = entry.slot.as_deref() {
            let Some(value) = filled_slot_value(slot_values, slot) else {
                rejections.push(ChildhoodRejection::SlotUnfilled {
                    slot: slot.to_string(),
                    ability: entry.ability.clone(),
                });
                continue;
            };
            if native_language_ability == Some(&entry.ability) && Some(value) == native_language {
                rejections.push(ChildhoodRejection::SlotIsNativeLanguage {
                    slot: slot.to_string(),
                    ability: entry.ability.clone(),
                    language: value.to_string(),
                });
                continue;
            }
            // Two entries of one Ability answered alike would merge into a single
            // row, so the second entry's experience would vanish.
            if let Some(other_slot) = answered.insert((&entry.ability, value), slot) {
                rejections.push(ChildhoodRejection::DuplicateSlotValue {
                    slot: slot.to_string(),
                    other_slot: other_slot.to_string(),
                    ability: entry.ability.clone(),
                    value: value.to_string(),
                });
                continue;
            }
            Some(value.to_string())
        } else {
            None
        };
        raise_score(&mut scores, &entry.ability, parameter, entry.score);
    }

    if !rejections.is_empty() {
        return Err(rejections);
    }

    let mut applied = entity.clone();
    applied.ability_scores = scores;
    if let Some(plan) = applied.life_stages.as_mut() {
        plan.childhood_package = Some(package.id.clone());
    }
    applied.normalize();
    Ok(applied)
}

/// The player's answer for `slot`, or `None` when the slot is unanswered.
///
/// Surrounding whitespace is the player's typing rather than part of the answer,
/// so it is trimmed off — which also makes a whitespace-only value count as no
/// value at all, as it should.
fn filled_slot_value<'a>(slot_values: &'a BTreeMap<String, String>, slot: &str) -> Option<&'a str> {
    let value = slot_values.get(slot)?.trim();
    (!value.is_empty()).then_some(value)
}

/// Raises the `(ability, parameter)` row to `score`, adding it when the
/// character has none. The raise is monotone — a higher bought score stands —
/// and an existing row keeps everything else it carries, its specialty included.
fn raise_score(scores: &mut Vec<AbilityScore>, ability: &Id, parameter: Option<String>, score: u8) {
    if let Some(existing) = scores
        .iter_mut()
        .find(|row| row.ability == *ability && row.parameter == parameter)
    {
        existing.score = existing.score.max(score);
        return;
    }
    scores.push(AbilityScore {
        ability: ability.clone(),
        score,
        specialty: None,
        parameter,
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    use std::collections::BTreeMap;

    use crate::life_stage::LifeStagePlan;
    use crate::ruleset::{Ruleset, RulesetSources};
    use crate::types::{AbilityScore, Entity, EntityKind, RulesetRef};

    /// "Athletic Childhood: Athletics 2, Brawl 2, Native Language 5, Swim 2"
    /// (Ars Magica - Definitive Edition (Core Rules).md:2384) as the shipped file will carry it — entries in
    /// `(ability, slot)` order, no slots to fill.
    const ATHLETIC: &str = r#"{ "id": "childhood.athletic",
      "entries": [
        { "ability": "ability.athletics", "score": 2 },
        { "ability": "ability.brawl", "score": 2 },
        { "ability": "ability.living_language", "score": 5, "native": true },
        { "ability": "ability.swim", "score": 2 }
      ],
      "source": { "file": "Ars Magica - Definitive Edition (Core Rules).md", "lines": [2384, 2384] } }"#;

    /// "Traveling Childhood: Area A Lore 1, Area B Lore 1, Folk Ken 2, Living
    /// Language 1, Native Language 5, Survival 2" (Ars Magica - Definitive Edition (Core Rules).md:2388) — the
    /// package that exercises every shape at once: two instances of one
    /// parameterized Ability, and a spread Living Language beside the native one.
    const TRAVELING: &str = r#"{ "id": "childhood.traveling",
      "entries": [
        { "ability": "ability.area_lore", "score": 1, "slot": "area_a" },
        { "ability": "ability.area_lore", "score": 1, "slot": "area_b" },
        { "ability": "ability.folk_ken", "score": 2 },
        { "ability": "ability.living_language", "score": 5, "native": true },
        { "ability": "ability.living_language", "score": 1, "slot": "language" },
        { "ability": "ability.survival", "score": 2 }
      ],
      "source": { "file": "Ars Magica - Definitive Edition (Core Rules).md", "lines": [2388, 2388] } }"#;

    fn athletic() -> ChildhoodPackage {
        serde_json::from_str(ATHLETIC).expect("the shipped package shape parses")
    }

    fn traveling() -> ChildhoodPackage {
        serde_json::from_str(TRAVELING).expect("the shipped package shape parses")
    }

    #[test]
    fn athletic_childhood_parses() {
        let package = athletic();
        assert_eq!(package.id, Id::new("childhood.athletic"));

        let abilities: Vec<&str> = package
            .entries
            .iter()
            .map(|entry| entry.ability.as_str())
            .collect();
        assert_eq!(
            abilities,
            vec![
                "ability.athletics",
                "ability.brawl",
                "ability.living_language",
                "ability.swim"
            ]
        );

        // One native entry (the language, at 5), and nothing to ask the player.
        let native = package
            .entries
            .iter()
            .find(|entry| entry.native)
            .expect("a native language entry");
        assert_eq!(native.ability, Id::new("ability.living_language"));
        assert_eq!(native.score, 5);
        assert!(package.entries.iter().all(|entry| entry.slot.is_none()));
        assert_eq!(
            package.entries.iter().filter(|e| e.native).count(),
            1,
            "exactly one entry is the native language"
        );

        let source = package.source.expect("provenance");
        assert_eq!(
            source.file,
            "Ars Magica - Definitive Edition (Core Rules).md"
        );
        assert_eq!((source.lines.start, source.lines.end), (2384, 2384));
    }

    #[test]
    fn traveling_childhood_parses_two_instances_of_one_ability() {
        let package = traveling();
        assert_eq!(package.id, Id::new("childhood.traveling"));
        assert_eq!(package.entries.len(), 6);

        // The two Area Lores are one Ability told apart by slot alone.
        let areas: Vec<Option<&str>> = package
            .entries
            .iter()
            .filter(|entry| entry.ability == Id::new("ability.area_lore"))
            .map(|entry| entry.slot.as_deref())
            .collect();
        assert_eq!(areas, vec![Some("area_a"), Some("area_b")]);

        // A spread Living Language at 1 sits beside the native one at 5.
        let languages: Vec<(u8, bool, Option<&str>)> = package
            .entries
            .iter()
            .filter(|entry| entry.ability == Id::new("ability.living_language"))
            .map(|entry| (entry.score, entry.native, entry.slot.as_deref()))
            .collect();
        assert_eq!(
            languages,
            vec![(5, true, None), (1, false, Some("language"))]
        );

        let source = package.source.expect("provenance");
        assert_eq!((source.lines.start, source.lines.end), (2388, 2388));
    }

    /// Absent `slot` and `native` are the defaults, so canonical JSON omits them
    /// and a round-trip is byte-stable.
    #[test]
    fn canonical_json_skips_absent_slot_and_native() {
        let package = athletic();
        let json = serde_json::to_string(&package).unwrap();
        assert!(
            !json.contains("slot"),
            "no entry has a slot, so the key must not appear: {json}"
        );
        assert_eq!(
            json.matches("\"native\":true").count(),
            1,
            "only the native entry carries the flag: {json}"
        );
        assert!(
            !json.contains("\"native\":false"),
            "a defaulted flag must be skipped: {json}"
        );

        let back: ChildhoodPackage = serde_json::from_str(&json).unwrap();
        assert_eq!(back, package);
        assert_eq!(serde_json::to_string(&back).unwrap(), json);
    }

    /// Entries keep file order rather than being re-sorted on load, because that
    /// order is what the parameter slots are asked for in.
    #[test]
    fn entries_keep_their_file_order() {
        let out_of_order: ChildhoodPackage = serde_json::from_str(
            r#"{ "id": "childhood.athletic",
              "entries": [
                { "ability": "ability.swim", "score": 2 },
                { "ability": "ability.athletics", "score": 2 }
              ] }"#,
        )
        .unwrap();
        let abilities: Vec<&str> = out_of_order
            .entries
            .iter()
            .map(|entry| entry.ability.as_str())
            .collect();
        assert_eq!(abilities, vec!["ability.swim", "ability.athletics"]);
        assert!(out_of_order.source.is_none());
    }

    /// The canonical "ABILITY To Buy" column, scores 1-10
    /// (Ars Magica - Definitive Edition (Core Rules).md:2408-2417).
    fn table() -> AdvancementTable {
        serde_json::from_str(
            r#"[
              { "score": 1, "total_xp": 5 },
              { "score": 2, "total_xp": 15 },
              { "score": 3, "total_xp": 30 },
              { "score": 4, "total_xp": 50 },
              { "score": 5, "total_xp": 75 },
              { "score": 6, "total_xp": 105 },
              { "score": 7, "total_xp": 140 },
              { "score": 8, "total_xp": 180 },
              { "score": 9, "total_xp": 225 },
              { "score": 10, "total_xp": 275 }
            ]"#,
        )
        .unwrap()
    }

    /// A package is a shortcut for spending the childhood's 45 points, so its
    /// spread must price to exactly 45 — Athletic 15+15+15, Traveling
    /// 5+5+15+5+15 (Ars Magica - Definitive Edition (Core Rules).md:2378, :2406-2427).
    #[test]
    fn every_package_spreads_exactly_45_experience_points() {
        let table = table();
        assert_eq!(athletic().spread_xp(&table), Some(45));
        assert_eq!(traveling().spread_xp(&table), Some(45));
    }

    /// And its native language to exactly the 75 the other block grants.
    #[test]
    fn the_native_entry_prices_at_75() {
        let table = table();
        assert_eq!(athletic().native_xp(&table), Some(75));
        assert_eq!(traveling().native_xp(&table), Some(75));
    }

    /// A score the table does not price makes the package unpriceable rather
    /// than free: the cost is unknown, and reporting 0 would understate it.
    #[test]
    fn an_off_table_score_cannot_be_priced() {
        let table = table();
        let too_high: ChildhoodPackage = serde_json::from_str(
            r#"{ "id": "childhood.impossible",
              "entries": [
                { "ability": "ability.athletics", "score": 11 },
                { "ability": "ability.living_language", "score": 11, "native": true }
              ] }"#,
        )
        .unwrap();
        assert_eq!(too_high.spread_xp(&table), None);
        assert_eq!(too_high.native_xp(&table), None);
    }

    /// Nothing in the shape requires a native language, so a package without one
    /// prices no native block at all — as distinct from pricing it at 0.
    #[test]
    fn a_package_without_a_native_entry_has_no_native_experience() {
        let package: ChildhoodPackage = serde_json::from_str(
            r#"{ "id": "childhood.spread_only",
              "entries": [{ "ability": "ability.athletics", "score": 2 }] }"#,
        )
        .unwrap();
        assert!(package.native_entry().is_none());
        assert_eq!(package.native_xp(&table()), None);
    }

    /// The native entry is the flagged one, whatever its position in the list.
    #[test]
    fn the_native_entry_is_the_flagged_one() {
        let native = traveling().native_entry().expect("a native entry").clone();
        assert_eq!(native.ability, Id::new("ability.living_language"));
        assert_eq!(native.score, 5);
        assert!(native.slot.is_none());
    }

    /// The spread is everything but the native language — including a second
    /// Living Language, which the flag distinguishes and the id cannot.
    #[test]
    fn spread_entries_exclude_the_native_language() {
        let package = traveling();
        let spread: Vec<(&str, u8)> = package
            .spread_entries()
            .map(|entry| (entry.ability.as_str(), entry.score))
            .collect();
        assert_eq!(
            spread,
            vec![
                ("ability.area_lore", 1),
                ("ability.area_lore", 1),
                ("ability.folk_ken", 2),
                ("ability.living_language", 1),
                ("ability.survival", 2)
            ]
        );
    }

    /// `slots()` is what a UI must ask for, in file order; a package with
    /// nothing parameterized asks nothing.
    #[test]
    fn slots_name_what_the_ui_must_ask_for() {
        let package = traveling();
        let slots: Vec<(&str, &str)> = package
            .slots()
            .map(|(slot, ability)| (slot, ability.as_str()))
            .collect();
        assert_eq!(
            slots,
            vec![
                ("area_a", "ability.area_lore"),
                ("area_b", "ability.area_lore"),
                ("language", "ability.living_language")
            ]
        );

        assert_eq!(athletic().slots().count(), 0);
    }

    // --- Applying a package ------------------------------------------------

    /// The abilities the two fixtures name, priced by the canonical "ABILITY To
    /// Buy" column for scores 1-5 (Ars Magica - Definitive Edition (Core Rules).md:2406-2427).
    const APPLY_ABILITIES: &str = r#"{
      "advancement": [
        { "score": 1, "total_xp": 5 },
        { "score": 2, "total_xp": 15 },
        { "score": 3, "total_xp": 30 },
        { "score": 4, "total_xp": 50 },
        { "score": 5, "total_xp": 75 }
      ],
      "abilities": [
        { "id": "ability.area_lore", "category": "general", "parameter": "area" },
        { "id": "ability.athletics", "category": "general" },
        { "id": "ability.brawl", "category": "general" },
        { "id": "ability.folk_ken", "category": "general" },
        { "id": "ability.living_language", "category": "general", "parameter": "language" },
        { "id": "ability.stealth", "category": "general" },
        { "id": "ability.survival", "category": "general" },
        { "id": "ability.swim", "category": "general" }
      ]
    }"#;

    /// The two childhood blocks a package is a shortcut for (Ars Magica - Definitive Edition (Core Rules).md:2378).
    const APPLY_LIFE_STAGES: &str = r#"{
      "childhood": {
        "years": 5,
        "native_language_ability": "ability.living_language",
        "native_language_xp": 75,
        "spread_xp": 45,
        "spread_abilities": [
          "ability.area_lore",
          "ability.athletics",
          "ability.brawl",
          "ability.folk_ken",
          "ability.living_language",
          "ability.stealth",
          "ability.survival",
          "ability.swim"
        ]
      },
      "later_life": { "xp_per_year": 15 }
    }"#;

    /// A ruleset shipping both fixtures as its packages, so an application can be
    /// looked up by id and is priced by the same load-time rules the real file
    /// passes.
    fn ruleset() -> Ruleset {
        let childhoods = format!(r#"{{ "packages": [{ATHLETIC}, {TRAVELING}] }}"#);
        Ruleset::from_sources(RulesetSources {
            id: "test",
            version: "1",
            point_items: "[]",
            type_profiles: "[]",
            abilities: Some(APPLY_ABILITIES),
            life_stages: Some(APPLY_LIFE_STAGES),
            childhoods: Some(&childhoods),
            ..RulesetSources::default()
        })
        .expect("the childhood fixtures load")
    }

    /// A companion built through its life stages, speaking `native_language`.
    fn child(native_language: Option<&str>) -> Entity {
        let mut entity = Entity::new(
            EntityKind::Character,
            Id::new("companion"),
            RulesetRef::new(Id::new("test"), "1"),
        );
        entity.age = Some(25);
        entity.life_stages = Some(LifeStagePlan {
            native_language: native_language.map(str::to_string),
            ..LifeStagePlan::default()
        });
        entity
    }

    /// The entity's Ability rows as `(ability, parameter, score)`, in the
    /// canonical order [`Entity::normalize`] leaves them in.
    fn rows(entity: &Entity) -> Vec<(&str, Option<&str>, u8)> {
        entity
            .ability_scores
            .iter()
            .map(|row| (row.ability.as_str(), row.parameter.as_deref(), row.score))
            .collect()
    }

    fn score(ability: &str, score: u8, specialty: Option<&str>) -> AbilityScore {
        AbilityScore {
            ability: Id::new(ability),
            score,
            specialty: specialty.map(str::to_string),
            parameter: None,
        }
    }

    /// Every entry becomes an ordinary bought Ability row, and the package the
    /// player took is recorded on the plan.
    #[test]
    fn applying_a_package_writes_its_ability_rows() {
        let applied = apply_package(
            &child(Some("German")),
            &athletic(),
            &BTreeMap::new(),
            &ruleset(),
        )
        .expect("a package with nothing to ask for applies");

        assert_eq!(
            rows(&applied),
            vec![
                ("ability.athletics", None, 2),
                ("ability.brawl", None, 2),
                ("ability.living_language", Some("German"), 5),
                ("ability.swim", None, 2),
            ]
        );
        assert_eq!(
            applied
                .life_stages
                .as_ref()
                .and_then(|plan| plan.childhood_package.as_ref()),
            Some(&Id::new("childhood.athletic"))
        );
    }

    /// Application is a monotone raise, so a score bought from later life stands
    /// and a row the package does not name survives untouched — including its
    /// specialty.
    #[test]
    fn applying_a_package_never_lowers_a_bought_score() {
        let mut entity = child(Some("German"));
        entity.ability_scores = vec![
            score("ability.athletics", 4, Some("running")),
            score("ability.stealth", 3, None),
            score("ability.swim", 1, None),
        ];
        entity.normalize();

        let applied = apply_package(&entity, &athletic(), &BTreeMap::new(), &ruleset())
            .expect("a package applies over hand-bought scores");

        assert_eq!(
            rows(&applied),
            vec![
                ("ability.athletics", None, 4),
                ("ability.brawl", None, 2),
                ("ability.living_language", Some("German"), 5),
                ("ability.stealth", None, 3),
                ("ability.swim", None, 2),
            ]
        );
        assert_eq!(
            applied
                .ability_scores
                .iter()
                .find(|row| row.ability == Id::new("ability.athletics"))
                .and_then(|row| row.specialty.as_deref()),
            Some("running"),
            "an existing row keeps its specialty"
        );
    }

    /// Because the raise is monotone, applying the same package with the same
    /// slot values twice yields an identical entity — nothing is charged twice.
    #[test]
    fn applying_the_same_package_twice_changes_nothing() {
        let ruleset = ruleset();
        let slots = filled(&[
            ("area_a", "Rhine"),
            ("area_b", "Provence"),
            ("language", "Italian"),
        ]);

        let once = apply_package(&child(Some("German")), &traveling(), &slots, &ruleset)
            .expect("the first application");
        let twice =
            apply_package(&once, &traveling(), &slots, &ruleset).expect("the second application");

        assert_eq!(once, twice);
    }

    /// A rejected application writes nothing: the applicator returns a new
    /// entity, so the caller's own is left exactly as it was.
    #[test]
    fn a_rejected_application_leaves_the_entity_untouched() {
        let entity = child(None);
        let before = entity.clone();

        let rejections = apply_package(&entity, &athletic(), &BTreeMap::new(), &ruleset())
            .expect_err("the native entry has no language to take");

        assert_eq!(rejections, vec![ChildhoodRejection::NativeLanguageUnset]);
        assert_eq!(entity, before);
    }

    /// A slot value is nothing more than the row's `parameter`, so Traveling's two
    /// Area Lore slots become two distinct rows and its spread language sits
    /// beside the native one (Ars Magica - Definitive Edition (Core Rules).md:2388). Surrounding whitespace is the
    /// player's typing, not part of the answer, so it is trimmed off.
    #[test]
    fn slot_values_become_ordinary_ability_parameters() {
        let applied = apply_package(
            &child(Some("German")),
            &traveling(),
            &filled(&[
                ("area_a", "  Rhine  "),
                ("area_b", "Provence"),
                ("language", "Italian"),
            ]),
            &ruleset(),
        )
        .expect("every slot is filled");

        assert_eq!(
            rows(&applied),
            vec![
                ("ability.area_lore", Some("Provence"), 1),
                ("ability.area_lore", Some("Rhine"), 1),
                ("ability.folk_ken", None, 2),
                ("ability.living_language", Some("Italian"), 1),
                ("ability.living_language", Some("German"), 5),
                ("ability.survival", None, 2),
            ]
        );
    }

    /// A slot the player left blank is unanswered, not answered with an empty
    /// string: writing `Area Lore ()` would be a row no one asked for.
    #[test]
    fn a_blank_slot_value_counts_as_unfilled() {
        let rejections = apply_package(
            &child(Some("German")),
            &traveling(),
            &filled(&[
                ("area_a", "Rhine"),
                ("area_b", "   "),
                ("language", "Italian"),
            ]),
            &ruleset(),
        )
        .expect_err("a whitespace-only value fills nothing");

        assert_eq!(
            rejections,
            vec![ChildhoodRejection::SlotUnfilled {
                slot: "area_b".to_string(),
                ability: Id::new("ability.area_lore"),
            }]
        );
    }

    /// A package is taken by id, so an id no ruleset ships is a rejection rather
    /// than a silent no-op; a known one is applied exactly as `apply_package`
    /// would.
    #[test]
    fn an_unknown_package_is_rejected() {
        let ruleset = ruleset();
        let rejections = apply_childhood_package(
            &child(Some("German")),
            &Id::new("childhood.nonesuch"),
            &BTreeMap::new(),
            &ruleset,
        )
        .expect_err("no such package");
        assert_eq!(
            rejections,
            vec![ChildhoodRejection::UnknownPackage {
                package: Id::new("childhood.nonesuch"),
            }]
        );

        let applied = apply_childhood_package(
            &child(Some("German")),
            &Id::new("childhood.athletic"),
            &BTreeMap::new(),
            &ruleset,
        )
        .expect("a shipped package applies");
        assert_eq!(
            applied,
            apply_package(
                &child(Some("German")),
                &athletic(),
                &BTreeMap::new(),
                &ruleset
            )
            .expect("the same package, looked up by hand")
        );
    }

    /// Childhood exists only in life-stage mode, and the native entry takes the
    /// plan's language — so neither a character without a plan nor one whose plan
    /// leaves the language blank can take a package.
    #[test]
    fn a_plan_without_a_native_language_cannot_take_a_package() {
        let ruleset = ruleset();

        let mut without_plan = child(None);
        without_plan.life_stages = None;
        assert_eq!(
            apply_package(&without_plan, &athletic(), &BTreeMap::new(), &ruleset)
                .expect_err("no plan at all"),
            vec![ChildhoodRejection::NativeLanguageUnset]
        );

        assert_eq!(
            apply_package(&child(Some("   ")), &athletic(), &BTreeMap::new(), &ruleset)
                .expect_err("a blank language is no language"),
            vec![ChildhoodRejection::NativeLanguageUnset]
        );
    }

    /// A slot with no value at all is as unanswered as a blank one.
    #[test]
    fn an_unfilled_slot_is_rejected() {
        let rejections = apply_package(
            &child(Some("German")),
            &traveling(),
            &filled(&[("area_a", "Rhine"), ("language", "Italian")]),
            &ruleset(),
        )
        .expect_err("area_b was never answered");

        assert_eq!(
            rejections,
            vec![ChildhoodRejection::SlotUnfilled {
                slot: "area_b".to_string(),
                ability: Id::new("ability.area_lore"),
            }]
        );
    }

    /// The childhood spread buys "Living Language (other than the character's
    /// native language)" (Ars Magica - Definitive Edition (Core Rules).md:2378), so a language slot answered with
    /// the native language is illegal — while an Area Lore that happens to be
    /// named after it is perfectly ordinary.
    #[test]
    fn a_language_slot_filled_with_the_native_language_is_rejected() {
        let ruleset = ruleset();
        let rejections = apply_package(
            &child(Some("German")),
            &traveling(),
            &filled(&[
                ("area_a", "Rhine"),
                ("area_b", "Provence"),
                ("language", "German"),
            ]),
            &ruleset,
        )
        .expect_err("the spread language may not be the native one");

        assert_eq!(
            rejections,
            vec![ChildhoodRejection::SlotIsNativeLanguage {
                slot: "language".to_string(),
                ability: Id::new("ability.living_language"),
                language: "German".to_string(),
            }]
        );

        apply_package(
            &child(Some("German")),
            &traveling(),
            &filled(&[
                ("area_a", "German"),
                ("area_b", "Provence"),
                ("language", "Italian"),
            ]),
            &ruleset,
        )
        .expect("an area named after the language is not the language");
    }

    /// Rows merge by `(ability, parameter)`, so two Area Lore slots answered
    /// "Bavaria" would collapse into ONE row at score 1: `duplicate_ability`
    /// would never fire, and 40 experience points would vanish into an anonymous
    /// unspent-pool warning. Hence the rejection.
    #[test]
    fn two_slots_with_the_same_value_are_rejected() {
        let rejections = apply_package(
            &child(Some("German")),
            &traveling(),
            &filled(&[
                ("area_a", "Bavaria"),
                ("area_b", "Bavaria"),
                ("language", "Italian"),
            ]),
            &ruleset(),
        )
        .expect_err("one area cannot fill both slots");

        assert_eq!(
            rejections,
            vec![ChildhoodRejection::DuplicateSlotValue {
                slot: "area_b".to_string(),
                other_slot: "area_a".to_string(),
                ability: Id::new("ability.area_lore"),
                value: "Bavaria".to_string(),
            }]
        );
    }

    /// Rejections are collected rather than short-circuited, so a UI can flag
    /// every bad field in one pass instead of one per attempt.
    #[test]
    fn every_bad_slot_is_reported_at_once() {
        let rejections = apply_package(
            &child(Some("German")),
            &traveling(),
            &filled(&[("language", "Italian")]),
            &ruleset(),
        )
        .expect_err("two areas were never answered");

        assert_eq!(
            rejections,
            vec![
                ChildhoodRejection::SlotUnfilled {
                    slot: "area_a".to_string(),
                    ability: Id::new("ability.area_lore"),
                },
                ChildhoodRejection::SlotUnfilled {
                    slot: "area_b".to_string(),
                    ability: Id::new("ability.area_lore"),
                },
            ]
        );
    }

    /// Slot values as a UI would supply them.
    fn filled(pairs: &[(&str, &str)]) -> BTreeMap<String, String> {
        pairs
            .iter()
            .map(|(slot, value)| ((*slot).to_string(), (*value).to_string()))
            .collect()
    }
}
