//! German-name guard for Virtues and Flaws (data-integrity, PHASE 1 — test +
//! triage only, see `tmp/de-names-handover.md`): every
//! `rules/core/virtues_flaws.json` entry's shipped DE name
//! (`rules/i18n/de/virtues_flaws.json`) must match the translation tables'
//! `Deutsch (DE)` value for its EN name (`rules/i18n/en/virtues_flaws.json`),
//! per CLAUDE.md "German translation tables" and
//! `docs/vf-audit/decisions.md` D6/D7/D18/D31/D65/D71/D86.
//!
//! No `rules/core/`, `rules/i18n/`, or `rules/source/` change lands in this
//! phase. [`RULED_EXCEPTIONS`] is the only departure from a literal
//! name-vs-table comparison (same shape as
//! `vf_descriptor_line.rs::RULED_EXCEPTIONS`: a row records a human reading,
//! not a license, and [`ruled_exceptions_still_mismatch`] asserts every row
//! still fails the plain comparison, so the list can only shrink).
//!
//! ## Precedence (CLAUDE.md "German translation tables"; D6, D7, D31, D86)
//!
//! **D86 (P8-6): D31 retires.** D31 made the table win on a NAME because the
//! `rules/source/de/` rulebook copy was then older than the tables. The P8
//! re-sync landed a current rulebook, so D7.1's original rule is back in
//! force: the rulebook heading wins by default, and the table wins only
//! where the heading is defective (collides with another entry's heading, or
//! never renders the term at all). This file does not parse the rulebook
//! heading programmatically — that comparison was done by hand, entry by
//! entry, against the current `rules/source/de/` file (see
//! `tmp/p8-6-names.md`) — so a heading-wins verdict shows up here either as a
//! corrected `rules/i18n/de/` name (so the plain table comparison below now
//! agrees too, needing no exception) or, where the table still disagrees for
//! a reason worth recording, a [`RULED_EXCEPTIONS`] row. D71's
//! `FORCED_COVERAGE_DESPITE_TAG` mechanism — which forced three `SdM:M`-tagged
//! rows into the table dispute despite precedence rule 4, back when the table
//! was the only current source — is retired along with it: those three now
//! resolve by rule 1 below, uncontested, exactly like any other tagged row.
//!
//! 1. A table row tagged to a book OTHER than the one an entry is sourced
//!    from is not in the dispute (precedence rule 4) — every entry in
//!    `rules/core/virtues_flaws.json` cites only the core rulebook (`ArMDE`),
//!    so a row tagged
//!    `HdH:WL`/`SdM:M`/`SdM:G`/`SdM:F`/`SdM:I`/`HM:RE`/`DM:ÜA`/`GotF` is
//!    always skipped.
//! 2. Among untagged rows, a thematic table wins over `tugenden-fehler.md`
//!    (precedence rule 3 — the broadest table is the likeliest to be wrong).
//!    "A thematic table" is narrowed to [`VF_RELEVANT_THEMATIC_FILES`], not
//!    every other file: most of the other ~19 files document a different
//!    catalogue entirely (creature powers, spell names, combat/unit/lab
//!    terms), so a same-spelled row there is homonymy, not a cross-reference
//!    — see that constant's doc comment for the concrete collision
//!    (`Delusion`) that proved this narrowing necessary.
//! 3. [`RULED_EXCEPTIONS`] overrides the plain comparison for a name
//!    disagreement that a specific ruling has already closed (D7's and D86's
//!    rulebook-heading-wins cases).
//!
//! ## Normalisations (see `tmp/de-names-handover.md` for the full rationale)
//!
//! A bracketed region is NOT always a parameter placeholder to erase: German
//! capitalises every noun, so `(Klein)`/`(Groß)`/`(Positiv)` (real,
//! name-distinguishing content — e.g. two different Reputation-level
//! entries, or `Cyclic Magic (positive)` vs `Cyclic Magic (negative)`, two
//! DIFFERENT catalogue entries) are exactly as capitalised as a genuine
//! category placeholder like `(Ability)`/`(Fertigkeit)`. Case alone cannot
//! tell them apart, so this guard uses a narrow, evidence-based whitelist
//! ([`PLACEHOLDER_WORDS`]) of the actual category words the catalogue's own
//! parameter keys and the tables' placeholder columns use, and strips a
//! bracketed region ONLY when its content is one of those words. Everything
//! else in parentheses is preserved verbatim, so it keeps distinguishing the
//! entries it was already distinguishing.
//!
//! - `{snake_case}` (our own shipped placeholder syntax) is always a
//!   placeholder; replaced with a single marker token so two different
//!   fixed-text neighbourhoods never collapse onto the same key (seen with
//!   `Poor {characteristic}`, which must NOT collapse to the unrelated Flaw
//!   `Poor`).
//! - A `(` directly wrapping a `{snake_case}` token, immediately closed by
//!   `)` — i.e. the shipped name itself writes the placeholder as
//!   `({power})` — is deleted ENTIRELY (not just marked), because several
//!   table rows (the `Power` family, `Social Contacts`) show the whole
//!   parenthetical dropped in the bare form. This is a structural signal
//!   (adjacent bracket characters), never a word-content guess, so it cannot
//!   create the same collision risk as deleting on content.
//! - A table row that spells its placeholder as a bare category word with NO
//!   brackets at all (`Puissant Ability`, `Affinity with Ability`, `Form
//!   Monstrosity`) is matched by a second EN lookup candidate: literally
//!   substituting our own parameter key's capitalised form into the shipped
//!   EN name. Narrow by construction — it only ever touches a name that
//!   itself contains `{some_key}`, using that exact key, so it cannot
//!   manufacture a collision between two unrelated entries. (It is NOT
//!   applied to the DE side too: the parameter key is an English word, so
//!   there is no German spelling to substitute. `virtue.great_characteristic`
//!   is the one entry this leaves as a genuine comparison gap — PARSER GAP,
//!   see the handover.)
//! - The table's `(Lat.)` tag (`Custos (Lat.)`) is removed by the same
//!   whitelist rule (`lat` is in [`PLACEHOLDER_WORDS`]) — no separate rule is
//!   needed for it.

