//! X2t (`docs/vf-audit/decisions.md` D60 point 3) — True Friend is a
//! first-class twin of True Love. ArMDE:5177 ("This Virtue may be renamed
//! 'True Friend' to cover equally close attachments which are not romantic")
//! and ArMDE:6875 ("This Story Hook may be renamed 'True Friend' to cover
//! characters with whom you are very closely linked, but not in a romantic
//! way") are the *only* two mentions of "True Friend" in the English core
//! rulebook (confirmed by grep against the full text) — there is no separate
//! `#### True Friend` heading in either language. So D60's premise holds
//! exactly: `virtue.true_friend_pc`, `flaw.true_friend_major` and
//! `flaw.true_friend_minor` are new ids that copy their twins'
//! (`virtue.true_love_pc`, `flaw.true_love_major`, `flaw.true_love_minor`)
//! mechanics, categories, classification, and source (same file, same lines,
//! same anchor — there is nothing else to cite), differing only in name:
//! "True Friend" (EN) / "Wahrer Freund" (DE, `tugenden-fehler.md:170`,
//! confirmed against the translation table per D31's precedence rule).
//!
//! GREEN (phase 2 data landed): all three ids are shipped in
//! `rules/core/virtues_flaws.json` and the two `rules/i18n/<lang>/
//! virtues_flaws.json` files, copying their twins' mechanics, categories,
//! classification and source as D60 requires — every test below passes
//! against the shipped catalogue. See `tmp/x2t-handover.md` for the data that
//! landed and the other guard lists
//! (`x4_incompatibilities.rs::ENTAILED_TWIN_PAIRS`,
//! `uncomputed_clauses.rs::NO_RULE_DESPITE_TOKEN`,
//! `data_integrity.rs::PENDING_D67_CLASSIFICATION`,
//! `rules/i18n/de/source_anchors.json`) that were updated alongside it.

use arm_rules::ruleset::{LocalizedRuleset, Ruleset, RulesetSources};
use arm_rules::types::*;
use arm_rules::validation::{ValidationResult, validate};

const SHIPPED_HOUSES: &str = include_str!("../../../rules/core/houses.json");
const EN_VF: &str = include_str!("../../../rules/i18n/en/virtues_flaws.json");
const DE_VF: &str = include_str!("../../../rules/i18n/de/virtues_flaws.json");

fn load_ruleset() -> Ruleset {
    Ruleset::from_sources(RulesetSources {
        id: "arm5-core",
        version: "2024.1",
        point_items: include_str!("../../../rules/core/virtues_flaws.json"),
        type_profiles: include_str!("../../../rules/core/character_types.json"),
        abilities: Some(include_str!("../../../rules/core/abilities.json")),
        houses: Some(SHIPPED_HOUSES),
        characteristics: Some(include_str!("../../../rules/core/characteristics.json")),
        life_stages: Some(include_str!("../../../rules/core/life_stages.json")),
        parameter_catalogues: Some(include_str!(
            "../../../rules/core/parameter_catalogues.json"
        )),
        ..RulesetSources::default()
    })
    .unwrap()
}

/// Same shape as `x4_incompatibilities.rs::entity` — duplicated since
/// integration test binaries cannot share private helpers.
fn entity(type_id: &str, selections: Vec<Selection>) -> Entity {
    let mut e = Entity::new(
        EntityKind::Character,
        Id::new(type_id),
        RulesetRef::new(Id::new("arm5-core"), "2024.1"),
    );
    e.selections = selections;
    e
}

fn sel(id: &str) -> Selection {
    Selection::new(Id::new(id))
}

fn issue_codes(result: &ValidationResult) -> Vec<&str> {
    result.issues.iter().map(|i| i.code.as_str()).collect()
}

