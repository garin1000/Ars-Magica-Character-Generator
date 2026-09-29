# Ars Magica 5th Edition Character Generator

## Project overview

A locally installed, cross-platform desktop character generator for Ars Magica
5th Edition. Builds **characters** (grog, companion, mythic companion, magus)
and **covenants** as first-class entity types with three input modes: guided
wizard, direct-validated, and direct-unchecked.

### This is a DESKTOP APPLICATION, not a web project — assess findings accordingly

The frontend is Svelte running in an **OS webview inside a native binary the user
installed**. It is not a website, not a hosted service, and not a multi-tenant
application. Web technology is an *implementation detail of the UI layer*, not the
deployment model. Every review, audit, severity rating, and design decision MUST be
made against the actual model:

- **Single local user, full trust, no privilege boundary.** There are no accounts, no
  sessions, no roles, no tenants, no login. The person running the app already owns the
  machine and the files. There is nothing to escalate *to*.
- **No server, no network service, no listening socket, no remote API, no origin.** The
  app ships no backend and makes no outbound calls in normal operation.
- **No untrusted remote content.** Nothing is fetched from the internet and rendered.
- **The trust boundary is the file the user opens** — a `.armc`/`.armcov` save, or a
  `rules/` directory beside the binary. That is the realistic hostile-input surface, and
  it is the one that deserves real scrutiny: reachable panics, unbounded recursion or
  allocation, integer overflow, and algorithmic blowup on crafted input, plus schema
  migration on an untrusted `schema_version`.

**Consequences for severity ratings.** Web-shaped concerns are either inapplicable or
land far lower than a web-app checklist would suggest, and must not be rated as if a
remote attacker existed:

- **Not applicable at all:** CSRF, session fixation/cookies, SQL injection, TLS/HSTS,
  CORS, clickjacking, SSRF, rate limiting, multi-tenant isolation, authn/authz flaws.
- **Rated by *local* impact only:** XSS-shaped findings. There is no remote injection
  vector and no cross-origin attacker; the CSP hardening is defence-in-depth for
  save-file-derived text, not a control against a remote adversary. A missing sanitiser
  with no reachable hostile source is at most LOW.
- **Availability findings cap low.** A crash or hang affects only the local user's own
  session, and they can simply reopen the app. A DoS reachable *only* by the user opening
  their own deliberately-corrupted file is not a HIGH — though a **panic** is still a real
  robustness defect worth fixing (it loses unsaved work).
- **`devDependencies` vulnerabilities are not shipped code.** They are a developer-machine
  and build-integrity concern, not an end-user exposure. Rate accordingly, and say
  explicitly whether a vulnerable package reaches the shipped bundle.

**What genuinely IS high-severity here**, and should be rated *up* rather than down:
data loss and silent data corruption (the unsaved-changes guard, save/load round-trip
fidelity, schema migrations), **wrong rules output** — the app's entire purpose is
computing correct Ars Magica characters, so a miscalculation is a product-integrity
failure, not a cosmetic one — accessibility and localization defects (real users, every
session, on every platform), and breaches of the invariants this file declares
load-bearing.

## Technical stack (locked)

| Concern            | Choice                                                     |
|--------------------|------------------------------------------------------------|
| Shell / packaging  | **Tauri 2** — native binary, OS webview                    |
| Core logic         | **Rust** — pure library crate (`arm-rules`)                |
| Frontend           | **Svelte 5 + Vite** (SPA, no SSR)                         |
| UI localization    | **Fluent** (`.ftl`): `fluent-bundle` (Rust) / `@fluent/bundle` (JS) |
| Rules + save format| **JSON** via `serde`                                       |
| Targets            | Windows, macOS, Linux                                      |

## Workspace layout

```
arm-char-gen/
  Cargo.toml                 # cargo workspace
  crates/
    arm-rules/               # PURE engine: types, parse, validate, evaluate. No tauri/IO.
    arm-app/                 # Tauri binary; depends on arm-rules; owns file IO + commands
  ui/                        # Svelte 5 + Vite frontend (SPA)
  rules/
    source/<lang>/*.md       # authoritative human-authored rules in Markdown (NOT loaded at runtime)
    core/*.json              # language-neutral mechanics
    i18n/<lang>/*.json       # translatable rules text, keyed by stable IDs
  locales/<lang>/*.ftl       # Fluent UI strings
  examples/                  # sample character and covenant saves for tests/demo
```

## Rules source files

