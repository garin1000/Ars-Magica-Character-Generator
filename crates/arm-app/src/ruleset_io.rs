//! Pure, webview-free command logic. Every function here takes explicit inputs
//! (paths, entities, refs) and no Tauri `State`/`AppHandle`, so the integration
//! tests in `tests/commands.rs` exercise the real logic without a running app.
//! The thin `#[tauri::command]` shims in `commands.rs` only resolve the rules
//! directory and managed state, then delegate here.
//!
//! The IPC read-out DTO layer (`EffectiveScores` and the assemblers that build
//! it) lives in [`crate::effective_dto`] instead: this module keeps the
//! ruleset loader, the save/export path helpers, and the aging commands.

use std::fs;
use std::path::{Path, PathBuf};

use std::collections::BTreeMap;

use arm_rules::validation::{aging_error_issue, childhood_rejection_issues};
use arm_rules::{
    AgingError, AgingNote, AgingOutcome, AgingTotal, AgingYearRequest, Characteristic,
    CrisisPreview, Entity, EntityKind, Id, LocalizedRuleset, Ruleset, RulesetSources,
    ValidationIssue, ValidationMode, ValidationResult, aging_total, apply_childhood_package,
    resolve_outcome, resolve_year, revert_year, validate,
};
use serde::{Deserialize, Serialize};

use crate::atomic_write::write_file_atomically;
use crate::error::AppError;

/// The outcome of applying a Sample Childhood package: either the entity with the
/// package's rows written, or the reasons it could not be applied.
///
/// The reasons cross the IPC edge as ordinary [`ValidationIssue`]s so the frontend
/// renders them through the `issue-<code>` Fluent path it already has, and no
/// English prose ever crosses the boundary. That is also why a rejection is a
/// perfectly ordinary `Ok` outcome rather than an [`AppError`]: an unanswered slot
/// is a finding about the form the player just submitted, not a failure of the
/// command.
/// The entity is boxed so the two variants stay comparable in size (an `Entity` is
/// far larger than a list of issues); `Box<Entity>` serializes exactly as `Entity`
/// does, so the JSON the frontend sees is unaffected.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum ChildhoodApplication {
    Applied { entity: Box<Entity> },
    Rejected { issues: Vec<ValidationIssue> },
}

/// Applies the Sample Childhood package `package` names to `entity` against a
/// loaded ruleset, localizing the engine's rejections on the way out.
///
/// The engine owns every decision here ([`apply_childhood_package`]): what the
/// package writes, that the write is a monotone raise, and what makes it
/// impossible. This only chooses the shape the frontend receives.
pub fn apply_childhood_package_loaded(
    entity: &Entity,
    package: &Id,
    slot_values: &BTreeMap<String, String>,
    ruleset: &Ruleset,
) -> ChildhoodApplication {
    match apply_childhood_package(entity, package, slot_values, ruleset) {
        Ok(entity) => ChildhoodApplication::Applied {
            entity: Box::new(entity),
        },
        Err(rejections) => ChildhoodApplication::Rejected {
            issues: childhood_rejection_issues(&rejections, ruleset),
        },
    }
}

/// The outcome of previewing one aging roll: the reading, or the reason there was
/// none.
///
/// The three aging commands are shaped on [`ChildhoodApplication`] throughout: a
/// refusal is an ordinary `Ok` outcome carrying localizable
/// [`ValidationIssue`]s, never an [`AppError`], because a die typed against a year
/// already rolled is a finding about the form the player just submitted rather
/// than a failure of the command. [`arm_rules::AgingError`] is plain data with no
/// prose of its own, so `aging_error_issue` turns it into something the frontend
/// renders through the `issue-<code>` path it already has, and no English string
/// crosses the boundary.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum AgingProjection {
    Previewed {
        total: AgingTotal,
        outcome: AgingOutcome,
        /// The Crisis this year would send the character to, read whole and
        /// **written nowhere** — present only once the row demands one, the
        /// player has thrown the Simple Die, and the year is one
        /// [`arm_rules::resolve_year`] would accept. See
        /// [`aging_preview_loaded`] for why the last condition is not optional.
        ///
        /// Boxed to keep the two variants comparable in size;
        /// `Box<CrisisPreview>` serializes exactly as `CrisisPreview` does.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        crisis: Option<Box<CrisisPreview>>,
    },
    Rejected {
        issues: Vec<ValidationIssue>,
    },
}

