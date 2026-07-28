<p align="center">
  <img src="arm5openlicenselogo.png" alt="Ars Magica Open License" width="420">
</p>

<h1 align="center">Ars Magica 5th Edition Character Generator</h1>

<p align="center">
  A locally installed, cross-platform desktop generator for <strong>Ars Magica 5th Edition</strong> characters and covenants.
</p>

<p align="center">
  <a href="#status"><img src="https://img.shields.io/badge/status-in%20development-orange" alt="Status"></a>
  <img src="https://img.shields.io/badge/platforms-Windows%20%7C%20macOS%20%7C%20Linux-blue" alt="Platforms">
  <img src="https://img.shields.io/badge/rust-2024%20edition-informational" alt="Rust 2024">
  <img src="https://img.shields.io/badge/license-MIT-green" alt="License">
</p>

---

## What it does

This is a native desktop application for building **Ars Magica 5th Edition**
characters — grogs, companions, mythic companions, and magi — as well as
**covenants**. Both characters and covenants are first-class entity types built
on one shared rules engine.

Three input modes cover the spectrum from hand-holding to power-user:

- **Guided wizard** — step through creation phase by phase, validated at each step.
- **Direct, validated** — enter choices freely with live rule checking.
- **Direct, unchecked** — full manual control, validation suppressed.

Everything runs locally. There is no server, no account, and no telemetry. Saves
are plain JSON that record your *choices* (plus the ruleset id and version), so
they stay portable and diff-friendly.

## Highlights

- **Two languages out of the box** — English and German, for both the UI and the
  rules text. English is the canonical source of truth; German terminology is
  matched against hand-curated EN↔DE glossary tables.
- **Rules-as-data.** Mechanics live in JSON keyed by stable slug IDs
  (`virtue.puissant_ability`, `art.creo`, `house.bjornaer`). Translatable text is
  kept strictly separate from mechanics and from UI strings.
- **One evaluation path.** The engine always computes the full validation result;
  a `ValidationMode` (Enforced / Advisory / Silent) decides how strictly it is
  applied — there is no separate "rules off" code branch. In Enforced mode a
  Virtue/Flaw that an already selected one excludes is greyed out with the reason,
  so the illegal pairing cannot be picked at all.
- **Referential integrity enforced at load.** Every prerequisite and
  incompatibility reference must resolve, and incompatibilities must be symmetric,
  or loading fails loudly with the offending IDs.
- **Source-backed rules.** Every implemented mechanic is traced to a line range
  in the authoritative rulebook Markdown — no rules from memory.
- **Works like a standard document app.** A tracked current file with New, Open,
  Save and Save As — plus the usual Ctrl/Cmd+N/O/S (Shift+S for Save As)
  shortcuts. The window title shows the file name and marks unsaved edits; Save
  writes straight to the current file while Save As always prompts.
- **Never lose work by accident.** Closing or quitting the app with unsaved
  changes — including via macOS Cmd+Q — prompts for confirmation before
  discarding, as does starting a new document or opening another file.
- **Export a readable character sheet.** One click writes the whole character to a
  Markdown file of your choosing — identity, Characteristics, Virtues and Flaws
  (including the ones a House or type granted), Abilities, Arts, spells,
  equipment, the computed combat/Soak/Fatigue/Wound values, the magic possessions,
  and the aging record — in the language the app is running in. Exporting is not a
  save: it never touches your current file or its unsaved-changes state.

## Architecture

The codebase is split so that all game logic is pure and independently testable,
with the desktop shell as a thin layer on top.

| Concern             | Choice                                                          |
|---------------------|----------------------------------------------------------------|
| Shell / packaging   | **Tauri 2** — native binary using the OS webview               |
| Core logic          | **Rust** — pure library crate (`arm-rules`), no IO, no UI deps |
| Frontend            | **Svelte 5 + Vite** (SPA, no SSR)                              |
| UI localization     | **Fluent** (`.ftl`)                                            |
| Rules + save format | **JSON** via `serde`                                           |