All provided rulebooks are released under the **Ars Magica Open License**
(based on CC-BY-SA 4.0), so the Markdown source text may be redistributed with
the repository — no proprietary-copyright restrictions apply. Attribution and
share-alike terms still apply to derived content.

The authoritative Markdown in `rules/source/<lang>/` is the human-readable
rulebook text. JSON in `rules/core/` and `rules/i18n/` is **generated** from it
by extraction (hand-authoring the full catalogue is infeasible); the engine's
referential-integrity + serde validation is the trust gate. **English is the
source of truth** — IDs are derived from the English source; other languages
only fill i18n text against IDs that already exist.

Current English sources (`rules/source/en/`):

```
Ars Magica - Definitive Edition (Core Rules).md
Ars Magica 5e - Houses of Hermes - Mystery Cults.md
Ars Magica 5e - Houses of Hermes - Societates.md
Ars Magica 5e - Houses of Hermes - True Lineages.md
Ars Magica 5e - Magic - Hedge Magic (Revised).md
Ars Magica 5e - Realms of Power - Faerie.md
Ars Magica 5e - Realms of Power - Magic.md
Ars Magica 5e - Realms of Power - The Divine (Revised).md
Ars Magica 5e - Realms of Power - The Infernal.md
```

German sources (`rules/source/de/`) are the **source of truth for German
translations** — both the full rulebook text and the term mappings used to
fill `rules/i18n/de/`. The German rulebook file set does **not** match English
1:1:

```
Ars Magica Definitive Edition Basisregeln.md            # ↔ Core Rules
Ars Magica 5e - Häuser des Hermes - Mysterienkulte.md   # ↔ Houses of Hermes - Mystery Cults
Ars Magica 5e - Häuser des Hermes - Societates.md       # ↔ Houses of Hermes - Societates
Ars Magica 5e - Häuser des Hermes - Wahre Linien.md     # ↔ Houses of Hermes - True Lineages
Ars Magica 5e - Magie - Heckenzauber (Überarbeitet).md  # ↔ Magic - Hedge Magic (Revised)
Ars Magica 5e - Sphären der Macht - Magie.md            # ↔ Realms of Power - Magic
Ars Magica 5e - Wächter des Waldes - Das Rhein-Tribunal.md  # Rhine Tribunal — no English source in repo
```

Not yet available in German: Realms of Power — Faerie, The Divine, The
Infernal. Conversely, the Rhine Tribunal book has no English counterpart here.
Because English is the source of truth for IDs, German-only books cannot
introduce new IDs; they only supply i18n text against existing English-derived
IDs (or wait until the English source is added).

### German translation tables

`rules/source/de/translation-tables/` holds 22 thematic, hand-curated **EN↔DE
glossary tables** (Markdown tables: `Englisch (EN) | Deutsch (DE) | Anmerkung`),
distilled from three master tables (main glossary of 22 sections, 30 additional
terms, 362 spell names). These are the **canonical EN→DE terminology mapping**:
when generating `rules/i18n/de/` text, the German label for any term whose
English form appears in a table MUST match the table's `Deutsch (DE)` value —
**subject to the precedence rules below, which are not optional.**

**This directory is a COPY, and `../arm-de-translation` is the
source of truth.** It is synced from there, not maintained here, and it *drifts*
between syncs — the V/F audit found rows that disagreed with the rulebook and
turned out to be stale rather than wrong. So a correction goes into the source
project first, and a row that looks wrong is checked against it **before** being
called an error. Re-sync by copying `translation-tables/*.md` across; the local
`README.md` then needs its audit warning re-applied, since the source project's
copy is worded for that project.

**The tables are AI-generated and fallible — but less often than the audit first
thought.** The V/F audit (2026-09-19 to -21, all 655 entries) recorded **ten**
proven errors. **D31 withdrew seven of them**: they were *name* disagreements
convicted with a heading from our **stale** German rulebook copy, which is
circular. What survives is the shape that actually matters — *a terminology table
making a factual claim about the rules* — and a translation revision cannot
excuse that, because it renames rather than reassigning mechanics. Hence the
precedence rules, which live in full in
`docs/vf-audit/decisions.md` **D6**, **D7** and **D31**:

1. A **factual claim about the rules** (which Virtue grants which Reputation, at
   which level, in which magnitude) — the **rulebook** wins, always.
2. An **entry's name** — **the table wins** (D31). This reverses the original
   rule, which gave the heading precedence: our German rulebook copy is **older**
   than the tables, so convicting a table row with a stale heading is circular.
   The heading regains precedence only once a current German rulebook lands, at
   which point D31 retires.