/// The outcome of applying one aging roll: the character it made plus the reading
/// that made it, or the reason it was refused.
///
/// The reading comes back with the entity so the UI can confirm what it applied
/// without recomputing it against a character that has since changed. The entity
/// is boxed so the two variants stay comparable in size; `Box<Entity>` serializes
/// exactly as `Entity` does.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum AgingApplication {
    Applied {
        entity: Box<Entity>,
        total: AgingTotal,
        outcome: AgingOutcome,
        /// The Crisis the year sent the character to, read whole — present only
        /// when the row demanded one *and* the player had thrown the Simple Die.
        /// The log records the same figures, but only from here can the UI show
        /// what surviving it would take, which the log has no room for.
        ///
        /// Boxed for the reason [`Self::Applied::entity`] is — it keeps the two
        /// variants comparable in size — and `Box<CrisisPreview>` serializes
        /// exactly as `CrisisPreview` does.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        crisis: Option<Box<CrisisPreview>>,
        /// What the year changed that the character itself cannot show — today,
        /// only the Longevity Ritual a Crisis spends (`ArMDE:16573`). Empty for almost
        /// every year, and each variant is rendered through Fluent by the caller.
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        notes: Vec<AgingNote>,
    },
    Rejected {
        issues: Vec<ValidationIssue>,
    },
}

/// The outcome of taking one applied aging roll back off.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum AgingReversion {
    Reverted { entity: Box<Entity> },
    Rejected { issues: Vec<ValidationIssue> },
}

/// Reads one year's aging roll without writing anything: the AGING TOTAL the typed
/// `die` makes at `age`, and the row it lands on.
///
/// Read-only on purpose. The die is player input the entity must never store — the
/// engine has no `rand` dependency and a stress die explodes, so no lookup table
/// could stand in for this — which is why the calculator asks the engine instead of
/// keeping the number on the character.
///
/// # The Crisis rides here, and there is no second command
///
/// A Crisis is not a second question about a second thing: it exists only because
/// this year's row demanded one (`ArMDE:16602`, `ArMDE:16611`), and its total counts the
/// Decrepitude this very year raised. A command of its own would let the frontend
/// pair a CRISIS TOTAL with an aging outcome that no longer calls for one, which is
/// exactly the pairing `crisis_preview` was composed to take out of a caller's
/// hands.
///
/// # Why it resolves the year rather than reading the character
///
/// > **Crisis:** Increase the character's Decrepitude first, and then roll on the
/// > Crisis Table. (`ArMDE:16619`)
///
/// The Aging Points the row awards ARE that increase, so a Crisis read off the
/// character *standing in front of you* is one Decrepitude short of the one the
/// year writes — the player would be shown 14 and then watch the log record 15. So
/// this builds the very [`AgingYearRequest`] the Apply would send, resolves it in
/// memory, and keeps only the reading: same request, same answer, and the entity
/// it made is dropped on the spot. Nothing is written, and nothing can drift.
///
/// A year the engine would refuse — an unplaced distribution, a year already
/// recorded — yields no crisis reading, while the AGING TOTAL and the outcome still
/// stand. The player has to be told a Crisis follows and what to place *before* he
/// can place it, which is the order `ArMDE:16619` itself asks for.
///
/// Source: ArMDE:16567-16615, :16619.
pub fn aging_preview_loaded(
    entity: &Entity,
    ruleset: &Ruleset,
    age: u32,
    die: i32,
    distribution: &BTreeMap<Characteristic, u8>,
    crisis_die: Option<i32>,
) -> AgingProjection {
    let reading = aging_total(entity, ruleset, age, die)
        .and_then(|total| resolve_outcome(entity, ruleset, total.total).map(|out| (total, out)));
    match reading {
        Some((total, outcome)) => AgingProjection::Previewed {
            total,
            outcome,
            crisis: previewed_crisis(entity, ruleset, age, die, distribution, crisis_die),
        },
        // The one thing that can be missing is the aging block itself; every other
        // input is the character's own.
        None => AgingProjection::Rejected {
            issues: vec![aging_error_issue(&AgingError::NoAgingRules)],
        },
    }
}

