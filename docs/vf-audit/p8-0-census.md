# P8-0 census (`design-p8-resync.md`, Phase P8-0)

Read-only pass, 2026-10-02, against HEAD `10e022f`. Dated record: the line
numbers below are a snapshot and are not maintained.

Shorthand used below:
- `CIT` = `"(ArMDE|HoH:TL|HoH:MC|HoH:S|HM:RE|RoP:M|RoP:F|RoP:D|RoP:I):[0-9]+(-[0-9]+)?"`
- `START` = the same without `(-[0-9]+)?` (yields acronym + first line only)
- `ACR` = `"^(ArMDE|HoH:TL|HoH:MC|HoH:S|HM:RE|RoP:M|RoP:F|RoP:D|RoP:I)"`
- scope groups: **src** = `crates/arm-rules/src crates/arm-app/src`; **tests** =
  `crates/arm-rules/tests crates/arm-app/tests` with `-g '!book_templates.rs'`;
  **bt** = `crates/arm-rules/tests/book_templates.rs`; **ui** = `ui/src`;
  **e2e** = `ui/e2e` (it carries citations and sits outside every citation guard);
  **RULES** = `crates/arm-rules/RULES.md`.

## 0. Is M0's census still current? No, and it never covered this scope

`measurements.md` § "Handover § 8 — entry-less `ArMDE:` citations" (M0, `6f34a70`,
2026-09-26) counts only `ArMDE:` citations, only in `corrections.md`, `decisions.md`,
`phase-2-handover.md` and `book_templates.rs`. Its one overlap with P8's scope:

| | M0 (2026-09-26) | today |
|---|---|---|
| `ArMDE:` occurrences in `book_templates.rs` | 208 | **232** |
| continuations (`, :NNNN`) | 8 | 8 |

Command: `grep -oE "ArMDE:[0-9]+" crates/arm-rules/tests/book_templates.rs | wc -l`.
M0 is stale (+24) and absent for every other scope, so the census below is re-derived.

## 1. Entry-less citation census (everything outside `rules/core/*.json`)

### 1a. Acronym citations, by scope and acronym

Per group: `rg -o -a --no-filename -N CIT <group> | grep -oE ACR | sort | uniq -c`;
occurrences `… | wc -l`; distinct specs `… | sort -u | wc -l`; distinct start lines
`rg -o -a --no-filename -N START <group> | sort -u | wc -l`.

| scope | ArMDE | RoP:M | RoP:I | RoP:D | HoH:* / HM:RE / RoP:F | **occurrences** | distinct specs | distinct start lines | files |
|---|---|---|---|---|---|---|---|---|---|
| src | 1800 | 12 | 17 | 5 | 0 | **1834** | 486 | 433 | 57 |
| tests (minus bt) | 1687 | 0 | 2 | 0 | 0 | **1689** | 895 | 834 | 58 (incl. bt) |
| bt | 232 | 0 | 0 | 0 | 0 | **232** | 208 | 208 | 1 |
| ui (`ui/src`) | 264 | 0 | 0 | 0 | 0 | **264** | 114 | 108 | 62 |
| e2e (`ui/e2e`) | 19 | 0 | 0 | 0 | 0 | **19** | 11 | 10 | 5 |
| RULES.md | 2178 | 8 | 7 | 0 | 0 | **2193** | 1248 | 1148 | 1 |
| **all** | 6180 | 20 | 26 | 5 | **0** | **6231** | **1792** | **1577** | — |

Heaviest files: RULES.md 2193, `tests/data_integrity.rs` 442, `src/aging.rs` 366,
`src/types.rs` 359, `tests/book_templates.rs` 232, `tests/uncomputed_clauses.rs` 228,
`tests/x2_reclassification.rs` 171, `tests/x3_trained_gate.rs` 130.

**Non-Core citations are tiny:** 51 occurrences, all RoP:M/RoP:I/RoP:D, on **13
distinct start lines**: RoP:M 1470, 1472, 5486; RoP:I 4120, 4122, 4136, 4142, 4908,
4912, 4924, 4942; RoP:D 1975, 1977. Zero HoH:*, HM:RE, RoP:F.

### 1b. Continuation citations (`, :NNNN`), not matched by CIT

| scope | broad `[,/] ?:[0-9]+(-[0-9]+)?` | chained to an acronym (lower bound) |
|---|---|---|
| src | 272 | 178 |
| tests (minus bt) | 106 | 68 |
| bt | 8 | 2 |
| ui | 20 | 11 |
| e2e | 1 | 1 |
| RULES.md | 31 (includes code-line refs) | **22** |

**Estimate: ~430 continuation citations in code + 22 in RULES.md.** T2 must parse
chains the way `rulebook_citations.rs` does.

### 1c. Line citations in other forms (also need relocation)

| form | where | count |
|---|---|---|
| `("id", NNNN)` tuple literals feeding `ArMDE:{line}` messages | `tests/data_integrity.rs` | 60 rows, 34 template sites |
| `Basisregeln.md:NNNN` | RULES.md 8, `i18n.test.ts` 10, `derive.test.ts` 2, `ParameterPicker.test.ts` 1, `x9b_bard_rename.rs` 2, `arm-app/tests/commands.rs` 1 | 24 |
| English full basename inside runtime error strings | `crates/arm-rules/src/ruleset/integrity.rs` | 7 |
| English full basename in comments | `ui/e2e` (two specs + `helpers.js`) | 3 |
| legacy `Core Rules.md:NNNN` shorthand | `ui/e2e/specs/` (four specs) | 7 |
| translation-table lines (`grundbegriffe.md:671`, …) | ui/src, tests, RULES.md | ~35, out of P8 scope |