3. **Other terminology** — thematic table before `tugenden-fehler.md`, which is
   the broadest and so the likeliest to be wrong.
4. A row tagged with **another book's acronym** (`HdH:WL`, `SdM:G`) is not in
   the dispute at all; it describes that book's terminology.

`README.md` indexes the tables, carries the audit warning and the corrected-error
log, and records resolved naming conflicts;
`uebersetzungsregeln.md` gives the prose/formatting rules (natural German
syntax, `n/a`→`n/v`, Latin terms kept untranslated, etc.). Key conventions:
Latin terms tagged `(Lat.)` stay untranslated; the type-label `Tainted` →
`Befleckt` (but the flaw name `Depraved` → `Verdorben`); supernatural abilities
are marked `*`. The `Anmerkung` column carries gender/number and disambiguation
notes.

**Provenance is per-language.** Each item's `source` field (`SourceRef`:
`{ file, lines: [start, end] }`) records the file basename plus an inclusive
line range. The `source` in `rules/core/` always points at the **English**
file — the canonical source. German source files mirror the English ones
**line-by-line throughout** — even lists that are alphabetically ordered in
English (e.g. Virtues, Flaws, Abilities) are **not** re-sorted into German
alphabetical order; each German entry keeps the same line position as its
English counterpart. So a German line number corresponds to the same item as
the English line number. Such per-language provenance, when needed, lives in
the `rules/i18n/<lang>/` layer, not in language-neutral `core/`.

**NEVER change the number of lines in a file under `rules/source/`.** Not to fix
a defect, not to improve formatting, not as part of any other change. An edit
that adds or removes a line renumbers everything below it and silently
invalidates **every** citation into that file — the `SourceRef` ranges in
`rules/core/` and `rules/i18n/`, the `// Source: ArMDE:NNNN` comments, `RULES.md`,
and every dated record under `docs/`. Nothing detects it:
`crates/arm-rules/tests/rulebook_citations.rs` checks only that a cited range
lands on **non-blank lines**, so after a shift every citation still *passes*
while pointing at the wrong text. A whole-file re-sync from upstream is a
different act and is not covered by this rule; a hand edit is.

**And `rules/source/` is a COPY — repair defects upstream, not here.**
`../arm-de-translation` holds the authoritative originals, for
the **English** text as well as the German: its
`original-english/reviewed/…Core Rules.md` is **newer than ours and 80 lines
longer** (25,803 against 25,723, measured 2026-09-24). So a hand repair in this
repo is throwaway work — the file is replaced wholesale at the next re-sync, and
the edit is lost along with it.

A source defect is therefore **recorded here and fixed there**, as a todo in
`arm-de-translation/docs/todo.md`. In-place same-line edits here are not *wrong*,
but they buy only the interval until the next sync, and they widen the drift
between the two copies meanwhile. Prefer the upstream fix; make a local one only
when something shipping depends on it now.

**Corollary, and it is uncomfortable: every `ArMDE:NNNN` citation in this
repository is already offset from the authoritative file.** That is not a reason
to renumber anything — it is the reason `source.anchor` exists and is mandatory
(see `docs/vf-audit/decisions.md` **D30**).

## Architecture invariants

- **Catalogue size is data, never code.** The engine and UI must never assume the
  *number* of Abilities, Virtues/Flaws, or any catalogue entry. Catalogue size is
  purely a function of the rules JSON, so Abilities and V/F are added by editing
  `rules/core/*.json` + `rules/i18n/<lang>/*.json` with **zero code changes** —
  this is what keeps the executable and the rules separable (e.g. pulling the full
  ability catalogue from the `full-abilities` branch is a data-only change). Tests
  assert structural invariants (a known item is present, a flag/category is read
  correctly), **never exact catalogue totals**. Fixed *taxonomies* the rules define
  (magnitude tiers, ability categories) stay Rust enums, but the UI must not
  re-hardcode their values — the engine surfaces them (`Magnitude::points` →
  `Ruleset.magnitude_points`, `AbilityCategory::ALL` → `Ruleset.ability_category_order`)
  so there is a single source of truth.
- **Engine purity.** `arm-rules` has NO dependency on `tauri`, filesystem, or UI.
  It operates on in-memory data (`&str` / `&[u8]`), parses rulesets and entities,
  evaluates them, and returns results. Fully testable with `cargo test`.
- **Entity-generic design.** Characters and covenants share one buildable-entity
  abstraction (`EntityKind`). No character-specific assumptions in the engine;
  covenant support = new data + thin UI, never an engine rewrite.