/// The Crisis one previewed year would produce, read off the character the year
/// would make rather than the one it started from (`ArMDE:16619`).
///
/// The whole of it is [`resolve_year`] run for its reading alone: the entity it
/// returns is dropped, so this stays as read-only as the preview it serves while
/// remaining, by construction, the same answer the Apply will give. A refusal is
/// simply no reading — the calculator's own findings already say why.
///
/// Source: ArMDE:16619.
fn previewed_crisis(
    entity: &Entity,
    ruleset: &Ruleset,
    age: u32,
    die: i32,
    distribution: &BTreeMap<Characteristic, u8>,
    crisis_die: Option<i32>,
) -> Option<Box<CrisisPreview>> {
    let request = AgingYearRequest {
        age,
        die,
        distribution: distribution.clone(),
        crisis_die: Some(crisis_die?),
    };
    resolve_year(entity, ruleset, &request)
        .ok()?
        .crisis
        .map(Box::new)
}

/// Applies one year's aging roll, returning the character it makes.
///
/// The engine owns every decision ([`resolve_year`] is the aging subsystem's
/// single writer): what the row awards, where the points may go, whether the
/// apparent age advances, and what makes the year impossible. This only chooses
/// the shape the frontend receives.
///
/// `crisis_die` is the **Simple Die** the player threw at the Crisis Table
/// (`ArMDE:16621`), or `None` for a Crisis nobody has rolled yet — a legitimate state,
/// because the aging roll happened whether or not the second die was thrown. A die
/// given for a year the table sent to no Crisis is simply unused: whether a Crisis
/// happened is `ArMDE:16602`/`ArMDE:16611`'s call, never the player's.
///
/// Source: ArMDE:16567-16621.
pub fn aging_apply_loaded(
    entity: &Entity,
    ruleset: &Ruleset,
    age: u32,
    die: i32,
    distribution: &BTreeMap<Characteristic, u8>,
    crisis_die: Option<i32>,
) -> AgingApplication {
    let request = AgingYearRequest {
        age,
        die,
        distribution: distribution.clone(),
        crisis_die,
    };
    match resolve_year(entity, ruleset, &request) {
        Ok(result) => AgingApplication::Applied {
            entity: Box::new(result.entity),
            total: result.total,
            outcome: result.outcome,
            crisis: result.crisis.map(Box::new),
            notes: result.notes,
        },
        Err(error) => AgingApplication::Rejected {
            issues: vec![aging_error_issue(&error)],
        },
    }
}

/// Takes one applied aging year back off, exactly.
///
/// Addresses the year by `age`, which is what the log entry records; a
/// hand-written free-text entry carries none and is out of reach here by
/// construction (the ordinary log editor removes it instead).
///
/// Source: ArMDE:16577-16617.
pub fn aging_revert_loaded(entity: &Entity, ruleset: &Ruleset, age: u32) -> AgingReversion {
    match revert_year(entity, ruleset, age) {
        Ok(entity) => AgingReversion::Reverted {
            entity: Box::new(entity),
        },
        Err(error) => AgingReversion::Rejected {
            issues: vec![aging_error_issue(&error)],
        },
    }
}

/// File extension for a saved entity of the given kind: `armc` for characters,
/// `armcov` for covenants ("Ars Magica character/covenant"). They are plain JSON
/// underneath, but a distinct extension lets the OS associate and filter them.
pub fn entity_extension(kind: EntityKind) -> &'static str {
    match kind {
        EntityKind::Character => "armc",
        EntityKind::Covenant => "armcov",
    }
}

/// Extension-free base file name for a kind, shared by the save default and the
/// Markdown-export default so the two can never drift apart. Deliberately not the
/// entity's own `name`: that is arbitrary user text and would need per-platform
/// filename sanitization to be safe, for no gain — the dialog lets the user type
/// whatever name they want.
fn entity_base_name(kind: EntityKind) -> &'static str {
    match kind {
        EntityKind::Character => "character",
        EntityKind::Covenant => "covenant",
    }
}

/// Default save file name for a kind, e.g. `character.armc`.
pub fn default_file_name(kind: EntityKind) -> String {
    format!("{}.{}", entity_base_name(kind), entity_extension(kind))
}

/// Extension of an exported Markdown character sheet.
pub const MARKDOWN_EXTENSION: &str = "md";

/// Default file name for a Markdown export, e.g. `character.md`.
pub fn default_markdown_file_name(kind: EntityKind) -> String {
    format!("{}.{MARKDOWN_EXTENSION}", entity_base_name(kind))
}