```
arm-char-gen/
├── crates/
│   ├── arm-rules/        # PURE engine: types, parse, validate, evaluate
│   └── arm-app/          # Tauri binary; file IO + commands
├── ui/                   # Svelte 5 + Vite frontend (SPA)
├── rules/
│   ├── source/<lang>/    # authoritative human-authored rulebook Markdown
│   ├── core/             # language-neutral mechanics (JSON)
│   └── i18n/<lang>/      # translatable rules text, keyed by stable IDs
├── locales/<lang>/       # Fluent UI strings
└── examples/             # sample saves for tests and demos
```

The engine (`arm-rules`) has **no dependency on Tauri, the filesystem, or the
UI**. It operates on in-memory data and is fully exercised with `cargo test`.
Characters and covenants share a single entity-generic abstraction, so covenant
support is new data plus thin UI — never an engine rewrite.

## Status

Active development. The engine, Tauri integration, and a direct-entry UI are in
place and tested end to end (engine unit tests, webview-free command integration
tests, and a real-binary `tauri-driver` E2E). Characters can be built with
virtues/flaws, point-buy Characteristics, whole bought Ability scores, and — for
magi — whole bought Hermetic Art scores, with Abilities and Arts drawing from one
shared experience pool, all validated live. A character-type selector switches
between grog, companion, mythic companion, and magus; the Arts, Spells, and Magic
Items tabs appear only for magus-capable types. The Magic Items tab stores a
magus's starting possessions — aura, enchanted devices (charged against the
item-level budget their Virtues grant), a familiar, a talisman, and a self-made
or external Longevity Ritual with its aging bonus, its focus, and the permanent
sterility it causes. Because the ritual's bonus was fixed by the Lab Total of the
season it was made, it is entered and stored rather than derived; beside the input
the engine suggests what a ritual made now would be worth, from the live Creo
Corpus Lab Total, halvings included. The talisman is the magus's personal
enchanted item: its shape and material, its shape-and-material attunements, and
the effects instilled in it, with a read-only capacity read-out (highest Technique
+ highest Form, in pawns of Vim vis) beside them. The familiar is entered as the
magical animal it is, in the rulebook's own creature format: the animal, its Magic
Might, the eight Characteristics, a (usually negative) Size, Personality Traits,
the Gold/Silver/Bronze bond cords, and the powers invested in the bond — with
read-only guidance for the bonding level (Might + 25 + 5 × Size), the best bonding
Lab Total, what the cords cost off the 5/15/30/50/75 curve, and the total level
invested. The bond's own grants (True Friend, Loyal (partner) +3, Intelligence -3)
are shown as a note rather than applied silently, and no number the familiar
carries can ever raise a validation error. An Equipment tab — available to every
character type — records the weapons, shields,
and armor a character carries from the Core Rules equipment catalogue (English +
German), marking each equipped. The full Core Rules Ability catalogue and the
full 15-Art catalogue (English + German) ship with localized descriptions;
abilities also carry example specialties and the "cannot be used untrained"
marker, and the complete Core Rules spell catalogue ships with each spell's
Range/Duration/Target and ritual legality. Score-boosting Virtues
(Puissant Ability, Puissant Art +3, Great Characteristic) compute an effective
score that drives prerequisites, the +5 characteristic ceiling, and a read-only
display badge. Virtues with experience effects are modelled too: Affinity
(Ability/Art) reduces the XP charged for its target, the restricted-pool Virtues
(Educated, Warrior, Privileged Upbringing) grant experience spendable only on
eligible Abilities (allocated by a max-flow solve so overlapping pools resolve
correctly), Improved Characteristics raises the Characteristic-buy budget, and
starting-score Virtues (Second Sight, Premonitions) confer their Ability at a
free floor. The Details tab captures identity/flavor fields (name, gender, birth
year, Wizard's sigil, covenant, parens) and an already-aged / already-warped
character's raw state — aging points and completed Characteristic reductions,
Warping Points, and free-text Twilight Scars — from which the engine derives the
Decrepitude and Warping scores (aging reductions lower derived/play stats but never
the point-buy the creation checks read). Every in-play Virtue/Flaw effect
(Magical Focus, Method Caster, Deficient Technique/Form, Tough, and the rest of
the 653-entry catalogue) is modelled, and a read-out panel computes the full
in-play totals in-engine — per-Technique/Form Lab and Casting Totals, per-spell
Penetration, per-Form Magic Resistance, combat lines (a one-handed weapon reads out
twice while a shield is equipped: with the shield, and bare), Soak,
Encumbrance, fatigue and wound ranges, and the stored Longevity bonus beside the
suggestion for a ritual made today — from a numeric aura input. With this any core-rules character is fully
enterable **and** fully computable in direct entry (milestone M5 complete). A
finished character can then be exported as a formatted Markdown sheet (M5.6): the
engine assembles the document from the character and the localized rules text while
the section headings come from the UI's own Fluent strings, so the sheet is
localized without any user-facing text living in the engine, and the per-line
casting/lab read-outs are deliberately left out as export noise. Next comes the
guided creation wizard with its life-stage XP flow, which merely orchestrates these
existing input surfaces — see [PLAN.md](PLAN.md) for the milestone breakdown.

## Getting started

### Prerequisites

- **Rust** (2024 edition) and Cargo
- **Node.js** + npm (for the Svelte frontend)
- The [Tauri 2 prerequisites](https://v2.tauri.app/start/prerequisites/) for your
  platform (on Linux, this includes WebKitGTK)

### Run the app in development

```bash
cargo tauri dev
```

### Run the test suites

```bash
# Rust engine tests
cargo test -p arm-rules

# Entire Rust workspace
cargo test --workspace

# Lint (warnings treated as errors) and format check
cargo clippy --workspace -- -D warnings
cargo fmt --check

# Frontend
cd ui && npm run dev          # dev server
cd ui && npm run lint         # eslint + prettier
cd ui && npm run test:e2e     # headless E2E (needs webkit2gtk-driver on Linux)
```

## Rules sources & licensing

This project deliberately separates two bodies of work under two licenses:

- **The generator's own source code** — the Rust crates, the Svelte/TypeScript
  frontend, build config, and the mechanics/UI data authored for this project
  (`rules/core/`, `rules/i18n/`, `locales/`, `examples/`) — is licensed under the
  **MIT License**. See [`LICENSE`](LICENSE).
- **The Ars Magica rulebook content** in `rules/source/` is derived from material
  ©1993–2024 Trident, Inc. d/b/a Atlas Games, redistributed under the
  **Ars Magica Open License** (Creative Commons Attribution-ShareAlike 4.0). See
  [`rules/source/LICENSE`](rules/source/LICENSE). The authoritative Markdown lives
  in `rules/source/<lang>/`; the JSON in `rules/core/` and `rules/i18n/` is
  generated from it.
- **Rule provenance** — every implemented mechanic is traced back to its source
  passage (book + line range) in
  [`crates/arm-rules/RULES.md`](crates/arm-rules/RULES.md), the single
  traceability map for the engine.

The rulebook Markdown originates from these Open License repositories:

- **English** — [OriginalMadman/Ars-Magica-Open-License](https://github.com/OriginalMadman/Ars-Magica-Open-License),
  via the fork at [garin1000/Ars-Magica-Open-License](https://github.com/garin1000/Ars-Magica-Open-License)
- **German** — [garin1000/Ars-Magica-Open-License-German](https://github.com/garin1000/Ars-Magica-Open-License-German)

*Ars Magica is a trademark of Trident, Inc. d/b/a Atlas Games. This is an
unofficial fan tool and is not affiliated with or endorsed by Atlas Games.*