- **Character types are data-driven profiles.** Each type (grog, companion,
  mythic companion, magus) is a JSON profile defining: V/F budget, caps,
  permitted/forbidden categories, required/forbidden traits, Gift policy, and
  ordered creation phases.
- **One evaluation path.** The engine always computes validation results.
  `ValidationMode` (Enforced / Advisory / Silent) governs enforcement at the
  caller level. No separate "without rule checking" branch.
- **Prerequisites as recursive boolean expressions.** `Prereq` enum with
  `All`, `Any`, `None`, `Has`, `House`, `AbilityMin`, `ArtMin`,
  `HermeticallyTrained`, `OrderMember`, `IsCompanion` —
  exhaustive `match` so adding a variant is a compile error until handled.
- **Two-file separation per rules domain.** Language-neutral mechanics in
  `rules/core/`, translatable text in `rules/i18n/<lang>/`, joined by stable
  slug-style IDs (`virtue.puissant_ability`).
- **Strict separation of data kinds.** Mechanics files: zero translatable strings.
  UI strings: only in Fluent `.ftl`. Rules text: only in `rules/i18n/<lang>/`.
  No user-facing string hardcoded in Rust or Svelte source. A raw ID or enum
  value (category, magnitude, kind, …) must **never** be rendered directly as a
  user-facing label — always map it through a Fluent key
  (`category-<id>`, `magnitude-<id>`). Rendering the slug itself is the same
  violation as hardcoding a string.

## Mandatory product behaviors

These are load-bearing user-facing guarantees. A change that touches the
relevant surface MUST preserve them and keep their tests green.

- **Unsaved-changes guard.** Closing **or** quitting the app with unsaved edits
  MUST prompt for confirmation before discarding — on every platform and every
  quit path, including macOS **Cmd+Q**. The dirty flag lives in the frontend
  store (`AppStore.dirty` in `ui/src/lib/state.svelte.ts`, a snapshot-compare
  against the last save/load baseline) and is mirrored to Rust via the
  `update_close_guard` command; the Rust `on_window_event` /
  `RunEvent::ExitRequested` handlers in `crates/arm-app/src/main.rs` own the
  actual confirmation dialog. Any change to save/load or the window/app
  lifecycle must keep this guard intact and the dirty-tracking tests in
  `state.svelte.test.ts` passing.

## Engineering conventions

- **TDD mandatory, and strictly test-first.** Red → green → refactor, in that
  order, for every unit of behavior — including pure helpers. No implementation
  code without a preceding failing test. Commits should reflect the cycle.
  - **Red must fail for the right reason.** Run the test and *read* the failure.
    What fails must be the assertion about the behavior under test — not a typo,
    a missing import, a wrong fixture path, an unrelated panic, or a test that
    never ran because the file name does not match its vitest project (see
    "Frontend test environments"). In Rust, a test naming a function that does
    not exist yet fails to **compile**; that is a legitimate *first* red only
    until the signature exists — you must still see the assertion itself fail
    before writing the body. A red nobody read is not a red.
  - **Green minimally.** Write the least implementation that turns that test
    green, then run it and confirm it did.
  - **Refactor green.** Restructure only with the suite passing, re-running it
    as you go. A refactor changes structure, never behavior; wanting new
    behavior means going back to red.
  - **Delegated work must show its reds.** An implementation subagent's report
    MUST contain the actual RED output for each test it wrote and the GREEN
    output afterwards — pasted verbatim, not summarized, not "I followed TDD".
    A report without them is rejected and the slice re-run. This is the repo's
    *only* TDD enforcement: there are deliberately no git hooks and no
    commit-shape checks, and CI can only ever prove the end state is green —
    it cannot distinguish test-first from test-after.
- **Human-readable code.** Intention-revealing names, small single-purpose
  functions, early returns over deep nesting. A domain expert should be able to
  follow the rules engine by reading it.
- **Code style enforced.**
  - Rust: `rustfmt` + `clippy` (warnings as errors: `#![deny(clippy::all)]`)
  - Svelte/TS: `prettier` + `eslint`
- **Edit files with the Edit/Write tools, never the shell.** Create or modify
  files (source, data, tests, docs) with the dedicated Edit/Write tools only.
  Never write or append through shell redirection — no `cat >`, `cat >> … <<EOF`
  heredocs, `echo >`, `sed -i`, `tee`, or similar. Shell file-writes are opaque to
  the permission system (so they force an approval prompt that the Edit/Write path
  avoids) and bypass the harness's file-state tracking. Use the shell only for
  running commands (build, test, git), not for producing file content.