use serde_json::Value;
use std::collections::BTreeMap;

const VF_JSON: &str = include_str!("../../../rules/core/virtues_flaws.json");
const EN_I18N: &str = include_str!("../../../rules/i18n/en/virtues_flaws.json");
const DE_I18N: &str = include_str!("../../../rules/i18n/de/virtues_flaws.json");

/// Every translation-table file that carries `Englisch (EN)` / `Deutsch (DE)`
/// glossary rows. `README.md` (prose + a differently-shaped correction log)
/// and `uebersetzungsregeln.md` (prose only, no glossary) are deliberately
/// excluded — confirmed empty of that header by grep before this list was
/// written.
const TABLE_FILES: &[(&str, &str)] = &[
    (
        "alterung-twilight.md",
        include_str!("../../../rules/source/de/translation-tables/alterung-twilight.md"),
    ),
    (
        "fertigkeiten.md",
        include_str!("../../../rules/source/de/translation-tables/fertigkeiten.md"),
    ),
    (
        "goettliche-kraefte.md",
        include_str!("../../../rules/source/de/translation-tables/goettliche-kraefte.md"),
    ),
    (
        "grundbegriffe.md",
        include_str!("../../../rules/source/de/translation-tables/grundbegriffe.md"),
    ),
    (
        "infernale-kraefte.md",
        include_str!("../../../rules/source/de/translation-tables/infernale-kraefte.md"),
    ),
    (
        "islamische-begriffe.md",
        include_str!("../../../rules/source/de/translation-tables/islamische-begriffe.md"),
    ),
    (
        "juedische-begriffe.md",
        include_str!("../../../rules/source/de/translation-tables/juedische-begriffe.md"),
    ),
    (
        "kampf.md",
        include_str!("../../../rules/source/de/translation-tables/kampf.md"),
    ),
    (
        "konvent-boons-hooks.md",
        include_str!("../../../rules/source/de/translation-tables/konvent-boons-hooks.md"),
    ),
    (
        "konvent.md",
        include_str!("../../../rules/source/de/translation-tables/konvent.md"),
    ),
    (
        "kreaturenkraefte.md",
        include_str!("../../../rules/source/de/translation-tables/kreaturenkraefte.md"),
    ),
    (
        "labor-fortschritt.md",
        include_str!("../../../rules/source/de/translation-tables/labor-fortschritt.md"),
    ),
    (
        "magie-regeln.md",
        include_str!("../../../rules/source/de/translation-tables/magie-regeln.md"),
    ),
    (
        "magische-qualitaeten.md",
        include_str!("../../../rules/source/de/translation-tables/magische-qualitaeten.md"),
    ),
    (
        "masseinheiten.md",
        include_str!("../../../rules/source/de/translation-tables/masseinheiten.md"),
    ),
    (
        "orden-tribunale.md",
        include_str!("../../../rules/source/de/translation-tables/orden-tribunale.md"),
    ),
    (
        "persoenlichkeitseigenschaften.md",
        include_str!(
            "../../../rules/source/de/translation-tables/persoenlichkeitseigenschaften.md"
        ),
    ),
    (
        "reputationen.md",
        include_str!("../../../rules/source/de/translation-tables/reputationen.md"),
    ),
    (
        "sphären-mächte.md",
        include_str!("../../../rules/source/de/translation-tables/sphären-mächte.md"),
    ),
    (
        "tiere-kreaturen.md",
        include_str!("../../../rules/source/de/translation-tables/tiere-kreaturen.md"),
    ),
    (
        "tugenden-fehler.md",
        include_str!("../../../rules/source/de/translation-tables/tugenden-fehler.md"),
    ),
    (
        "zauber-nach-form.md",
        include_str!("../../../rules/source/de/translation-tables/zauber-nach-form.md"),
    ),
];

/// Every book acronym the translation tables use to tag a row to a
/// supplement OTHER than the core rulebook (see
/// `rules/source/de/translation-tables/README.md` "Rangfolge bei
/// Widerspruch" rule 4). Distinct from the English `ArMDE`/`HoH:TL`/…
/// acronyms `CLAUDE.md` defines for Rust source citations — these are the
/// translation tables' own German-book abbreviations.
const BOOK_ACRONYMS: &[&str] = &[
    "HdH:WL", "HdH:TL", "HdH:MC", "HdH:S", "SdM:M", "SdM:G", "SdM:F", "SdM:I", "HM:RE", "DM:ÜA",
    "GotF", "GdW",
];

#[derive(Debug, Clone)]
struct TableRow {
    file: &'static str,
    /// [`match_key`] of the row's `Englisch (EN)` cell, computed once per row:
    /// [`find_match`] compares it against every catalogue entry, so
    /// recomputing it per (entry, row) pair cost seconds.
    en_key: String,
    de: String,
    book_tag: Option<&'static str>,
}

