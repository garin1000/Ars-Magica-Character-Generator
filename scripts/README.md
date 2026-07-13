# `scripts/` — developer / build tooling

These scripts are **dev/build tools**. They are **never loaded at runtime** by
`arm-rules` or `arm-app`; they regenerate hand-checkable data from the
authoritative Markdown in `rules/source/`, consistent with CLAUDE.md's rule that
"JSON is generated from the Markdown by extraction". The engine's load-time
referential-integrity + serde validation remains the trust gate over whatever
these scripts emit.

## `extract_spells.py`

Extracts the full Hermetic spell catalogue from the Spells chapter of
`rules/source/en/Ars Magica - Definitive Edition (Core Rules).md` and writes:

- `rules/core/spells.json` — language-neutral mechanics (id, technique, form,
  level, requisites, ritual, range/duration/target, creates_lasting, source
  line-range).
- `rules/i18n/en/spells.json` — English name + description prose, keyed by id.
- `rules/i18n/de/spells.json` — German names from the canonical translation table
  `rules/source/de/translation-tables/zauber-nach-form.md`. A spell absent from
  the table falls back to its English name (documented policy — the extractor
  reports every fallback); German is never invented.

Run it from anywhere:

```bash
python3 scripts/extract_spells.py
```

Properties:

- **Deterministic & canonical.** Output is sorted by id with stable formatting,
  so re-running produces a zero-noise diff.
- **In-loop checks fail loudly.** Technique resolves to a Technique-class Art and
  Form to a Form-class Art (against `rules/core/arts.json`); requisites are known
  arts; ritual ⇒ level ≥ 20; non-ritual ⇒ level ≤ 50; Year ⇒ ritual; Boundary ⇒
  ritual; ids unique; source ranges within the chapter. On any violation it
  prints the offending ids and exits non-zero without writing.
- **Reports.** Total spell count, per-Technique and per-Form counts, ritual
  count, and DE coverage (with the list of any English fallbacks).

The script tolerates the source's formatting inconsistencies (mixed CRLF/LF,
stray `.`/missing `,` in R/D/T lines, compound durations like "Sun & Year", a
`## Creo Mentem Spells` heading with two hashes, and the "Svells" typo). See
`crates/arm-rules/RULES.md` (Spells section) for the provenance and mapping
detail.