- **Read files with the Read tool, never `cat`/`head`/`tail`/`sed -n`.** To look at
  a file's content — source, data, docs, a log, a JSON report — use Read (with
  `offset`/`limit` for a slice of a big file) or Grep/Glob to search it. Shelling
  out to `cat file`, `cat file | tail -50`, `head -n 100 file`, or `sed -n '1,40p'
  file` is the wrong tool: it wastes a Bash round-trip, drops the harness's
  file-state tracking (so a later Edit can fail or clobber), and `cat | tail`
  additionally discards everything above the tail so the next question needs
  another call. `cat`/`head`/`tail` are legitimate only for trimming a **command's
  stdout** in a pipeline (`cargo tarpaulin … | tail -3`), never for reading a path.
- **Scratch and log artifacts live in the project-local `tmp/`, never `/tmp`.**
  `tmp/` is gitignored and is the single home for anything a run produces that is
  not committed: coverage reports, review findings, staged portable builds, test
  logs. Writing to the system `/tmp` scatters a run's artifacts outside the repo
  where they are invisible to `git status`, unreachable by the read-path
  containment check (so every read of them prompts), and shared with every other
  project on the machine. The one sanctioned exception is the e2e save/load
  fixture: the specs resolve it through `os.tmpdir()` (`ui/e2e/wdio.conf.js`)
  because the app's own file dialogs must write somewhere OS-native.
- **Negative signs are ASCII hyphens.** Every displayed negative sign (and the
  minus glyph on decrement/stepper buttons) uses the ASCII hyphen-minus `-`
  (U+002D), never the mathematical minus `−` (U+2212) — ASCII stays copy-paste
  clean and reads correctly in assistive tech. `formatSigned(n)` in
  `ui/src/lib/derive.ts` is the single source of truth for signed values;
  `.ftl` strings that prepend a sign to a value use the hyphen too.
- **A menu action's keyboard shortcut is the menu item's accelerator, and is
  declared nowhere else.** One chord, one owner. Every shortcut for an action
  the native menu offers is attached to that item
  (`accelerator_for`, `crates/arm-app/src/menu.rs`), in muda's cross-platform
  `CmdOrCtrl` notation rather than a hand-written branch per platform, and the
  webview must **not** carry a competing `keydown` handler for the same chord —
  two owners mean one press runs the action twice, which on a dirty document is
  two discard prompts for one Ctrl+N. Three things follow and are part of the
  rule. The OS *draws* the chord beside the label, which is the only reason the
  shortcuts are discoverable at all, so a shortcut that is not on the menu is
  a shortcut nobody can find — give it a menu item instead of a handler. The OS
  *dispatches* it above the webview (GTK matches the window's accel group before
  the focused widget sees the key), so `inert` and DOM state do not gate it; the
  gate travels to the OS as the item's enabled state (`MenuFlags`), and the
  platform refuses to activate a disabled item's accelerator. And the chord is
  therefore **not reachable from an e2e spec** — WebDriver delivers keys into
  the webview, not to the window — so a new chord is proved by the accelerator
  test in `crates/arm-app/tests/menu.rs` (which parses it with muda's own
  parser, because `MenuItem::with_id` silently drops one it cannot parse) plus
  an activation of the item itself, never by `browser.keys`. Widget-local keys
  — arrow navigation in a tablist, Escape in a dialog — are not shortcuts in
  this sense and stay in the component that owns them.
- **English** for all code, identifiers, comments, commit messages, and all
  assistant communication (chat responses, PR descriptions, status updates) —
  regardless of the language the user writes in.
- **Current dependencies.** Latest stable versions. Pin below latest only with
  documented reason.
- **Canonical serialization.** Sort object keys and arrays by `id`/`ref` before
  writing JSON. Use `BTreeMap` / explicit sort. Zero-noise git diffs.
- **Referential integrity validated at load.** Every `has`, `incompatible_with`,
  and parameter `ref` must resolve. Incompatibilities must be symmetric. Fail
  loudly with clear error listing offending IDs.
- **Saves store choices, not resolved values.** Record ruleset `id` + `version`.
  Schema-versioned (`schema_version` field).
- **YAGNI / KISS.** Build what is needed now, nothing speculative.
- **Committing on `main` is fine.** This repo's workflow commits directly to
  `main`; do not branch first or open a PR unless explicitly asked.
- **Keep `README.md` current.** `README.md` is the GitHub-facing project
  overview. When a change alters what the project does, its status, the tech
  stack, or the build/run commands, update `README.md` in the same change so it
  never drifts from reality.

## Rules provenance

