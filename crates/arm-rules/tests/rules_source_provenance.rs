//! V22: nothing previously checked that `rules/core/*.json` stays in sync
//! with `rules/source/**` — the authoritative, human-authored Markdown
//! (`CLAUDE.md` → "Rules source files" / "Rules provenance"). Drift (a
//! citation left stale after the source Markdown was edited, a `source`
//! block naming the wrong file, a line range shifted out of bounds) would go
//! undetected until a user noticed wrong rules data.
//!
//! Deliberately bounded: this does **not** re-derive the catalogue or
//! re-parse the rulebooks into structured data (fragile, out of proportion —
//! see `CLAUDE.md` → "Catalogue size is data, never code" and the
//! `rules/core/` extraction note). Instead it walks every `source: { file,
//! lines: [start, end] }` block that already exists in `rules/core/*.json`
//! — whatever catalogue it is in, however many there are — and asserts each
//! one actually brackets real content in its named source file: the file
//! exists, the range is well-formed and in-bounds, and the bracketed lines
//! are not blank. This catches the concrete failure modes drift produces
//! (renamed/moved source file, an edit that shifted everything below it so
//! the old line numbers now point at the wrong passage or past EOF, a
//! transposed start/end) without asserting anything about catalogue *size*.
//!
//! `CLAUDE.md` → "Rules provenance": `source` in `rules/core/` always points
//! at the **English** file, so every reference below resolves against
//! `rules/source/en/`.
//!
//! A second, sharper guard lives beside that one:
//! [`every_guarded_effect_cites_a_passage_that_names_its_own_mechanic`]. The
//! bracketing check above proves a citation points *somewhere*; it cannot
//! prove it points at a passage that supports the mechanic encoded beside it,
//! which is what `CLAUDE.md` → "Rules provenance" makes the whole purpose of
//! the `source` field. That is a **cross-site** property — it is not a
//! statement about any one catalogue entry but about the relation between
//! every entry encoding the same effect kind — and it is the shape of test
//! this repo already uses for enum↔Fluent-key parity and profile
//! conformance, aimed for the first time at the rules-semantics axis.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use serde_json::Value;

/// `crates/arm-rules` (this crate's manifest dir) -> repo root -> `rules/`.
fn rules_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../rules")
}

/// One `source` block found somewhere in a `rules/core/*.json` file, plus
/// enough context to name it in a failure message.
struct FoundSourceRef {
    /// The `rules/core/<name>.json` file it was found in (for error messages).
    core_file: String,
    /// The sibling `"id"` of the object the `source` block sits on, if any
    /// (most catalogue entries have one; falls back to a positional label).
    entry_label: String,
    source_file: String,
    /// The `anchor` recorded beside the line range, when the entry has been
    /// swept. See `types.rs::SourceRef::anchor`.
    anchor: Option<String>,
    start: i64,
    end: i64,
    /// The `type` of every entry in the sibling `effects` array on the same
    /// object, in document order — the mechanic(s) this citation is being
    /// offered as the source for. Empty for the many entries that carry a
    /// `source` block and no effects at all (narrative Virtues/Flaws, spells,
    /// abilities, …).
    effect_kinds: Vec<String>,
}

/// Recursively walks a parsed JSON document looking for `source: { "file":
/// ..., "lines": [start, end] }` blocks — the shape [`SourceRef`] serializes
/// to (see `crates/arm-rules/src/types.rs`). Generic over which catalogue it
/// is walking: every `rules/core/*.json` file uses the same shape for
/// provenance, so one walker covers virtues/flaws, abilities, arts, spells,
/// spell mastery abilities, houses, mythic companion types, equipment,
/// childhoods, and aging rows alike, with zero per-catalogue code.
fn collect_source_refs(core_file: &str, value: &Value, out: &mut Vec<FoundSourceRef>) {
    match value {
        Value::Object(map) => {
            if let Some(source) = map.get("source")
                && let (Some(file), Some(lines)) = (
                    source.get("file").and_then(Value::as_str),
                    source.get("lines").and_then(Value::as_array),
                )
                && let [start, end] = lines.as_slice()
                && let (Some(start), Some(end)) = (start.as_i64(), end.as_i64())
            {
                let entry_label = map
                    .get("id")
                    .and_then(Value::as_str)
                    .map(str::to_string)
                    .unwrap_or_else(|| format!("<no id, cites {file}:{start}-{end}>"));
                let effect_kinds = map
                    .get("effects")
                    .and_then(Value::as_array)
                    .map(|effects| {
                        effects
                            .iter()
                            .filter_map(|effect| effect.get("type").and_then(Value::as_str))
                            .map(str::to_string)
                            .collect()
                    })
                    .unwrap_or_default();
                out.push(FoundSourceRef {
                    core_file: core_file.to_string(),
                    entry_label,
                    source_file: file.to_string(),
                    anchor: source
                        .get("anchor")
                        .and_then(Value::as_str)
                        .map(str::to_string),
                    start,
                    end,
                    effect_kinds,
                });
            }
            for nested in map.values() {
                collect_source_refs(core_file, nested, out);
            }
        }
        Value::Array(items) => {
            for item in items {
                collect_source_refs(core_file, item, out);
            }
        }
        _ => {}
    }
}

/// Validates one [`FoundSourceRef`] against the actual Markdown file, pushing
/// a descriptive message onto `errors` for every way it can be stale:
/// malformed range, missing file, out-of-bounds range, or a range that
/// brackets nothing but blank lines.
fn validate_source_ref(source_dir: &Path, found: &FoundSourceRef, errors: &mut Vec<String>) {
    let FoundSourceRef {
        core_file,
        entry_label,
        source_file,
        anchor: _,
        start,
        end,
        effect_kinds: _,
    } = found;

    if *start < 1 || end < start {
        errors.push(format!(
            "{core_file}: \"{entry_label}\" cites {source_file}:{start}-{end}, which is not a \
             well-formed 1-based inclusive range"
        ));
        return;
    }

    let path = source_dir.join(source_file);
    let content = match fs::read_to_string(&path) {
        Ok(content) => content,
        Err(e) => {
            errors.push(format!(
                "{core_file}: \"{entry_label}\" cites source file {source_file}, which could not \
                 be read at {path:?}: {e}"
            ));
            return;
        }
    };

    let lines: Vec<&str> = content.lines().collect();
    let total = lines.len() as i64;
    if *end > total {
        errors.push(format!(
            "{core_file}: \"{entry_label}\" cites {source_file}:{start}-{end}, but the file has \
             only {total} lines — the citation is stale or the file was trimmed"
        ));
        return;
    }

    let bracketed = &lines[(*start as usize - 1)..(*end as usize)];
    if bracketed.iter().all(|line| line.trim().is_empty()) {
        errors.push(format!(
            "{core_file}: \"{entry_label}\" cites {source_file}:{start}-{end}, but every line in \
             that range is blank — the citation no longer brackets any passage"
        ));
    }
}

/// Every `rules/core/*.json` file, discovered rather than hardcoded — a new
/// catalogue file is picked up automatically, matching "catalogue size is
/// data, never code".
fn core_json_files() -> Vec<PathBuf> {
    let dir = rules_dir().join("core");
    let mut files: Vec<PathBuf> = fs::read_dir(&dir)
        .unwrap_or_else(|e| panic!("rules/core is readable: {e}"))
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "json"))
        .collect();
    files.sort();
    assert!(
        !files.is_empty(),
        "expected at least one rules/core/*.json file to exist"
    );
    files
}

/// Every `source` block in every `rules/core/*.json` file, walked once.
fn all_source_refs() -> Vec<FoundSourceRef> {
    let mut all_refs = Vec::new();
    for core_path in core_json_files() {
        let core_file = core_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap()
            .to_string();
        let text = fs::read_to_string(&core_path)
            .unwrap_or_else(|e| panic!("{core_file} is readable: {e}"));
        let value: Value = serde_json::from_str(&text)
            .unwrap_or_else(|e| panic!("{core_file} is valid JSON: {e}"));
        collect_source_refs(&core_file, &value, &mut all_refs);
    }
    all_refs
}

