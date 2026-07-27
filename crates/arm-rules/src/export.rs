//! Markdown export of a finished entity (M5.6).
//!
//! A **pure formatter**: `(&Entity, &LocalizedRuleset, &labels) → String`. It
//! implements no rules mechanic — it selects values the engine already computes
//! ([`crate::derived`], [`crate::effective`], [`crate::validation`]) and lays them
//! out as a GitHub-Flavored-Markdown document. Nothing is mutated, nothing is
//! persisted, and no file is touched: the caller owns IO.
//!
//! # The localization split
//!
//! Two different kinds of text meet in the document, and they come from two
//! different places:
//!
//! - **Item names** (Virtues, Abilities, Arts, spells, Houses, equipment) come from
//!   the rules i18n via [`LocalizedRuleset::display_name`]. An id with no entry
//!   falls back to the raw slug — reachable only for a foreign or hand-edited id,
//!   and better than dropping the row.
//! - **Document chrome** (section headings, field labels, column headers, yes/no
//!   markers) is **passed in** by the caller as a `key → text` map keyed by Fluent
//!   message name, so the engine hardcodes no user-facing string. A key missing from
//!   the map degrades to printing the key itself, mirroring the frontend's own
//!   fallback in `ui/src/lib/i18n.ts`.
//!
//! [`LABEL_KEYS`] declares every chrome key the formatter can ask for, so the
//! caller can assemble the map without reading this source. Only **argument-free**
//! Fluent keys are usable: the engine links no Fluent formatter, so a key whose
//! message interpolates `{ $arg }` can never be resolved here.
//!
//! # Entity kinds
//!
//! The formatter is entity-generic. Every section is omitted when its collection or
//! value is empty, so a **covenant** — which has selections (Boons/Hooks) and
//! free-text identity but no Characteristics, Abilities, Arts, spells, equipment or
//! magic — renders exactly the sections it has, with the character-only ones absent.
//! No section is gated on [`crate::types::EntityKind`]; emptiness does the work.
//!
//! # Determinism
//!
//! Output is byte-deterministic: every collection is iterated in `BTreeMap` order,
//! in the entity's own normalized `Vec` order, or in a fixed rules taxonomy's order
//! ([`Characteristic::ALL`], [`ArtType::ALL`]). Two consecutive renders of the same
//! entity are byte-identical.

use std::collections::BTreeMap;

use crate::characteristics::Characteristic;
use crate::effective::effective_characteristic_score;
use crate::ruleset::{LocalizedRuleset, Ruleset};
use crate::types::{Entity, Id};

/// Every document-chrome label key [`character_markdown`] can ask for, sorted and
/// duplicate-free. The caller resolves these against its own Fluent bundle and
/// hands the results in; a key absent from the map prints as the key itself.
///
/// The list is closed over the keys the formatter *names* and over every key it
/// composes from a fixed Rust taxonomy (`characteristic-<slug>` and friends), so a
/// new taxonomy variant cannot silently surface as a raw slug — a unit test walks
/// each family and asserts membership here.
///
/// One family is deliberately **outside** the list: `type-<profile id>`, the
/// character-type label in the subtitle. Profile ids are catalogue *data*, and
/// enumerating them here would bake the catalogue's size into code. The locales
/// already ship one key per shipped profile.
pub const LABEL_KEYS: &[&str] = &[
    "ability-score-label",
    "age-label",
    "apparent-age-label",
    "characteristic-com",
    "characteristic-description-label",
    "characteristic-dex",
    "characteristic-int",
    "characteristic-per",
    "characteristic-pre",
    "characteristic-qik",
    "characteristic-sta",
    "characteristic-str",
    "characteristics-title",
    "export-col-effective",
    "export-untitled",
    "house-label",
    "identity-birth-year",
    "identity-concept",
    "identity-covenant",
    "identity-description",
    "identity-gender",
    "identity-label",
    "identity-name",
    "identity-parens",
    "identity-sigil",
];