- **Rules backed by source, never memory.** Every rule implemented in code or
  encoded as data MUST be taken from the authoritative Markdown in
  `rules/source/<lang>/` (English is the source of truth). This applies to
  *every* sourcebook, not just the core rules — e.g. hedge-wizard mechanics come
  from *Hedge Magic (Revised)*, House mysteries from *Houses of Hermes — Mystery
  Cults*. Implementing a rule from training-data recollection is prohibited: if
  the passage is not in the source files, the rule is not implemented until the
  source is added. A rule from a book with no English source in
  `rules/source/en/` yet (e.g. the Rhine Tribunal book) cannot be implemented,
  because English is the source of truth for IDs.
- **Cite the source at the implementation site, by acronym.** When implementing a
  mechanic in Rust, add a comment citing the book's **acronym** + inclusive line
  range — the acronym identifies which book, e.g. `// Source: ArMDE:2774`, or a
  range `ArMDE:16563-16617`. A citation naming a second book in the same
  sentence gets its own acronym (`RoP:M:5486`, `HoH:TL:1234`); a colon-bearing
  acronym is fine, since the line number is always the trailing run of digits
  after the *last* colon, which is what makes the form unambiguously parseable.
  A continuation within one citation repeats only the line number, not the
  acronym: `ArMDE:16565, :2232, :16575`. Never cite a bare line number with no
  acronym at all (`` `:2774` ``, `(:2774)`) — it is meaningless without the
  book — and never spell the book out by its full basename
  (`Ars Magica - Definitive Edition (Core Rules).md:2774`); the acronym is the
  only spelling. Verify every line range against the actual file before
  committing it — do not trust recalled numbers; the guard below only checks
  that a range lands on non-blank lines, not that it says what the comment
  claims.

  The nine acronyms, canonically defined in
  `rules/source/de/translation-tables/grundbegriffe.md:212-229` (whose `:212`
  row offers both `ArM5` and `ArMDE` for the core book — this repo uses `ArMDE`
  only, so `ArM5` is a rejected spelling):

  | Acronym | File under `rules/source/en/` |
  |---|---|
  | `ArMDE` | `Ars Magica - Definitive Edition (Core Rules).md` |
  | `HoH:TL` | `Ars Magica 5e - Houses of Hermes - True Lineages.md` |
  | `HoH:MC` | `Ars Magica 5e - Houses of Hermes - Mystery Cults.md` |
  | `HoH:S` | `Ars Magica 5e - Houses of Hermes - Societates.md` |
  | `HM:RE` | `Ars Magica 5e - Magic - Hedge Magic (Revised).md` |
  | `RoP:M` | `Ars Magica 5e - Realms of Power - Magic.md` |
  | `RoP:F` | `Ars Magica 5e - Realms of Power - Faerie.md` |
  | `RoP:D` | `Ars Magica 5e - Realms of Power - The Divine (Revised).md` |
  | `RoP:I` | `Ars Magica 5e - Realms of Power - The Infernal.md` |

  This is the second spelling the repo has used for this citation — an earlier
  pass (`docs/audit-2026-08.md`) normalized ~470 shorthand citations
  (`Core Rules.md`, `Core:NNNN`, `RoP:Magic`) that matched no real file, and
  caught four wrong line ranges hiding behind that shorthand along the way,
  each found only by opening the source file. Shorthand is safe here *only*
  because it is now mechanically resolvable:
  `crates/arm-rules/tests/rulebook_citations.rs` guards it — one test resolves
  every acronym to a real file (and rejects `ArM5`), one checks every cited
  range actually lands on non-blank lines in that file, one forbids the bare
  form, and one forbids the full-basename form — so a citation can no longer
  drift to a spelling nothing checks. Do not re-normalize back to full
  basenames; that would undo the guard's whole point. The guard is scoped
  incrementally by slice (D1a: `crates/arm-rules/src` and `crates/arm-app/src`;
  later slices widen it to `crates/*/tests`, `crates/arm-rules/RULES.md`,
  `ui/src`, and `docs/`) — see the roots function's doc comment for the current
  boundary.