/// A note is a book tag only if the acronym sits at the very start of the
/// note AND is not immediately followed by `-Hinweis` — five rows in
/// `tugenden-fehler.md` (`Giant Blood`, `Large`, `Age Quickly`,
/// `Poor Concentration`, `Small Frame`) start their note with
/// `SdM:M-Hinweis:`, an explanatory aside about how that supplement treats a
/// CORE entry differently, not a claim that the row belongs to that
/// supplement.
fn inline_tag(note: &str) -> Option<&'static str> {
    BOOK_ACRONYMS.iter().copied().find(|acr| {
        note.strip_prefix(acr)
            .is_some_and(|rest| !rest.starts_with("-Hinweis"))
    })
}

/// A `### Ergänzungen aus <Book> (ACRONYM)` heading tags every row under it
/// (until the next level-1/2/3 heading) to that supplement. A heading that
/// says "Ergänzungen" but names no acronym (`### Ergänzungen aus Basisregeln
/// (Feenwesen-Statblöcke)`) is core-book material presented out of the main
/// list, not a supplement — it returns `None`, same as a heading with no
/// "Ergänzungen" at all.
fn heading_book_tag(heading_text: &str) -> Option<&'static str> {
    if !heading_text.contains("Ergänzungen") {
        return None;
    }
    BOOK_ACRONYMS
        .iter()
        .copied()
        .find(|acr| heading_text.contains(acr))
}

/// Splits a markdown table row `| a | b | c |` into trimmed cells, dropping
/// the leading/trailing empty cells the outer `|`s produce.
fn row_cells(line: &str) -> Vec<&str> {
    line.trim()
        .trim_matches('|')
        .split('|')
        .map(|c| c.trim())
        .collect()
}

fn is_separator_row(cells: &[&str]) -> bool {
    cells
        .iter()
        .all(|c| !c.is_empty() && c.chars().all(|ch| ch == '-' || ch == ':'))
}

/// Parses every `Englisch (EN)` / `Deutsch (DE)` / `Anmerkung`-shaped table in
/// one file. Column positions are read from each table's own header (not
/// hardcoded), because `grundbegriffe.md` alone uses three different column
/// layouts (a 7-column Latin-declension table, a 4-column abbreviation
/// table, and the common 3-column form) for its glossary rows.
fn parse_file(file: &'static str, text: &str) -> Vec<TableRow> {
    let mut rows = Vec::new();
    let mut section_tag: Option<&'static str> = None;
    let mut de_idx: Option<usize> = None;
    let mut note_idx: Option<usize> = None;

    for line in text.lines() {
        let trimmed = line.trim();

        if trimmed.is_empty() {
            de_idx = None;
            note_idx = None;
            continue;
        }

        if let Some(rest) = trimmed.strip_prefix('#') {
            let level = 1 + rest.chars().take_while(|&c| c == '#').count();
            let heading_text = trimmed.trim_start_matches('#').trim();
            if level == 3 {
                section_tag = heading_book_tag(heading_text);
            } else if level <= 2 {
                section_tag = None;
            } // level >= 4: subsection of the current (possibly tagged) block, leave section_tag alone
            de_idx = None;
            note_idx = None;
            continue;
        }

        if !trimmed.starts_with('|') {
            de_idx = None;
            note_idx = None;
            continue;
        }

        let cells = row_cells(trimmed);
        if cells.first() == Some(&"Englisch (EN)") {
            de_idx = cells.iter().position(|c| *c == "Deutsch (DE)");
            note_idx = cells.iter().position(|c| *c == "Anmerkung");
            continue;
        }

        let Some(de_col) = de_idx else { continue };
        if is_separator_row(&cells) {
            continue;
        }
        let en = cells.first().copied().unwrap_or("");
        if en.is_empty() {
            continue;
        }
        let de = cells.get(de_col).copied().unwrap_or("").to_string();
        let note = note_idx.and_then(|i| cells.get(i)).copied().unwrap_or("");
        let book_tag = section_tag.or_else(|| inline_tag(note));
        rows.push(TableRow {
            file,
            en_key: match_key(en),
            de,
            book_tag,
        });
    }
    rows
}

fn all_rows() -> Vec<TableRow> {
    TABLE_FILES
        .iter()
        .flat_map(|(file, text)| parse_file(file, text))
        .collect()
}

/// The actual vocabulary of category/parameter words the catalogue and the
/// tables use inside a placeholder bracket — see the module doc comment for
/// why this must be a whitelist rather than "any bracketed content".
/// Evidence-based: every entry is a word observed wrapped in `(...)` in
/// either `rules/i18n/*/virtues_flaws.json`'s own parameter keys or a
/// translation-table row. Deliberately EXCLUDES a German word for "power"
/// (`Macht`/`Kraft`): the `{power}` family (`Deteriorating`/`Restricted`/
/// `Slow`/`Variable Power`) is handled by the structural
/// parens-around-curly rule instead, and whitelisting `macht`/`kraft` as
/// content-based words would risk silently equating two DIFFERENT German
/// words for "power" — exactly the kind of terminology error this guard
/// exists to catch (see `flaw.deteriorating_power`'s real, surfaced
/// mismatch).
const PLACEHOLDER_WORDS: &[&str] = &[
    "ability",
    "art",
    "form",
    "being",
    "beings",
    "land",
    "realm",
    "characteristic",
    "subject",
    "sense",
    "role",
    "virtue",
    "terrain",
    "faculty",
    "medium",
    "hazard",
    "commodity",
    "condition",
    "sin",
    "category",
    "social group",
    "technique",
    "lat",
    "fertigkeit",
    "kunst",
    "sphäre",
    "wesen",
    "fach",
    "fakultät",
    "sinn",
    "eigenschaft",
];

