//! T2 (try-out finding 23, decision C3) — the Art-parameterized Virtues and
//! Flaws, as the Markdown export names them, in both locales, against the
//! SHIPPED rules i18n.
//!
//! Unfilled, an entry reads as its book heading (EN) or its translation-table
//! row (DE; D31, the table wins on a name). Filled, the chosen Art sits in the
//! template's slot. The two Deficiencies are the case C3 decided: EN
//! "Deficient Creo" / "Deficient Ignem", DE "Defizitäre Technik: Creo" /
//! "Defizitäre Form: Ignem" (no adjective agreement with a Latin Art name);
//! unfilled "Deficient Technique" / "Deficient Form" (ArMDE:5913, :5909) and
//! "Defizitäre Technik" / "Defizitäre Form". Before T2, `flaw.deficient_form`
//! had no `{form}` slot, so the sheet appended the Art ("Deficient Form
//! (Ignem)") while its sibling read "Deficient Creo".
//!
//! The sheet and the in-app row share one wording through `name_unfilled`
//! (`export/resolve.rs::Doc::unfilled_name`); the UI side is pinned by
//! `ui/src/lib/art-parameterized-names.test.ts`.

use arm_rules::export::character_markdown;
use arm_rules::ruleset::{LocalizedRuleset, Ruleset, RulesetSources};
use arm_rules::types::*;
use std::collections::BTreeMap;

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

/// Every chrome key resolved to itself, except the three slot labels these
/// names use, which carry the values `locales/<lang>/main.ftl` ships
/// (`param-label-art`, `-technique`, `-form`), so an unfilled hint reads as it
/// does in the app.
fn labels(
    rs: &LocalizedRuleset,
    art: &str,
    technique: &str,
    form: &str,
) -> BTreeMap<String, String> {
    let mut keys: std::collections::BTreeSet<String> = arm_rules::export::LABEL_KEYS
        .iter()
        .map(|k| k.to_string())
        .collect();
    for profile in rs.ruleset.profiles() {
        keys.insert(format!("type-{}", profile.id));
    }
    for item in rs.ruleset.items() {
        for category in &item.categories {
            keys.insert(format!("category-{category}"));
        }
        for param in &item.parameters {
            keys.insert(format!("param-label-{}", param.key));
        }
    }
    let mut labels: BTreeMap<String, String> = keys.into_iter().map(|k| (k.clone(), k)).collect();
    labels.insert("restricted-xp-list-separator".to_string(), ",".to_string());
    labels.insert("param-label-art".to_string(), art.to_string());
    labels.insert("param-label-technique".to_string(), technique.to_string());
    labels.insert("param-label-form".to_string(), form.to_string());
    labels
}

/// One Art-parameterized item: its id, parameter key, the Art chosen for the
/// filled case, a name prefix that locates its row whether or not the name is
/// right, and the expected filled and unfilled names.
struct Case {
    id: &'static str,
    key: &'static str,
    art: &'static str,
    row_prefix: &'static str,
    filled: &'static str,
    unfilled: &'static str,
}

const EN: &[Case] = &[
    Case {
        id: "flaw.deficient_technique",
        key: "technique",
        art: "art.creo",
        row_prefix: "Deficient",
        filled: "Deficient Creo",
        unfilled: "Deficient Technique",
    },
    Case {
        id: "flaw.deficient_form",
        key: "form",
        art: "art.ignem",
        row_prefix: "Deficient",
        filled: "Deficient Ignem",
        unfilled: "Deficient Form",
    },
    Case {
        id: "virtue.puissant_art",
        key: "art",
        art: "art.creo",
        row_prefix: "Puissant",
        filled: "Puissant Creo",
        unfilled: "Puissant Art",
    },
    Case {
        id: "virtue.affinity_art",
        key: "art",
        art: "art.creo",
        row_prefix: "Affinity with",
        filled: "Affinity with Creo",
        unfilled: "Affinity with Art",
    },
    Case {
        id: "virtue.imbued_with_the_spirit_of_form",
        key: "form",
        art: "art.ignem",
        row_prefix: "Imbued with",
        filled: "Imbued with the Spirit of Ignem",
        unfilled: "Imbued with the Spirit of (Form)",
    },
    Case {
        id: "virtue.extractor_of_form_vis",
        key: "form",
        art: "art.ignem",
        row_prefix: "Extractor of",
        filled: "Extractor of Ignem Vis",
        unfilled: "Extractor of (Form) Vis",
    },
    Case {
        id: "virtue.master_of_form_creatures",
        key: "form",
        art: "art.animal",
        row_prefix: "Master of",
        filled: "Master of Animal Creatures",
        unfilled: "Master of (Form) Creatures",
    },
    Case {
        id: "flaw.hunger_for_form_magic",
        key: "form",
        art: "art.ignem",
        row_prefix: "Hunger for",
        filled: "Hunger for Ignem Magic",
        unfilled: "Hunger for (Form) Magic",
    },
];

