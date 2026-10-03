//! V/F `summary` factual fidelity — the two WRONG and four INCOMPLETE English
//! summaries found by the night-2 summary audit (`tmp/summary-audit.md`; the
//! verbatim rulebook passages and the reasoning per row are in
//! `tmp/sumfix-handover.md`), fixed in both locales together.
//!
//! Each target keeps what the old summary already said correctly and adds or
//! corrects the missing clause in the rulebook's own words: the English from
//! the English book, the German from the German book at the same line (the
//! German file mirrors the English line by line). German game terms follow the
//! translation tables where they differ from the book's spelling (CLAUDE.md,
//! precedence rule 3): *Kleine Tugend* (`magische-qualitaeten.md`),
//! *Formulaische* (`grundbegriffe.md`, `alterung-twilight.md`) and
//! *Kampffertigkeiten* (`kampf.md`). The experience pool is the app's own
//! label, *EP-Vorrat* (`xp-pool` in `locales/de/main.ftl`), *EP* being the
//! tables' abbreviation of *Erfahrungspunkte* (`labor-fortschritt.md`). A cited
//! Virtue carries its shipped name (`virtue.unaging`: *Unaging* /
//! *Nicht alternd*), and the German uses the lower-case *du* of the other
//! summaries.
//!
//! Table-driven against the REAL shipped `rules/i18n/{en,de}/virtues_flaws.json`
//! (the `de_summary_terms.rs` idiom), so the test proves the shipped data.

use arm_rules::ruleset::{LocalizedRuleset, Ruleset, RulesetSources};
use arm_rules::types::Id;

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
        spells: Some(include_str!("../../../rules/core/spells.json")),
        spell_mastery_abilities: Some(include_str!(
            "../../../rules/core/spell_mastery_abilities.json"
        )),
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

fn localized_en() -> LocalizedRuleset {
    LocalizedRuleset::from_merged(
        shipped_ruleset(),
        &[
            include_str!("../../../rules/i18n/en/virtues_flaws.json"),
            include_str!("../../../rules/i18n/en/abilities.json"),
            include_str!("../../../rules/i18n/en/arts.json"),
            include_str!("../../../rules/i18n/en/houses.json"),
            include_str!("../../../rules/i18n/en/mythic_companion_types.json"),
            include_str!("../../../rules/i18n/en/spells.json"),
            include_str!("../../../rules/i18n/en/spell_mastery_abilities.json"),
            include_str!("../../../rules/i18n/en/equipment.json"),
            include_str!("../../../rules/i18n/en/aging.json"),
        ],
    )
    .expect("shipped English rules text loads")
}

fn localized_de() -> LocalizedRuleset {
    LocalizedRuleset::from_merged(
        shipped_ruleset(),
        &[
            include_str!("../../../rules/i18n/de/virtues_flaws.json"),
            include_str!("../../../rules/i18n/de/abilities.json"),
            include_str!("../../../rules/i18n/de/arts.json"),
            include_str!("../../../rules/i18n/de/houses.json"),
            include_str!("../../../rules/i18n/de/mythic_companion_types.json"),
            include_str!("../../../rules/i18n/de/spells.json"),
            include_str!("../../../rules/i18n/de/spell_mastery_abilities.json"),
            include_str!("../../../rules/i18n/de/equipment.json"),
            include_str!("../../../rules/i18n/de/aging.json"),
        ],
    )
    .expect("shipped German rules text loads")
}

struct SummaryCase {
    id: &'static str,
    /// The target English summary.
    en: &'static str,
    /// The target German summary, parallel to the English.
    de: &'static str,
}

