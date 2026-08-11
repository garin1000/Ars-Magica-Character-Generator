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
//! list (Core Rules.md:2378). This module therefore *prices* a package but never
//! charges anything — the restricted 45/75 pools in
//! [`crate::effective::xp_allocation`] already do the funding.

use serde::{Deserialize, Serialize};

use crate::ability::AdvancementTable;
use crate::types::{Id, SourceRef, is_false};

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
    /// Core Rules.md:2388). `None` for an entry that needs nothing asked.
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
    /// (Core Rules.md:2378), which the Traveling package takes up — so
    /// membership is decided by the `native` flag, never by the Ability id.
    pub fn spread_entries(&self) -> impl Iterator<Item = &ChildhoodEntry> {
        self.entries.iter().filter(|entry| !entry.native)
    }

    /// What the package's spread costs off the Ability advancement table
    /// ("ABILITY To Buy", Core Rules.md:2406-2427). Every package in the
    /// rulebook prices to exactly the 45 points early childhood grants
    /// (Core Rules.md:2378), so taking one can never smuggle in experience the
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
    /// for the score of 5 every package grants (Core Rules.md:2378, :2384-2388).
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

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    /// "Athletic Childhood: Athletics 2, Brawl 2, Native Language 5, Swim 2"
    /// (Core Rules.md:2384) as the shipped file will carry it — entries in
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
    /// Language 1, Native Language 5, Survival 2" (Core Rules.md:2388) — the
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
    /// (Core Rules.md:2408-2417).
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
    /// 5+5+15+5+15 (Core Rules.md:2378, :2406-2427).
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
}
