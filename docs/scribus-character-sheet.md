# Scribus fillable-PDF character sheet

Reference detail for **PLAN.md Milestone 11**. Tracks the work to translate,
extend, audit, and integrate the existing Scribus character sheet as a bilingual,
auto-fillable PDF export target for arm-char-gen.

## Source asset

`arm-de-translation/character-sheet/ArM5-Charakterbogen-Deutsch.sla`
(also `.pdf`, `arm5openlicenselogo.png`).

- Scribus **1.6.6** `.sla` (plain XML, ~1.9 MB, ~9.8k lines), 5 pages, 2134 page
  objects.
- **670 AcroForm fields**, all text fields (`ANTYPE="3"`).
- AcroForm JavaScript is stored **entity-encoded** (`&quot;`, `&#10;`, `&gt;`)
  inside annotation attributes:
  - `ANCACT` — calculate (70 non-empty)
  - `ANVACT` — validate (54)
  - `ANKACT` — keystroke / input mask (177; e.g. `/^\d{0,4}$/`)
  - other action slots (`ANACTION`, `ANEACT`, `ANFACT`, …) are present but empty.
- Field names (`ANNAME`) are German with umlauts (`Größe`, `Präsenz`) and
  copy-paste cruft (`Kopie von Details (3)`, `Kopie von EigenschaftBeschreibung (5)`).

Example calc (`Konvent` field, age from years):
`event.value = (parseInt(AktJahr)||0) - (parseInt(GebJahr)||0)`.
Combat fields follow the same style (e.g. attack total = Qik + Skill + WeaponAtk).

## The three "German" layers (translate differently, different risk)

1. **Visible labels** — static text in frame story text (`<ITEXT CH="…">`).
   Cosmetic; safe to translate freely.
2. **Tooltips** (`ANTOOLTIP`) — cosmetic; safe.
3. **Internal field names** (`ANNAME`) — **load-bearing**. JS references them by
   exact string via `getField("…")`. Renaming requires updating every reference
   in lockstep or calculations break. This is the #1 risk.

## Decided approach

- **Bilingual**: DE + EN sheets, sharing identical field names + identical JS;
  only labels/tooltips differ ("label skins").
- **Field names → English-ASCII IDs**, mapped to arm-rules engine slugs where a
  correspondence exists (characteristics, abilities, arts, V/F, combat totals).
  The map is the single source of truth, driving both the rename and auto-fill.
- **Template-patch**, not PDF-from-scratch: hand-maintained `.sla`, populated by
  arm-char-gen at export time.

## Work items

1. **Field-ID schema & map.** German `ANNAME` → English-ASCII ID → engine slug.
   Normalize cruft and umlauts. Store as a checked-in mapping (e.g. JSON).
2. **Atomic rename.** Apply the map to every `ANNAME` AND every `getField("…")`
   inside all `AN*ACT` scripts, in lockstep.
3. **Audit & fix the JS.** Verify each calc against authoritative rules source;
   fix bugs and inconsistent `parseInt` radix / empty-string handling. Fix
   Scribus's unreliable **calculation-order** emission so derived-feeding-derived
   fields evaluate in the correct order.
4. **Extend missing fields.** Diff against the official core-rules copy-template
   PDF; add the gaps (enumerate during implementation).
5. **Bilingual label skins.** Translate visible `ITEXT` + tooltips. EN follows the
   English source of truth; DE follows the German translation tables
   (`rules/source/de/translation-tables/`).
6. **Export integration (ties to M10).** arm-char-gen emits FDF/XFDF (or a filled
   PDF) keyed by the canonical English field IDs, mapping engine output → field
   IDs via the schema in step 1.

## Scribus quirks to watch

- JS is entity-encoded inside `AN*ACT` attributes.
- `getField()` is exact-name-coupled — rename breaks calcs unless refs updated.
- Calculation-order emission is unreliable; dependent fields can mis-order.
- Umlauts / special chars in field names are fragile across PDF readers → ASCII.
- Copy-paste yields duplicate `Kopie von …` names.
- Workflow is edit `.sla` → re-export PDF via Scribus to regenerate the form.

## Open decisions (resolve at implementation time)

- **Vendor** the sheet into arm-char-gen (e.g. `assets/character-sheet/`) vs keep
  it in `arm-de-translation`? It becomes an export target, arguing for vendoring.
- **One** source `.sla` + a label-swap build step vs **two** maintained `.sla`
  files (DRY vs KISS).

## Verification

- Static gate script: every `getField()` target ∈ field-name set (0 broken refs);
  all field names unique + ASCII.
- Open the exported PDF in ≥2 readers (e.g. Acrobat + Okular/Firefox); enter
  Characteristics + weapon values; confirm derived fields compute correctly and in
  order.
- Integration: generated character → FDF → filled PDF; confirm fields populate.