/// Prefill file name for the Markdown-export dialog: the document's own save file
/// re-extensioned, so `gerhard.armc` exports as `gerhard.md`. Falls back to the
/// kind default for a document that was never saved, or a path with no usable base
/// name — the entity's `name` stays out of it for the reason
/// [`entity_base_name`] documents.
pub fn markdown_file_name_for(current_path: Option<&str>, kind: EntityKind) -> String {
    let base = current_path
        .and_then(|path| path.rsplit(['/', '\\']).next())
        .unwrap_or_default();
    // Split at the LAST dot, but only past the first byte: a base name starting
    // with the dot (`.armc`) is nothing but an extension and leaves no stem.
    let stem = match base.rfind('.') {
        Some(dot) if dot > 0 => &base[..dot],
        Some(_) => "",
        None => base,
    };
    if stem.is_empty() {
        return default_markdown_file_name(kind);
    }
    format!("{stem}.{MARKDOWN_EXTENSION}")
}

/// Directory the document's save file sits in, used as the export dialog's starting
/// folder so the export is offered next to the character file. `None` when the path
/// names no directory at all (a bare file name, or a document never saved).
pub fn save_file_directory(current_path: Option<&str>) -> Option<&str> {
    let path = current_path?;
    let separator = path.rfind(['/', '\\'])?;
    // The separator is kept, so a root-level file yields "/" rather than "".
    Some(&path[..=separator])
}

/// Appends `ext` when the chosen path has no extension, so a user who types just
/// "testchar" still gets "testchar.armc". An explicit extension (`.armc`,
/// `.json`, …) the user typed is respected.
pub fn ensure_extension(path: PathBuf, ext: &str) -> PathBuf {
    if path.extension().is_none() {
        path.with_extension(ext)
    } else {
        path
    }
}

/// The rules file each ruleset declares its own identity in. Slug-style
/// identifiers, not user-facing text, so no Fluent key is involved.
///
/// It is a file rather than a pair of `const`s in this binary (which is what it
/// was until full-audit V6) because a ruleset's identity is a property of the
/// **data**, not of the executable that read it. A `rules/` directory beside the
/// binary is a supported layout, so house-ruled data is loadable — and while the
/// id was compiled in, every character built against it was stamped with the
/// shipped ruleset's id and version, leaving the provenance a save records
/// (`types.rs::RulesetRef`) unable to tell the two apart.
const RULESET_IDENTITY_FILE: &str = "core/ruleset.json";

/// A ruleset's own declaration of which ruleset it is, as read from
/// [`RULESET_IDENTITY_FILE`]. Unknown fields are tolerated (a future ruleset may
/// declare more about itself); the two that must be there are checked below.
#[derive(Deserialize)]
struct RulesetIdentity {
    /// Stable ruleset id, e.g. `arm5-core`.
    id: String,
    /// Ruleset version string, e.g. `2024.1`.
    version: String,
}

/// Parses [`RULESET_IDENTITY_FILE`]'s contents, failing loudly — as a ruleset
/// **parse** failure naming the file, exactly like any other malformed rules
/// file — rather than falling back to an identity nothing declared. A blank id
/// or version counts as malformed: it would stamp every save with a provenance
/// that can never be compared against anything.
fn parse_ruleset_identity(json: &str) -> Result<RulesetIdentity, AppError> {
    let identity: RulesetIdentity =
        serde_json::from_str(json).map_err(|e| ruleset_identity_error(&e.to_string()))?;
    if identity.id.trim().is_empty() || identity.version.trim().is_empty() {
        return Err(ruleset_identity_error(
            "\"id\" and \"version\" must both be non-empty",
        ));
    }
    Ok(identity)
}

/// A parse-kind [`AppError::Ruleset`] naming [`RULESET_IDENTITY_FILE`], matching
/// the engine's own `"{source}: {message}"` diagnostic shape so the frontend and
/// the stderr listing treat it like every other rules-file failure.
fn ruleset_identity_error(message: &str) -> AppError {
    AppError::Ruleset {
        ruleset_kind: "parse".to_string(),
        errors: vec![format!("{RULESET_IDENTITY_FILE}: {message}")],
    }
}

/// Every core rules file a valid `rules/` directory must carry — exactly the
/// set [`load_ruleset_from_dir`] unconditionally reads via `fs::read_to_string`
/// regardless of language. An empty *file* is a legitimate "this ruleset ships
/// none of this subsystem" signal there (characteristics/life_stages/
/// childhoods/aging may all be `""`), but the file itself must still exist —
/// that is exactly the presence this list checks.
const REQUIRED_CORE_FILES: [&str; 14] = [
    RULESET_IDENTITY_FILE,
    "core/virtues_flaws.json",
    "core/character_types.json",
    "core/abilities.json",
    "core/arts.json",
    "core/houses.json",
    "core/mythic_companion_types.json",
    "core/spells.json",
    "core/spell_mastery_abilities.json",
    "core/equipment.json",
    "core/characteristics.json",
    "core/life_stages.json",
    "core/childhoods.json",
    "core/aging.json",
];

