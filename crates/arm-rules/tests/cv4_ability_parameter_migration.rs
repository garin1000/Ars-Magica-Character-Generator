//! CV4 (`docs/vf-audit/design-cv-catalogued-values.md`): `AbilityScore.parameter`
//! widens from a bare `Option<String>` to `Option<AbilityParameterValue>` (D14
//! fix). § 5.1's raw pre-pass (bare string → `{"text": …}`), § 5.3's
//! catalogue-name-matching fold, and § 4 rule 1's Literal-only (`Catalogued`,
//! never `Text`) matching are all implemented and green below.

use arm_rules::migration::load_entity_migrating;
use arm_rules::ruleset::{Ruleset, RulesetSources};
use arm_rules::types::{
    AbilityParameterValue, AbilityScore, Entity, EntityKind, Id, RulesetRef, Selection,
};
use arm_rules::{DEFAULT_SAGA_YEAR, checked_xp_allocation};
use std::collections::BTreeMap;

/// The shipped core ruleset, loaded exactly as `book_templates.rs` does.
fn full_ruleset() -> Ruleset {
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
        spell_mastery_abilities: None,
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

/// The shipped catalogue names, both locales — CV4's dependency for
/// `load_entity_migrating` (design § 5.6).
fn catalogue_names(ruleset: &Ruleset) -> BTreeMap<Id, Vec<String>> {
    let en = include_str!("../../../rules/i18n/en/parameter_catalogue.json");
    let de = include_str!("../../../rules/i18n/de/parameter_catalogue.json");
    arm_rules::load_catalogue_names(ruleset.parameter_catalogues(), en, de)
        .expect("catalogue names load")
}

/// A minimal companion save, one `ability.dead_language` row, `parameter` spliced
/// in verbatim (so a caller can pass a bare string, an already-tagged object, or
/// a hostile shape).
fn companion_with_dead_language_parameter(schema_version: u32, parameter_json: &str) -> String {
    format!(
        r#"{{
          "schema_version": {schema_version},
          "ruleset": {{ "id": "arm5-core", "version": "2024.1" }},
          "entity_kind": "character",
          "type_id": "companion",
          "ability_scores": [
            {{ "ability": "ability.dead_language", "score": 1, "parameter": {parameter_json} }}
          ]
        }}"#
    )
}

/// The design note's own first-red pin (§ 10's CV4 row): the real shipped
/// `examples/magus_sample.json` (schema 16, bare-string `"parameter": "Latin"`)
/// loads through the real path and resolves to `Catalogued{"language.latin"}` —
/// not a fresh hand-written value.
#[test]
fn magus_sample_json_latin_resolves_to_the_catalogue_id() {
    let ruleset = full_ruleset();
    let names = catalogue_names(&ruleset);
    let json = include_str!("../../../examples/magus_sample.json");
    let loaded = load_entity_migrating(json, DEFAULT_SAGA_YEAR, &ruleset, &names)
        .expect("the shipped magus sample loads");

    let dead_language = loaded
        .entity
        .ability_scores
        .iter()
        .find(|s| s.ability == Id::new("ability.dead_language"))
        .expect("the sample carries a Dead Language row");
    assert_eq!(
        dead_language.parameter,
        Some(AbilityParameterValue::Catalogued {
            id: Id::new("language.latin")
        }),
        "the human-typed \"Latin\" must resolve to the catalogue id, not stay free text"
    );
}

/// "Latin", "latin", and " Latein " (German, mixed case, padded) all name the
/// same catalogue value in one locale or the other, trimmed and case-folded
/// (design § 5.3).
#[test]
fn latin_name_variants_all_resolve_to_the_same_catalogue_id() {
    let ruleset = full_ruleset();
    let names = catalogue_names(&ruleset);
    for typed in ["Latin", "latin", " Latein "] {
        let json = companion_with_dead_language_parameter(18, &format!("\"{typed}\""));
        let loaded = load_entity_migrating(&json, DEFAULT_SAGA_YEAR, &ruleset, &names)
            .unwrap_or_else(|e| panic!("{typed:?} loads: {e}"));
        assert_eq!(
            loaded.entity.ability_scores[0].parameter,
            Some(AbilityParameterValue::Catalogued {
                id: Id::new("language.latin")
            }),
            "{typed:?} must resolve to language.latin"
        );
    }
}