/// The displayed rules text for `id` under `loc` — `description` if present,
/// else `summary`, same precedence `x2_reclassification.rs::displayed_text`
/// uses.
fn displayed_text<'a>(loc: &'a LocalizedRuleset, id: &str) -> Option<&'a str> {
    let id = Id::new(id);
    loc.description(&id).or_else(|| loc.summary(&id))
}

fn item<'a>(rs: &'a Ruleset, id: &str) -> &'a PointItem {
    rs.item(&Id::new(id))
        .unwrap_or_else(|| panic!("{id} must exist in the shipped catalogue (D60 point 3, X2t)"))
}

#[test]
fn true_friend_ids_exist_in_the_shipped_catalogue() {
    let rs = load_ruleset();
    for id in [
        "virtue.true_friend_pc",
        "flaw.true_friend_major",
        "flaw.true_friend_minor",
    ] {
        assert!(
            rs.item(&Id::new(id)).is_some(),
            "{id} must exist (D60 point 3): True Friend is a first-class twin of True Love, \
             not just a rename of the True Love entry"
        );
    }
}

#[test]
fn flaw_true_friend_major_copies_true_love_majors_mechanics() {
    let rs = load_ruleset();
    let twin = item(&rs, "flaw.true_love_major");
    let new = item(&rs, "flaw.true_friend_major");

    assert_eq!(
        new.kind, twin.kind,
        "flaw.true_friend_major must be a Flaw, like its twin"
    );
    assert_eq!(
        new.magnitude, twin.magnitude,
        "D60: True Friend [Major] copies True Love [Major]'s magnitude"
    );
    assert_eq!(
        new.categories, twin.categories,
        "D60: True Friend [Major] copies True Love [Major]'s categories"
    );
    assert_eq!(
        new.classification, twin.classification,
        "D60: True Friend [Major] copies True Love [Major]'s classification (narrative)"
    );
    assert_eq!(
        new.entity_kinds, twin.entity_kinds,
        "D60: True Friend [Major] copies True Love [Major]'s entity_kinds"
    );
    assert_eq!(
        new.source, twin.source,
        "D60: True Friend [Major] cites the SAME source (file, lines, anchor) as True Love \
         [Major] — there is no separate '#### True Friend' heading, only a rename sentence \
         inside True Love's own passage (ArMDE:6875)"
    );
}

#[test]
fn flaw_true_friend_minor_copies_true_love_minors_mechanics() {
    let rs = load_ruleset();
    let twin = item(&rs, "flaw.true_love_minor");
    let new = item(&rs, "flaw.true_friend_minor");

    assert_eq!(new.kind, twin.kind);
    assert_eq!(
        new.magnitude, twin.magnitude,
        "D60: True Friend [Minor] copies True Love [Minor]'s magnitude"
    );
    assert_eq!(new.categories, twin.categories);
    assert_eq!(
        new.classification, twin.classification,
        "D60: True Friend [Minor] copies True Love [Minor]'s classification (narrative)"
    );
    assert_eq!(new.entity_kinds, twin.entity_kinds);
    assert_eq!(
        new.source, twin.source,
        "D60: True Friend [Minor] cites the SAME source as True Love [Minor]"
    );
}

#[test]
fn virtue_true_friend_pc_copies_true_love_pcs_mechanics() {
    let rs = load_ruleset();
    let twin = item(&rs, "virtue.true_love_pc");
    let new = item(&rs, "virtue.true_friend_pc");

    assert_eq!(
        new.kind, twin.kind,
        "virtue.true_friend_pc must be a Virtue, like its twin"
    );
    assert_eq!(
        new.magnitude, twin.magnitude,
        "D60: True Friend (PC) copies True Love (PC)'s magnitude (minor)"
    );
    assert_eq!(new.categories, twin.categories);
    assert_eq!(
        new.classification, twin.classification,
        "D60: True Friend (PC) copies True Love (PC)'s classification (uncomputed_rule)"
    );
    assert_eq!(new.entity_kinds, twin.entity_kinds);
    assert_eq!(
        new.prerequisites, twin.prerequisites,
        "True Love (PC) carries no prerequisites; its twin must not invent one"
    );
    assert_eq!(
        new.source, twin.source,
        "D60: True Friend (PC) cites the SAME source as True Love (PC) (ArMDE:5173-5178, the \
         rename sentence is ArMDE:5177, inside True Love (PC)'s own passage)"
    );
}

