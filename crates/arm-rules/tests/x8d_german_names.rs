//! X8d (`tmp/x8d-handover.md`; D31, D71) — the German V/F names below were
//! either corrected to match `rules/source/de/translation-tables/tugenden-fehler.md`
//! (the "adopt" set), reverted to the pre-correction table name because D31
//! withdrew the table-vs-stale-rulebook conviction (the "revert" set), or
//! resolved by D71's three held items (the `SdM:M` rows and Gender Shift).
//!
//! Table-driven against the REAL shipped `rules/i18n/de/virtues_flaws.json` so
//! the fixture proves the shipped data actually changed, not a synthetic
//! stand-in. Two entries (`flaw.deteriorating_power`, `flaw.vulnerable_magic`)
//! carry a `name_unfilled` alongside `name`; both are asserted.

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

/// (id, expected `name`, expected `name_unfilled`)
const EXPECTED: &[(&str, &str, Option<&str>)] = &[
    // -- Adopt the table name (D31) --
    ("virtue.fabric_ripper", "Stoffzerreißer", None),
    ("virtue.leather_ripper", "Lederzerreißer", None),
    (
        "flaw.susceptibility_to_divine_power",
        "Anfälligkeit für Göttliche Kraft",
        None,
    ),
    (
        "flaw.susceptibility_to_faerie_power",
        "Anfälligkeit für Feenkraft",
        None,
    ),
    (
        "flaw.susceptibility_to_infernal_power",
        "Anfälligkeit für Infernale Kraft",
        None,
    ),
    (
        "flaw.imagined_folk_tradition_vulnerability",
        "Eingebildete Volksmagie-Verwundbarkeit",
        None,
    ),
    ("flaw.tainted_offspring", "Befleckter Nachkomme", None),
    ("flaw.stigmatic_catalyst", "Stigmatischer Katalysator", None),
    ("virtue.independent_study", "Eigenständiges Studium", None),
    ("virtue.lone_redcap", "Einzelgänger-Rotkappe", None),
    ("flaw.feral_scent", "Wildgeruch", None),
    ("flaw.apostate", "Abtrünniger", None),
    (
        "flaw.gender_nonconforming_major",
        "Geschlechtsnonkonform (Groß)",
        None,
    ),
    (
        "flaw.gender_nonconforming_minor",
        "Geschlechtsnonkonform (Klein)",
        None,
    ),
    // -- Revert to the pre-correction table name (D31) --
    (
        "flaw.deteriorating_power",
        "Schwindende Kraft ({power})",
        Some("Schwindende Kraft"),
    ),
    ("flaw.disorientating_magic", "Desorientierungsmagie", None),
    ("flaw.enfeebled", "Entkräftet", None),
    (
        "flaw.environmental_magic_condition",
        "Magische Umgebungsbedingung",
        None,
    ),
    (
        "flaw.environmental_sensitivity",
        "Umgebungsempfindlichkeit",
        None,
    ),
    (
        "flaw.vulnerable_magic",
        "Verwundbare Magie ({condition})",
        Some("Verwundbare Magie"),
    ),
    (
        "flaw.vulnerable_to_folk_tradition",
        "Anfällig für Volkszauber",
        None,
    ),
    // -- D71's three held `SdM:M` rows (rule 4 applied despite the "Keep" list) --
    ("virtue.homing_instinct", "Ortsgespür", None),
    ("virtue.magical_warder", "Magischer Wächter", None),
    (
        "virtue.unaffected_by_the_gift",
        "Unempfindlich gegenüber der Gabe",
        None,
    ),
    // -- D71's third held item --
    ("virtue.gender_shift", "Geschlechtswandel", None),
    // -- Untouched: table row is a table error (D71 item 2); stays distinct
    //    from flaw.crippled's "Verkrüppelt" --
    ("flaw.hobbled", "Humpelnd", None),
];

#[test]
fn german_vf_names_match_d31_and_d71() {
    let ruleset = localized_de();
    for (id, expected_name, expected_unfilled) in EXPECTED {
        let id = Id::new(*id);
        let got_name = ruleset
            .display_name(&id)
            .unwrap_or_else(|| panic!("{id} must carry a German name"));
        assert_eq!(
            got_name, *expected_name,
            "{id}'s German `name` must match D31/D71"
        );

        let got_unfilled = ruleset.entry(&id).and_then(|e| e.name_unfilled.as_deref());
        assert_eq!(
            got_unfilled, *expected_unfilled,
            "{id}'s German `name_unfilled` must match D31/D71"
        );
    }
}