#[test]
fn every_core_source_citation_brackets_real_content_in_its_named_file() {
    let source_dir = rules_dir().join("source/en");
    let all_refs = all_source_refs();

    // This is the property that makes the test meaningful rather than
    // vacuous: if nothing carries a `source` block, every check below passes
    // trivially without having verified anything.
    assert!(
        all_refs.len() > 100,
        "expected many source citations across rules/core/*.json, found {}",
        all_refs.len()
    );

    let mut errors = Vec::new();
    for found in &all_refs {
        validate_source_ref(&source_dir, found, &mut errors);
    }

    assert!(
        errors.is_empty(),
        "{} stale/invalid source citation(s) in rules/core/*.json:\n{}",
        errors.len(),
        errors.join("\n")
    );
}

/// Every `source.file` named anywhere in `rules/core/*.json` must be one of
/// the English sourcebooks `CLAUDE.md` documents as shipped — catching a
/// citation that names a book with no English source at all (which
/// `CLAUDE.md` → "Rules provenance" forbids: "A rule from a book with no
/// English source in `rules/source/en/` yet ... cannot be implemented").
#[test]
fn every_cited_source_file_exists_under_rules_source_en() {
    let source_dir = rules_dir().join("source/en");
    let cited_files: BTreeSet<String> = all_source_refs()
        .into_iter()
        .map(|r| r.source_file)
        .collect();

    assert!(
        !cited_files.is_empty(),
        "expected at least one cited source file across rules/core/*.json"
    );

    let missing: Vec<&String> = cited_files
        .iter()
        .filter(|file| !source_dir.join(file).is_file())
        .collect();
    assert!(
        missing.is_empty(),
        "rules/core/*.json cites source file(s) not present under rules/source/en/: {missing:?}"
    );
}

// --- The heading-anchor guard ----------------------------------------------

/// The per-language sidecar that carries a non-English item's heading anchor,
/// found under `rules/i18n/<lang>/`. English anchors key `rules/core/`'s
/// `source` block instead, because `rules/core/` *is* the canonical-ID language
/// (`CLAUDE.md` → "Rules provenance").
///
/// Shape: `id -> { "anchor": <slug>, "file": <basename under rules/source/<lang>/> }`.
/// Deliberately a **sidecar** rather than a field on the localized entries in
/// `virtues_flaws.json`:
///
/// - those entries deserialize into `types.rs::I18nEntry`, whose serialized
///   shape is a stable IPC contract the frontend binds to, and an extraction
///   coordinate has no business crossing that boundary into every tooltip;
/// - it is provenance, not rules text, and `CLAUDE.md` → "Strict separation of
///   data kinds" keeps those apart;
/// - `ruleset_io.rs::read_i18n_sources` reads a fixed ten-file list, so the
///   sidecar is not loaded at runtime at all — which is correct: it exists for
///   the re-sync tooling and for this guard.
///
/// It records **no line range**, on purpose. German line numbers are already
/// derivable from the English ones by the line-parity invariant
/// (`CLAUDE.md` → "Rules provenance"), and
/// [`german_anchors_sit_on_the_same_line_as_their_english_counterparts`] is what
/// now checks that. Copying the numbers here would create a second coordinate to
/// rot.
const ANCHOR_SIDECAR: &str = "source_anchors.json";

/// The language `rules/core/` citations resolve against.
const CANONICAL_LANGUAGE: &str = "en";

/// Slugifies a Markdown heading the way the generator that produced these files
/// does — the algorithm is not guessed: the sources carry their own generated
/// cross-links (`[Anchored to the (Land)](#anchored-to-the-land)`,
/// `[Seite 15](#haus-mercere)`, `(#hitze--und-ätzungstabelle)`), and this
/// reproduces every one of them.
///
/// Lowercase; drop everything that is not alphanumeric, `-`, `_` or a space;
/// spaces become `-`. Unicode-aware, so `Ähnliche Zauber` becomes
/// `ähnliche-zauber` rather than losing its umlaut.
fn slugify_heading(text: &str) -> String {
    text.to_lowercase()
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '-' || *c == '_' || *c == ' ')
        .map(|c| if c == ' ' { '-' } else { c })
        .collect()
}

/// One Markdown heading, located and slugified.
#[derive(Clone, Debug)]
struct Heading {
    /// 1-based line number the `#` sits on.
    line: usize,
    /// How many `#` characters opened it — 4 for the `####` a catalogue item is
    /// defined under.
    level: usize,
    /// Whether the heading sits inside a blockquote (`> #### …`). Those are the
    /// books' **sidebars** — a boxed table or example belonging to the section
    /// it is printed in, not a divider between sections.
    blockquoted: bool,
    /// The anchor the generator gives it, `-N` disambiguation included.
    anchor: String,
}

/// Every Markdown heading in `text`, in document order, including the
/// generator's `-1`/`-2` disambiguation for repeated headings (which is what
/// `#die-gabe-2` and `#abilities-1` in these files are).
fn headings(text: &str) -> Vec<Heading> {
    let mut seen: BTreeMap<String, usize> = BTreeMap::new();
    let mut found = Vec::new();

    for (index, line) in text.lines().enumerate() {
        let trimmed = line.trim_start();
        // A blockquoted heading (`> #### Environmental Temperatures`) is a
        // sidebar, and the generator anchors it like any other heading.
        let blockquoted = trimmed.starts_with("> ");
        let trimmed = trimmed.strip_prefix("> ").unwrap_or(trimmed).trim_start();
        if !trimmed.starts_with('#') {
            continue;
        }
        let level = trimmed.chars().take_while(|c| *c == '#').count();
        let title = trimmed.trim_start_matches('#').trim();
        if title.is_empty() {
            continue;
        }
        let base = slugify_heading(title);
        let count = seen.entry(base.clone()).or_insert(0);
        let anchor = if *count == 0 {
            base.clone()
        } else {
            format!("{base}-{count}")
        };
        *count += 1;
        found.push(Heading {
            line: index + 1,
            level,
            blockquoted,
            anchor,
        });
    }

    found
}

/// Every Markdown heading in `text`, as `anchor -> 1-based line number`.
fn heading_anchors(text: &str) -> BTreeMap<String, usize> {
    let mut anchors = BTreeMap::new();
    for heading in headings(text) {
        anchors.entry(heading.anchor).or_insert(heading.line);
    }
    anchors
}

/// `rules/source/<lang>/<file>` -> its heading anchors, read once per file.
fn anchors_for(
    cache: &mut BTreeMap<(String, String), BTreeMap<String, usize>>,
    lang: &str,
    file: &str,
) -> BTreeMap<String, usize> {
    cache
        .entry((lang.to_string(), file.to_string()))
        .or_insert_with(|| {
            let path = rules_dir().join("source").join(lang).join(file);
            match fs::read_to_string(&path) {
                Ok(text) => heading_anchors(&text),
                Err(_) => BTreeMap::new(),
            }
        })
        .clone()
}

/// `rules/source/<lang>/<file>` -> its headings in document order, read once
/// per file. The list form (rather than [`anchors_for`]'s map) is what lets a
/// caller ask "which heading opens this line" and "where does the next one
/// start", which is the pair the two range guards below are built on.
fn headings_for(
    cache: &mut BTreeMap<(String, String), Vec<Heading>>,
    lang: &str,
    file: &str,
) -> Vec<Heading> {
    cache
        .entry((lang.to_string(), file.to_string()))
        .or_insert_with(|| {
            let path = rules_dir().join("source").join(lang).join(file);
            match fs::read_to_string(&path) {
                Ok(text) => headings(&text),
                Err(_) => Vec::new(),
            }
        })
        .clone()
}

/// The catalogues whose entries are *all* required to carry a heading anchor
/// in both stores, paired with the heading level their catalogue is defined
/// under (`####` for most books' entries, `###` for mythic companion types,
/// `#####` for spells — a structural fact, not a count). Scoped incrementally
/// on purpose: the anchor sweep is incremental (`docs/rules-source-resync.md`),
/// and a catalogue earns a place here only once *every* entry resolves — a
/// catalogue with even one entry pending a human reading (see
/// `CATALOGUES_TO_SWEEP` below) stays out until that reading lands. Widen this
/// list as later catalogues are swept to completion — never loosen the
/// assertion instead.
const FULLY_ANCHORED_CATALOGUES: &[(&str, usize)] = &[
    ("virtues_flaws.json", 4),
    ("arts.json", 4),
    ("spell_mastery_abilities.json", 4),
    ("mythic_companion_types.json", 3),
    ("abilities.json", 4),
    // `spells.json` and `parameter_catalogues.json` are NOT here yet: each has
    // one or more entries a heading-only sweep cannot resolve
    // (`spell.piercing_the_magical_veil`; `profession.poet`,
    // `profession.storyteller`, `organization.church`) and needs a human
    // reading first (`tmp/x9a-spike.md` §4 Q2, Q7) before every entry in the
    // file carries an anchor. They join once that reading lands.
];