const DE: &[Case] = &[
    Case {
        id: "flaw.deficient_technique",
        key: "technique",
        art: "art.creo",
        row_prefix: "Defizitäre",
        filled: "Defizitäre Technik: Creo",
        unfilled: "Defizitäre Technik",
    },
    Case {
        id: "flaw.deficient_form",
        key: "form",
        art: "art.ignem",
        row_prefix: "Defizitäre",
        filled: "Defizitäre Form: Ignem",
        unfilled: "Defizitäre Form",
    },
    Case {
        id: "virtue.puissant_art",
        key: "art",
        art: "art.creo",
        row_prefix: "Begabung in",
        filled: "Begabung in Creo",
        unfilled: "Begabung in (Kunst)",
    },
    Case {
        id: "virtue.affinity_art",
        key: "art",
        art: "art.creo",
        row_prefix: "Affinität zu",
        filled: "Affinität zu Creo",
        unfilled: "Affinität zu (Kunst)",
    },
    // D57 apposition when filled; the table row (D31) when unfilled.
    Case {
        id: "virtue.imbued_with_the_spirit_of_form",
        key: "form",
        art: "art.ignem",
        row_prefix: "Durchdrungen vom",
        filled: "Durchdrungen vom Geist der Form, Ignem",
        unfilled: "Durchdrungen vom Geist der (Form)",
    },
    Case {
        id: "virtue.extractor_of_form_vis",
        key: "form",
        art: "art.ignem",
        row_prefix: "Vis-Gewinner",
        filled: "Vis-Gewinner der Form, Ignem",
        unfilled: "Vis-Gewinner der (Form)",
    },
    Case {
        id: "virtue.master_of_form_creatures",
        key: "form",
        art: "art.animal",
        row_prefix: "Meister der",
        filled: "Meister der Animal-Kreaturen",
        unfilled: "Meister der (Form-)Kreaturen",
    },
    Case {
        id: "flaw.hunger_for_form_magic",
        key: "form",
        art: "art.ignem",
        row_prefix: "Hunger nach",
        filled: "Hunger nach Ignem-Magie",
        unfilled: "Hunger nach (Form-)Magie",
    },
];

fn magus_with(selection: Selection) -> Entity {
    let mut e = Entity::new(
        EntityKind::Character,
        Id::new("magus"),
        RulesetRef::new(Id::new("arm5-core"), "2024.1"),
    );
    e.selections = vec![selection];
    e.normalize();
    e
}

/// The name cell (first column) of the first table row whose name starts with
/// `name_prefix`.
fn name_cell(rendered: &str, name_prefix: &str) -> String {
    let row_start = format!("| {name_prefix}");
    let row = rendered
        .lines()
        .find(|line| line.starts_with(&row_start))
        .unwrap_or_else(|| panic!("no table row starting {row_start:?} in:\n{rendered}"));
    row.trim_start_matches("| ")
        .split(" | ")
        .next()
        .expect("a table row has a first cell")
        .to_string()
}

/// Exports each case filled and unfilled, and returns every name that differs
/// from the expectation, so one run reports them all.
fn mismatches(
    ruleset: &LocalizedRuleset,
    labels: &BTreeMap<String, String>,
    cases: &[Case],
) -> Vec<String> {
    let mut out = Vec::new();
    for case in cases {
        let filled = Selection::with_params(
            Id::new(case.id),
            BTreeMap::from([(case.key.to_string(), Id::new(case.art))]),
        );
        let unfilled = Selection::new(Id::new(case.id));
        for (selection, expected, shape) in [
            (filled, case.filled, "filled"),
            (unfilled, case.unfilled, "unfilled"),
        ] {
            let rendered = character_markdown(&magus_with(selection), ruleset, labels)
                .unwrap_or_else(|e| panic!("{} exports: {e:?}", case.id));
            let got = name_cell(&rendered, case.row_prefix);
            if got != expected {
                out.push(format!(
                    "{} {shape}: got {got:?}, expected {expected:?}",
                    case.id
                ));
            }
        }
    }
    out
}

#[test]
fn english_sheet_names_art_parameterized_items_by_heading_and_chosen_art() {
    let ruleset = localized_en();
    let labels = labels(&ruleset, "Art", "Technique", "Form");
    let wrong = mismatches(&ruleset, &labels, EN);
    assert!(
        wrong.is_empty(),
        "English export names:\n{}",
        wrong.join("\n")
    );
}

#[test]
fn german_sheet_names_art_parameterized_items_by_table_row_and_chosen_art() {
    let ruleset = localized_de();
    let labels = labels(&ruleset, "Kunst", "Technik", "Form");
    let wrong = mismatches(&ruleset, &labels, DE);
    assert!(
        wrong.is_empty(),
        "German export names:\n{}",
        wrong.join("\n")
    );
}