- **Maintain the traceability map.** `crates/arm-rules/RULES.md` maps each rule
  → verbatim excerpt → source file:line → implementing function/file (and the
  JSON data value where the rule's number lives), organized by book. Update it
  in the same change as any mechanic. JSON files carry no comments, so RULES.md
  is the provenance home for rule values encoded as data (e.g. character-type
  budgets in `rules/core/character_types.json`).
- **A cross-reference to another *source file* names the symbol, never the
  line.** This is the opposite rule to the one above, and for the opposite
  reason: a rulebook line is stable because the book is never edited, while a
  source line moves under every edit above it and nothing tells the reader it
  moved. The form is `` `<file>::<symbol>` `` — the file (a full path, or any
  suffix of one that resolves: `types.rs`, `effective/xp.rs`) and the declared
  name, joined by `::` — as in `` `types.rs::Entity::selections` ``,
  `` `effective/xp.rs::spell_mastery_flow_pool` ``,
  `` `menu.rs::accelerator_for` ``. It cannot be confused with a rulebook
  citation (`ArMDE:3899`): this one requires a source extension before its `::`
  and carries no digits. Naming a file with **no** line and no symbol is also
  fine — imprecise, but it cannot rot. `crates/arm-rules/tests/source_citations.rs`
  enforces both halves: no comment in `crates/*/src`, `crates/*/tests` or
  `ui/src` may pin a source file by line, and every `::` symbol it does name
  must still be declared in the named file. Two things are deliberately outside
  that guard and must stay so. **`docs/`** — its several hundred line citations
  sit in dated historical records (implementation plans, reviews, findings
  sheets), where the line number is part of the snapshot and rewriting it to
  today's symbol would falsify the record, exactly as D1c decided for
  `docs/audit-2026-08.md`. **`crates/arm-rules/RULES.md`** — it has its own
  line-citation convention (`` `validate_caps` (:22) ``) that
  `rules_md_citations.rs` already *verifies*, so a blanket ban there would
  contradict a working guard rather than add to one. A reference into a
  **vendored dependency** at a pinned version
  (`muda-0.19.3/src/accelerator.rs:539-541`) keeps its line number: those bytes
  are frozen, the version is in the path, and the guard checks that version
  against `Cargo.lock`.

## Build & test commands

```bash
# Rust engine tests
cargo test -p arm-rules

# Rust workspace (all crates)
cargo test --workspace

# Clippy (warnings as errors) — `--all-targets` so tests and benches are linted too
cargo clippy --workspace --all-targets -- -D warnings

# Format check
cargo fmt --check

# Frontend dev server
cd ui && npm run dev

# Frontend type-check (svelte-check) — vitest does NOT type-check
cd ui && npm run check

# Frontend lint/format
cd ui && npm run lint && npm run format:check

# Frontend unit tests (both projects — see "Frontend test environments" below)
cd ui && npm run test:unit

# Full Tauri dev build
cargo tauri dev

# E2E tests (wdio + tauri-driver, drives the REAL release binary)
cd ui && npm run test:e2e

# Portable-layout smoke check: stages the binary + rules/ outside target/ and
# proves the app boots and loads its ruleset there. Separate run, not part of the
# suite above (it needs the staging step first).
cd ui && npm run test:e2e:portable
```

### Frontend test environments — pick the right one

`npm run test:unit` runs **two vitest projects**, declared in
`ui/vitest.workspace.ts`. Output lines are tagged `|ssr|` or `|client|` so you can
see which project a test ran under. `ui/vitest.config.ts` holds only the shared
Svelte plugin setup — it deliberately sets no `environment`, `include` or
`exclude`, because `extends` deep-merges that block into *both* projects.

| Project | Environment | Matches | Use it for |
|---|---|---|---|
| `ssr` | `node` | `src/**/*.test.ts`, `e2e/**/*.test.js` | **The default.** Pure logic, and components rendered with `render` from `svelte/server`. Fast, no DOM. |
| `client` | `happy-dom` | `src/**/*.client.test.ts` | Only when you must observe something that needs a **live component instance**. |

**Default to `ssr`.** Name a file `*.client.test.ts` only when the thing under
test cannot be observed without a mounted component:

- an **`$effect` body running** — SSR never executes one
- lifecycle hooks, focus management, event listeners, `bind:` two-way updates
- anything asserting on live DOM state rather than rendered markup

**Why this is two projects and not one flag.** SSR rendering and client mounting
need *different module resolution*, not just a different environment. `mount()`
exists only in Svelte's client build, which requires the `browser` resolve
condition — and that condition cannot be set globally: it resolves the whole
`svelte` package to its client build, after which every component calling
`onMount` fails under the SSR renderer with `lifecycle_outside_component`. So the
condition is scoped to the `client` project alone. A per-file *environment* switch
(`environmentMatchGlobs`) is not enough on its own; it changes the DOM, not the
resolution. Both failure modes were hit and confirmed while setting this up, so
please do not "simplify" the split back into one config.

**Why the `client` project exists at all.** Before it, the entire frontend suite
was SSR-only, so **no `$effect` in the codebase was executed by any test**. That
is a silent gap, not a loud one: an assertion placed after an effect that never
fires still reports green. It hid the fact that the unsaved-changes guard's
`update_close_guard` IPC mirror — an `$effect` keyed on `store.dirty`, and a
mandatory product behavior — had zero coverage.

`ui/src/lib/client-env.client.test.ts` guards the mechanism itself: it asserts a
real `document` exists and that `$effect` actually runs on a mounted component
(via the `EffectProbe.svelte` fixture, which is test-only and imported by nothing
in the app). If the resolution or environment mapping regresses, that file fails
loudly instead of effects silently never running. Keep it.

Writing a client test: `mount()` from `svelte`, then `flushSync()` to run the
scheduled effects before asserting, and `unmount()` afterwards. `happy-dom` is a
devDependency; it ships in nothing.

E2E tests drive the **real release binary** — the only layer that exercises the
shipped production binary through real IPC + bundled rules resources, so run it
when a change touches the app's runtime behavior. (Machine-specific setup —
installed tool paths, display availability — lives in the gitignored
`CLAUDE.local.md`, imported at the end of this file.) The standard suite runs from
`target/release`, where `BaseDirectory::Resource` already resolves to the exe dir,
so it does **not** cover the **portable** layout — where Resource points at a
system path that does not exist and only `load_ruleset`'s exe-dir fallback finds
the rules (see `crates/arm-app/src/commands.rs`). `npm run test:e2e:portable`
(`ui/e2e/wdio.portable.conf.js` + `stage-portable.js`) stages the binary and
`rules/` into the gitignored `tmp/portable/` and drives that copy.