/// Renders `entity` as a Markdown document.
///
/// `ruleset` supplies localized **item names**; `labels` supplies localized
/// **document chrome** keyed by the names in [`LABEL_KEYS`]. See the module docs for
/// the split and for the emptiness rule that governs which sections appear.
pub fn character_markdown(
    entity: &Entity,
    ruleset: &LocalizedRuleset,
    labels: &BTreeMap<String, String>,
) -> String {
    let doc = Doc {
        entity,
        ruleset,
        labels,
    };
    let mut out = String::new();
    doc.write_title(&mut out);
    doc.write_identity(&mut out);
    doc.write_characteristics(&mut out);
    out
}

/// The formatter's inputs, bundled so each section reads as one small method.
struct Doc<'a> {
    entity: &'a Entity,
    ruleset: &'a LocalizedRuleset,
    labels: &'a BTreeMap<String, String>,
}

impl<'a> Doc<'a> {
    /// The language-neutral ruleset behind the localized one.
    fn rules(&self) -> &'a Ruleset {
        &self.ruleset.ruleset
    }

    /// The localized chrome text for `key`, or the key itself when the caller did
    /// not supply it (the frontend's own fallback behavior).
    fn label(&self, key: &str) -> String {
        self.labels
            .get(key)
            .cloned()
            .unwrap_or_else(|| key.to_string())
    }

    /// The localized display name for a catalogue id, falling back to the raw slug
    /// so a foreign id costs a readable label rather than a whole row.
    fn name(&self, id: &Id) -> String {
        self.ruleset
            .display_name(id)
            .unwrap_or_else(|| id.as_str())
            .to_string()
    }

    // --- Sections ----------------------------------------------------------

    /// The `# ` title and the subtitle line (character type, House, ages).
    fn write_title(&self, out: &mut String) {
        let title = if self.entity.name.trim().is_empty() {
            self.label("export-untitled")
        } else {
            escape_cell(&self.entity.name)
        };
        out.push_str("# ");
        out.push_str(&title);
        out.push_str("\n\n");

        let mut parts: Vec<String> = Vec::new();
        // The type label is composed from the profile id — catalogue data, so it is
        // the one key family LABEL_KEYS does not enumerate (see its docs).
        parts.push(self.label(&format!("type-{}", self.entity.type_id)));
        if let Some(house) = &self.entity.house {
            parts.push(pair(&self.label("house-label"), &self.name(house)));
        }
        if let Some(age) = self.entity.age {
            parts.push(pair(&self.label("age-label"), &age.to_string()));
        }
        if let Some(age) = self.entity.apparent_age {
            parts.push(pair(&self.label("apparent-age-label"), &age.to_string()));
        }
        out.push('*');
        out.push_str(&parts.join(SUBTITLE_SEPARATOR));
        out.push_str("*\n\n");
    }

    /// The free-text identity fields, each omitted when empty.
    fn write_identity(&self, out: &mut String) {
        let e = self.entity;
        let mut body = String::new();
        for (key, value) in [
            ("identity-description", e.description.as_str()),
            ("identity-concept", e.concept.as_str()),
            ("identity-gender", e.gender.as_str()),
            ("identity-sigil", e.sigil.as_str()),
            ("identity-covenant", e.covenant_name.as_str()),
            ("identity-parens", e.parens.as_str()),
        ] {
            if !value.trim().is_empty() {
                field(&mut body, &self.label(key), &escape_cell(value));
            }
        }
        if let Some(year) = e.birth_year {
            field(
                &mut body,
                &self.label("identity-birth-year"),
                &year.to_string(),
            );
        }
        if body.is_empty() {
            return;
        }
        heading(out, 2, &self.label("identity-label"));
        out.push_str(&body);
        out.push('\n');
    }

    /// The bought Characteristic scores, their effective values where a Virtue moves
    /// them, and the sheet's free-text descriptions.
    fn write_characteristics(&self, out: &mut String) {
        let e = self.entity;
        let mut rows: Vec<Vec<String>> = Vec::new();
        for c in Characteristic::ALL {
            let bought = e.characteristics.get(&c).copied();
            let description = e
                .characteristic_descriptions
                .get(&c)
                .map(String::as_str)
                .unwrap_or_default();
            if bought.is_none() && description.trim().is_empty() {
                continue;
            }
            let bought = i32::from(bought.unwrap_or(0));
            let effective = effective_characteristic_score(e, self.rules(), c);
            rows.push(vec![
                self.label(&format!("characteristic-{c}")),
                signed(bought),
                if effective == bought {
                    String::new()
                } else {
                    signed(effective)
                },
                escape_cell(description),
            ]);
        }
        if rows.is_empty() {
            return;
        }
        heading(out, 2, &self.label("characteristics-title"));
        table(
            out,
            &[
                self.label("identity-name"),
                self.label("ability-score-label"),
                self.label("export-col-effective"),
                self.label("characteristic-description-label"),
            ],
            &rows,
        );
    }
}