/// Every `rules/i18n/<lang>/source_anchors.json`, as
/// `lang -> id -> (file, anchor)`. The canonical language is skipped: its
/// anchors live in `rules/core/`.
fn localized_anchors() -> BTreeMap<String, BTreeMap<String, (String, String)>> {
    let i18n_dir = rules_dir().join("i18n");
    let mut by_language = BTreeMap::new();

    for entry in fs::read_dir(&i18n_dir).unwrap_or_else(|e| panic!("rules/i18n is readable: {e}")) {
        let lang_path = entry.expect("a readable rules/i18n child").path();
        if !lang_path.is_dir() {
            continue;
        }
        let lang = lang_path
            .file_name()
            .expect("a language directory has a name")
            .to_string_lossy()
            .to_string();
        if lang == CANONICAL_LANGUAGE {
            continue;
        }

        let path = lang_path.join(ANCHOR_SIDECAR);
        if !path.is_file() {
            continue;
        }
        let text = fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("{} is readable: {e}", path.display()));
        let parsed: BTreeMap<String, Value> = serde_json::from_str(&text)
            .unwrap_or_else(|e| panic!("{} is valid JSON: {e}", path.display()));

        let rows = parsed
            .into_iter()
            .map(|(id, value)| {
                let file = value["file"]
                    .as_str()
                    .unwrap_or_else(|| panic!("{}: \"{id}\" has a string file", path.display()))
                    .to_string();
                let anchor = value["anchor"]
                    .as_str()
                    .unwrap_or_else(|| panic!("{}: \"{id}\" has a string anchor", path.display()))
                    .to_string();
                (id, (file, anchor))
            })
            .collect();
        by_language.insert(lang, rows);
    }

    by_language
}

/// The floor that stops the anchor guards passing vacuously. A count, not a
/// total — `CLAUDE.md` → "Catalogue size is data, never code" forbids asserting
/// exact catalogue sizes, and the sweep that records anchors is incremental, so
/// this only ever needs raising.
const MIN_RECORDED_ANCHORS: usize = 40;

/// **Every recorded heading anchor resolves to a real heading, and that heading
/// is inside the line range recorded beside it.**
///
/// This is the anchor-shaped sibling of `rulebook_citations.rs`, and the second
/// half is what makes it worth more than a spell-check. `CLAUDE.md`'s citation
/// guards can only prove a range lands on non-blank lines; the upstream re-sync
/// `docs/rules-source-resync.md` describes will shift every range at once and
/// leave that check green while hundreds of citations point at the wrong rule.
/// Binding the anchor to the range means the two coordinates have to agree:
/// after a re-sync, a shifted range no longer contains its own heading and this
/// fails immediately, and the fix is mechanical — relocate the anchor, take its
/// new line number.
#[test]
fn every_recorded_source_anchor_resolves_to_its_own_heading() {
    let all_refs = all_source_refs();
    let mut cache = BTreeMap::new();
    let mut recorded = 0usize;
    let mut errors = Vec::new();

    for found in &all_refs {
        let Some(anchor) = &found.anchor else {
            continue;
        };
        recorded += 1;
        let anchors = anchors_for(&mut cache, CANONICAL_LANGUAGE, &found.source_file);
        match anchors.get(anchor) {
            None => errors.push(format!(
                "{}: \"{}\" records anchor \"#{anchor}\", but {} has no heading with that \
                 anchor — the heading was renamed, or the anchor was mistyped",
                found.core_file, found.entry_label, found.source_file
            )),
            Some(&line) => {
                let line = line as i64;
                if line < found.start || line > found.end {
                    errors.push(format!(
                        "{}: \"{}\" records anchor \"#{anchor}\", whose heading is at \
                         {}:{line} — outside the cited range {}-{}. The two halves of this \
                         citation disagree, so one of them is stale.",
                        found.core_file,
                        found.entry_label,
                        found.source_file,
                        found.start,
                        found.end
                    ));
                }
            }
        }
    }

    assert!(
        recorded >= MIN_RECORDED_ANCHORS,
        "only {recorded} source citation(s) in rules/core/*.json record a heading anchor, below \
         the floor of {MIN_RECORDED_ANCHORS} — so this guard is checking almost nothing. Either \
         the sweep recording them regressed, or the floor is stale."
    );

    assert!(
        errors.is_empty(),
        "{} recorded heading anchor(s) in rules/core/*.json do not resolve:\n{}",
        errors.len(),
        errors.join("\n")
    );
}

/// Every anchor in a `rules/i18n/<lang>/source_anchors.json` resolves to a real
/// heading in that language's own source file.
#[test]
fn every_localized_source_anchor_resolves_to_a_heading() {
    let by_language = localized_anchors();
    let mut cache = BTreeMap::new();
    let mut recorded = 0usize;
    let mut errors = Vec::new();

    for (lang, rows) in &by_language {
        for (id, (file, anchor)) in rows {
            recorded += 1;
            let path = rules_dir().join("source").join(lang).join(file);
            if !path.is_file() {
                errors.push(format!(
                    "{lang}/{ANCHOR_SIDECAR}: \"{id}\" names source file {file}, which does not \
                     exist under rules/source/{lang}/"
                ));
                continue;
            }
            if !anchors_for(&mut cache, lang, file).contains_key(anchor) {
                errors.push(format!(
                    "{lang}/{ANCHOR_SIDECAR}: \"{id}\" records anchor \"#{anchor}\", but {file} \
                     has no heading with that anchor"
                ));
            }
        }
    }

    assert!(
        recorded >= MIN_RECORDED_ANCHORS,
        "only {recorded} localized heading anchor(s) recorded across rules/i18n/*/{ANCHOR_SIDECAR}, \
         below the floor of {MIN_RECORDED_ANCHORS} — this guard is checking almost nothing"
    );

    assert!(
        errors.is_empty(),
        "{} localized heading anchor(s) do not resolve:\n{}",
        errors.len(),
        errors.join("\n")
    );
}

/// **The German line-parity invariant, tested for the first time.**
///
/// `CLAUDE.md` → "Rules provenance" declares that the German sources mirror the
/// English line-by-line throughout, so a German line number identifies the same
/// item as the English one — which is why `rules/core/` can carry a single line
/// range and both locales' extractions can use it. `docs/rules-source-resync.md`
/// records that nothing tested this, and that the invariant breaks silently if
/// the two languages ever shift differently.
///
/// Anchors make it testable: for every item recording an anchor in both
/// languages, the two headings must sit on the *same* line. That is the
/// invariant stated exactly, item by item, over whatever subset has been swept.
#[test]
fn german_anchors_sit_on_the_same_line_as_their_english_counterparts() {
    let english: BTreeMap<String, (String, Option<String>)> = all_source_refs()
        .into_iter()
        .map(|found| (found.entry_label, (found.source_file, found.anchor)))
        .collect();
    let by_language = localized_anchors();
    let mut cache = BTreeMap::new();
    let mut compared = 0usize;
    let mut errors = Vec::new();

    for (lang, rows) in &by_language {
        for (id, (file, anchor)) in rows {
            let Some((english_file, Some(english_anchor))) = english.get(id) else {
                continue;
            };
            let english_line = anchors_for(&mut cache, CANONICAL_LANGUAGE, english_file)
                .get(english_anchor)
                .copied();
            let localized_line = anchors_for(&mut cache, lang, file).get(anchor).copied();
            let (Some(english_line), Some(localized_line)) = (english_line, localized_line) else {
                // Unresolvable anchors are the other two tests' finding.
                continue;
            };
            compared += 1;
            if english_line != localized_line {
                errors.push(format!(
                    "\"{id}\": #{english_anchor} is at {english_file}:{english_line} but \
                     #{anchor} is at {lang}/{file}:{localized_line} — the line-parity invariant \
                     CLAUDE.md declares for the German sources does not hold here, so a line \
                     number no longer identifies the same item in both languages"
                ));
            }
        }
    }

    assert!(
        compared >= MIN_RECORDED_ANCHORS,
        "only {compared} item(s) have an anchor in both languages, below the floor of \
         {MIN_RECORDED_ANCHORS} — this guard is checking almost nothing"
    );

    assert!(
        errors.is_empty(),
        "{} item(s) break the German line-parity invariant:\n{}",
        errors.len(),
        errors.join("\n")
    );
}