fn is_placeholder_word(inner: &str) -> bool {
    let cleaned = inner
        .trim()
        .trim_end_matches('-')
        .trim_end_matches('.')
        .to_lowercase();
    PLACEHOLDER_WORDS.contains(&cleaned.as_str())
}

/// Deletes an EXACT `(` + `{snake_case}` + `}` + `)` span entirely (both
/// brackets and the key), leaving no marker. Purely structural — triggered
/// only by adjacent bracket characters, never by inspecting word content —
/// so it is safe from the collision risk a content-based rule would carry.
/// See the module doc comment.
fn delete_parenthesized_curly(s: &str) -> String {
    let chars: Vec<char> = s.chars().collect();
    let mut out = String::new();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '('
            && chars.get(i + 1) == Some(&'{')
            && let Some(close_brace) = (i + 1..chars.len()).find(|&j| chars[j] == '}')
            && chars.get(close_brace + 1) == Some(&')')
        {
            i = close_brace + 2;
            continue;
        }
        out.push(chars[i]);
        i += 1;
    }
    out
}

/// Replaces every remaining bare `{snake_case}` token with a single marker
/// character (never deleted to nothing — see the module doc comment on why
/// `Poor {characteristic}` must not collapse onto the unrelated Flaw
/// `Poor`).
fn curly_to_marker(s: &str) -> String {
    let mut out = String::new();
    let mut in_curly = false;
    for ch in s.chars() {
        match ch {
            '{' => {
                in_curly = true;
                out.push('@');
            }
            '}' => in_curly = false,
            _ if in_curly => {}
            _ => out.push(ch),
        }
    }
    out
}

/// Strips a `(...)` group to a single marker ONLY when its content is a
/// known placeholder word ([`PLACEHOLDER_WORDS`]); any other parenthetical
/// (`(Klein)`, `(positive)`, `(Anruth)`, an unclosed stray `(`) is kept
/// verbatim, since it is real distinguishing content, not a placeholder.
fn strip_whitelisted_parens(s: &str) -> String {
    let mut out = String::new();
    let mut chars = s.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '(' {
            let mut inner = String::new();
            let mut closed = false;
            for c2 in chars.by_ref() {
                if c2 == ')' {
                    closed = true;
                    break;
                }
                inner.push(c2);
            }
            if closed && is_placeholder_word(&inner) {
                out.push('@');
            } else {
                out.push('(');
                out.push_str(&inner);
                if closed {
                    out.push(')');
                }
            }
        } else {
            out.push(ch);
        }
    }
    out
}

fn collapse_ws(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Deletes every `{...}` group entirely (no marker).
fn delete_curly(s: &str) -> String {
    let mut out = String::new();
    let mut in_curly = false;
    for ch in s.chars() {
        match ch {
            '{' => in_curly = true,
            '}' => in_curly = false,
            _ if in_curly => {}
            _ => out.push(ch),
        }
    }
    out
}

/// Deletes a `(...)` group entirely (no marker) when its content is empty
/// (a placeholder shell hollowed out by [`delete_curly`] already running,
/// e.g. `({power})` -> `()`) or a known placeholder word
/// ([`PLACEHOLDER_WORDS`], e.g. the `(Lat.)` tag); any other parenthetical
/// (`(Klein)`, `(positive)`, `(Anruth)`) is kept verbatim. Unlike
/// [`strip_whitelisted_parens`], this deletes rather than marks — safe here
/// because [`normalize_de`] only ever compares ONE already-identified pair
/// (the EN lookup, which DOES carry a cross-entry collision risk, already
/// happened and used the marker-preserving form instead).
fn delete_whitelisted_parens(s: &str) -> String {
    let mut out = String::new();
    let mut chars = s.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '(' {
            let mut inner = String::new();
            let mut closed = false;
            for c2 in chars.by_ref() {
                if c2 == ')' {
                    closed = true;
                    break;
                }
                inner.push(c2);
            }
            if closed && (inner.trim().is_empty() || is_placeholder_word(&inner)) {
                // deleted — push nothing
            } else {
                out.push('(');
                out.push_str(&inner);
                if closed {
                    out.push(')');
                }
            }
        } else {
            out.push(ch);
        }
    }
    out
}

/// The primary normalisation used for both the EN lookup key and the DE
/// equality check: delete a parens-wrapped curly placeholder structurally,
/// mark any remaining bare curly placeholder, strip a whitelisted-word
/// parenthetical, collapse whitespace. Case-preserving (callers lowercase it
/// themselves for the case-insensitive EN lookup).
fn marker_form(s: &str) -> String {
    let step1 = delete_parenthesized_curly(s);
    let step2 = curly_to_marker(&step1);
    let step3 = strip_whitelisted_parens(&step2);
    collapse_ws(&step3)
}