/// D60: "the major/minor exclusion is mirrored" — ArMDE:6877 states alternate,
/// mutually exclusive competence levels for the SAME named NPC, exactly the
/// D44-entailment shape `x4_incompatibilities.rs::ENTAILED_TWIN_PAIRS` already
/// pins for `flaw.true_love_major`/`_minor`. True Friend cites the identical
/// passage, so the same entailment applies and this stays a hard block.
#[test]
fn flaw_true_friend_major_and_minor_exclude_each_other() {
    let rs = load_ruleset();
    let major = item(&rs, "flaw.true_friend_major");
    let minor = item(&rs, "flaw.true_friend_minor");

    assert!(
        major
            .incompatible_with
            .contains(&Id::new("flaw.true_friend_minor")),
        "flaw.true_friend_major must declare incompatible_with flaw.true_friend_minor, mirroring \
         True Love's own mutual exclusion"
    );
    assert!(
        minor
            .incompatible_with
            .contains(&Id::new("flaw.true_friend_major")),
        "flaw.true_friend_minor must declare incompatible_with flaw.true_friend_major \
         (symmetric — CLAUDE.md: incompatibilities must be symmetric)"
    );

    let e = entity(
        "companion",
        vec![sel("flaw.true_friend_major"), sel("flaw.true_friend_minor")],
    );
    let result = validate(&e, &rs);
    assert!(
        issue_codes(&result).contains(&"incompatible"),
        "holding both True Friend [Major] and [Minor] must be a hard block, like True Love's \
         own pair, got: {:?}",
        issue_codes(&result)
    );
}

/// D60: "The book states no incompatibility with True Love, so none is
/// added." Neither direction, and validating both together raises no
/// `incompatible` issue.
#[test]
fn flaw_true_friend_carries_no_incompatibility_with_true_love() {
    let rs = load_ruleset();
    let true_friend_major = item(&rs, "flaw.true_friend_major");
    let true_friend_minor = item(&rs, "flaw.true_friend_minor");
    let true_love_major = item(&rs, "flaw.true_love_major");
    let true_love_minor = item(&rs, "flaw.true_love_minor");

    for (id, set) in [
        (
            "flaw.true_friend_major",
            &true_friend_major.incompatible_with,
        ),
        (
            "flaw.true_friend_minor",
            &true_friend_minor.incompatible_with,
        ),
    ] {
        assert!(
            !set.contains(&Id::new("flaw.true_love_major"))
                && !set.contains(&Id::new("flaw.true_love_minor")),
            "{id} must NOT declare incompatible_with either True Love entry (D60: the book \
             states no such incompatibility)"
        );
    }
    for (id, set) in [
        ("flaw.true_love_major", &true_love_major.incompatible_with),
        ("flaw.true_love_minor", &true_love_minor.incompatible_with),
    ] {
        assert!(
            !set.contains(&Id::new("flaw.true_friend_major"))
                && !set.contains(&Id::new("flaw.true_friend_minor")),
            "{id} must NOT gain an incompatible_with pointing at True Friend either \
             (symmetric absence)"
        );
    }

    let e = entity(
        "companion",
        vec![sel("flaw.true_love_major"), sel("flaw.true_friend_minor")],
    );
    let result = validate(&e, &rs);
    assert!(
        !issue_codes(&result).contains(&"incompatible"),
        "True Love [Major] + True Friend [Minor] together must not be blocked as incompatible, \
         got: {:?}",
        issue_codes(&result)
    );
}

