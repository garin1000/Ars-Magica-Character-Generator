//! Ability rules-text fidelity — the WRONG, MISMATCH and invented-specialty
//! findings of the night-2 summary audit (`tmp/summary-audit-2.md`; the
//! verbatim rulebook passages and the consumer analysis are in
//! `tmp/abfix-handover.md`), fixed in both locales together.
//!
//! Descriptions keep what the old text already said correctly and correct the
//! one wrong clause in the rulebook's own words: the English from the English
//! book, the German from the German book at the same line (the German file
//! mirrors the English line by line). Where the German book names a trait
//! differently from the app, the app's shipped name wins: Awareness is
//! *Aufmerksamkeit* (`ability.awareness`, `fertigkeiten.md`), Perception is
//! *Wahrnehmung* (`characteristic-per`, `grundbegriffe.md`).
//!
//! Specialty lists are the book's own, item for item and in the book's order:
//! the English as printed in the English book, the German as printed in the
//! German book at the same line. An Ability the book gives no specialties
//! carries no `specialties` key at all, as the shipped entries without
//! specialties already do (`types.rs::I18nEntry::specialties` skips an empty
//! list on serialization).
//!
//! Table-driven against the REAL shipped `rules/i18n/{en,de}/abilities.json`.

use std::collections::BTreeMap;

use arm_rules::types::I18nEntry;
use serde_json::Value;

const EN_ABILITIES: &str = include_str!("../../../rules/i18n/en/abilities.json");
const DE_ABILITIES: &str = include_str!("../../../rules/i18n/de/abilities.json");

fn entries(json: &str) -> BTreeMap<String, I18nEntry> {
    serde_json::from_str(json).expect("shipped abilities i18n parses")
}

fn entry<'a>(entries: &'a BTreeMap<String, I18nEntry>, id: &str) -> &'a I18nEntry {
    entries
        .get(id)
        .unwrap_or_else(|| panic!("{id} must be present in the shipped abilities i18n"))
}

struct DescriptionCase {
    id: &'static str,
    en: &'static str,
    de: &'static str,
}

const DESCRIPTIONS: &[DescriptionCase] = &[
    // WRONG: she cannot tell holy from unholy, only that something is one or
    // the other (ArMDE:7735, second sentence; the German book agrees at the
    // same line).
    DescriptionCase {
        id: "ability.sense_passions",
        en: "Sense an intelligent being's strongest emotion or dominant Personality Trait, and \
             tell that something is either holy or unholy, but not distinguish between the two; \
             associated with the Infernal or a false power.",
        de: "Die stärkste Empfindung oder dominante Persönlichkeitseigenschaft eines \
             vernunftbegabten Wesens erspüren und erkennen, dass etwas entweder heilig oder \
             unheilig ist, ohne zwischen den beiden unterscheiden zu können; mit dem Infernalen \
             oder einer falschen Macht verbunden.",
    },
    // MISMATCH: the target rolls Perception + Awareness, not Awareness alone
    // (ArMDE:7607, fourth sentence; the German book at the same line and
    // sentence).
    DescriptionCase {
        id: "ability.legerdemain",
        en: "Sleight of hand and confidence tricks such as filching, cutting purses, and picking \
             locks; the target rolls Perception + Awareness to detect your actions.",
        de: "Geschickte Hände und Trickbetrügereien wie Mausen, Beutelschneiden und \
             Schlösserknacken; das Ziel würfelt Wahrnehmung + Aufmerksamkeit, um deine \
             Handlungen zu bemerken.",
    },
];

struct SpecialtyCase {
    id: &'static str,
    en: &'static [&'static str],
    de: &'static [&'static str],
}

