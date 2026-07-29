# Contributing

Thanks for looking at this. It is an unofficial fan tool for **Ars Magica 5th
Edition** — a locally installed desktop character and covenant generator, not
affiliated with, sponsored by, or endorsed by Atlas Games. See the
[README](README.md) for what it does and how it is built.

## Licensing — please read this first

This repository holds two differently licensed bodies of work, and the split
follows **what the content is**, not which folder it lives in. That makes
"which license does my change fall under?" less obvious than usual, so here it
is spelled out:

| What you are changing | License your contribution is under |
|---|---|
| Rust, Svelte/TypeScript, Fluent UI strings (`locales/`), sample saves (`examples/`), tooling (`scripts/`), build config | **MIT** |
| The rules JSON's *form* — schema, key names, the slug-ID scheme, file layout — including new mechanical fields | **MIT** |
| `rules/core/` entries: IDs, numbers, enums, prerequisite and grant structures | **MIT** |
| The rules *text* in `rules/i18n/<lang>/` string values — names, descriptions, summaries, specialties | **CC BY-SA 4.0** |
| Anything under `rules/source/` | **CC BY-SA 4.0** |

A few consequences worth stating outright:

- **Translating the rules text produces a derivative.** A brand-new
  `rules/i18n/<lang>/` full of translated descriptions is CC BY-SA 4.0, even
  though the schema it follows is MIT. Attribution must be preserved and it
  cannot be relicensed.
- **Reusing the format is free under MIT.** Writing your own data against this
  schema, or building tooling that reads it, needs only the MIT License.
- **Rulebook text may come only from the Ars Magica Open License editions** —
  [English](https://github.com/garin1000/Ars-Magica-Open-License) and
  [German](https://github.com/garin1000/Ars-Magica-Open-License-German). Never
  paste text from a PDF, from a non-Open-License Atlas product, or from memory.
- **Trademarks are not licensed.** Ars Magica, Mythic Europe, Order of Hermes
  and the rest are trademarks and are not covered by CC BY-SA 4.0. Do not add
  trademark-heavy naming or artwork.
- By opening a pull request you confirm you have the right to contribute the
  material under the license that applies to it.

Full terms: [`LICENSE`](LICENSE), [`rules/source/LICENSE`](rules/source/LICENSE),
[`rules/i18n/LICENSE`](rules/i18n/LICENSE),
[`rules/core/LICENSE`](rules/core/LICENSE).

> **If you edit `LICENSE`, keep it ASCII** and free of `\`, `{` and `}`. The
> Windows MSI installer splices that file's raw bytes into an RTF wrapper, so
> anything else renders as mojibake on the installer's license page.

## Rules provenance

Every rule in this project is backed by the source text, never by recollection.

- Take the rule from the authoritative Markdown in `rules/source/<lang>/`.
  **English is the source of truth** — IDs derive from the English text, so a
  book with no English source here cannot introduce new IDs.
- Cite the **source file basename plus an inclusive line range** at the
  implementation site, e.g.
  `// Source: Ars Magica - Definitive Edition (Core Rules).md:2774`. A bare line
  number is meaningless without the book. Verify the range against the actual
  file — do not trust remembered numbers.
- Record the rule in [`crates/arm-rules/RULES.md`](crates/arm-rules/RULES.md),
  the traceability map, in the same change. JSON carries no comments, so RULES.md
  is where provenance lives for values encoded as data.

If the passage is not in the source files, the rule is not implemented yet.

## How the project is shaped

A few invariants that reviews will hold you to:

- **The engine is pure.** `arm-rules` has no dependency on Tauri, the
  filesystem, or the UI. It operates on in-memory data and is fully testable
  with `cargo test`.
- **Catalogue size is data, never code.** Neither the engine nor the UI may
  assume how many Abilities or Virtues exist. Tests assert structural
  invariants, never exact totals.
- **Three kinds of data stay separate.** Mechanics in `rules/core/` (no
  translatable strings), rules text in `rules/i18n/<lang>/`, UI strings in
  `locales/*.ftl`. No user-facing string is hardcoded in Rust or Svelte, and a
  raw ID or enum value is never rendered as a label.
- **Saves store choices, not resolved values**, and are schema-versioned.
- **Canonical serialization.** Sort keys and arrays by `id`/`ref` before writing
  JSON, so diffs stay noise-free.
- **TDD.** Red, green, refactor — a failing test comes before the implementation.

## Development setup

You will need Rust (2024 edition) with Cargo, Node.js with npm, and the
[Tauri 2 prerequisites](https://v2.tauri.app/start/prerequisites/) for your
platform (on Linux that includes WebKitGTK 4.1).

```bash
cargo tauri dev
```

## Run the gate before opening a pull request

**There is no CI that runs these checks on your PR.** The only workflow in this
repository is the release build, which fires on version tags. A pull request
that has not been gated locally will be sent back, so please run all of it:

```bash
cargo test --workspace
cargo clippy --workspace -- -D warnings
cargo fmt --check
cd ui && npm run test:unit && npm run lint && npm run format:check && cd ..

# The authoritative step. `cargo test` and vitest do NOT type-check the frontend
# or build the production app, so type errors and production-only breakage slip
# past every other check — only this one catches them.
cargo tauri build --no-bundle
```

If your change touches the app's runtime behavior, also run the end-to-end suite
(`cd ui && npm run test:e2e`), which drives the real release binary.

## Pull requests

Keep them small and focused — one concern per PR, with the commit history
reflecting the red/green/refactor cycle. The maintainer
([`.github/CODEOWNERS`](.github/CODEOWNERS)) reviews everything, and always
reviews changes under `rules/`.

> You may notice `CLAUDE.md` says committing directly to `main` is fine. That is
> the maintainer's own workflow — they are a bypass actor on the branch ruleset.
> Contributions from everyone else come in as pull requests, where the code-owner
> review applies.

## Bugs and feature requests

Open a GitHub issue. For bugs, please include your OS, how you installed the app
(`.deb` / `.rpm` / `.AppImage` / `.msi` / `.exe` / portable archive), the version
shown in the app, and — if the problem involves a specific character — the
`.armc` save file.