/// A value that spells out no catalogue entry's name in either locale is never
/// guessed at: it stays `Text`, unchanged, and is reported so the player is told
/// rather than left with a silently-unrecognized value (design § 5.3/§ 5.5).
#[test]
fn an_unmatched_free_text_value_stays_text_and_is_reported() {
    let ruleset = full_ruleset();
    let names = catalogue_names(&ruleset);
    let json = companion_with_dead_language_parameter(18, "\"Klingon\"");
    let loaded = load_entity_migrating(&json, DEFAULT_SAGA_YEAR, &ruleset, &names)
        .expect("an unmatched value still loads");

    assert_eq!(
        loaded.entity.ability_scores[0].parameter,
        Some(AbilityParameterValue::text("Klingon")),
        "an unmatched value must stay free text, never guessed"
    );
    assert!(
        loaded
            .unresolved_catalogued_parameters
            .contains(&(Id::new("ability.dead_language"), "Klingon".to_string())),
        "the unmatched value must be reported so the player is told, not left \
         silently unrecognized: {:?}",
        loaded.unresolved_catalogued_parameters
    );
}

/// § 3.3's hostile-input rejection: a value naming keys from more than one
/// `AbilityParameterValue` variant at once is not a shape any writer of this
/// format produces. `deny_unknown_fields` makes every variant's parse attempt
/// fail, so the untagged enum as a whole fails with a clear "no matching
/// variant" error — the whole load fails, rather than silently matching the
/// first variant whose subset of fields fits and dropping the rest.
///
/// Uses `schema_version: 17` (not 18) deliberately: an out-of-range claimed
/// version would itself refuse the load before the parse is even reached,
/// masking exactly the behavior this test exists to pin.
#[test]
fn a_multi_variant_parameter_value_is_rejected_not_misread() {
    let ruleset = full_ruleset();
    let names = catalogue_names(&ruleset);
    let json = companion_with_dead_language_parameter(
        17,
        r#"{ "id": "language.latin", "item": "virtue.craft_guild_training", "param": "guild" }"#,
    );
    let err = load_entity_migrating(&json, DEFAULT_SAGA_YEAR, &ruleset, &names)
        .expect_err("a value naming more than one variant's keys must be rejected");
    // A parse failure, never a panic — the whole point of `deny_unknown_fields`
    // over silently matching `Catalogued` and dropping `item`/`param`. Asserted
    // on the message content, not just "any Err", so this cannot be satisfied by
    // an unrelated rejection (e.g. a future-schema refusal) instead.
    assert!(
        err.to_string().contains("did not match any variant"),
        "must be rejected as an unrecognised shape, not some other error: {err}"
    );
}

/// § 5.1/§ 5.2: the raw pre-pass does not even read `schema_version` — a
/// hand-edited save can claim any version alongside a legacy bare-string
/// `parameter`, and it is still wrapped before the typed parse.
#[test]
fn a_bare_string_parameter_on_a_save_claiming_schema_18_is_still_wrapped() {
    let ruleset = full_ruleset();
    let names = catalogue_names(&ruleset);
    let json = companion_with_dead_language_parameter(18, "\"Klingon\"");
    let loaded = load_entity_migrating(&json, DEFAULT_SAGA_YEAR, &ruleset, &names)
        .expect("a bare string on a save claiming schema 18 must still load");
    assert_eq!(
        loaded.entity.ability_scores[0].parameter,
        Some(AbilityParameterValue::text("Klingon")),
        "the pre-pass must wrap a bare string regardless of the claimed schema_version"
    );
}

/// § 4: a `Literal` instance (`virtue.clan_ilfetu`'s pool, ArMDE:3565, funding
/// `ability.dead_language` at the `language.gothic` catalogue instance) is
/// satisfied ONLY by `Catalogued{id}`, never by `Text` holding the identical
/// letters — a literal can never mean "whichever spelling happens to match".
#[test]
fn a_literal_is_funded_by_catalogued_not_by_text_with_the_same_spelling() {
    let ruleset = full_ruleset();

    let funds = |parameter: AbilityParameterValue| -> bool {
        let mut entity = Entity::new(
            EntityKind::Character,
            Id::new("companion"),
            RulesetRef::new(Id::new("arm5-core"), "2024.1"),
        );
        entity.xp_pool = 0;
        entity.selections = vec![Selection::new(Id::new("virtue.clan_ilfetu"))];
        entity.ability_scores = vec![AbilityScore {
            ability: Id::new("ability.dead_language"),
            score: 1,
            specialty: None,
            parameter: Some(parameter),
            banked_xp: 0,
        }];
        let allocation = checked_xp_allocation(&entity, &ruleset).expect("solve stays in bounds");
        allocation.max_flow == allocation.total_demand
    };

    assert!(
        funds(AbilityParameterValue::Catalogued {
            id: Id::new("language.gothic")
        }),
        "Catalogued{{id: language.gothic}} must satisfy the Literal and be funded"
    );
    assert!(
        !funds(AbilityParameterValue::text("language.gothic")),
        "Text holding the identical letters must NOT satisfy a Literal"
    );
}