**Two guard gaps (recorded, not fixed):** `ui/e2e` is not a root of
`rulebook_citations.rs`, so the forbidden full-basename and `Core Rules.md` forms survive
there; and the 7 `integrity.rs` citations live in string literals, not comments, so the
full-basename ban does not see them.

### 1d. Relocatable by anchor vs. bare

| scope | heading-adjacent | on a line naming a catalogue id | quote-adjacent | remainder (bare) |
|---|---|---|---|---|
| src | 0 | 11 | 327 | ~1500 |
| tests (minus bt) | 0 | 371 | 342 | ~1000 |
| **bt** | **221** | — | 36 | **11** |
| ui | 0 | 1 | 48 | ~215 |
| e2e | 0 | 0 | 0 | 19 |
| RULES.md | 0 | 748 | 179 | ~1270 |

Columns overlap and are heuristics. Outside `book_templates.rs` (95% anchored by X9d)
essentially nothing carries a `####` anchor; ~1,100 sit beside an id and ~900 beside a
quoted excerpt, which is what T2/T3's text matching can use.

## 2. Tests that pin a rulebook line number

### 2a. Hard pins

These **go red (or silently vacuous) when P8-3 relocates `source.lines`**, so they
belong in the P8-3 commit. This contradicts the design's claim that P8-3 touches only
`rules/core/` and the sidecar.

| file | pins | what |
|---|---|---|
| `data_integrity.rs` | **43** | `core_rules_virtues_cite_the_core_rules_file` 11; two single-line pins; living-conditions rows 10; Aging Roll rows 11; crisis die; attendant; Crisis rows 7 |
| `x2_reclassification.rs` | **8 ranges** (4 ids) | `COMPOSED_DESCRIPTIONS`, read from the real EN **and** DE rulebooks |
| `ability_category_line.rs` | **3 ranges** | `check_rejects_synthetic_wrong_entries`: asserts `is_err()`, so a shift makes it **vacuously green**, not red |
| `x9b_bard_rename.rs` | **1** + anchor pin | `LineRange::new(3476, 3478)`; anchor `"rard"` breaks by design when upstream's Bard fix lands |
| `x9c_descriptor_sweep.rs` | **1** + 12 RULES.md-text pins | `rules_md_carries_the_known_source_errata_note` asserts RULES.md contains given line numbers: couples to P8-4's RULES.md rewrite |
| `uncomputed_clauses.rs` | **1 range** | `SWEPT_BLOCKS` `(Core Rules, 3360, 7113)` |
| `rules_source_provenance.rs` | **1** | fixture `"lines": [16601, 16601]` resolved against the real rulebook |
| **total** | **58 line literals in 7 files** (+13 text/anchor pins) | |

`data_integrity.rs` also has 60 message-only `(id, line)` rows (stale text after a
sync, never red). Data-driven readers (`rules_source_provenance.rs`,
`aging_table_row.rs`, `equipment_table_row.rs`, `spell_stat_line.rs`,
`vf_descriptor_line.rs`, …) follow `source.lines` and are P8-3's gates. **No test
under `ui/src` or `ui/e2e` pins a line.**

## 3. Row-key catalogues cite Core Rules only (confirmed)

| catalogue | source blocks | `source.file` | row-key anchors |
|---|---|---|---|
| `equipment.json` | 46 | 46 × Core Rules | 46 |
| `childhoods.json` | 5 | 5 × Core Rules | 5 |
| `aging.json` | 30 | 30 × Core Rules | 29 (+ `simple-die` heading) |
| `spells.json` | 361 | 361 × Core Rules | **1** (`criamon/piercing-the-magical-veil`) |
| every other catalogue | 802 | 802 × Core Rules | 0 |
| **all `rules/core/`** | **1244** | **1244 × Core Rules** | 81 |
| DE sidecar `source_anchors.json` | 1244 | 1244 × Basisregeln | — |

Command: `jq -r ".. | objects | select(has(\"source\")) | .source | objects | .file" rules/core/<file>.json | sort | uniq -c`.
No anchored data entry cites a structurally-heavy book or any HoH book, so the heavy
group matters only to the 13 bare RoP lines in § 1a.

Our Core Rules copy has `### Identified Issues From Source PDF Release` at ArMDE:47,
where 85% of upstream's +80 lines landed. So virtually every Core citation moves,
mostly by one constant offset: an ideal case for exact-text search.

## 4. Re-estimates (S ≈ ½ session, M ≈ 1, L ≈ 3)

**P8-4 (bare citations): M, or L if T2 only parses `ACRONYM:NNNN`.** Almost all of the
volume is Core text displaced by the one errata block; hand work is the 13 RoP lines,
Core lines inside upstream's reworded hunks, and ~110 non-standard forms. If T2 skips
continuations and tuple literals, ~550 occurrences go to hand editing.

**P8-5 (verbatim description refresh): S.** Zero anchored entries cite a heavy book,
and Core's non-errata body change is small: a refresh list in the tens.

**Knock-on for P8-3:** the 58 hard pins in § 2a must change in the same commit as the
data, or P8-3 is red; x9c's RULES.md-text pins change with P8-4's RULES.md rewrite.