### Required gate (must pass before any commit / "done" claim)

A change is not verified until **all** of these pass. The last command is the
authoritative one and is mandatory — it is the only step that exercises the
shipped production code path.

```bash
cargo test --workspace
# `--all-targets` lints the test and integration-test crates as well as the
# libraries. Without it test code goes unlinted, which is exactly where rot hides:
# the flag's introduction (M6/6b8d) turned up a `#[cfg(test)] mod` sitting in the
# middle of a source file, silently swallowing the doc comment of the function
# below it.
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --check
cd ui && npm run test:unit && npm run lint && npm run format:check && cd ..

# FULL RELEASE APP COMPILE — non-negotiable final gate.
# Runs the frontend build (svelte-check type-check + vite) via beforeBuildCommand
# AND compiles the release binary. `cargo test`/`clippy` and `npm run test:unit`
# (vitest) do NOT type-check the frontend or build the production app, so type
# errors and production-only breakage slip past every other gate — only this
# command catches them. This is what `./arm-char-gen.sh` runs to launch.
cargo tauri build --no-bundle
```

## Data model quick reference

### IDs
Slug-style, never translated: `virtue.puissant_ability`, `flaw.deficient_technique`,
`ability.awareness`, `art.creo`, `house.bjornaer`.

### Magnitudes
`free` = 0 points, `minor` = 1 point, `major` = 3 points.
Combined with `kind` (virtue costs, flaw grants) for balance computation.

### Prereq enum (Rust)
```rust
enum Prereq {
    All(Vec<Prereq>),
    Any(Vec<Prereq>),
    Nor(Vec<Prereq>),        // NOR — none may be present (serde tag "none")
    Has(Id),
    House(Id),
    AbilityMin { ability: Id, score: u8 },
    ArtMin { art: Id, score: u8 },
    HermeticallyTrained,
    OrderMember,
    IsCompanion,       // profile flag, true for companion + mythic_companion
    HasCategory(String),   // holds (bought or granted) an item of this category
    AgeMin(u32),       // entity.age >= this; unset age is Unknown, not False
    HasCategoryAtMagnitude { category: String, magnitude: Magnitude, item_kind: ItemKind },
        // magnitude/kind-filtered twin of HasCategory (e.g. "a Major Supernatural Virtue")
}
```

### ValidationMode
```rust
enum ValidationMode { Enforced, Advisory, Silent }
```
- `Enforced` — guided + direct-validated modes (blocks illegal states)
- `Advisory` — shows violations as non-blocking warnings
- `Silent` — suppresses validation display (unchecked mode)

## Local, machine-specific guidance

Personal, machine-specific notes that must **not** be published live in a
gitignored `CLAUDE.local.md`, imported here. This import silently no-ops on a
fresh clone that lacks the file, so the reference is safe to ship.

@CLAUDE.local.md