/// The [`REQUIRED_CORE_FILES`] that `dir` does NOT carry, in their fixed
/// order — empty when `dir` is a complete, valid rules directory (including
/// when `dir` does not exist at all, in which case every file is "missing").
/// Exposed so a caller can build an error message naming exactly what is
/// wrong with a rejected candidate (V9), rather than a bare "not found".
pub fn missing_core_files(dir: &Path) -> Vec<&'static str> {
    REQUIRED_CORE_FILES
        .iter()
        .filter(|file| !dir.join(file).is_file())
        .copied()
        .collect()
}

/// Given ordered candidate rules directories, returns the first one that
/// actually holds the rules data, or `None` when none do.
///
/// Tauri's `BaseDirectory::Resource` does not resolve to the executable's own
/// directory for a portable Linux build: `resource_dir` there falls back to a
/// system path (`/usr/lib/<name>`) that a portable extract never populates. So
/// the command offers both the resource path and the directory next to the
/// executable as candidates and lets this pick whichever is real.
///
/// A candidate qualifies when it carries every
/// [`REQUIRED_CORE_FILES`] entry (V9). A candidate missing even one — a
/// stale or partially-staged directory — is skipped rather than accepted on
/// the strength of a single file and left to fail later, deep inside
/// [`load_ruleset_from_dir`], with a raw "file not found" that names neither
/// the directory nor what was actually missing from it.
pub fn pick_rules_dir(candidates: &[PathBuf]) -> Option<PathBuf> {
    candidates
        .iter()
        .find(|dir| missing_core_files(dir).is_empty())
        .cloned()
}

/// Loads the shipped ruleset for `lang` from a rules directory laid out as
/// `core/*.json` + `i18n/<lang>/*.json`, parsing and integrity-checking it via
/// the engine. Returns the ruleset paired with localized display text.
pub fn load_ruleset_from_dir(rules_dir: &Path, lang: &str) -> Result<LocalizedRuleset, AppError> {
    // Array-driven, matching read_i18n_sources below: REQUIRED_CORE_FILES *is*
    // the ordered list of files this function reads (see its doc comment), so
    // reading it back destructures in exactly that fixed order rather than
    // repeating the filenames a second time. Deliberately no count here — the
    // array is the count, and the one below is checked by the compiler.
    let core: Vec<String> = REQUIRED_CORE_FILES
        .iter()
        .map(|file| fs::read_to_string(rules_dir.join(file)).map_err(AppError::from))
        .collect::<Result<_, _>>()?;
    let [
        identity_json,
        point_items_json,
        type_profiles_json,
        abilities_json,
        arts_json,
        houses_json,
        mythic_types_json,
        spells_json,
        spell_mastery_abilities_json,
        equipment_json,
        characteristics_json,
        life_stages_json,
        childhoods_json,
        aging_json,
    ]: [String; 14] = core
        .try_into()
        .expect("REQUIRED_CORE_FILES has exactly 14 entries");

    // The identity the DATA declares, never one this binary holds: see
    // [`RULESET_IDENTITY_FILE`].
    let identity = parse_ruleset_identity(&identity_json)?;

    let ruleset = Ruleset::from_sources(RulesetSources {
        id: &identity.id,
        version: &identity.version,
        point_items: &point_items_json,
        type_profiles: &type_profiles_json,
        abilities: Some(&abilities_json),
        arts: Some(&arts_json),
        houses: Some(&houses_json),
        mythic_types: Some(&mythic_types_json),
        spells: Some(&spells_json),
        spell_mastery_abilities: Some(&spell_mastery_abilities_json),
        equipment: Some(&equipment_json),
        // An empty characteristics file means the ruleset ships no characteristic
        // rules (the `Option` is the engine's honest "absent" signal).
        characteristics: (!characteristics_json.is_empty())
            .then_some(characteristics_json.as_str()),
        // Likewise: an empty life-stages file means the ruleset ships no life
        // stages, leaving `Entity::xp_pool` the only source of experience.
        life_stages: (!life_stages_json.is_empty()).then_some(life_stages_json.as_str()),
        // And likewise: an empty Sample Childhood file means the ruleset offers no
        // ready-made packages, leaving the childhood blocks to be divided by hand —
        // which the rules explicitly allow ("you can spend the 45 experience points
        // for yourself, as well",
        // ArMDE:2382).
        childhoods: (!childhoods_json.is_empty()).then_some(childhoods_json.as_str()),
        // And likewise: an empty aging file means the ruleset ships no aging
        // tables, which stands the aging subsystem down rather than letting the
        // engine invent a table.
        aging: (!aging_json.is_empty()).then_some(aging_json.as_str()),
    })?;
    // Load the requested language's rules text. For any non-English language,
    // English is loaded as a per-field fallback so a not-yet-translated string
    // (e.g. a missing German spell description) surfaces the English text rather
    // than rendering empty. English needs no fallback (it is the source of truth).
    let i18n = read_i18n_sources(rules_dir, lang)?;
    let i18n_refs: Vec<&str> = i18n.iter().map(String::as_str).collect();
    let localized = if lang == "en" {
        LocalizedRuleset::from_merged(ruleset, &i18n_refs)?
    } else {
        let fallback = read_i18n_sources(rules_dir, "en")?;
        let fallback_refs: Vec<&str> = fallback.iter().map(String::as_str).collect();
        LocalizedRuleset::from_merged_with_fallback(ruleset, &i18n_refs, &fallback_refs)?
    };
    Ok(localized)
}

