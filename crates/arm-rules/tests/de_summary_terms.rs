//! German V/F `summary` fidelity — the six DE-specific deviations found by the
//! night-2 description audit (`tmp/de-desc-audit.md`, Appendix; full reasoning
//! and the verbatim rulebook passages in `tmp/desum-handover.md`).
//!
//! Two shapes, one table:
//!
//! - **Verbatim**: where the English summary is the book's sentence verbatim,
//!   the German is the line-parallel German sentence verbatim, cut at the same
//!   point as the English.
//! - **Term (D36, D53)**: where a sentence names a game element, it uses the
//!   glossary's term — *Reputation* (`reputationen.md:20`,
//!   `grundbegriffe.md:104`: „nicht ‚Ruf‘“), *Fertigkeit* for an Ability
//!   (F-65) — and changes nothing else.
//!
//! The English summaries are pinned alongside, unchanged: the German follows
//! the English's cut and content, so a fix here must not drift from it.
//!
//! Table-driven against the REAL shipped `rules/i18n/{en,de}/virtues_flaws.json`
//! (the `x8a_german_text.rs` idiom), so the test proves the shipped data.

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
    /// The unchanged English summary the German runs parallel to.
    en: &'static str,
    /// The target German summary.
    de: &'static str,
}

const CASES: &[SummaryCase] = &[
    // Verbatim: ArMDE:3969, first sentence in both locales.
    SummaryCase {
        id: "virtue.the_gift",
        en: "You have the ability to work magic.",
        de: "Du besitzt die Fähigkeit, Magie zu wirken.",
    },
    // Verbatim: ArMDE:7074, second sentence cut at the comma in both locales.
    SummaryCase {
        id: "flaw.weak_parens",
        en: "You gain 60 fewer experience points and 30 fewer spell levels from apprenticeship.",
        de: "Du erhältst 60 Erfahrungspunkte und 30 Zauberstufen weniger aus der Lehrlingschaft.",
    },
    // Term + verbatim: the second sentence is ArMDE:4073's last, verbatim in
    // both locales; the book's own German says Reputation, as the table does.
    SummaryCase {
        id: "virtue.hermetic_prestige",
        en: "Other magi look up to you. You gain a Reputation of level 4 within the Order.",
        de: "Andere Magi blicken zu dir auf. Du erhältst eine Reputation der Stufe 4 innerhalb des Ordens.",
    },
    // Term only: the English is app-condensed (ArMDE:5705), so the German
    // stays a translation of it; Ruf -> Reputation, with agreement.
    SummaryCase {
        id: "flaw.black_sheep",
        en: "Estranged from a prestigious family; a bad Reputation at level 2 among those who respect them.",
        de: "Von einer angesehenen Familie entfremdet; eine schlechte Reputation der Stufe 2 bei denen, die sie achten.",
    },
    // Term only: the English is app-condensed (ArMDE:6312).
    SummaryCase {
        id: "flaw.infamous",
        en: "A level-4 bad Reputation for horrible deeds of your choosing.",
        de: "Eine schlechte Reputation der Stufe 4 für frei wählbare schändliche Taten.",
    },
    // Term (F-65's first defect): Fähigkeit -> Fertigkeit. Since T1
    // (after-deadline answer 5) both locales also carry the granted score, as
    // F-65 gave Dowsing; the Ability is named by its glossary term
    // (`fertigkeiten.md:83`) rather than as „die gleichnamige“ (ArMDE:4928).
    SummaryCase {
        id: "virtue.sense_holiness_and_unholiness",
        en: "Feel the presence of good and evil. Confers the Sense Holiness and Unholiness Ability at 1.",
        de: "Spüre die Gegenwart von Gut und Böse. Verleiht die Fertigkeit Gespür für Heiliges und Unheiliges auf 1.",
    },
];

#[test]
fn english_summaries_stay_as_they_are() {
    let en = localized_en();
    for case in CASES {
        let id = Id::new(case.id);
        let got = en
            .summary(&id)
            .unwrap_or_else(|| panic!("{id} must carry an English summary"));
        assert_eq!(
            got, case.en,
            "{id}: the English summary is out of scope here and must not change"
        );
    }
}

#[test]
fn german_summaries_use_the_rulebook_wording_and_the_glossary_terms() {
    let de = localized_de();
    let mismatches: Vec<String> = CASES
        .iter()
        .filter_map(|case| {
            let id = Id::new(case.id);
            let got = de
                .summary(&id)
                .unwrap_or_else(|| panic!("{id} must carry a German summary"));
            (got != case.de).then(|| format!("{id}\n   got: {got}\n  want: {}", case.de))
        })
        .collect();
    assert!(
        mismatches.is_empty(),
        "German summaries deviate from the verbatim rulebook sentence or the glossary \
         term (D36/D53):\n{}",
        mismatches.join("\n")
    );
}
