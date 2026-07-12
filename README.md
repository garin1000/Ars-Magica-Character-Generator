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
  applied — there is no separate "rules off" code branch.
- **Referential integrity enforced at load.** Every prerequisite and
  incompatibility reference must resolve, and incompatibilities must be symmetric,
  or loading fails loudly with the offending IDs.
- **Source-backed rules.** Every implemented mechanic is traced to a line range
  in the authoritative rulebook Markdown — no rules from memory.

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
item-level budget their Virtues grant), a familiar with its Gold/Silver/Bronze
bond cords, talisman attunements, and a self-made or external Longevity Ritual. A seed Ability catalogue and the full 15-Art catalogue
(English + German) ship with localized descriptions; abilities also carry example
specialties and the "cannot be used untrained" marker. The complete Core Rules
ability catalogue is staged for a later milestone. Score-boosting Virtues
(Puissant Ability, Puissant Art +3, Great Characteristic) compute an effective
score that drives prerequisites, the +5 characteristic ceiling, and a read-only
display badge. Virtues with experience effects are modelled too: Affinity
(Ability/Art) reduces the XP charged for its target, the restricted-pool Virtues
(Educated, Warrior, Privileged Upbringing) grant experience spendable only on
eligible Abilities (allocated by a max-flow solve so overlapping pools resolve
correctly), Improved Characteristics raises the Characteristic-buy budget, and
starting-score Virtues (Second Sight, Premonitions) confer their Ability at a
free floor. Next comes full mechanical completeness (the remaining V/F effects,
the full ability + spell catalogues, and derived combat/casting/lab totals), so
any core-rules character is fully enterable and computable in direct entry; the
guided creation wizard with its life-stage XP flow follows — see
[PLAN.md](PLAN.md) for the milestone breakdown.

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