/// Reads the ten `i18n/<lang>/*.json` rules-text files for a language, in the
/// stable domain order the localized ruleset merges them. A missing file is an
/// error (each language ships the full set), surfaced to the caller.
fn read_i18n_sources(rules_dir: &Path, lang: &str) -> Result<Vec<String>, AppError> {
    const FILES: [&str; 10] = [
        "virtues_flaws.json",
        "abilities.json",
        "arts.json",
        "houses.json",
        "mythic_companion_types.json",
        "spells.json",
        "spell_mastery_abilities.json",
        "equipment.json",
        "childhoods.json",
        "aging.json",
    ];
    let lang = validated_language_tag(lang)?;
    FILES
        .iter()
        .map(|file| {
            fs::read_to_string(rules_dir.join(format!("i18n/{lang}/{file}")))
                .map_err(AppError::from)
        })
        .collect()
}

/// `lang` back again, once it is a single path component and nothing else.
///
/// **This is the codebase's only string-to-path join fed by an unvalidated
/// string** (Klaus F5), and it has two sources: the `load_ruleset` command,
/// which takes a bare `String` over IPC, and `settings.json`, whose deliberately
/// lenient deserializer (`settings.rs::lenient`) accepts any string a
/// hand-edited file offers. `"../../.."` would resolve outside the rules
/// directory.
///
/// What it can actually reach is narrow — the read is read-only and the final
/// component is one of the ten fixed filenames above — so by this app's threat
/// model (`CLAUDE.md`: single local user, no remote attacker) the exposure is
/// near-inert. The guard is here because an unchecked join is worth closing
/// while it IS inert, and because [`read_i18n_sources`] is the one place both
/// sources meet: putting it in the command shim would leave the settings path
/// unchecked.
///
/// The shape, not a list of languages: a language directory is named by a
/// BCP-47-style tag (`en`, `de`, `pt-BR`), so ASCII alphanumerics plus `-` and
/// `_` is the whole alphabet. That rejects `/`, `\` and `.` — and with `.` gone
/// there is no `..` to climb with — while keeping which languages exist a
/// property of the rules data, never of this code.
fn validated_language_tag(lang: &str) -> Result<&str, AppError> {
    let is_tag = !lang.is_empty()
        && lang
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_');
    if is_tag {
        return Ok(lang);
    }
    Err(AppError::Io {
        message: format!("not a language tag: {lang}"),
    })
}