// --- Formatting primitives -------------------------------------------------

/// Separator between the subtitle's parts. Punctuation, not prose — no locale owns
/// it.
const SUBTITLE_SEPARATOR: &str = " · ";

/// A signed modifier for display: `+` for positive, an ASCII hyphen-minus `-`
/// (U+002D) for negative, plain for zero. The engine-side twin of `formatSigned`
/// in `ui/src/lib/derive.ts` — the mathematical minus U+2212 is never used, so
/// values stay copy-paste clean and read correctly in assistive tech.
fn signed(n: i32) -> String {
    if n > 0 {
        format!("+{n}")
    } else {
        n.to_string()
    }
}

/// Makes an arbitrary free-text value safe to print inside the document.
///
/// Every value the formatter prints from the entity — the name, concept, an
/// Ability's specialty, a familiar's animal, an aging-log entry — is an
/// unconstrained `String` the user typed. Three characters would otherwise change
/// the document's *structure* rather than its content: a `|` ends a table cell, a
/// newline ends a table row (or the paragraph), and a leading `#` becomes a
/// heading. So every cell goes through here.
fn escape_cell(text: &str) -> String {
    let flattened: String = text
        .chars()
        .map(|c| if c == '\n' || c == '\r' { ' ' } else { c })
        .collect();
    // Backslash first: escaping it after adding our own would double-escape them.
    let escaped = flattened.replace('\\', "\\\\").replace('|', "\\|");
    let trimmed = escaped.trim();
    if trimmed.starts_with('#') {
        format!("\\{trimmed}")
    } else {
        trimmed.to_string()
    }
}

/// `label: value`, the inline pairing used in the subtitle and in read-out lines.
fn pair(label: &str, value: &str) -> String {
    format!("{label}: {value}")
}

/// Appends an ATX heading of `level` and the blank line after it.
fn heading(out: &mut String, level: usize, text: &str) {
    for _ in 0..level {
        out.push('#');
    }
    out.push(' ');
    out.push_str(text);
    out.push_str("\n\n");
}

/// Appends a `- **label**: value` bullet.
fn field(out: &mut String, label: &str, value: &str) {
    out.push_str("- **");
    out.push_str(label);
    out.push_str("**: ");
    out.push_str(value);
    out.push('\n');
}