/// Capitalises a `snake_case` parameter key into the table's `Title Case`
/// word spelling: `social_group` -> `Social Group`.
fn capitalize_key(key: &str) -> String {
    key.split('_')
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Replaces every `{snake_case}` token with its capitalised literal form,
/// leaving everything else (including any surrounding brackets) untouched.
/// Feeds the second EN lookup candidate for a table row that spells the
/// placeholder as a bare word with no brackets at all (`Puissant Ability`) —
/// see the module doc comment.
fn literal_substituted(name: &str) -> String {
    let mut out = String::new();
    let mut chars = name.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '{' {
            let mut key = String::new();
            for c2 in chars.by_ref() {
                if c2 == '}' {
                    break;
                }
                key.push(c2);
            }
            out.push_str(&capitalize_key(&key));
        } else {
            out.push(ch);
        }
    }
    out
}

/// The bracketed magnitude a split "Minor or Major" entry carries in its
/// display name (D84.3: "Ambitious [Major]", „Ehrgeizig [Klein]").
const BRACKETED_MAGNITUDES: &[&str] = &["[Minor]", "[Major]", "[Klein]", "[Groß]"];

/// Removes a bracketed magnitude (and the space before it), so a split
/// entry is compared with the table's base-name row: "Ambitious [Major]" ->
/// "Ambitious", "False Power [Minor]: {virtue}" -> "False Power: {virtue}".
/// A separate step on purpose, NOT a [`PLACEHOLDER_WORDS`] entry: the
/// magnitude is a display convention, not a parameter, and a parenthesised
/// `(Klein)` keeps distinguishing what it distinguished before.
fn strip_bracketed_magnitude(s: &str) -> String {
    BRACKETED_MAGNITUDES
        .iter()
        .fold(s.to_string(), |acc, bracket| {
            acc.replace(&format!(" {bracket}"), "").replace(bracket, "")
        })
}

fn match_key(s: &str) -> String {
    marker_form(&strip_bracketed_magnitude(s)).to_lowercase()
}

fn literal_match_key(s: &str) -> String {
    let base = strip_bracketed_magnitude(s);
    collapse_ws(&strip_whitelisted_parens(&literal_substituted(&base))).to_lowercase()
}

/// The DE equality check, used only AFTER [`find_match`] has already
/// identified the one table row that corresponds to an entry. Deletes a
/// placeholder entirely (curly, or a whitelisted-word/empty parenthetical)
/// rather than leaving [`marker_form`]'s marker — safe here because this is a
/// 1:1 confirmation of an already-selected pair, not a many-row lookup, so it
/// does not carry the collision risk that made the EN lookup key
/// marker-preserving (`Poor {characteristic}` vs the unrelated Flaw `Poor`).
fn normalize_de(s: &str) -> String {
    let base = strip_bracketed_magnitude(s);
    let step1 = delete_curly(&base);
    let step2 = delete_whitelisted_parens(&step1);
    collapse_ws(&step2)
}

#[derive(Debug)]
enum Match<'a> {
    Uncovered,
    Found(&'a str),
    /// Distinct `(file, DE value)` candidates that all matched the same EN
    /// key — a genuine ambiguity (duplicate/contradicting table rows), not a
    /// data defect in `rules/i18n/de/`.
    Ambiguous(Vec<(&'static str, &'a str)>),
}

/// The only OTHER file (besides `tugenden-fehler.md` itself) this guard
/// treats as a genuine source of Virtue/Flaw names, confirmed by cross-
/// checking actual matches: `grundbegriffe.md` ("General Game Terms") is
/// where `Custos`, `Hobbled`, `Primogeniture Lineage`, `Alluring to
/// (Beings)`, `Vulnerable Magic`, and `Vulnerable to Folk Tradition` live
/// (all confirmed real entries in `rules/core/virtues_flaws.json`).
///
/// Every one of the other ~19 translation-table files documents a DIFFERENT
/// catalogue entirely — creature powers (`infernale-kraefte.md`,
/// `goettliche-kraefte.md`, `kreaturenkraefte.md`), spell names
/// (`zauber-nach-form.md`), combat/units/lab terms, religious vocabulary — so
/// an English-word match there is coincidental homonymy, not a genuine
/// cross-reference. This was caught concretely: `infernale-kraefte.md` has
/// an untagged `Delusion -> Verblendung` row (a demonic Corruption power from
/// Realms of Power: The Infernal), which would otherwise SILENTLY outrank
/// the real Personality Flaw `flaw.delusion`'s own translation with no
/// ambiguity signal at all (unlike the `reputationen.md` collisions, which
/// at least surface as [`Match::Ambiguous`] because that file lists the same
/// name at two magnitudes). `reputationen.md` is excluded for the same
/// reason once that pattern was visible: none of its matches were ever a
/// clean, unambiguous `Found`, only ever `Ambiguous` (`Outlaw`, `Hermetic
/// Prestige`, `Senior Bard`) — it documents EXAMPLE reputations, not a V/F
/// name glossary.
const VF_RELEVANT_THEMATIC_FILES: &[&str] = &["grundbegriffe.md"];

fn find_match<'a>(en_name: &str, rows: &'a [TableRow]) -> Match<'a> {
    let direct_key = match_key(en_name);
    let literal_key = literal_match_key(en_name);

    let mut thematic: Vec<&TableRow> = Vec::new();
    let mut vf_table: Vec<&TableRow> = Vec::new();
    for row in rows {
        if row.book_tag.is_some() {
            continue;
        }
        if row.en_key != direct_key && row.en_key != literal_key {
            continue;
        }
        if row.file == "tugenden-fehler.md" {
            vf_table.push(row);
        } else if VF_RELEVANT_THEMATIC_FILES.contains(&row.file) {
            thematic.push(row);
        } // else: a real file, but not a V/F name source — not in the dispute at all
    }

    let winners = if !thematic.is_empty() {
        thematic
    } else {
        vf_table
    };
    if winners.is_empty() {
        return Match::Uncovered;
    }
    let mut distinct_de: Vec<&str> = Vec::new();
    for w in &winners {
        if !distinct_de.contains(&w.de.as_str()) {
            distinct_de.push(&w.de);
        }
    }
    if distinct_de.len() > 1 {
        Match::Ambiguous(winners.iter().map(|w| (w.file, w.de.as_str())).collect())
    } else {
        Match::Found(winners[0].de.as_str())
    }
}