const SPECIALTIES: &[SpecialtyCase] = &[
    // ArMDE:7425, and the German book at the same line.
    SpecialtyCase {
        id: "ability.curse_throwing",
        en: &["diseases", "faerie curses", "livestock"],
        de: &["Krankheiten", "Feenflüche", "Vieh"],
    },
    // ArMDE:7441, and the German book at the same line.
    SpecialtyCase {
        id: "ability.dowsing",
        en: &[
            "searching for a particular kind of thing (water, gold, etc.)",
            "searching in a particular kind of place",
        ],
        de: &[
            "Suche nach einer bestimmten Art von Sache (Wasser, Gold etc.)",
            "Suche an einem bestimmten Ort",
        ],
    },
    // ArMDE:9937, and the German book at the same line.
    SpecialtyCase {
        id: "ability.enigmatic_wisdom",
        en: &["interpreting signs", "explaining the Enigma", "Twilight"],
        de: &["Zeichen deuten", "das Enigma erklären", "Zwielicht"],
    },
    // ArMDE:10003, and the German book at the same line.
    SpecialtyCase {
        id: "ability.faerie_magic",
        en: &[
            "faerie vis",
            "experimenting",
            "inventing spells",
            "charms",
            "lore",
        ],
        de: &[
            "Feenvis",
            "Experimentieren",
            "Zauber erfinden",
            "Amulette",
            "Kunde",
        ],
    },
    // The book prints none: neither the Ability entry (ArMDE:7501-7502) nor
    // the Heartbeast mystery (ArMDE:9886-9898) lists any.
    SpecialtyCase {
        id: "ability.heartbeast",
        en: &[],
        de: &[],
    },
];

fn description_mismatches(json: &str, target: fn(&DescriptionCase) -> &'static str) -> Vec<String> {
    let entries = entries(json);
    DESCRIPTIONS
        .iter()
        .filter_map(|case| {
            let got = entry(&entries, case.id)
                .description
                .as_deref()
                .unwrap_or("");
            let want = target(case);
            (got != want).then(|| format!("{}\n   got: {got}\n  want: {want}", case.id))
        })
        .collect()
}

fn specialty_mismatches(
    json: &str,
    target: fn(&SpecialtyCase) -> &'static [&'static str],
) -> Vec<String> {
    let entries = entries(json);
    SPECIALTIES
        .iter()
        .filter_map(|case| {
            let got = &entry(&entries, case.id).specialties;
            let want = target(case);
            (got.as_slice() != want)
                .then(|| format!("{}\n   got: {got:?}\n  want: {want:?}", case.id))
        })
        .collect()
}

/// Ids whose raw JSON entry still carries a `specialties` key although the
/// book prints none for them.
fn ids_with_a_specialties_key_the_book_does_not_print(json: &str) -> Vec<&'static str> {
    let raw: Value = serde_json::from_str(json).expect("shipped abilities i18n parses");
    SPECIALTIES
        .iter()
        .filter(|case| case.en.is_empty() && case.de.is_empty())
        .filter(|case| raw[case.id].get("specialties").is_some())
        .map(|case| case.id)
        .collect()
}

#[test]
fn english_descriptions_state_the_rulebook_facts() {
    let wrong = description_mismatches(EN_ABILITIES, |case| case.en);
    assert!(
        wrong.is_empty(),
        "English Ability descriptions misstate the rulebook:\n{}",
        wrong.join("\n")
    );
}

#[test]
fn german_descriptions_state_the_rulebook_facts() {
    let wrong = description_mismatches(DE_ABILITIES, |case| case.de);
    assert!(
        wrong.is_empty(),
        "German Ability descriptions misstate the rulebook:\n{}",
        wrong.join("\n")
    );
}

#[test]
fn english_specialties_are_the_books_own() {
    let wrong = specialty_mismatches(EN_ABILITIES, |case| case.en);
    assert!(
        wrong.is_empty(),
        "English Ability specialties differ from the rulebook's list:\n{}",
        wrong.join("\n")
    );
}

#[test]
fn german_specialties_are_the_books_own() {
    let wrong = specialty_mismatches(DE_ABILITIES, |case| case.de);
    assert!(
        wrong.is_empty(),
        "German Ability specialties differ from the rulebook's list:\n{}",
        wrong.join("\n")
    );
}

#[test]
fn an_ability_without_book_specialties_carries_no_specialties_key() {
    let en = ids_with_a_specialties_key_the_book_does_not_print(EN_ABILITIES);
    let de = ids_with_a_specialties_key_the_book_does_not_print(DE_ABILITIES);
    assert!(
        en.is_empty() && de.is_empty(),
        "the book prints no specialties for these, yet a key is present — en: {en:?}, de: {de:?}"
    );
}