/// D60: the name differs, everything else is a copy. EN "True Love" ->
/// "True Friend"; DE "Wahre Liebe" -> "Wahrer Freund"
/// (`tugenden-fehler.md:170`, D31). The [Major]/[Minor]/(PC) qualifier stays,
/// matching the twin's own naming convention exactly.
#[test]
fn en_de_names_follow_the_true_love_to_true_friend_naming_convention() {
    let rs = load_ruleset();
    let loc_en = LocalizedRuleset::new(rs.clone(), EN_VF).unwrap();
    let loc_de = LocalizedRuleset::new(rs.clone(), DE_VF).unwrap();

    let cases: &[(&str, &str, &str, &str)] = &[
        (
            "flaw.true_love_major",
            "flaw.true_friend_major",
            "True Love",
            "True Friend",
        ),
        (
            "flaw.true_love_minor",
            "flaw.true_friend_minor",
            "True Love",
            "True Friend",
        ),
        (
            "virtue.true_love_pc",
            "virtue.true_friend_pc",
            "True Love",
            "True Friend",
        ),
    ];
    for (twin_id, new_id, en_from, en_to) in cases {
        let twin_en = loc_en
            .display_name(&Id::new(*twin_id))
            .unwrap_or_else(|| panic!("{twin_id} must carry an EN display name"));
        let expected_en = twin_en.replace(en_from, en_to);
        let got_en = loc_en
            .display_name(&Id::new(*new_id))
            .unwrap_or_else(|| panic!("{new_id} must carry an EN display name"));
        assert_eq!(
            got_en, expected_en,
            "{new_id}'s EN name must be {twin_id}'s EN name (\"{twin_en}\") with \"{en_from}\" \
             renamed to \"{en_to}\""
        );

        let twin_de = loc_de
            .display_name(&Id::new(*twin_id))
            .unwrap_or_else(|| panic!("{twin_id} must carry a DE display name"));
        let expected_de = twin_de.replace("Wahre Liebe", "Wahrer Freund");
        let got_de = loc_de
            .display_name(&Id::new(*new_id))
            .unwrap_or_else(|| panic!("{new_id} must carry a DE display name"));
        assert_eq!(
            got_de, expected_de,
            "{new_id}'s DE name must be {twin_id}'s DE name (\"{twin_de}\") with \"Wahre Liebe\" \
             renamed to \"Wahrer Freund\" (tugenden-fehler.md:170, D31)"
        );
    }
}

/// D60: "copy their twins' mechanics" — the displayed rules text (summary for
/// the narrative Flaws, description for the uncomputed_rule Virtue) is the
/// SAME cited passage, so it is copied verbatim, in both locales. Only the
/// name differs (checked above); the body text does not, because there is no
/// separate rulebook passage for True Friend to draw a different text from.
#[test]
fn en_de_summary_and_description_are_copied_verbatim_from_the_true_love_twin() {
    let rs = load_ruleset();
    let loc_en = LocalizedRuleset::new(rs.clone(), EN_VF).unwrap();
    let loc_de = LocalizedRuleset::new(rs.clone(), DE_VF).unwrap();

    for (twin_id, new_id) in [
        ("flaw.true_love_major", "flaw.true_friend_major"),
        ("flaw.true_love_minor", "flaw.true_friend_minor"),
        ("virtue.true_love_pc", "virtue.true_friend_pc"),
    ] {
        for (loc, lang) in [(&loc_en, "en"), (&loc_de, "de")] {
            let twin_text = displayed_text(loc, twin_id)
                .unwrap_or_else(|| panic!("{twin_id} must carry displayed text in {lang}"));
            let new_text = displayed_text(loc, new_id)
                .unwrap_or_else(|| panic!("{new_id} must carry displayed text in {lang}"));
            assert_eq!(
                new_text, twin_text,
                "{new_id}'s {lang} displayed text must be copied verbatim from {twin_id} — \
                 same cited passage, D60"
            );
        }
    }
}