/// The path as the text the frontend will carry, or a failure naming it.
///
/// **Refusing is the point** (Gerda #5). A file name on Linux is an arbitrary
/// byte string, so a path chosen in the native dialog need not be UTF-8 — and
/// `to_string_lossy`, which this used to be, substitutes U+FFFD for every byte
/// that is not. The frontend adopts the result as `currentPath`, and every later
/// plain Save writes straight to it with no dialog: the user would edit for an
/// hour, press Ctrl+S, and have the app write to a path containing a literal
/// replacement character while the file they believe they updated still held the
/// old version. The dirty flag is cleared on that "successful" save, so the
/// close guard would not warn them either.
///
/// The honest fix is the cheap one: refuse at the boundary and say which path,
/// so the user is told rather than silently misdirected. The message is
/// technical detail (`AppError::Io`'s `message`), not a user-facing sentence —
/// the frontend renders the localized text for the variant, as it does for every
/// other IO failure.
pub fn path_text(path: &Path) -> Result<String, AppError> {
    match path.to_str() {
        Some(text) => Ok(text.to_owned()),
        // Lossy HERE is correct and nowhere else: this string is a diagnostic
        // for a human, never a path anything writes to.
        None => Err(AppError::Io {
            message: format!(
                "the chosen path is not valid UTF-8 and cannot be tracked as the \
                 current file: {}",
                path.to_string_lossy()
            ),
        }),
    }
}

/// Validates an entity against a loaded ruleset and applies the caller's mode
/// (Enforced keeps errors, Advisory downgrades them to warnings, Silent clears).
pub fn validate_loaded(
    entity: &Entity,
    ruleset: &Ruleset,
    mode: ValidationMode,
) -> ValidationResult {
    validate(entity, ruleset).apply_mode(mode)
}

/// Writes an entity to `path` as canonical, pretty JSON. The entity is
/// normalized first (sorting selections and parameters) so the output is
/// byte-stable for zero-noise git diffs — the engine no longer sorts implicitly
/// on serialize.
///
/// The write replaces the file rather than truncating it
/// (`atomic_write.rs::write_file_atomically`): this is the path a plain Save
/// takes straight to the current file, with no dialog and no other copy of the
/// character anywhere, so a write that empties it first is one interruption away
/// from losing the document outright.
pub fn save_entity_to_path(entity: &Entity, path: &Path) -> Result<(), AppError> {
    let mut canonical = entity.clone();
    // Stamp the current schema version so app-written saves never drift from the
    // engine's `SCHEMA_VERSION` (a save always reflects the shape it was written by).
    canonical.schema_version = arm_rules::SCHEMA_VERSION;
    canonical.normalize();
    let json = serde_json::to_string_pretty(&canonical)?;
    write_file_atomically(path, &json)
}

/// Renders `entity` as Markdown and writes it to `path`.
///
/// `ruleset` is the cached localized ruleset that supplies item display names, and
/// `labels` the frontend's localized document chrome (see
/// [`arm_rules::export::LABEL_KEYS`]) — no user-facing string originates here.
///
/// The ruleset is an `Option` rather than a reference because "no ruleset loaded"
/// is a real outcome the export must report: without it not one display name can be
/// resolved, so it fails with [`AppError::NotLoaded`] and writes nothing. Deciding
/// that here rather than in the command shim keeps the case testable without a
/// Tauri runtime.
pub fn export_markdown_to_path(
    entity: &Entity,
    ruleset: Option<&LocalizedRuleset>,
    labels: &BTreeMap<String, String>,
    path: &Path,
) -> Result<(), AppError> {
    let ruleset = ruleset.ok_or(AppError::NotLoaded)?;
    let markdown = arm_rules::character_markdown(entity, ruleset, labels)?;
    write_file_atomically(path, &markdown)
}

