# Surviving an upstream rules-source re-sync

**Status: preparation, not yet executed.** Norbert has confirmed the rulebook
Markdown in `rules/source/` will be **updated upstream**, with **line numbers
changing throughout**, and that the German edition will be re-synced to match.
This document records how the repository prepares for that, and why.

## The problem, stated plainly

Every rule in this codebase is located by **file + line range**:

- Code comments cite `ArMDE:547`, `HoH:TL:383`, and so on.
- Every item in `rules/core/*.json` carries
  `source: { file, lines: [start, end] }`.
- `crates/arm-rules/RULES.md` maps rules to line ranges.

That convention is only safe because a published rulebook never moves. Once
upstream edits land, **every one of those references shifts at once**, and the
guards cannot save us:

- `rulebook_citations.rs` checks that a cited range lands on **non-blank lines**
  — not that those lines say what the citation claims. A wholesale shift
  therefore produces a **green suite and hundreds of citations silently pointing
  at the wrong rule.** This is the exact failure `docs/audit-2026-08.md` records:
  four wrong ranges hid behind unverifiable shorthand and were each found only by
  opening the file.
- The **German line-parity invariant** (`CLAUDE.md`) — German sources mirror the
  English line-by-line, so a German line number identifies the same item — breaks
  if the two languages shift differently. Nothing tests that.

## The durable key: heading anchors

The sources already give us one, and it is per-item:

- **Every virtue and flaw has its own `####` heading** in both languages —
  `#### Clumsy` at `ArMDE:5797`, `#### Humpelnd` in the German edition.
- The Markdown **already carries generated anchor slugs**, visible in its own
  cross-links: `(#abandoned-apprentice)`, `(#haus-mercere)`, `(#abilities-1)`.
  So anchor-by-heading is an existing convention of these files, not an invention.

An anchor survives an edit anywhere else in the book. It breaks only if the
heading itself is renamed — which is rare, is a *semantic* change worth noticing,
and fails **loudly** (anchor not found) rather than silently pointing at the
wrong text. That asymmetry is the whole argument: a rotted line number still
resolves to *something*, which is why it is dangerous.

## What to capture during extraction, starting now

Any pass that touches an item's provenance should record the **heading anchor
alongside the existing line range**, not instead of it:

- `lines` stays. The guards depend on it, and it is a useful fast path.
- The **anchor becomes the source of truth** for *which item* a reference means.
- After an upstream re-sync, line ranges are **recomputed mechanically** by
  locating each anchor — a data migration, not a re-audit.

Per-language, because the headings differ: the English anchor keys
`rules/core/`'s `source` (English is the source of truth for IDs), and the German
anchor belongs with the German text in the `rules/i18n/de/` layer.

## Why this is cheap now and expensive later

The 655-entry description sweep (open-todos row 38) reads **every** cited range in
**both** languages anyway. Capturing the anchor during that read costs almost
nothing. Doing it afterwards means reading all 1310 ranges a second time — and if
the re-sync lands first, reading them against a book whose line numbers no longer
match what the JSON says.

## The recorded shape

Decided and in use since the Flaws sweep (2026-09-15):

- **English anchor** — `rules/core/*.json`, as a third key inside the existing
  `source` block: `{ "anchor": "poor-concentration", "file": "…", "lines": [6602, 6605] }`.
  Modelled by `types.rs::SourceRef::anchor`, an `Option<String>` that is skipped
  when absent, so an unswept entry serializes byte-identically to before the
  field existed and the rollout stays incremental. Alphabetical field order keeps
  `CLAUDE.md` → "Canonical serialization" satisfied without a custom `Serialize`.
- **Per-language anchor** — a sidecar, `rules/i18n/<lang>/source_anchors.json`,
  mapping `id -> { anchor, file }`. A sidecar rather than a field on the
  localized entries because those deserialize into `ruleset.rs::I18nEntry`, whose
  serialized shape is a stable IPC contract the frontend binds to, and an
  extraction coordinate has no business travelling into every tooltip. It is not
  in `ruleset_io.rs::read_i18n_sources`'s fixed file list, so it is never loaded
  at runtime — correct for something that exists for re-sync tooling and guards.
- **No line range on the localized side.** German line numbers are derivable from
  the English by the line-parity invariant, and that invariant is now itself
  tested (below); recording them again would create a second coordinate to rot.
- **No `SCHEMA_VERSION` bump, and none is owed.** `SCHEMA_VERSION` versions
  *saves*, and `SourceRef` appears only on ruleset catalogue types — never on
  `types.rs::Entity`, which is what a save serializes. Verified before assuming
  it.

Three guards in `crates/arm-rules/tests/rules_source_provenance.rs` enforce it:

- `every_recorded_source_anchor_resolves_to_its_own_heading` — the anchor names a
  real heading **and that heading lies inside the cited line range**. The second
  half is what makes the anchor load-bearing rather than decorative: after a
  re-sync a shifted range no longer contains its own heading, so the two
  coordinates disagree loudly instead of the range silently pointing elsewhere.
- `every_localized_source_anchor_resolves_to_a_heading` — same, per language.
- `german_anchors_sit_on_the_same_line_as_their_english_counterparts` — **the
  German line-parity invariant, tested for the first time.** `CLAUDE.md` declares
  it and this document recorded that nothing checked it; anchors make it
  checkable item by item.

The slug algorithm is not guessed: `the_heading_slug_matches_the_sources_own_generated_links`
pins it against the sources' own generated cross-links (`#anchored-to-the-land`,
`#hitze--und-ätzungstabelle`, the `-N` disambiguation behind `#die-gabe-2`).

## Still owed

- **Rollout.** 51 of ~1000 `source` blocks carry an anchor — the core rulebook's
  Flaws block. Every later sweep records them as it reads, per the "cheap now,
  expensive later" argument above.
- A decision on code comments: `ArMDE:547` is the citation form `CLAUDE.md`
  mandates and four guards enforce. Anchors may suit data better than prose
  comments; do not change the comment convention unilaterally.
- The re-sync itself. The German line-parity re-verification it used to owe is
  now a test, so it re-verifies itself over whatever is swept. See open-todos P8.