const CASES: &[SummaryCase] = &[
    // WRONG: the Ability is not exempt from the age-based cap; it may exceed
    // it by two points during character generation (ArMDE:3374, last
    // sentence, verbatim in both locales minus the page reference).
    SummaryCase {
        id: "virtue.affinity_ability",
        en: "Experience put into one Ability counts for half again, and you may exceed the \
             normal age-based cap during character generation by two points for that Ability.",
        de: "Erfahrung, die du in eine Fertigkeit steckst, zählt anderthalbfach, und du darfst \
             während der Charaktererschaffung die normale altersbedingte Obergrenze für diese \
             Fertigkeit um zwei Punkte überschreiten.",
    },
    // WRONG: Verditius Magic does not cast enchantments through craft; it
    // incorporates craft Abilities into the magus's magic (ArMDE:10078, first
    // sentence) and needs casting tools for Formulaic and Ritual spells
    // (ArMDE:10090, first sentence). The Virtue itself is ArMDE:5215-5217.
    SummaryCase {
        id: "virtue.verditius_magic",
        en: "Initiation into the Outer Mystery of House Verditius, which allows you to \
             incorporate craft Abilities into your magic. You need casting tools to cast \
             Formulaic or Ritual spells.",
        de: "Einweihung in das Äußere Mysterium des Hauses Verditius, das es dir erlaubt, \
             Handwerksfertigkeiten in deine Magie zu integrieren. Du benötigst \
             Zauberwerkzeuge, um Formulaische Zauber oder Ritualzauber zu wirken.",
    },
    // INCOMPLETE: missile-weapon attacks are excluded (ArMDE:4189, second
    // sentence, verbatim in both locales).
    SummaryCase {
        id: "virtue.keen_vision",
        en: "+3 to all rolls involving sight, not including attacks with missile weapons.",
        de: "+3 auf alle Würfe, die das Sehen beinhalten, ausgenommen Angriffe mit \
             Fernkampfwaffen.",
    },
    // INCOMPLETE: the bonus does not apply to learning, teaching or writing
    // (ArMDE:4820, third sentence, verbatim in both locales).
    SummaryCase {
        id: "virtue.puissant_art",
        en: "+3 to all totals using one Art; it does not apply when learning, teaching, or \
             writing about the Art. May be taken twice, for two Arts.",
        de: "+3 auf alle Summen mit einer Kunst; es gilt nicht beim Lernen, Unterrichten oder \
             Schreiben über die Kunst. Kann zweimal gewählt werden, für zwei Künste.",
    },
    // INCOMPLETE: Academic and Martial Abilities stay closed to the normal
    // pool (ArMDE:4808, third sentence, verbatim in both locales).
    SummaryCase {
        id: "virtue.privileged_upbringing",
        en: "+50 experience points, spent only on General, Academic, or Martial Abilities. \
             You may not, however, buy Academic or Martial Abilities with your normal pool of \
             experience points unless you have another Virtue or Flaw permitting that.",
        de: "+50 Erfahrungspunkte, nur für Allgemeine, Akademische oder Kampffertigkeiten. \
             Du kannst jedoch Akademische oder Kampffertigkeiten nicht mit deinem normalen \
             EP-Vorrat erwerben, sofern du keine andere Tugend oder keinen anderen Fehler \
             hast, der dies erlaubt.",
    },
    // INCOMPLETE: no immunity to aging (ArMDE:4011, third sentence, verbatim
    // in both locales minus the page reference).
    SummaryCase {
        id: "virtue.greater_immunity",
        en: "You are completely immune to one common, potentially deadly hazard. You may not \
             take immunity to aging — see the Unaging Minor Virtue instead.",
        de: "Du bist gegen eine verbreitete, potenziell tödliche Gefahr völlig immun. Du darfst \
             keine Immunität gegen Altern nehmen – siehe stattdessen die Kleine Tugend Nicht \
             alternd.",
    },
];

/// Every case whose shipped summary in `locale` differs from its target,
/// formatted for the failure message.
fn mismatches(locale: &LocalizedRuleset, target: fn(&SummaryCase) -> &'static str) -> Vec<String> {
    CASES
        .iter()
        .filter_map(|case| {
            let id = Id::new(case.id);
            let got = locale
                .summary(&id)
                .unwrap_or_else(|| panic!("{id} must carry a summary"));
            let want = target(case);
            (got != want).then(|| format!("{id}\n   got: {got}\n  want: {want}"))
        })
        .collect()
}

#[test]
fn english_summaries_state_the_rulebook_facts() {
    let wrong = mismatches(&localized_en(), |case| case.en);
    assert!(
        wrong.is_empty(),
        "English summaries misstate or omit a rulebook clause:\n{}",
        wrong.join("\n")
    );
}

#[test]
fn german_summaries_state_the_rulebook_facts() {
    let wrong = mismatches(&localized_de(), |case| case.de);
    assert!(
        wrong.is_empty(),
        "German summaries misstate or omit a rulebook clause:\n{}",
        wrong.join("\n")
    );
}