fn core_ids() -> Vec<String> {
    let all: Value = serde_json::from_str(VF_JSON).expect("virtues_flaws.json is valid JSON");
    all.as_array()
        .expect("virtues_flaws.json is a top-level array")
        .iter()
        .map(|v| v["id"].as_str().expect("entry has an id").to_string())
        .collect()
}

/// Reads an i18n file's `virtue.*`/`flaw.*`-keyed entries' `name` field. Both
/// `rules/i18n/en/virtues_flaws.json` and `rules/i18n/de/virtues_flaws.json`
/// are one shared-shape object also carrying unrelated keys
/// (`ability_category.*`, `being.*`, …), so this filters to the two V/F
/// prefixes.
fn names(json: &str) -> BTreeMap<String, String> {
    let map: Value = serde_json::from_str(json).expect("i18n file is valid JSON");
    map.as_object()
        .expect("i18n file is a top-level object")
        .iter()
        .filter(|(k, _)| k.starts_with("virtue.") || k.starts_with("flaw."))
        .map(|(k, v)| {
            (
                k.clone(),
                v["name"].as_str().expect("entry has a name").to_string(),
            )
        })
        .collect()
}

/// Entries whose own translation-table name is known, by a recorded ruling,
/// to disagree with the shipped DE name — see `tmp/de-names-handover.md` for
/// the triage that produced this list. Each row must still fail
/// [`check`] ([`ruled_exceptions_still_mismatch`]), so the list can only
/// shrink as a reclassification lands, never grow silently to hide a new
/// defect.
const RULED_EXCEPTIONS: &[(&str, &str)] = &[
    (
        "flaw.primogeniture_lineage",
        "D7 (docs/vf-audit/decisions.md): the DE rulebook heading wins for an \
         entry's name when the heading is not itself defective. ArMDE:6705 heads \
         this Flaw 'Erstgeburts-Abstammung', which is what ships; \
         grundbegriffe.md:355's 'Primogenitur-Abstammungslinie' is the glossary's \
         broader catalogue reconciliation, and its own Anmerkung already notes \
         the heading form is what the generator ships.",
    ),
    // --- Parser cannot safely compare the comma-phrase/colon rendering
    // styles below, not a shipped-data defect: see tmp/de-names-handover.md
    // for the full rationale. Each is re-confirmed against the current (P8
    // re-synced) rulebook heading in tmp/p8-6-names.md: the heading uses the
    // same bracket-placeholder form as the table, so these remain pure
    // parser limitations, not naming disputes.
    (
        "flaw.bound_to_realm",
        "Parser cannot compare the comma-phrase style: shipped 'Gebunden an \
         eine Sphäre, {realm}' uses a deliberate, widespread (14-occurrence) \
         DE convention — a descriptive label clause before ', {param}' — that \
         contains words (e.g. 'eine Sphäre') with no counterpart in the \
         table's bare '(Sphäre)' placeholder notation. ArMDE:5802's heading \
         'Gebunden an (Sphäre)' uses the identical bracket form, confirming \
         this is a rendering-style gap, not a name dispute.",
    ),
    (
        "virtue.enchanting_ability",
        "Parser cannot compare the comma-phrase style — same convention as \
         flaw.bound_to_realm ('Bezaubernde Fertigkeit, {medium}' vs the \
         table's '(Fertigkeit)'). ArMDE:3818's heading 'Bezaubernde \
         (Fertigkeit)' confirms the same bracket form.",
    ),
    (
        "virtue.extractor_of_form_vis",
        "Parser cannot compare the comma-phrase style — same convention as \
         flaw.bound_to_realm ('Vis-Gewinner der Form, {form}' vs the table's \
         '(Form)'). ArMDE:3850's heading 'Vis-Gewinner der (Form)' confirms \
         the same bracket form.",
    ),
    (
        "virtue.student_of_realm",
        "Parser cannot compare the comma-phrase style — same convention as \
         flaw.bound_to_realm ('Student einer Sphäre, {realm}' vs the table's \
         '(Sphäre)'). ArMDE:5123's heading 'Student der (Sphäre)' confirms \
         the same bracket form (the shipped 'einer' vs the heading's 'der' is \
         the pre-existing comma-phrase convention, not a new finding).",
    ),
    (
        "flaw.deficient_technique",
        "Parser cannot compare: the table spells the placeholder as a bare \
         word with no brackets on BOTH the EN ('Deficient Technique') and DE \
         ('Defizitäre Technik') columns, unlike every other parameterized row \
         confirmed in this guard, so there is no safe bracket- or word-based \
         signal to bridge the English parameter key to its German spelling. \
         Shipped 'Defizitäre Technik: {technique}' (T2, C3) is the table name \
         plus the chosen Technique after a colon; the unfilled name is the \
         table's 'Defizitäre Technik' exactly. ArMDE:5984's heading \
         'Defizitäre Technik' confirms this (bare, no bracket, same word).",
    ),
    (
        "flaw.deficient_form",
        "Parser cannot compare — same bare-placeholder limitation as \
         flaw.deficient_technique: shipped 'Defizitäre Form: {form}' (T2, C3) \
         is the table's 'Defizitäre Form' plus the chosen Form after a colon; \
         the unfilled name is the table's 'Defizitäre Form' exactly. \
         ArMDE:5980's heading 'Defizitäre Form' confirms this.",
    ),
    (
        "virtue.great_characteristic",
        "Parser cannot compare — same bare-bare-placeholder limitation as \
         flaw.deficient_technique ('Hervorragende {characteristic}' vs the \
         table's bare 'Hervorragende Eigenschaft'). ArMDE:4058's heading \
         'Hervorragende (Eigenschaft)' confirms the same word, bracketed.",
    ),
    (
        "flaw.realm_stigmatic",
        "Parser cannot compare the comma-phrase style — same family as \
         flaw.bound_to_realm. Shipped 'Stigmatisierter, {realm}' vs the \
         table's '(Sphäre-)Stigmatisierter'; ArMDE:6725's heading \
         '(Sphäre-)Stigmatisierter' confirms the 'Stigmatisierter' root and \
         the table agree — the comma-phrase rendering just isn't literally \
         comparable. New P8-6 case (D86): not part of D18/D31's original 30.",
    ),
    // --- D86 (docs/vf-audit/decisions.md): D31 retires because a current
    // German rulebook landed in the P8 re-sync. D7.1 applies again: the
    // rulebook heading wins for a NAME unless defective, and each row below
    // was individually re-checked against the current heading
    // (tmp/p8-6-names.md). The table still disagrees for each, pending its
    // own upstream re-sync (arm-de-translation), so these stay exceptions.
    (
        "virtue.leather_ripper",
        "D86: ArMDE:4316's heading 'Lederreißer' wins over the untagged \
         tugenden-fehler.md:120 row 'Lederzerreißer' (not defective — no \
         collision, no absence). Shipped corrected to 'Lederreißer'.",
    ),
    (
        "flaw.stigmatic_catalyst",
        "D86: ArMDE:6854's heading 'Stigma-Katalysator' wins over the \
         untagged grundbegriffe.md:338 row 'Stigmatischer Katalysator'. \
         Shipped corrected to 'Stigma-Katalysator'.",
    ),
    (
        "virtue.independent_study",
        "D86: ArMDE:4186's heading 'Unabhängiges Studium' wins over the \
         untagged grundbegriffe.md:370 row 'Eigenständiges Studium'. Shipped \
         corrected to 'Unabhängiges Studium'.",
    ),
    (
        "flaw.form_monstrosity",
        "D86 (resolves the former 'pending Norbert's ruling' row): ArMDE:6233's \
         heading '(Form) Missgeburt' confirms shipped '{form} Missgeburt' is \
         correct; the table's '(Form-)Monstrosität' (tugenden-fehler.md:350) \
         is outvoted since the heading is not defective.",
    ),
    (
        "virtue.alluring_to_beings",
        "D86 (resolves the former 'pending Norbert's ruling' row): ArMDE:3459's \
         heading 'Anziehend auf (Wesen)' confirms shipped 'Anziehend auf \
         {being}' is correct; the table's 'Anziehend für (Wesen)' \
         (grundbegriffe.md:371) is outvoted since the heading is not \
         defective.",
    ),
];