/// The anchor guards above are only as good as the slug algorithm they resolve
/// with, and a resolver that is wrong in the same way the data is wrong would
/// pass while checking nothing. These cases are taken from the sources' **own
/// generated cross-links**, so they are the generator's output rather than my
/// reading of it.
#[test]
fn the_heading_slug_matches_the_sources_own_generated_links() {
    // `[Anchored to the (Land)](#anchored-to-the-land)`, ArMDE:5568 — brackets
    // and parentheses are dropped, not transliterated.
    assert_eq!(
        slugify_heading("Anchored to the (Land)"),
        "anchored-to-the-land"
    );
    assert_eq!(
        slugify_heading("Fish Out of Water (Terrain)"),
        "fish-out-of-water-terrain"
    );
    // German links keep their umlauts: `(#ähnliche-zauber)`,
    // `(#hitze--und-ätzungstabelle)` — note the doubled hyphen, which survives
    // because a hyphen already in the heading is kept and the space becomes a
    // second one.
    assert_eq!(slugify_heading("Ähnliche Zauber"), "ähnliche-zauber");
    assert_eq!(
        slugify_heading("Hitze- und Ätzungstabelle"),
        "hitze--und-ätzungstabelle"
    );
    assert_eq!(slugify_heading("Haus Mercere"), "haus-mercere");

    // Repeated headings take the generator's `-N` suffix, zero-based on the
    // *second* occurrence: `(#die-gabe-2)` is the third "Die Gabe".
    let anchors = heading_anchors("# Die Gabe\n\n## Die Gabe\n\n#### Die Gabe\n");
    assert_eq!(anchors.get("die-gabe"), Some(&1));
    assert_eq!(anchors.get("die-gabe-1"), Some(&3));
    assert_eq!(anchors.get("die-gabe-2"), Some(&5));

    // A sidebar heading inside a blockquote is still a heading.
    let sidebar = heading_anchors("> #### Environmental Temperatures\n");
    assert_eq!(sidebar.get("environmental-temperatures"), Some(&1));

    // Not headings: a hash inside prose, and a bare hash rule.
    let prose = heading_anchors("the # sign\n#\n");
    assert!(prose.is_empty(), "{prose:?}");
}

/// **Guard A — the anchor is the heading the range opens on, in both
/// languages.**
///
/// [`every_recorded_source_anchor_resolves_to_its_own_heading`] only asks that
/// the anchor's heading fall *somewhere* inside the cited range. That is too
/// loose to survive a re-sync: a range that shifted by a line or two still
/// contains its heading. The extraction actually produces a tighter fact —
/// `source.lines[0]` **is** the heading line the item is defined under — so
/// this asserts exactly that, for the English anchor in `rules/core/` and for
/// the German one in `rules/i18n/de/source_anchors.json` alike. The expected
/// heading *level* is per catalogue (recorded beside its name in
/// [`FULLY_ANCHORED_CATALOGUES`]): `####` for most, `###` for mythic
/// companion types, `#####` for spells — a structural fact about how each
/// book lays that catalogue out, not a count.
///
/// It is also the coverage assertion for the backfill: every entry of a
/// catalogue in [`FULLY_ANCHORED_CATALOGUES`] must record an anchor in *both*
/// stores, so a Virtue or Flaw added later cannot quietly arrive without the
/// only coordinate that survives re-pagination. This is a per-entry
/// requirement, not a count — `CLAUDE.md` → "Catalogue size is data, never
/// code" is untouched.
///
/// The German half leans on the line-parity invariant `CLAUDE.md` declares:
/// the German book mirrors the English line-by-line, so the same
/// `source.lines[0]` locates the item's heading in both files.
#[test]
fn every_anchored_catalogue_entry_records_the_heading_that_opens_its_range() {
    const GERMAN: &str = "de";

    let german = localized_anchors();
    let german_rows = german.get(GERMAN).cloned().unwrap_or_default();
    let mut cache = BTreeMap::new();
    let mut checked = 0usize;
    let mut errors = Vec::new();

    for found in all_source_refs() {
        let Some(&(_, expected_level)) = FULLY_ANCHORED_CATALOGUES
            .iter()
            .find(|(name, _)| *name == found.core_file)
        else {
            continue;
        };
        checked += 1;

        let english = headings_for(&mut cache, CANONICAL_LANGUAGE, &found.source_file);
        let opening = english
            .iter()
            .find(|heading| heading.line as i64 == found.start);

        match (&found.anchor, opening) {
            (None, _) => errors.push(format!(
                "{}: \"{}\" records no source.anchor. Every entry of this catalogue must carry \
                 the heading anchor of its own definition — it is the only half of the citation \
                 that survives a re-paginated rulebook.",
                found.core_file, found.entry_label
            )),
            (Some(anchor), None) => errors.push(format!(
                "{}: \"{}\" records anchor \"#{anchor}\", but {}:{} is not a heading line at all \
                 — the range no longer opens on the item's definition",
                found.core_file, found.entry_label, found.source_file, found.start
            )),
            (Some(anchor), Some(heading)) => {
                if heading.level != expected_level {
                    errors.push(format!(
                        "{}: \"{}\" opens on {}:{}, which is a level-{} heading — this \
                         catalogue's items are defined under a level-{expected_level} heading \
                         (`{}`)",
                        found.core_file,
                        found.entry_label,
                        found.source_file,
                        found.start,
                        heading.level,
                        "#".repeat(expected_level),
                    ));
                } else if *anchor != heading.anchor {
                    errors.push(format!(
                        "{}: \"{}\" records anchor \"#{anchor}\", but the heading at {}:{} slugs \
                         to \"#{}\" — the anchor and the line range name different items",
                        found.core_file,
                        found.entry_label,
                        found.source_file,
                        found.start,
                        heading.anchor
                    ));
                }
            }
        }

        match german_rows.get(&found.entry_label) {
            None => errors.push(format!(
                "de/{ANCHOR_SIDECAR}: \"{}\" has no German heading anchor. Both locales are in \
                 scope of every slice, so a swept entry carries an anchor in each.",
                found.entry_label
            )),
            Some((file, anchor)) => {
                let localized = headings_for(&mut cache, GERMAN, file);
                match localized
                    .iter()
                    .find(|heading| heading.line as i64 == found.start)
                {
                    None => errors.push(format!(
                        "de/{ANCHOR_SIDECAR}: \"{}\" records anchor \"#{anchor}\", but {file}:{} \
                         — the line its English counterpart is defined on — is not a heading, so \
                         the line-parity invariant does not hold here",
                        found.entry_label, found.start
                    )),
                    Some(heading) => {
                        if *anchor != heading.anchor {
                            errors.push(format!(
                                "de/{ANCHOR_SIDECAR}: \"{}\" records anchor \"#{anchor}\", but \
                                 the heading at {file}:{} slugs to \"#{}\"",
                                found.entry_label, found.start, heading.anchor
                            ));
                        }
                    }
                }
            }
        }
    }

    assert!(
        checked >= MIN_RECORDED_ANCHORS,
        "only {checked} entry/entries came from {FULLY_ANCHORED_CATALOGUES:?} — this guard is \
         checking almost nothing"
    );

    assert!(
        errors.is_empty(),
        "{} anchor/heading disagreement(s):\n{}",
        errors.len(),
        errors.join("\n")
    );
}

/// The heading that ends `opening`'s section: the next one that is neither a
/// blockquoted sidebar nor a deeper sub-heading. `None` when the section runs to
/// the end of the file.
fn section_boundary_after<'a>(
    file_headings: &'a [Heading],
    opening: &Heading,
) -> Option<&'a Heading> {
    file_headings.iter().find(|heading| {
        heading.line > opening.line && !heading.blockquoted && heading.level <= opening.level
    })
}

/// Items whose own section is cut into same-level `####` pieces, so the "next
/// heading at the same level" is still part of the item rather than the start of
/// the next one. Each row records why, because an unexplained exemption is
/// indistinguishable from a silenced bug.
///
/// [`every_subdivided_item_really_is_subdivided`] stops a row outliving its
/// reason.
const SUBDIVIDED_ITEMS: &[(&str, &str)] = &[(
    "ability.hex",
    "ArMDE:7504-7542 is the whole Hex entry, and the book lays its two tables out as \
     sibling `####` headings rather than sidebars: `#### Hex Delay Modifiers` (:7511), which \
     :7509 sends the reader to (\"apply the delay modifier\"), and `#### Hex Effects` (:7533), \
     which :7525 sends the reader to (\"compare this to the Ease Factor on the Hex Effects \
     chart\"). The next *Ability* is `#### Hunt` at :7543, exactly one line past the range's \
     end, so the citation stops where it should.",
)];

