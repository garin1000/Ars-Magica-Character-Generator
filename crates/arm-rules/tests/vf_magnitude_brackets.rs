//! N11 / D84.3 — display convention for a split "Minor or Major" entry.
//!
//! Where the book prints ONE Virtue/Flaw entry taken at either magnitude and
//! the catalogue splits it into two items, the magnitude is shown in square
//! brackets: "Ambitious [Major]", "Potent Magic [Minor]",
//! "False Power [Major]: {virtue}" (DE „Ehrgeizig [Groß]" / „[Klein]").
//! Square brackets keep the magnitude visually apart from a parameter in
//! parentheses ("Potent Magic [Minor] (Fire)"). A name whose book spelling
//! itself carries the magnitude ("Minor Magical Focus") is a plain name and
//! is untouched.
//!
//! The guard is defined BY NAME PATTERN, not by ids or flags (the id and
//! flag sets do not coincide with the split pairs):
//! - no V/F name in either locale still uses the old parenthesised form;
//! - every bracketed name has a sibling carrying the other bracket;
//! - an id's EN and DE names agree on the bracket, and the bracket agrees
//!   with the item's `magnitude` in `rules/core/virtues_flaws.json`.

use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

const VF_JSON: &str = include_str!("../../../rules/core/virtues_flaws.json");
const EN_I18N: &str = include_str!("../../../rules/i18n/en/virtues_flaws.json");
const DE_I18N: &str = include_str!("../../../rules/i18n/de/virtues_flaws.json");

/// One locale's magnitude vocabulary: the bracket text for each magnitude,
/// and the retired parenthesised spellings that must no longer appear.
struct Locale {
    label: &'static str,
    json: &'static str,
    minor: &'static str,
    major: &'static str,
    retired: [&'static str; 2],
}

const EN: Locale = Locale {
    label: "en",
    json: EN_I18N,
    minor: "[Minor]",
    major: "[Major]",
    retired: ["(Minor)", "(Major)"],
};

const DE: Locale = Locale {
    label: "de",
    json: DE_I18N,
    minor: "[Klein]",
    major: "[Groß]",
    retired: ["(Klein)", "(Groß)"],
};

/// Every `name` and `name_unfilled` of the locale's `virtue.*`/`flaw.*`
/// entries, keyed by `(id, field)`.
fn vf_names(json: &str) -> BTreeMap<(String, &'static str), String> {
    let map: Value = serde_json::from_str(json).expect("i18n file is valid JSON");
    let mut out = BTreeMap::new();
    for (id, entry) in map.as_object().expect("i18n file is a top-level object") {
        if !(id.starts_with("virtue.") || id.starts_with("flaw.")) {
            continue;
        }
        for field in ["name", "name_unfilled"] {
            if let Some(text) = entry[field].as_str() {
                out.insert((id.clone(), field), text.to_string());
            }
        }
    }
    out
}

/// Names that still use a retired parenthesised magnitude.
fn retired_spellings<'a>(names: impl Iterator<Item = &'a String>, locale: &Locale) -> Vec<String> {
    names
        .filter(|n| locale.retired.iter().any(|r| n.contains(r)))
        .cloned()
        .collect()
}

/// Bracketed names whose sibling (the same name with the other bracket) is
/// missing from `names`.
fn orphans(names: &BTreeSet<String>, locale: &Locale) -> Vec<String> {
    names
        .iter()
        .filter_map(|n| {
            let sibling = if n.contains(locale.minor) {
                n.replace(locale.minor, locale.major)
            } else if n.contains(locale.major) {
                n.replace(locale.major, locale.minor)
            } else {
                return None;
            };
            (!names.contains(&sibling)).then(|| format!("\"{n}\" has no sibling \"{sibling}\""))
        })
        .collect()
}

/// The bracket a name carries, as the core `magnitude` value it stands for.
fn bracket_magnitude(name: &str, locale: &Locale) -> Option<&'static str> {
    if name.contains(locale.minor) {
        Some("minor")
    } else if name.contains(locale.major) {
        Some("major")
    } else {
        None
    }
}

fn core_magnitudes() -> BTreeMap<String, String> {
    let all: Value = serde_json::from_str(VF_JSON).expect("virtues_flaws.json is valid JSON");
    all.as_array()
        .expect("virtues_flaws.json is a top-level array")
        .iter()
        .map(|v| {
            (
                v["id"].as_str().expect("entry has an id").to_string(),
                v["magnitude"]
                    .as_str()
                    .expect("entry has a magnitude")
                    .to_string(),
            )
        })
        .collect()
}

#[test]
fn no_vf_name_uses_a_parenthesised_magnitude() {
    for locale in [&EN, &DE] {
        let names = vf_names(locale.json);
        let offenders = retired_spellings(names.values(), locale);
        assert!(
            offenders.is_empty(),
            "{}: {} V/F names still carry the magnitude in parentheses; a split \
             'Minor or Major' entry shows it in brackets ({} / {}) (D84.3):\n{}",
            locale.label,
            offenders.len(),
            locale.minor,
            locale.major,
            offenders.join("\n")
        );
    }
}

#[test]
fn every_bracketed_magnitude_name_has_its_sibling() {
    for locale in [&EN, &DE] {
        let names: BTreeSet<String> = vf_names(locale.json).into_values().collect();
        assert!(
            names.iter().any(|n| n.contains(locale.minor)),
            "{}: no V/F name carries {} — the bracket convention (D84.3) is not applied",
            locale.label,
            locale.minor
        );
        let missing = orphans(&names, locale);
        assert!(
            missing.is_empty(),
            "{}: a bracketed magnitude marks one half of a split entry, so its \
             other half must exist:\n{}",
            locale.label,
            missing.join("\n")
        );
    }
}

#[test]
fn bracket_agrees_across_locales_and_with_core_magnitude() {
    let magnitudes = core_magnitudes();
    let en = vf_names(EN_I18N);
    let de = vf_names(DE_I18N);
    let mut failures = Vec::new();
    for (key, en_name) in &en {
        let (id, field) = key;
        let en_bracket = bracket_magnitude(en_name, &EN);
        let de_name = de.get(key).map(String::as_str).unwrap_or("");
        let de_bracket = bracket_magnitude(de_name, &DE);
        if en_bracket != de_bracket {
            failures.push(format!(
                "{id}.{field}: EN \"{en_name}\" and DE \"{de_name}\" disagree on the bracket"
            ));
        }
        let Some(bracket) = en_bracket else { continue };
        let core = magnitudes.get(id).map(String::as_str);
        if core != Some(bracket) {
            failures.push(format!(
                "{id}.{field}: name \"{en_name}\" says {bracket}, core magnitude is {core:?}"
            ));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

/// The two pattern helpers must actually catch what they look for — a
/// synthetic list with a retired spelling and an orphan.
#[test]
fn self_test_helpers_catch_retired_spelling_and_orphan() {
    let retired = [
        "Ehrgeizig (Groß)".to_string(),
        "Ehrgeizig [Klein]".to_string(),
    ];
    assert_eq!(
        retired_spellings(retired.iter(), &DE),
        vec!["Ehrgeizig (Groß)".to_string()]
    );

    let names: BTreeSet<String> = [
        "Ambitious [Major]",
        "Ambitious [Minor]",
        "False Power [Minor]: {virtue}",
        "Minor Magical Focus",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect();
    assert_eq!(
        orphans(&names, &EN),
        vec![
            "\"False Power [Minor]: {virtue}\" has no sibling \"False Power [Major]: {virtue}\""
                .to_string()
        ]
    );
}