fn exception_reason(id: &str) -> Option<&'static str> {
    RULED_EXCEPTIONS
        .iter()
        .find(|(i, _)| *i == id)
        .map(|(_, r)| *r)
}

#[test]
fn every_entry_matches_translation_table_name() {
    let ids = core_ids();
    let en = names(EN_I18N);
    let de = names(DE_I18N);
    assert_eq!(
        ids.len(),
        en.len(),
        "rules/core/virtues_flaws.json has {} entries but rules/i18n/en/virtues_flaws.json has {} \
         virtue.*/flaw.* keys",
        ids.len(),
        en.len()
    );
    assert_eq!(
        ids.len(),
        de.len(),
        "rules/core/virtues_flaws.json has {} entries but rules/i18n/de/virtues_flaws.json has {} \
         virtue.*/flaw.* keys",
        ids.len(),
        de.len()
    );

    let rows = all_rows();
    let mut failures = Vec::new();
    let mut uncovered = 0usize;

    for id in &ids {
        if exception_reason(id).is_some() {
            continue;
        }
        let en_name = en
            .get(id)
            .unwrap_or_else(|| panic!("{id}: no EN name in rules/i18n/en/virtues_flaws.json"));
        let de_name = de
            .get(id)
            .unwrap_or_else(|| panic!("{id}: no DE name in rules/i18n/de/virtues_flaws.json"));

        match find_match(en_name, &rows) {
            Match::Uncovered => uncovered += 1,
            Match::Ambiguous(candidates) => failures.push(format!(
                "{id}: EN name \"{en_name}\" matches {} differing translation-table rows {:?} — \
                 ambiguous, needs triage (duplicate/contradicting table rows, not a shipped-data defect \
                 by itself)",
                candidates.len(),
                candidates
            )),
            Match::Found(table_de) => {
                if normalize_de(de_name) != normalize_de(table_de) {
                    failures.push(format!(
                        "{id}: shipped DE name \"{de_name}\" does not match the translation table's \
                         \"{table_de}\" for EN \"{en_name}\"",
                    ));
                }
            }
        }
    }

    eprintln!(
        "de_vf_names: {uncovered} of {} Virtue/Flaw entries have no translation-table coverage \
         (not a failure)",
        ids.len()
    );

    assert!(
        failures.is_empty(),
        "{} Virtue/Flaw entries disagree with the translation tables:\n{}",
        failures.len(),
        failures.join("\n")
    );
}