/// **Guard B — a citation stops before the next heading.**
///
/// Guard A pins the *start* of a range to the item's own `####`. Nothing pinned
/// the *end*, and that is a distinct defect: B19's F-540 had `flaw.wrathful_*`
/// citing ArMDE:7106-7119, a correct start and an extent that swallowed the
/// `# Chapter 5: Abilities` heading at :7114 plus three paragraphs belonging to
/// another chapter. Every existing guard stayed green, because the range does
/// land on non-blank lines and does contain its own heading.
///
/// A range may legitimately stop a line or two early — on its last body line
/// rather than the line before the next heading — which B09, B10 and B11 all
/// confirmed. So this forbids *overrunning* the next heading, and requires
/// nothing about landing exactly on its doorstep.
///
/// Scoped to citations that open on a heading: a range that brackets table rows
/// (aging, spell levels) has no "own section" for this question to be about.
///
/// Two kinds of heading are deliberately **not** section boundaries, both
/// established by running this guard over the catalogue and reading every
/// passage it flagged:
///
/// - a **blockquoted** heading (`> #### Environmental Temperatures`,
///   ArMDE:7041) is a printed sidebar belonging to the section it sits in —
///   Warped Senses' own text says "see sidebar" — so a range that reaches it is
///   right to. Nineteen of the twenty-two first-run flags were this;
/// - a **deeper** heading is a sub-section of the item, not the next item:
///   `mythic_type.faerie_doctor` opens on `### Faerie Doctors` (ArMDE:2668) and
///   its `#### Faerie Doctors as Mythic Companions` at ArMDE:2676 is part of it.
///
/// So the boundary is the next non-blockquoted heading at the same level or
/// shallower.
#[test]
fn no_source_range_runs_past_the_heading_that_follows_it() {
    let mut cache = BTreeMap::new();
    let mut checked = 0usize;
    let mut errors = Vec::new();

    for found in all_source_refs() {
        let file_headings = headings_for(&mut cache, CANONICAL_LANGUAGE, &found.source_file);
        let Some(opening) = file_headings
            .iter()
            .find(|heading| heading.line as i64 == found.start)
        else {
            continue;
        };
        if SUBDIVIDED_ITEMS
            .iter()
            .any(|(id, _)| *id == found.entry_label)
        {
            continue;
        }
        let Some(next) = section_boundary_after(&file_headings, opening) else {
            continue;
        };
        checked += 1;
        if found.end >= next.line as i64 {
            errors.push(format!(
                "{}: \"{}\" cites {}:{}-{}, but the next heading (\"#{}\") starts at :{}. The \
                 range runs past the end of the item's own section and into text that belongs to \
                 something else.",
                found.core_file,
                found.entry_label,
                found.source_file,
                found.start,
                found.end,
                next.anchor,
                next.line
            ));
        }
    }

    assert!(
        checked >= MIN_RECORDED_ANCHORS,
        "only {checked} citation(s) open on a heading — this guard is checking almost nothing"
    );

    assert!(
        errors.is_empty(),
        "{} citation(s) overrun their own section:\n{}",
        errors.len(),
        errors.join("\n")
    );
}

/// [`SUBDIVIDED_ITEMS`] silences Guard B for a handful of citations, so every
/// row must still *need* silencing. If a row's range stops before its section
/// boundary after all — the citation was tightened, the source re-paginated, or
/// the id renamed — the row has become a stale excuse, and the guard should say
/// so rather than quietly cover one entry less.
#[test]
fn every_subdivided_item_really_is_subdivided() {
    let all_refs = all_source_refs();
    let mut cache = BTreeMap::new();

    for (id, _reason) in SUBDIVIDED_ITEMS {
        let found = all_refs
            .iter()
            .find(|found| found.entry_label == *id)
            .unwrap_or_else(|| {
                panic!(
                    "SUBDIVIDED_ITEMS row \"{id}\" names an entry that no longer carries a source \
                     citation — delete the row"
                )
            });
        let file_headings = headings_for(&mut cache, CANONICAL_LANGUAGE, &found.source_file);
        let opening = file_headings
            .iter()
            .find(|heading| heading.line as i64 == found.start)
            .unwrap_or_else(|| {
                panic!("SUBDIVIDED_ITEMS row \"{id}\" no longer opens on a heading")
            });
        let boundary = section_boundary_after(&file_headings, opening).unwrap_or_else(|| {
            panic!("SUBDIVIDED_ITEMS row \"{id}\" has no following section boundary — delete it")
        });
        assert!(
            found.end >= boundary.line as i64,
            "SUBDIVIDED_ITEMS row \"{id}\" now stops at :{} without reaching \"#{}\" at :{} — it \
             no longer needs the exemption, so delete the row and let Guard B cover it",
            found.end,
            boundary.anchor,
            boundary.line
        );
    }
}

// --- The regenerator -------------------------------------------------------

/// Catalogues the regenerator knows how to sweep, paired with the heading
/// level their entries are defined under (`####` for most, `###` for mythic
/// companion types, `#####` for spells — a structural fact, not a count). A
/// superset of [`FULLY_ANCHORED_CATALOGUES`]: a catalogue enters this list as
/// soon as its anchors are mechanically derivable, even before *every* entry
/// resolves — a few need a human reading first (`tmp/x9a-spike.md`,
/// `docs/rules-source-resync.md`), and the loop below leaves each of those
/// exactly as it was rather than failing the whole catalogue. A catalogue
/// earns a place in `FULLY_ANCHORED_CATALOGUES`, and Guard A's stricter
/// promise, only once every one of its entries resolves.
///
/// `virtues_flaws.json` is deliberately **not** listed here even though it is
/// in [`FULLY_ANCHORED_CATALOGUES`]: it was swept to completion by an earlier
/// slice, so this regenerator has nothing left to derive for it, and leaving
/// it out of the write path means a run for the newer catalogues below never
/// touches a file that a concurrent workstream may be mid-edit on.
const CATALOGUES_TO_SWEEP: &[(&str, usize)] = &[
    ("arts.json", 4),
    ("spell_mastery_abilities.json", 4),
    ("mythic_companion_types.json", 3),
    ("abilities.json", 4),
    ("spells.json", 5),
    ("parameter_catalogues.json", 4),
];