/// Appends a GFM table and the blank line after it. A no-op when there are no
/// rows, so a section can never leave a headerless or bodyless table behind.
fn table(out: &mut String, headers: &[String], rows: &[Vec<String>]) {
    if rows.is_empty() {
        return;
    }
    out.push_str("| ");
    out.push_str(&headers.join(" | "));
    out.push_str(" |\n|");
    for _ in headers {
        out.push_str(" --- |");
    }
    out.push('\n');
    for row in rows {
        out.push_str("| ");
        out.push_str(&row.join(" | "));
        out.push_str(" |\n");
    }
    out.push('\n');
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ruleset::RulesetSources;
    use crate::types::{EntityKind, RulesetRef, Selection};
    use pretty_assertions::assert_eq;

    /// A small magus-capable ruleset with one Characteristic-moving Virtue.
    fn ruleset() -> LocalizedRuleset {
        let items = r#"[
          { "id": "virtue.giant_blood", "kind": "virtue", "classification": "creation_effect",
            "magnitude": "major", "category": "general", "entity_kinds": ["character"],
            "effects": [
              { "type": "characteristic_score_delta", "characteristic": "characteristic.str", "amount": 1 }
            ] },
          { "id": "boon.rich_vis_source", "kind": "boon", "classification": "narrative",
            "magnitude": "minor", "category": "general", "entity_kinds": ["covenant"] },
          { "id": "flaw.optimistic", "kind": "flaw", "classification": "narrative",
            "magnitude": "minor", "category": "personality", "entity_kinds": ["character"] }
        ]"#;
        let types = r#"[
          { "id": "magus", "is_magus": true,
            "budget": { "virtue_points": 10, "flaw_points": 10 },
            "permitted_categories": [], "forbidden_categories": [],
            "required_traits": [], "forbidden_traits": [],
            "confidence_score": 1, "confidence_points": 3,
            "gift_policy": "required", "gift_categories": [], "creation_phases": ["concept"] },
          { "id": "covenant",
            "budget": { "virtue_points": 10, "flaw_points": 10 },
            "permitted_categories": [], "forbidden_categories": [],
            "required_traits": [], "forbidden_traits": [], "creation_phases": ["concept"] }
        ]"#;
        let abilities = r#"{
          "advancement": [
            { "score": 1, "total_xp": 5 }, { "score": 2, "total_xp": 15 },
            { "score": 3, "total_xp": 30 }, { "score": 4, "total_xp": 50 },
            { "score": 5, "total_xp": 75 }
          ],
          "abilities": [
            { "id": "ability.awareness", "category": "general" },
            { "id": "ability.artes_liberales", "category": "academic" },
            { "id": "ability.magic_theory", "category": "arcane" },
            { "id": "ability.parma_magica", "category": "arcane" },
            { "id": "ability.penetration", "category": "arcane" },
            { "id": "ability.philosophiae", "category": "academic" }
          ]
        }"#;
        let rs = Ruleset::from_sources(RulesetSources {
            id: "arm5-core",
            version: "2024.1",
            point_items: items,
            type_profiles: types,
            abilities: Some(abilities),
            arts: None,
            houses: None,
            mythic_types: None,
            spells: None,
            spell_mastery_abilities: None,
            equipment: None,
            characteristics: None,
        })
        .unwrap();
        let i18n = r#"{
          "virtue.giant_blood": { "name": "Giant Blood" },
          "boon.rich_vis_source": { "name": "Rich Vis Source" },
          "ability.awareness": { "name": "Awareness" },
          "house.bonisagus": { "name": "Bonisagus" }
        }"#;
        LocalizedRuleset::new(rs, i18n).unwrap()
    }

    /// A label map that resolves every key to itself plus a marker, so a test can
    /// tell "the formatter asked for this key" from "this text came from the data".
    fn labels(pairs: &[(&str, &str)]) -> BTreeMap<String, String> {
        pairs
            .iter()
            .map(|(k, v)| ((*k).to_string(), (*v).to_string()))
            .collect()
    }

    fn no_labels() -> BTreeMap<String, String> {
        BTreeMap::new()
    }

    fn magus() -> Entity {
        Entity::new(
            EntityKind::Character,
            Id::new("magus"),
            RulesetRef::new(Id::new("arm5-core"), "2024.1"),
        )
    }

    fn covenant() -> Entity {
        Entity::new(
            EntityKind::Covenant,
            Id::new("covenant"),
            RulesetRef::new(Id::new("arm5-core"), "2024.1"),
        )
    }

    // --- escape_cell ------------------------------------------------------

    #[test]
    fn escape_cell_escapes_a_pipe_so_it_cannot_end_a_table_cell() {
        assert_eq!(escape_cell("a | b"), "a \\| b");
    }

    #[test]
    fn escape_cell_flattens_newlines_so_they_cannot_end_a_row() {
        assert_eq!(escape_cell("first\nsecond\r\nthird"), "first second  third");
    }

    #[test]
    fn escape_cell_escapes_a_leading_hash_so_it_cannot_inject_a_heading() {
        assert_eq!(escape_cell("# Not a heading"), "\\# Not a heading");
    }

    #[test]
    fn escape_cell_escapes_a_backslash_before_adding_its_own() {
        assert_eq!(escape_cell("back\\slash"), "back\\\\slash");
    }

    #[test]
    fn escape_cell_trims_surrounding_whitespace() {
        assert_eq!(escape_cell("  padded  "), "padded");
    }

    #[test]
    fn escape_cell_leaves_ordinary_text_alone() {
        assert_eq!(escape_cell("Bjornaer of Crintera"), "Bjornaer of Crintera");
    }

    // --- signed -----------------------------------------------------------

    #[test]
    fn signed_prefixes_a_plus_for_a_positive_value() {
        assert_eq!(signed(3), "+3");
    }

    #[test]
    fn signed_leaves_zero_unsigned() {
        assert_eq!(signed(0), "0");
    }

    #[test]
    fn signed_uses_an_ascii_hyphen_for_a_negative_value() {
        assert_eq!(signed(-2), "-2");
        assert!(!signed(-2).contains('\u{2212}'), "no mathematical minus");
    }

    // --- labels -----------------------------------------------------------

    #[test]
    fn a_missing_label_key_degrades_to_the_key_itself() {
        let mut e = magus();
        e.name = "Marcus".to_string();
        let doc = character_markdown(&e, &ruleset(), &no_labels());
        assert!(
            doc.contains("type-magus"),
            "the unresolved key prints verbatim: {doc}"
        );
    }

    #[test]
    fn label_keys_are_sorted_and_free_of_duplicates() {
        let mut sorted = LABEL_KEYS.to_vec();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(
            sorted.as_slice(),
            LABEL_KEYS,
            "LABEL_KEYS must be sorted and duplicate-free (canonical order)"
        );
    }

    // --- title ------------------------------------------------------------

    #[test]
    fn the_title_is_the_entity_name() {
        let mut e = magus();
        e.name = "Marcus of Bonisagus".to_string();
        let doc = character_markdown(&e, &ruleset(), &no_labels());
        assert!(
            doc.starts_with("# Marcus of Bonisagus\n"),
            "unexpected title: {doc}"
        );
    }

    #[test]
    fn an_unnamed_entity_uses_the_untitled_label_rather_than_a_bare_hash() {
        let doc = character_markdown(
            &magus(),
            &ruleset(),
            &labels(&[("export-untitled", "Untitled character")]),
        );
        assert!(
            doc.starts_with("# Untitled character\n"),
            "unexpected title: {doc}"
        );
    }

    #[test]
    fn the_subtitle_names_the_character_type_the_house_and_both_ages() {
        let mut e = magus();
        e.house = Some(Id::new("house.bonisagus"));
        e.age = Some(35);
        e.apparent_age = Some(30);
        let doc = character_markdown(
            &e,
            &ruleset(),
            &labels(&[
                ("type-magus", "Magus"),
                ("house-label", "House"),
                ("age-label", "Age"),
                ("apparent-age-label", "Apparent age"),
            ]),
        );
        assert!(
            doc.contains("*Magus · House: Bonisagus · Age: 35 · Apparent age: 30*"),
            "unexpected subtitle: {doc}"
        );
    }

    #[test]
    fn an_unknown_house_id_falls_back_to_its_slug() {
        let mut e = magus();
        e.house = Some(Id::new("house.diedne"));
        let doc = character_markdown(&e, &ruleset(), &no_labels());
        assert!(doc.contains("house.diedne"), "expected the slug: {doc}");
    }

    // --- identity ---------------------------------------------------------

    #[test]
    fn the_identity_section_lists_only_the_filled_fields() {
        let mut e = magus();
        e.concept = "A bookish theorist".to_string();
        e.birth_year = Some(1194);
        let doc = character_markdown(
            &e,
            &ruleset(),
            &labels(&[
                ("identity-label", "Identity"),
                ("identity-concept", "Concept"),
                ("identity-birth-year", "Birth year"),
                ("identity-gender", "Gender"),
            ]),
        );
        assert!(doc.contains("## Identity\n"), "missing section: {doc}");
        assert!(doc.contains("- **Concept**: A bookish theorist\n"), "{doc}");
        assert!(doc.contains("- **Birth year**: 1194\n"), "{doc}");
        assert!(!doc.contains("Gender"), "an empty field is omitted: {doc}");
    }

    #[test]
    fn the_identity_section_is_omitted_when_every_field_is_empty() {
        let doc = character_markdown(
            &magus(),
            &ruleset(),
            &labels(&[("identity-label", "Identity")]),
        );
        assert!(!doc.contains("## Identity"), "stray heading: {doc}");
    }

    #[test]
    fn free_text_identity_is_escaped() {
        let mut e = magus();
        e.description = "Knight | Crusader".to_string();
        let doc = character_markdown(
            &e,
            &ruleset(),
            &labels(&[("identity-description", "Short description")]),
        );
        assert!(doc.contains("Knight \\| Crusader"), "{doc}");
    }

    // --- characteristics --------------------------------------------------

    #[test]
    fn the_characteristics_table_lists_bought_scores_and_descriptions() {
        let mut e = magus();
        e.characteristics.insert(Characteristic::Int, 3);
        e.characteristic_descriptions
            .insert(Characteristic::Int, "quick-witted".to_string());
        let doc = character_markdown(
            &e,
            &ruleset(),
            &labels(&[
                ("characteristics-title", "Characteristics"),
                ("characteristic-int", "Intelligence"),
            ]),
        );
        assert!(doc.contains("## Characteristics\n"), "{doc}");
        assert!(
            doc.contains("| Intelligence | +3 |  | quick-witted |"),
            "unexpected row: {doc}"
        );
    }

    #[test]
    fn the_effective_column_is_filled_only_when_a_virtue_moves_the_score() {
        let mut e = magus();
        e.characteristics.insert(Characteristic::Str, 2);
        e.selections = vec![Selection::new(Id::new("virtue.giant_blood"))];
        let doc = character_markdown(
            &e,
            &ruleset(),
            &labels(&[("characteristic-str", "Strength")]),
        );
        assert!(
            doc.contains("| Strength | +2 | +3 |  |"),
            "unexpected row: {doc}"
        );
    }

    #[test]
    fn a_described_characteristic_with_no_bought_score_still_gets_a_row() {
        let mut e = magus();
        e.characteristic_descriptions
            .insert(Characteristic::Pre, "unremarkable".to_string());
        let doc = character_markdown(
            &e,
            &ruleset(),
            &labels(&[("characteristic-pre", "Presence")]),
        );
        assert!(doc.contains("| Presence | 0 |  | unremarkable |"), "{doc}");
    }

    #[test]
    fn the_characteristics_section_is_omitted_when_nothing_is_entered() {
        let doc = character_markdown(
            &magus(),
            &ruleset(),
            &labels(&[("characteristics-title", "Characteristics")]),
        );
        assert!(!doc.contains("## Characteristics"), "stray heading: {doc}");
    }

    // --- edge cases -------------------------------------------------------

    #[test]
    fn an_empty_entity_renders_a_minimal_document() {
        let doc = character_markdown(&magus(), &ruleset(), &no_labels());
        assert!(doc.starts_with("# export-untitled\n"), "{doc}");
        assert!(!doc.contains("##"), "no section headings: {doc}");
        assert!(!doc.contains("| --- |"), "no empty tables: {doc}");
    }

    #[test]
    fn a_covenant_omits_the_character_only_sections() {
        let mut e = covenant();
        e.name = "Semita Errabunda".to_string();
        e.description = "A covenant in the Rhine".to_string();
        let doc = character_markdown(
            &e,
            &ruleset(),
            &labels(&[
                ("identity-label", "Identity"),
                ("identity-description", "Short description"),
                ("characteristics-title", "Characteristics"),
            ]),
        );
        assert!(doc.contains("# Semita Errabunda\n"), "{doc}");
        assert!(doc.contains("## Identity\n"), "{doc}");
        assert!(
            !doc.contains("## Characteristics"),
            "a covenant has no Characteristics: {doc}"
        );
    }

    #[test]
    fn two_renders_of_the_same_entity_are_byte_identical() {
        let mut e = magus();
        e.name = "Marcus".to_string();
        e.characteristics.insert(Characteristic::Int, 3);
        e.characteristics.insert(Characteristic::Str, -1);
        let rs = ruleset();
        assert_eq!(
            character_markdown(&e, &rs, &no_labels()),
            character_markdown(&e, &rs, &no_labels())
        );
    }
}