/// Every [`RULED_EXCEPTIONS`] row exists to silence
/// [`every_entry_matches_translation_table_name`] for one entry whose
/// translation-table name genuinely disagrees with the shipped DE name by a
/// recorded ruling. If a future data change makes the comparison agree again
/// (the exception's rationale no longer applies), the row must be deleted —
/// this test fails loudly instead of leaving a dead, unverifiable row sitting
/// in the list.
#[test]
fn ruled_exceptions_still_mismatch() {
    let en = names(EN_I18N);
    let de = names(DE_I18N);
    let rows = all_rows();
    let mut stale = Vec::new();

    for (id, _) in RULED_EXCEPTIONS {
        let en_name = en
            .get(*id)
            .unwrap_or_else(|| panic!("RULED_EXCEPTIONS names \"{id}\", which has no EN name"));
        let de_name = de
            .get(*id)
            .unwrap_or_else(|| panic!("RULED_EXCEPTIONS names \"{id}\", which has no DE name"));
        let still_mismatches = match find_match(en_name, &rows) {
            Match::Found(table_de) => normalize_de(de_name) != normalize_de(table_de),
            Match::Ambiguous(_) | Match::Uncovered => true,
        };
        if !still_mismatches {
            stale.push(*id);
        }
    }

    assert!(
        stale.is_empty(),
        "RULED_EXCEPTIONS row(s) {stale:?} now MATCH the translation table — the defect they \
         recorded is gone, so remove the row(s)"
    );
}

/// Proves [`find_match`]/[`normalize_de`] actually reject a wrong name rather
/// than trivially agreeing with everything — modelled on
/// `vf_descriptor_line.rs`'s self-test. Uses a real, currently-matching pair
/// (`flaw.ability_block`, untagged in `tugenden-fehler.md`) as the known-good
/// fixture, so this test also fails loudly if the lookup machinery itself
/// breaks.
#[test]
fn self_test_rejects_a_wrong_synthetic_name() {
    let en = names(EN_I18N);
    let rows = all_rows();
    let en_name = en
        .get("flaw.ability_block")
        .expect("flaw.ability_block has an EN name");
    let table_de = match find_match(en_name, &rows) {
        Match::Found(de) => de,
        other => panic!("fixture \"{en_name}\" did not resolve to a single table match: {other:?}"),
    };
    assert_eq!(
        normalize_de(table_de),
        normalize_de("Fähigkeitsblock"),
        "sanity check: the real shipped name must still match the table"
    );
    assert_ne!(
        normalize_de(table_de),
        normalize_de("Völlig falscher Synthetischer Name"),
        "self-test: the comparison must reject a wrong synthetic name"
    );
}

/// [`strip_bracketed_magnitude`] removes exactly a bracketed magnitude and
/// the space before it — nothing else: a parenthesised `(Groß)` and an
/// unrelated bracket stay.
#[test]
fn self_test_strips_only_a_bracketed_magnitude() {
    assert_eq!(strip_bracketed_magnitude("Ambitious [Major]"), "Ambitious");
    assert_eq!(strip_bracketed_magnitude("Ehrgeizig [Klein]"), "Ehrgeizig");
    assert_eq!(
        strip_bracketed_magnitude("Falsche Macht [Groß]: {virtue}"),
        "Falsche Macht: {virtue}"
    );
    assert_eq!(
        strip_bracketed_magnitude("Potent Magic [Minor]"),
        "Potent Magic"
    );
    assert_eq!(
        strip_bracketed_magnitude("Ehrgeizig (Groß)"),
        "Ehrgeizig (Groß)"
    );
    assert_eq!(strip_bracketed_magnitude("Foo [Bar]"), "Foo [Bar]");
    assert_eq!(
        strip_bracketed_magnitude("Minor Magical Focus"),
        "Minor Magical Focus"
    );
}

/// The split "Minor or Major" entries ARE compared with the table's
/// base-name rows (D84.3): both halves of a real pair resolve to the
/// untagged `tugenden-fehler.md` row `Ambitious | Ehrgeizig`, and both
/// shipped DE names agree with it once the bracket is stripped.
#[test]
fn bracketed_magnitude_pairs_are_compared_with_the_table_base_name() {
    let en = names(EN_I18N);
    let de = names(DE_I18N);
    let rows = all_rows();
    for id in ["flaw.ambitious_major", "flaw.ambitious_minor"] {
        let en_name = en.get(id).expect("EN name exists");
        let de_name = de.get(id).expect("DE name exists");
        let table_de = match find_match(en_name, &rows) {
            Match::Found(de) => de,
            other => panic!(
                "{id}: EN \"{en_name}\" must resolve to the table's base-name row, got {other:?}"
            ),
        };
        assert_eq!(table_de, "Ehrgeizig", "{id}: wrong table row");
        assert_eq!(
            normalize_de(de_name),
            normalize_de(table_de),
            "{id}: shipped DE \"{de_name}\" must match the table's \"{table_de}\""
        );
    }
}