/// **The tool that satisfies the two guards above — not a test, and never run
/// by the gate.**
///
/// `docs/rules-source-resync.md` describes the job this exists for: a newer
/// rulebook edition shifts every line number at once, and the fix is to relocate
/// each item by its heading and take the heading's new line. Doing that by hand
/// across a 655-entry catalogue in two languages is not realistic, and it was not
/// realistic to *create* the anchors by hand either — which is why the sweep sat
/// at 94 of 655 for so long.
///
/// So this derives the anchor of every entry of every catalogue in
/// [`CATALOGUES_TO_SWEEP`] from the heading its `source.lines[0]` lands on, at
/// that catalogue's own level, in English and (via the line-parity invariant)
/// in German, and rewrites both stores. It is `#[ignore]`d because it **writes
/// into the repository**: `cargo test --workspace` must never mutate the tree,
/// and running it is a deliberate act (`cargo test -p arm-rules --test
/// rules_source_provenance -- --ignored regenerate_source_anchors`) whose output
/// is then reviewed as a diff like any other change.
///
/// An entry whose `lines[0]` does **not** land on a heading of the expected
/// level is left exactly as it was — no `anchor` added, no German row written —
/// rather than aborting the whole catalogue. That is a deliberate design point,
/// not a laxity: a few entries across these catalogues need a human reading
/// before they can be anchored at all (a citation into body prose, a `-N`
/// disambiguation that needs a name check), and the fix for those is
/// Norbert's, not a crash.
///
/// It deliberately rewrites each `rules/core/*.json` catalogue **line by
/// line** rather than reserializing it. Round-tripping the whole document
/// through `serde_json` would reflow every entry and bury the anchors in
/// thousands of lines of formatting churn, against `CLAUDE.md` → "Canonical
/// serialization"'s whole purpose of zero-noise diffs. The `source` block is
/// always one line, so replacing that one line is both sufficient and
/// minimal.
///
/// Correctness is not this function's claim to make:
/// [`every_anchored_catalogue_entry_records_the_heading_that_opens_its_range`]
/// is, and it re-derives the same fact independently from the sources.
#[test]
#[ignore = "writes into rules/; run deliberately after a rulebook re-sync"]
fn regenerate_source_anchors() {
    // The German book each English source file is mirrored by, learned from
    // the rows the sidecar already carries rather than hardcoded here.
    let existing = localized_anchors();
    let german_rows = existing.get("de").cloned().unwrap_or_default();
    let mut german_book: BTreeMap<String, String> = BTreeMap::new();
    let english_files: BTreeMap<String, String> = all_source_refs()
        .into_iter()
        .map(|found| (found.entry_label, found.source_file))
        .collect();
    for (id, (file, _)) in &german_rows {
        if let Some(english) = english_files.get(id) {
            german_book.insert(english.clone(), file.clone());
        }
    }

    let mut cache = BTreeMap::new();
    // Seeded from the existing sidecar so a catalogue outside
    // `CATALOGUES_TO_SWEEP` keeps its rows; every id this run touches is then
    // overwritten with a freshly-derived one.
    let mut sidecar_rows: BTreeMap<String, (String, String)> = german_rows
        .iter()
        .map(|(id, (file, anchor))| (id.clone(), (file.clone(), anchor.clone())))
        .collect();

    for &(catalogue_name, level) in CATALOGUES_TO_SWEEP {
        let catalogue = rules_dir().join("core").join(catalogue_name);
        let text = fs::read_to_string(&catalogue)
            .unwrap_or_else(|e| panic!("{catalogue_name} is readable: {e}"));

        // A handful of entries carry the block across several lines because it
        // grew long; joining them first lets the rewrite below treat every
        // block alike, and emits them all in the file's dominant one-line form.
        let mut rewritten = Vec::new();
        let mut source_lines = text.lines().peekable();
        while let Some(first) = source_lines.next() {
            let Some((prefix, rest)) = first.split_once("\"source\": {") else {
                rewritten.push(first.to_string());
                continue;
            };
            let mut block_text = rest.to_string();
            while !block_text.contains('}') {
                block_text.push(' ');
                block_text.push_str(source_lines.next().expect("the block closes").trim());
            }
            // The *first* `}` in `block_text` is always `source`'s own closing
            // brace: a `source` object is flat (string/array values only, no
            // nested `{`), so nothing earlier in `block_text` can close before
            // it does. Some catalogues (arts, spell mastery, ...) serialize an
            // entry compactly on one line, so `source` is not the entry's last
            // key and a *last*-`}` split would swallow the entry's own closing
            // brace into `body` instead of leaving it in `suffix`.
            let (body, suffix) = block_text.split_once('}').expect("the block closes");
            let block: Value = serde_json::from_str(&format!("{{{body}}}"))
                .unwrap_or_else(|e| panic!("{catalogue_name}: source block parses: {e}"));
            let file = block["file"].as_str().expect("source.file is a string");
            let start = block["lines"][0]
                .as_u64()
                .expect("source.lines[0] is a number") as usize;
            let end = block["lines"][1]
                .as_u64()
                .expect("source.lines[1] is a number") as usize;

            match anchor_opening(&mut cache, CANONICAL_LANGUAGE, file, start, level) {
                Some(anchor) => rewritten.push(format!(
                    "{prefix}\"source\": {{ \"anchor\": {}, \"file\": {}, \"lines\": [{start}, {end}] }}{suffix}",
                    serde_json::to_string(&anchor).expect("a string serializes"),
                    serde_json::to_string(file).expect("a string serializes"),
                )),
                None => rewritten.push(format!(
                    "{prefix}\"source\": {{ \"file\": {}, \"lines\": [{start}, {end}] }}{suffix}",
                    serde_json::to_string(file).expect("a string serializes"),
                )),
            }
        }

        fs::write(&catalogue, format!("{}\n", rewritten.join("\n")))
            .unwrap_or_else(|e| panic!("{catalogue_name} is writable: {e}"));

        // Re-read the now-anchored catalogue so the sidecar is keyed by id,
        // which only the parsed document knows — reusing the same walker that
        // finds every `source` block regardless of how deeply this catalogue
        // nests its entries (a flat array for virtues_flaws.json, a
        // `{ "catalogues": [ { "values": [...] } ] }` tree for parameter
        // catalogues).
        let reparsed_text = fs::read_to_string(&catalogue)
            .unwrap_or_else(|e| panic!("{catalogue_name} is readable: {e}"));
        let reparsed: Value = serde_json::from_str(&reparsed_text)
            .unwrap_or_else(|e| panic!("the rewritten {catalogue_name} is valid JSON: {e}"));
        let mut refs = Vec::new();
        collect_source_refs(catalogue_name, &reparsed, &mut refs);

        for found in &refs {
            // Unresolved this run (a reading is still pending) — no German
            // counterpart to derive either.
            if found.anchor.is_none() {
                continue;
            }
            let german_file = german_book.get(&found.source_file).unwrap_or_else(|| {
                panic!("no German counterpart recorded for {}", found.source_file)
            });
            let german_anchor =
                anchor_opening(&mut cache, "de", german_file, found.start as usize, level)
                    .unwrap_or_else(|| {
                        panic!(
                            "de/{german_file}:{} does not open a level-{level} heading",
                            found.start
                        )
                    });
            sidecar_rows.insert(
                found.entry_label.clone(),
                (german_file.clone(), german_anchor),
            );
        }
    }

    let body: Vec<String> = sidecar_rows
        .iter()
        .map(|(id, (file, anchor))| {
            format!(
                "  {}: {{ \"anchor\": {}, \"file\": {} }}",
                serde_json::to_string(id).expect("a string serializes"),
                serde_json::to_string(anchor).expect("a string serializes"),
                serde_json::to_string(file).expect("a string serializes"),
            )
        })
        .collect();
    fs::write(
        rules_dir().join("i18n/de").join(ANCHOR_SIDECAR),
        format!("{{\n{}\n}}\n", body.join(",\n")),
    )
    .expect("the German sidecar is writable");
}

/// **X9a-1 red.** Before generalising `anchor_opening` and Guard A to a
/// per-catalogue heading level, nothing in this file can recognise anything
/// but a `####` (level-4) heading. Spells are defined under a `#####`
/// (level-5) heading, so a level-5 catalogue entry needs a heading finder
/// that takes the level as a parameter instead of hardcoding 4.
#[test]
fn heading_opening_recognises_a_level_five_spell_heading() {
    let text = "#### Some Ability\n\n##### Piercing the Faerie Veil\n";
    let all = headings(text);
    let opening = heading_opening(&all, 3, 5);
    assert_eq!(
        opening.map(|heading| heading.anchor.as_str()),
        Some("piercing-the-faerie-veil")
    );
}

/// The heading at `line` whose level is exactly `level`, or `None` if that
/// line opens no heading or opens one at a different level. Pulled out of
/// `anchor_opening` so the level-matching rule is unit-testable without
/// reading a real file from `rules/source/`.
fn heading_opening(headings: &[Heading], line: usize, level: usize) -> Option<&Heading> {
    headings
        .iter()
        .find(|heading| heading.line == line && heading.level == level)
}

/// The anchor of the heading of the expected `level` that opens at `line` in
/// `rules/source/<lang>/<file>`, or `None` if that line is not a heading of
/// that level.
fn anchor_opening(
    cache: &mut BTreeMap<(String, String), Vec<Heading>>,
    lang: &str,
    file: &str,
    line: usize,
    level: usize,
) -> Option<String> {
    let found = headings_for(cache, lang, file);
    heading_opening(&found, line, level).map(|heading| heading.anchor.clone())
}

// --- The mechanic-vs-passage guard -----------------------------------------