/// Reads and deserializes an entity from `path`, applying save migrations, and
/// hands back the engine's outcome **including what the migration rewrote**.
///
/// A pre-schema-10 save's manual `aging_reductions` are folded into `aging_points`
/// (aging drops are now derived). See [`arm_rules::load_entity_migrating`].
///
/// **The report is returned, not swallowed** (Viktor #4). It used to go only to
/// an `eprintln!`, which a GUI binary started from a desktop launcher sends
/// nowhere: the user was never told that their character had been rewritten,
/// even though the fold is lossy (the engine reconstructs the MINIMAL
/// aging-point total that reproduces the recorded drops) and the next save makes
/// it permanent. The engine's own contract says the caller surfaces a localized
/// notice — `migration.rs::LoadedEntity` — and this is the caller. The stderr line
/// stays as a SECOND surface for whoever did launch from a terminal, exactly as
/// `error.rs::AppError::reported` writes to stderr *and* returns the failure.
///
/// `default_saga_year` is what a **pre-schema-17** save inherits: before C8 the saga
/// year lived in `settings.json`, machine-globally, so the honest value for a
/// document that never recorded one is the year the user has configured for new
/// documents. The engine cannot read that file — it has no filesystem at all — so
/// this crate, which owns the settings, hands it in. The caller passes
/// [`crate::settings::Settings::default_saga_year`].
pub fn load_entity_from_path(
    path: &Path,
    default_saga_year: i32,
) -> Result<arm_rules::LoadedEntity, AppError> {
    let json = fs::read_to_string(path)?;
    let loaded = arm_rules::load_entity_migrating(&json, default_saga_year)?;
    if !loaded.migrated_aging_characteristics.is_empty() {
        let characteristics: Vec<String> = loaded
            .migrated_aging_characteristics
            .iter()
            .map(|c| c.to_string())
            .collect();
        eprintln!(
            "save migration: folded legacy aging_reductions into aging_points for {} ({})",
            path.display(),
            characteristics.join(", ")
        );
    }
    Ok(loaded)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extension_and_default_name_per_kind() {
        assert_eq!(entity_extension(EntityKind::Character), "armc");
        assert_eq!(entity_extension(EntityKind::Covenant), "armcov");
        assert_eq!(default_file_name(EntityKind::Character), "character.armc");
        assert_eq!(default_file_name(EntityKind::Covenant), "covenant.armcov");
    }

    #[test]
    fn markdown_default_name_shares_the_base_name_with_the_save_default() {
        assert_eq!(
            default_markdown_file_name(EntityKind::Character),
            "character.md"
        );
        assert_eq!(
            default_markdown_file_name(EntityKind::Covenant),
            "covenant.md"
        );
    }

    #[test]
    fn markdown_name_follows_the_current_save_file() {
        // The save file's base name, re-extensioned: an export is named after the
        // document it came from.
        assert_eq!(
            markdown_file_name_for(Some("/home/u/gerhard.armc"), EntityKind::Character),
            "gerhard.md"
        );
        // A path recorded on Windows separates with backslashes.
        assert_eq!(
            markdown_file_name_for(Some("C:\\saves\\gerhard.armcov"), EntityKind::Covenant),
            "gerhard.md"
        );
        // A base name without an extension is already the stem.
        assert_eq!(
            markdown_file_name_for(Some("/x/gerhard"), EntityKind::Character),
            "gerhard.md"
        );
    }

    #[test]
    fn markdown_name_falls_back_to_the_kind_default() {
        // Never saved: nothing to name the export after.
        assert_eq!(
            markdown_file_name_for(None, EntityKind::Character),
            default_markdown_file_name(EntityKind::Character)
        );
        assert_eq!(
            markdown_file_name_for(None, EntityKind::Covenant),
            default_markdown_file_name(EntityKind::Covenant)
        );
        // A base name that is nothing but an extension leaves an empty stem.
        assert_eq!(
            markdown_file_name_for(Some("/x/.armc"), EntityKind::Character),
            default_markdown_file_name(EntityKind::Character)
        );
        // A directory-only path has no base name at all.
        assert_eq!(
            markdown_file_name_for(Some("/x/"), EntityKind::Covenant),
            default_markdown_file_name(EntityKind::Covenant)
        );
    }

    #[test]
    fn save_file_directory_is_the_path_up_to_its_last_separator() {
        assert_eq!(
            save_file_directory(Some("/home/u/gerhard.armc")),
            Some("/home/u/")
        );
        assert_eq!(
            save_file_directory(Some("C:\\saves\\gerhard.armcov")),
            Some("C:\\saves\\")
        );
        // A root-level file keeps the separator, so the directory is not empty.
        assert_eq!(save_file_directory(Some("/gerhard.armc")), Some("/"));
        // A bare file name names no directory; so does a document never saved.
        assert_eq!(save_file_directory(Some("gerhard.armc")), None);
        assert_eq!(save_file_directory(None), None);
    }

    #[test]
    fn ensure_extension_only_fills_when_missing() {
        // No extension -> append the kind's extension.
        assert_eq!(
            ensure_extension(PathBuf::from("/tmp/testchar"), "armc"),
            PathBuf::from("/tmp/testchar.armc")
        );
        // An explicit extension the user typed is kept (incl. .armc and .json).
        assert_eq!(
            ensure_extension(PathBuf::from("/tmp/testchar.armc"), "armc"),
            PathBuf::from("/tmp/testchar.armc")
        );
        assert_eq!(
            ensure_extension(PathBuf::from("/tmp/testchar.json"), "armc"),
            PathBuf::from("/tmp/testchar.json")
        );
    }
}
