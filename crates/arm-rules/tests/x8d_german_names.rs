//! X8d (`tmp/x8d-handover.md`; D31, D71) set these German V/F names to match
//! D31's table-wins regime. P8-6 (D86, `docs/vf-audit/decisions.md`) retires
//! D31: a current German rulebook landed in the P8 re-sync, so D7.1 applies
//! again and the rulebook heading wins a NAME dispute unless the heading is
//! itself defective. Every value below was re-derived from the current
//! `rules/source/de/Ars Magica Definitive Edition Basisregeln.md` heading at
//! the entry's own source line (`tmp/p8-6-names.md` has the full per-entry
//! evidence); see `crates/arm-rules/tests/de_vf_names.rs` for the mechanical
//! guard this fixture complements.
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
    // -- Unaffected by D31/D71/D86: adopted the table name all along --
    ("virtue.fabric_ripper", "Stoffzerreißer", None),
    (
        "flaw.gender_nonconforming_major",
        "Geschlechtsnonkonform [Groß]",
        None,
    ),
    (
        "flaw.gender_nonconforming_minor",
        "Geschlechtsnonkonform [Klein]",
        None,
    ),
    ("virtue.lone_redcap", "Einzelgänger-Rotkappe", None),
    // -- D86: the current rulebook heading wins over the table name D31 had
    //    adopted (tugenden-fehler.md/grundbegriffe.md pending their own
    //    re-sync) --
    ("virtue.leather_ripper", "Lederreißer", None),
    (
        "flaw.susceptibility_to_divine_power",
        "Anfälligkeit für Göttliche Macht",
        None,
    ),
    (
        "flaw.susceptibility_to_faerie_power",
        "Anfälligkeit für Feenmacht",
        None,
    ),
    (
        "flaw.susceptibility_to_infernal_power",
        "Anfälligkeit für Infernale Macht",
        None,
    ),
    (
        "flaw.imagined_folk_tradition_vulnerability",
        "Eingebildete Volksüberlieferungsanfälligkeit",
        None,
    ),
    ("flaw.tainted_offspring", "Befleckter Nachwuchs", None),
    ("flaw.stigmatic_catalyst", "Stigma-Katalysator", None),
    ("virtue.independent_study", "Unabhängiges Studium", None),
    ("flaw.feral_scent", "Wilder Geruch", None),
    ("flaw.apostate", "Apostat", None),
    ("flaw.depressed", "Schwermütig", None),
    ("virtue.gossip", "Gut unterrichtet", None),
    (
        "virtue.indescribable_face",
        "Unbeschreibliches Gesicht",
        None,
    ),
    ("virtue.templar_office_holder", "Templer-Amtsträger", None),
    ("virtue.templar_prestige", "Templer-Ansehen", None),
    // -- D86: D31's revert undone — the re-synced rulebook heading and the
    //    re-synced table now agree, so these seven adopt that (shared) name --
    (
        "flaw.deteriorating_power",
        "Schwindende Macht ({power})",
        Some("Schwindende Macht"),
    ),
    ("flaw.disorientating_magic", "Desorientierende Magie", None),
    ("flaw.enfeebled", "Geschwächt", None),
    (
        "flaw.environmental_magic_condition",
        "Magische Umweltbedingung",
        None,
    ),
    (
        "flaw.environmental_sensitivity",
        "Umweltempfindlichkeit",
        None,
    ),
    (
        "flaw.vulnerable_magic",
        "Anfällige Magie ({condition})",
        Some("Anfällige Magie"),
    ),
    (
        "flaw.vulnerable_to_folk_tradition",
        "Anfällig für Volksüberlieferungen",
        None,
    ),
    // -- D86: D71's `FORCED_COVERAGE_DESPITE_TAG` retired — these `SdM:M`- and
    //    `SdM:I`-tagged rows are no longer forced into the table dispute, so
    //    the rulebook heading now ships uncontested --
    ("virtue.homing_instinct", "Heimfindungsinstinkt", None),
    ("virtue.magical_warder", "Magischer Hüter", None),
    (
        "virtue.unaffected_by_the_gift",
        "Unbeeindruckt von der Gabe",
        None,
    ),
    ("virtue.gender_shift", "Geschlechtswechsel", None),
    // -- Untouched: table row is a table error (D71 item 2, still valid under
    //    D86 — the current rulebook heading agrees); stays distinct from
    //    flaw.crippled's "Verkrüppelt" --
    ("flaw.hobbled", "Humpelnd", None),
];

#[test]
fn german_vf_names_match_d86() {
    let ruleset = localized_de();
    for (id, expected_name, expected_unfilled) in EXPECTED {
        let id = Id::new(*id);
        let got_name = ruleset
            .display_name(&id)
            .unwrap_or_else(|| panic!("{id} must carry a German name"));
        assert_eq!(
            got_name, *expected_name,
            "{id}'s German `name` must match D86"
        );

        let got_unfilled = ruleset.entry(&id).and_then(|e| e.name_unfilled.as_deref());
        assert_eq!(
            got_unfilled, *expected_unfilled,
            "{id}'s German `name_unfilled` must match D86"
        );
    }
}