/// The effect families this guard covers, each with the phrase(s) the
/// rulebook uses for that mechanic. An item encoding one of these effects
/// must cite a passage containing at least one of its phrases (matched
/// case-insensitively).
///
/// Deliberately a **subset** of `types.rs::Effect`, not all of it. These are
/// the families where the rulebook's own vocabulary is consistent enough for
/// the check to be crisp, and a guard that is loud on a few families beats one
/// that is noisy on thirty-five and grows an exemption list nobody reads. Add
/// a family here when its vocabulary is shown to be as consistent — by reading
/// the cited passages, not speculatively.
///
/// Each phrase list is the vocabulary the book actually uses for that mechanic,
/// established by opening the citations:
///
/// - `magic_total_halving` — "halve"/"half" ("must halve their Lab Total",
///   ArMDE:5964; "Halve your Lab Total", ArMDE:7062; "You halve the normal
///   Penetration Total", ArMDE:7066). Deliberately *not* widened to "divide":
///   that would silently
///   absorb Weak Spontaneous Magic's ÷5, whose exemption is worth having
///   written down, and would stop discriminating a halving from any other
///   division.
/// - `magic_resistance_mod` — "Magic Resistance" and "Parma Magica" are the
///   book's proper nouns for the mechanic and are never used loosely.
/// - `aging_mod` — the aging system's own named terms: the "aging
///   roll"/"aging table"/"Aging Crisis"/"aging points" family (which also
///   catches "Unaging" as a substring), the "Living Conditions" Modifier, the
///   "Decrepitude" track, and the "Longevity" Ritual. Bare "age" is
///   deliberately excluded — it is a substring of "damage", "village", and
///   "average", so it would match unrelated prose and make the family vacuous.
/// - `grants_reputation` — "Reputation" is a rulebook proper noun and this is
///   the catalogue's largest family. Every entry states it with that one word
///   ("a bad Reputation at 4", ArMDE:5677; "an Academic Reputation of 1",
///   ArMDE:3472; "a Local Reputation of 1", ArMDE:3478), so one phrase suffices
///   and anything broader would only add slack.
/// - `combat_mod` — the `CombatStat` names plus the book's own collective noun.
///   ArMDE:16656 fixes the taxonomy ("Characters have five combat scores:
///   Initiative, Attack, Defense, Damage, and Soak"), and the encodings split
///   between naming a total ("Attack rolls", ArMDE:6436; "+9 to your Initiative
///   Total", ArMDE:4313) and naming the group ("Dodge and other combat rolls",
///   ArMDE:6262; "-1 on other combat scores", ArMDE:6332). So "combat" is
///   load-bearing rather than slack: without it three correctly-encoded items
///   would be false positives. "defend" is listed beside "defense" because
///   ArMDE:6608 uses the verb ("rolls to attack and defend").
///
///   Two deliberate exclusions. **"damage"**, though a `CombatStat`, is left
///   out: no entry encodes that target today, and the word is ordinary English
///   throughout these books ("legs are severely damaged", ArMDE:6262), so it
///   would let a passage pass for a reason unrelated to the mechanic. The day a
///   `damage` encoding lands it will flag, and whoever adds it should read the
///   passage and widen this list deliberately — which is the review moment the
///   guard exists to force. **"soak"** is excluded because Soak is *not* in this
///   family: ArMDE:16656 groups it with the combat scores, but the engine models
///   it as its own `Effect::SoakMod`, and `virtue.berserk` shows the correct
///   shape by carrying `combat_mod` attack/defense *and* a separate `soak_mod`
///   off the same passage. Admitting "soak" here would let a Soak-only passage
///   validate a `combat_mod` encoding — precisely the mis-encoding to catch.
/// - `health_mod` — the health system's mechanical nouns, one per `HealthTrack`
///   axis: "fatigue" (the FatiguePenalty / FatigueRoll / CastingFatigue tracks),
///   "wound" (WoundPenalty), and "recover" (Recovery, stated as "rolls to
///   recover from wounds", ArMDE:6188 and ArMDE:4836). All three are sharp
///   mechanical terms here rather than loose prose. This family flagged nothing
///   on its first run — every entry states its own mechanic — so its liveness
///   was proved by mutation instead: repointing `flaw.obese`'s citation at an
///   unrelated passage (the Gabai social-status Flaw, ArMDE:6198-6201) makes the
///   guard fail. The mutation was reverted immediately.
const GUARDED_EFFECT_PHRASES: &[(&str, &[&str])] = &[
    ("lab_total_mod", &["lab total"]),
    ("casting_total_mod", &["casting total", "casting score"]),
    ("magic_total_halving", &["halve", "half"]),
    (
        "magic_resistance_mod",
        &["magic resistance", "parma magica"],
    ),
    (
        "aging_mod",
        &["aging", "living conditions", "decrepitude", "longevity"],
    ),
    ("grants_reputation", &["reputation"]),
    (
        "combat_mod",
        &["attack", "defense", "defend", "initiative", "combat"],
    ),
    ("health_mod", &["fatigue", "wound", "recover"]),
];

/// Encodings whose cited passage states the mechanic in different words, with
/// the reason the paraphrase is sound. Every row is a sentence somebody had
/// to mean: writing one down is the point of the exemption, not its cost.
const PARAPHRASE_EXEMPTIONS: &[(&str, &str, &str)] = &[
    (
        "flaw.poor_formulaic_magic",
        "casting_total_mod",
        "ArMDE:6612 says \"Subtract 5 from every roll that you make to cast Formulaic \
         spells. This does not apply to Ritual spells.\"; ArMDE:9103 defines FORMULAIC \
         CASTING TOTAL = Casting Score + Die Roll, so the roll the passage penalizes IS \
         the Formulaic Casting Total, and the excluded Rituals are exactly what the \
         effect's \"formulaic\" scope excludes. A wording difference, not a mechanic \
         difference.",
    ),
    (
        "flaw.weak_spontaneous_magic",
        "magic_total_halving",
        "ArMDE:7086 says \"you always divide your Casting Score by five\" — a division of \
         a whole in-play total, which is what this family encodes, stated with the \
         divisor spelled out instead of the word \"halve\". The family name says \
         *halving* but the variant it carries is the discriminator: \
         `HalvableTotal::SpontaneousCasting` is read by `derived/casting.rs::casting_totals` \
         as \"the fatiguing (exert-yourself) option does not exist\", so the fatiguing slot \
         reports the div_euclid(5) figure rather than halve(base). The encoded behaviour is \
         the passage's own /5, not a /2 — verified at the consumption site. A wording \
         difference, not a mechanic difference.",
    ),
    (
        "flaw.palsied_hands",
        "combat_mod",
        "ArMDE:6580 states the mechanic in terms of the totals' shared *input* instead of their \
         names: \"All rolls involving holding or wielding an object are made at -2, including \
         weapon skills.\" The book's own formulas are what resolve that to this family: \
         ArMDE:16660 gives ATTACK TOTAL = Dexterity + Combat Ability + Weapon Attack Modifier + \
         Stress Die and ArMDE:16662 gives DEFENSE TOTAL = Quickness + Combat Ability + Weapon \
         Defense Modifier + Stress Die. The weapon skill (Combat Ability) is a term of both and \
         of no other combat total, so a flat -2 on it is -2 Attack and -2 Defense and nothing \
         else — which is exactly the pair encoded, and is why Initiative (which takes no Combat \
         Ability) is correctly absent. A wording difference, not a mechanic difference.",
    ),
    (
        "virtue.magister_in_medicina",
        "grants_reputation",
        "ArMDE:4397 states the mechanic by *delegation* rather than by wording: \"This Virtue \
         offers the same benefits as Doctor in (Faculty).\" The referent, ArMDE:3687, says \"He \
         also begins the game with an Academic Reputation of 3\" — and the two entries' effects \
         are identical (academic/3 plus 300 restricted academic XP), so the delegation is \
         honoured in full rather than approximated. The citation is deliberately NOT repointed \
         at ArMDE:3683-3698: `source` records where the item is *defined*, and this Virtue is \
         defined at :4395. A wording difference, not a mechanic difference.",
    ),
    (
        "virtue.bee_king",
        "aging_mod",
        "ArMDE:3488 states the mechanic in plain English instead of the system's noun: \
         \"Bee Kings do not appear to age after reaching maturity\". That sentence IS \
         `AgingEffect::NoApparentAging` (\"the apparent age never advances\"), and it is \
         the passage `types.rs::AgingEffect::NoApparentAging` already cites. The entry \
         carries that kind alone and correctly does *not* carry `NoAging` — the Bee King \
         still loses Characteristics, which is the distinction M6/6b6 split the two \
         immunities apart to preserve. The family's other 16 encodings all use the book's \
         \"aging\"/\"Living Conditions\"/\"Decrepitude\"/\"Longevity\" vocabulary; this one \
         narrative sentence is the sole plain-English statement of an aging immunity.",
    ),
];

/// Encodings the guard flags that are **genuinely wrong** and are not yet fixed,
/// because the correct mechanic has no `types.rs::Effect` representation and
/// inventing one is exactly what `CLAUDE.md` → "Rules provenance" forbids
/// ("Implementing a rule from training-data recollection is prohibited"). Each
/// row is a recorded wrong-output defect with an open row in `docs/open-todos.md`
/// — parked in the open, never silently absolved as a paraphrase.
///
/// This list is deliberately separate from [`PARAPHRASE_EXEMPTIONS`]: conflating
/// "the book says it differently" with "the book says something else entirely" is
/// how a guard quietly becomes decorative. A row here is an admission of a bug,
/// not a justification.
///
/// It also cannot rot: [`known_misencodings_still_fail_the_guard`] asserts every
/// row still fails the phrase check, so the day one is genuinely fixed the test
/// tells you to delete its row instead of leaving a stale excuse behind.
///
/// **Empty is the goal state, and it is the current one.** The two rows this list
/// was created for — `flaw.weak_magic_resistance` (halving a Magic Resistance
/// `ArMDE:7070` never halves) and `flaw.susceptibility_to_divine_power` (filed
/// under Magic Resistance though `ArMDE:6817` never mentions it) — were fixed
/// rather than kept: both are now encoded against the mechanic their own passage
/// states, and `known_misencodings_still_fail_the_guard` is what forced their
/// removal from here. Keep the list (and its test) for the next parked defect; do
/// not delete the mechanism because it currently has nothing to hold.
const KNOWN_MISENCODINGS: &[(&str, &str, &str)] = &[];

/// The minimum number of items each guarded family must actually match. A
/// floor, never a total — `CLAUDE.md` → "Catalogue size is data, never code"
/// forbids asserting exact catalogue counts, but without *some* floor this
/// test passes vacuously the day a family is renamed and matches nothing.
///
/// Lowered from 5 to 4 when `flaw.flawed_parma_magica` left `magic_total_halving`
/// for `magic_resistance_mod`: it halves one *addend* of Magic Resistance (the
/// Parma contribution) against one *Form*, which is not the "halve a whole
/// in-play total" this family means. That leaves the family with the four
/// genuine whole-total halvings the catalogue states (spontaneous casting, lab
/// enchanting, lab longevity, penetration), so the floor follows the data down
/// rather than the data being padded up to the floor.
const MIN_ITEMS_PER_GUARDED_FAMILY: usize = 4;

/// Reads the lines a citation brackets, lowercased and joined, caching each
/// source file so the guard reads a multi-megabyte rulebook once rather than
/// once per citation.
fn bracketed_passage(
    source_dir: &Path,
    cache: &mut BTreeMap<String, Vec<String>>,
    found: &FoundSourceRef,
) -> Option<String> {
    let lines = cache.entry(found.source_file.clone()).or_insert_with(|| {
        fs::read_to_string(source_dir.join(&found.source_file))
            .map(|text| text.lines().map(str::to_lowercase).collect())
            .unwrap_or_default()
    });
    if found.start < 1 || found.end < found.start || found.end as usize > lines.len() {
        // Malformed or out-of-bounds ranges are the other test's finding; do
        // not report the same citation twice.
        return None;
    }
    Some(lines[(found.start as usize - 1)..(found.end as usize)].join("\n"))
}

/// The cross-site invariant: **every effect encoded against a guarded family
/// must cite a passage that actually talks about that family's mechanic.**
///
/// `every_core_source_citation_brackets_real_content_in_its_named_file` proves
/// a citation points somewhere real. It cannot prove it points at a passage
/// that *supports the mechanic encoded beside it* — and that is the property
/// `CLAUDE.md` → "Rules provenance" actually demands ("Every rule … encoded as
/// data MUST be taken from the authoritative Markdown … Implementing a rule
/// from training-data recollection is prohibited"). JSON carries no comments,
/// so there is no second place the claim is written down and could be
/// cross-read.
///
/// The failure mode it exists for: a slice encodes a Virtue or Flaw against
/// the nearest available `types.rs::Effect` variant because no exact one
/// exists. Every gate stays green — the JSON parses, referential integrity
/// holds, the citation is in bounds and non-blank — and the wrong number then
/// reaches every derived read-out that consumes that family, on every
/// character carrying the item, in both locales. This test is the only thing
/// that can see it without a human reading the rulebook beside the JSON.
#[test]
fn every_guarded_effect_cites_a_passage_that_names_its_own_mechanic() {
    let source_dir = rules_dir().join("source/en");
    let all_refs = all_source_refs();
    let mut cache: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut matched: BTreeMap<&str, usize> = GUARDED_EFFECT_PHRASES
        .iter()
        .map(|(kind, _)| (*kind, 0))
        .collect();
    let mut errors = Vec::new();

    for found in &all_refs {
        for kind in &found.effect_kinds {
            let Some((_, phrases)) = GUARDED_EFFECT_PHRASES
                .iter()
                .find(|(guarded, _)| guarded == kind)
            else {
                continue;
            };
            *matched.get_mut(kind.as_str()).expect("family is seeded") += 1;

            if PARAPHRASE_EXEMPTIONS
                .iter()
                .chain(KNOWN_MISENCODINGS)
                .any(|(id, exempt_kind, _)| *id == found.entry_label && exempt_kind == kind)
            {
                continue;
            }
            let Some(passage) = bracketed_passage(&source_dir, &mut cache, found) else {
                continue;
            };
            if !phrases.iter().any(|phrase| passage.contains(phrase)) {
                errors.push(format!(
                    "{}: \"{}\" encodes a {kind} effect, but the passage it cites \
                     ({}:{}-{}) never says {}. Either the effect is encoded against the \
                     wrong mechanic, or the citation points at the wrong passage, or the \
                     passage paraphrases the mechanic — in which case add a row to \
                     PARAPHRASE_EXEMPTIONS stating why.",
                    found.core_file,
                    found.entry_label,
                    found.source_file,
                    found.start,
                    found.end,
                    phrases
                        .iter()
                        .map(|p| format!("\"{p}\""))
                        .collect::<Vec<_>>()
                        .join(" or "),
                ));
            }
        }
    }

    for (kind, count) in &matched {
        assert!(
            *count >= MIN_ITEMS_PER_GUARDED_FAMILY,
            "the {kind} guard matched only {count} item(s) in rules/core/*.json — below the \
             floor of {MIN_ITEMS_PER_GUARDED_FAMILY}, so this test would pass without having \
             checked anything. Was the effect kind renamed?"
        );
    }

    assert!(
        errors.is_empty(),
        "{} effect(s) in rules/core/*.json are encoded against a mechanic their own cited \
         passage does not state:\n{}",
        errors.len(),
        errors.join("\n")
    );
}

/// [`KNOWN_MISENCODINGS`] is a list of parked bugs, so every row must still *be*
/// a bug. If one starts passing the phrase check — because the encoding was
/// corrected, the citation repointed, or the family's phrase list widened — the
/// row has become a stale excuse that silences a guard over nothing. Fail then,
/// so the fix ends with the row deleted rather than outliving the defect.
#[test]
fn known_misencodings_still_fail_the_guard() {
    let source_dir = rules_dir().join("source/en");
    let all_refs = all_source_refs();
    let mut cache: BTreeMap<String, Vec<String>> = BTreeMap::new();

    for (id, kind, _) in KNOWN_MISENCODINGS {
        let phrases = GUARDED_EFFECT_PHRASES
            .iter()
            .find(|(guarded, _)| guarded == kind)
            .map(|(_, phrases)| *phrases)
            .unwrap_or_else(|| {
                panic!(
                    "KNOWN_MISENCODINGS row \"{id}\" names {kind}, which no longer appears in \
                        GUARDED_EFFECT_PHRASES — the guard it silences is gone, so delete the row"
                )
            });

        let found = all_refs
            .iter()
            .find(|found| found.entry_label == *id && found.effect_kinds.iter().any(|k| k == kind))
            .unwrap_or_else(|| {
                panic!(
                    "KNOWN_MISENCODINGS row \"{id}\" claims a {kind} effect that no longer \
                        exists in rules/core/*.json — the defect was fixed, so delete the row"
                )
            });

        let passage = bracketed_passage(&source_dir, &mut cache, found)
            .unwrap_or_else(|| panic!("\"{id}\" cites an out-of-bounds range"));
        assert!(
            !phrases.iter().any(|phrase| passage.contains(phrase)),
            "KNOWN_MISENCODINGS row \"{id}\" ({kind}) now PASSES the phrase check — the cited \
             passage does name the mechanic. The parked defect is resolved (or the citation \
             moved), so delete the row and let the guard cover it normally."
        );
    }
}
