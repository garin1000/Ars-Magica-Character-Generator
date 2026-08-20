---
name: full-review
description: Run architecture, UI, and QA review agents in parallel, fix all findings in a loop until zero remain. Enforces 95% test coverage.
whenToUse: When you want a comprehensive codebase review with automated fixing. Covers architecture invariants, API surface, and test quality/coverage.
---

# Full Review Skill

Run this inline — do NOT use the Workflow tool. Use Agent tool calls directly so that
permission prompts surface in the chat and progress updates are visible to the user.

## Instructions

Follow these steps exactly. Between every step, post a short status update in the chat
so the user can see progress.

### Command hygiene (ALL agents — required, paste into every agent prompt)

The project's `.claude/settings.local.json` pre-approves a set of command *prefixes* and
pre-sets `PATH` (cargo + node/npm are already on it). Background agents cannot raise
interactive approval prompts, so any command containing a stage that does not match an
allowlisted prefix is **auto-DENIED** (not queued for approval).

**Pipes and chains are fine** — the checker splits a command on `|`, `&&`, and `;` and
requires **every stage to match an allowlisted prefix**. So compose freely with pipelines.

**But matching a prefix is necessary, not sufficient.** Understand this before you start,
because misreading it is what sends agents hunting for workarounds. An allowlisted prefix
only clears the *first* of several independent checks; any later one can still deny the
command:

1. **Prefix match** — the stage must begin with an allowlisted prefix. `git -C <path> diff`
   fails, because it does not begin with `git diff`.
2. **Per-command flag whitelist** (`safeFlags`) — every known command has a table of
   permitted flags, git subcommands included. An unlisted flag is denied.
3. **Argument path containment** — commands classed as file *reads* (`grep`, `rg`, `cat`,
   `head`, `tail`, `od`, `jq`, `git`, …) have their positional file arguments extracted and
   checked against the working directory. Anything outside it is denied with *"Path is
   outside allowed working directories"* — `/dev/null` included.
4. **Special guards**, each with its own reason code: `cd-git-compound` (a `cd` and a `git`
   in one command), `multi-cd`, `cd-compound-redirect`, `shell-expansion`,
   `process-substitution`, `sed-dangerous`, `shell-operators`, `too-complex`.

So a command that looks entirely legitimate can still be denied, and that is **not**
evidence that the permission system is broken or that you need a cleverer spelling. Work
out which layer refused it and satisfy all four — or use the native Read/Grep/Glob tools,
which bypass this machinery completely. To stay inside it, every agent MUST:

- **A denial is information, not an obstacle to route around.** This is the governing
  rule; the specific bans below are only its worked examples. When a command is denied,
  do NOT construct a different spelling of the same thing — reaching for `perl` because
  `grep` returned nothing, or `git -C <path>` because a `cd` was refused, turns one
  blocked command into a second blocked command and burns the run. Stop and pick one of:
  (a) the native **Read/Grep/Glob** tools, which need no approval inside the repo and are
  usually the better answer anyway; (b) a genuinely different, allowlisted approach; or
  (c) report the blocker in your findings and move on. A refusal often means your premise
  is wrong — the `perl` incident began with a `grep` that was silently skipping a corrupt
  file, and no amount of tool-swapping would have revealed that.
- **Every stage must be an allowlisted command.** Allowed filter/util tools you can pipe
  through: `jq`, `grep`, `rg`, `sed`, `awk`, `sort`, `uniq`, `cut`, `tr`, `wc`, `head`,
  `tail`, `cat`, `diff`, `cmp`, `od`, `xxd`, `file`, `find`, `ls`, `echo`, `mkdir`.
  Build tools: `cargo …` (any
  subcommand), `npm run <script>`, `npx prettier/eslint/svelte-check/vitest/wdio` — `npx
  eslint <file>` is the way to lint a single file, since `npm run lint` is whole-tree.
  Read-only
  git: `git status`, `git diff`, `git log`, `git show`, `git check-ignore` — use these
  to inspect the working diff. A pipeline like `cargo tarpaulin … | tail -1` or
  `jq '.files | length' tmp/tarpaulin-report.json` auto-approves because each stage is
  allowlisted.
- **Never mutate git state.** `git push`, `checkout`, `reset`, `rebase`, `stash`, and
  `clean` are NOT allowlisted and are auto-denied, deliberately — they move refs or
  discard work. `git add`/`git commit` are allowlisted for the main session but a
  reviewer or fixer agent must NOT use them: committing is the orchestrator's call
  after the gates pass, never a subagent's. Leave your changes in the working tree.
- **An empty `grep` result does NOT mean "not present".** `grep` here is not GNU grep:
  Claude Code injects a shell function that re-execs its bundled **ugrep** as
  `grep -G --ignore-files --hidden -I …`. **`-I` is hardcoded**, so a file containing a
  NUL byte is skipped *silently* — no match, no warning, exit 0. If a search for
  something you are confident exists comes back empty, re-run it with **`grep -a`**
  (overrides `-I`) before concluding the symbol is missing or that your tools are
  broken. Also, ugrep's regex dialect is not GNU's — a bracket expression with hex
  ranges may fail with `ugrep: error at position N`; fall back to `rg -a` (NOT
  `command grep` — only `command -v` is allowlisted, so that is denied). The native
  Grep tool is ripgrep and skips binary files too.
- **Inspect bytes with `od`/`xxd`/`cat -v`/`cmp`/`file` — all allowlisted.** If a file
  looks corrupt or a search behaves impossibly, that is the toolkit. Escalating to an
  interpreter to hunt control characters is never justified and will be denied.
- **Parse JSON with `jq`, never an interpreter.** Do NOT shell out through `python`,
  `python3`, `perl`, `ruby`, `node -e`, `bash -c`, or `sh -c` — an interpreter is an
  arbitrary-code escape hatch that defeats the allowlist, and it is not allowlisted anyway.
  For the tarpaulin coverage %, read it directly: `cargo tarpaulin` prints a
  `XX.XX% coverage, N/M lines covered` summary line to stdout during the same run that
  writes the JSON — capture it (e.g. `… | tail -3`) rather than post-processing the
  1.5 MB report. If you truly need a field from the JSON, use `jq`.
- **Avoid arbitrary command executors when a direct form exists.** `xargs`, `find -exec`,
  `bash -c`, `sh -c`, and interpreter `-c`/`-e` flags run a command the allowlist cannot
  vet, so they are intentionally NOT allowlisted and will prompt/deny. Almost always there
  is a plain equivalent: prefer a shell glob or the tool's own file arguments over piping
  into an executor — e.g. `wc -l crates/arm-rules/src/*.rs` instead of
  `find … | xargs wc -l`, and `grep -r PATTERN <dir>` instead of `find … -exec grep …`.
  Only reach for an executor if there is genuinely no allowlisted alternative.
- **Avoid `$`-expansion syntax in commands.** The permission analyzer flags any `$(…)`,
  backticks, `$'…'` (ANSI-C quoting), and even `$VAR`/`$0`/`$1` field refs as unverifiable
  and prompts **regardless of the allowlist** — you cannot allowlist past them. Use plain
  forms: rely on `sort`'s default whitespace (space/tab) field-splitting, e.g.
  `… | sort -k2 -nr`, instead of `sort -t$'\t' …`. **A regex end-of-line anchor `$`
  counts as well** — the analyzer cannot distinguish an anchor from an expansion, so an
  otherwise fully allowlisted `grep -n "^## Heading$" file.md` is denied. Drop the anchor
  and tolerate the extra hits (`grep -n "^## Heading"`), or use the native Grep tool,
  which takes a real regex and needs no approval.
- **Scan code with `grep`/`rg`, not `awk`/`sed`.** `awk`/`sed` programs are built around
  `$1`/`$0`/`$` field references, which trip the `$`-expansion guard above and prompt even
  though `awk`/`sed` are allowlisted. To find items, use `grep`/`rg` with an extended
  regex — e.g. `grep -rnE '^\s*pub (fn|struct|enum|const|mod|trait)' src/` — or the native
  Grep tool. Reserve `awk`/`sed` for the rare transform with no `$` in it.
- **No shell control-flow — `for`, `while`, `if`, `case`.** Loops and conditionals cannot
  be decomposed into allowlisted prefixes, so the analyzer always prompts on them (and they
  usually carry `$var` too). Iterate with a glob or the tool's own multi-file arguments
  (`grep -nE … src/*.rs`, `cargo test -p arm-rules`), or use the native Read/Grep tools —
  never a `for f in …; do …; done` loop.
- **No brace groups or subshells — `{ … ; }` and `( … )`.** Compound commands built with
  `|`, `&&`, `||`, and `;` ARE allowed: the analyzer splits on those separators and vets
  each stage against the allowlist, so `cargo tarpaulin … | tail -3` and
  `cd <repo>/ui && npm run check` auto-approve. Grouping constructs do not decompose that
  way — the first token the analyzer sees is a bare `{` or `(`, which matches no
  allowlisted prefix, so the whole line is denied. Write the stages out flat with `&&`/`;`
  instead of wrapping them, and never use `{ … } > file` (redirects are banned anyway —
  use the Write tool). Brace *expansion* in an argument (`src/*.{rs,toml}`) is likewise
  out: use a plain glob or repeat the argument.
- **Never prefix a repo-root command with `cd <repo> &&`.** You already start in
  the repository root (`<repo>`), so that `cd` is dead weight — and it makes the
  analyzer prompt even though both stages are allowlisted, because relocating the shell
  changes what the next stage operates on. Observed repeatedly: `git log --oneline …`
  and `git diff … -- crates/…` run silently on their own, but
  `cd <repo> && git diff …` raises a dialog every time. Just
  issue the command with repo-relative paths (`git diff b49d49b..HEAD -- crates/…`,
  `sed -n '1,40p' crates/arm-rules/src/derived.rs`). The **only** place a `cd` is
  warranted is the UI gate, which genuinely must run inside `ui/` — see below.
  The precise rule (`bashMissKind: "cd-git-compound"`): a compound holding **both a `cd`
  and a `git`** is denied *regardless* of the allowlist and *regardless* of the path — it
  is not a containment check, so pointing at the repo you are already in does not help.
  Rationale: after a `cd`, git resolves its repository from the new directory, whose
  hooks are arbitrary executables. Two neighbours: **multiple `cd`s** in one command are
  denied (`multi-cd`), and a **`cd` plus an output redirect** is denied
  (`cd-compound-redirect`). A `cd` with a **non-git** command is unaffected, which is why
  the UI gate auto-approves.
  **`git -C <path> …` is NOT the way around this.** The allowlist grants the prefixes
  `git diff`, `git log`, `git show`, … and `git -C <path> diff` does not start with
  `git diff`, so no rule matches and it is denied just the same. There is no workaround
  to find here, because none is needed: you start in the repo root, so write the command
  bare — `git diff --stat -- crates/`, `git log --oneline b49d49b..HEAD`.
- **Every file argument must be inside the repo — including `/dev/null`.** File-reading
  commands (`grep`, `rg`, `cat`, `head`, `tail`, `od`, `jq`, `git`, …) have their
  positional file arguments extracted and containment-checked against the working
  directory; a path outside it is refused with *"Path is outside allowed working
  directories"* however well allowlisted the command is. `/dev/null` is exempt only as a
  redirect target, never as a file to read — `grep -n "PLAN.md" -m2 /dev/null` is denied.
  Never pass a placeholder file argument; just omit it.
- **Write artifacts with the Write tool, not shell redirects.** Redirects (`>`, `>>`,
  `tee`) are not allowlisted. To create `tmp/review-findings.json` or any file, use the
  Write/Edit tools (they work in-repo and under `tmp/` without approval). This includes
  parking a large diff in the scratchpad: `git diff … > …/derived.diff` is denied — pipe
  it through `head`/`grep`, or read the file with the Read tool instead.
- **Keep the output of an expensive run — via the tool's own log option, in `tmp/`.**
  Never throw away a full e2e or release-build log and re-run the suite to recover a
  line. Redirects are denied for you, so do not reach for `>`; use the option the tool
  already provides. For e2e that is wdio's `outputDir`, wired to `tmp/e2e-logs/` in
  `ui/e2e/wdio.conf.js` — a plain `cd <repo>/ui && npm run test:e2e` leaves the
  complete launcher/worker logs there, which you then inspect with the **Read** tool.
  If a tool offers no log option and the output is too long to keep in context, trim it
  in the same pipeline (`… | tail -40`) rather than re-running later.
- **All scratch output goes under the repo-local `tmp/`, never the system `/tmp`.**
  `tmp/` is gitignored and is where coverage reports, findings JSON, and staged builds
  belong. A `/tmp/...` path fails the argument-containment check, so every subsequent
  read of it is denied — you would be creating a file you cannot then look at. (The e2e
  specs' own `os.tmpdir()` fixture is a deliberate exception in committed code; do not
  imitate it for your own artifacts.)
- **To read a file, use the Read tool — not `cat`, `head`, `tail`, or `sed -n`.** Those
  are allowlisted as *pipeline filters for a command's stdout* (`cargo tarpaulin … |
  tail -3`), and that is the only thing they are for here. Pointing them at a path is a
  denial risk for no benefit: `Read` (with `offset`/`limit` for a slice) and
  `Grep`/`Glob` need no approval anywhere in the repo or under `tmp/`, return line
  numbers you can cite in a finding, and keep the harness's file-state tracking intact
  so a later Edit cannot silently clobber. `cat <file> | tail -50` is the specific
  anti-pattern: it burns a Bash call and throws away the 90% of the file you will ask
  for next.
- **Never prepend environment setup.** Do NOT add `source ~/.cargo/env`, `export PATH=…`,
  or `nvm use` — `PATH` is already configured via the settings `env`, so `cargo`, `npm`,
  and `node` resolve directly. (This supersedes any "source cargo/nvm first" note in older
  run recipes.)
- **Use the allowlisted build-command forms:** `cargo test/clippy/fmt/tarpaulin/tauri …`
  and `npm run <script>`. NEVER bare `npm`/`node`, `npm install`, `npm ci`, or
  **`npm --prefix <dir> run …`** — none of those match `npm run:*`, so they are denied.
- **Run the UI gate as one compound call:** `cd <repo>/ui && npm run <script>` — e.g.
  `cd <repo>/ui && npm run check`, where `<repo>` is the repository root spelled out
  as an absolute path. Both stages are allowlisted
  (`cd:*` + `npm run:*`) so the whole line auto-approves, and using the absolute path makes
  it independent of the current working directory. Do NOT rely on a bare `cd ui` from an
  earlier call persisting, and do NOT use `npm --prefix ui run …` (not allowlisted). Run
  `npm run check`, `npm run test:unit`, `npm run lint`, `npm run format:check` this way.
- File reads inside the repo and under the session scratchpad/`tmp/` need no approval —
  use the **Read** tool for them (see the `cat`/`head`/`tail` rule above); do not read
  unrelated out-of-repo paths.

### Phase 1: Setup

Run these bash commands (no agent needed):
1. `mkdir -p tmp`
2. Check `cargo tarpaulin --version` — if not installed, run `cargo install cargo-tarpaulin`
3. Confirm `tmp/` is in `.gitignore`

Post: "Setup complete."

### Phase 2: Review (parallel)

Launch **3 Agent calls in a single message** (so they run in parallel). Every agent prompt
MUST include the **Command hygiene** rules above (verbatim) in addition to PROJECT_CONTEXT,
so agents only emit allowlisted, single commands and never trip auto-denials. Each agent
must return structured JSON with this schema:

```json
{
  "reviewer": "<name>",
  "findings": [
    {
      "category": "<string>",
      "severity": "critical" | "major" | "minor",
      "file": "<path>",
      "line": <number>,
      "description": "<string>",
      "suggested_fix": "<string>"
    }
  ]
}
```

#### Agent 1: Architecture Reviewer

Prompt context (paste PROJECT_CONTEXT below + this):

> You are a world-class Architecture Reviewer with 30+ years of experience designing
> mission-critical systems. You are EXTREMELY picky. If something smells wrong, it IS wrong.
>
> Read CLAUDE.md and every source file in crates/arm-rules/src/.
>
> Check for ALL of the following — report every violation including pre-existing ones:
> 1. Engine purity violations (IO, filesystem, tauri, UI dependency)
> 2. Separation of concerns issues
> 3. Data-driven design violations (hardcoded character-type assumptions)
> 4. Entity-generic design violations (character-specific assumptions)
> 5. Prerequisite enum exhaustiveness (no catch-all wildcards)
> 6. Canonical serialization compliance (BTreeMap, sorted keys)
> 7. Module structure problems
> 8. Error handling quality
> 9. YAGNI violations
> 10. Code readability (naming, function size, early returns)
> 11. Consistency between CLAUDE.md and actual code

#### Agent 2: API Surface & UI-Readiness Reviewer

> You are a legendary API Surface & UI-Readiness Reviewer with decades of experience.
> Ruthlessly picky about ergonomics, naming, documentation, and type safety.
>
> Read every source file in crates/arm-rules/src/.
>
> Check for ALL of the following:
> 1. Public type ergonomics
> 2. Missing pub items needed by Tauri+Svelte consumer
> 3. Overly-public items leaking implementation details
> 4. Naming conventions
> 5. Documentation — public types/functions need /// doc comments
> 6. Serde attributes — will JSON shapes work for Svelte frontend?
> 7. Display/Debug implementations
> 8. Missing convenience constructors or builder patterns
> 9. Error type usability
> 10. Missing From/Into/AsRef implementations

#### Agent 3: QA Reviewer

> You are an elite QA Reviewer with 25+ years of experience. MERCILESS about coverage
> and test quality.
>
> Run: cargo test, cargo clippy (warnings as errors), cargo fmt --check,
> cargo tarpaulin -p arm-rules --out json --output-dir tmp/
>
> If ui/ exists, also run its type-check, tests, lint, and format — each as a
> `cd <repo>/ui && npm run <script>` compound (see Command hygiene), e.g.
> `cd <repo>/ui && npm run check` (with `<repo>` as the absolute repository root),
> then `npm run test:unit`,
> `npm run lint`, `npm run format:check`.
> `npm run check` (svelte-check) is MANDATORY: vitest does NOT type-check, so TS
> type errors (including in test files) pass `test:unit` yet break the build.
> If ui/ does not exist, skip — do NOT report it.
>
> Then run the FULL RELEASE APP COMPILE (mandatory, the authoritative gate):
> `cargo tauri build --no-bundle`. This runs the frontend build (svelte-check +
> vite) via beforeBuildCommand AND compiles the release binary — the only step
> that exercises the shipped production code path. A failure here is ALWAYS a
> critical finding, even if every other gate is green.
>
> Check for ALL of the following:
> 1. Test coverage must reach 95%. Report exact percentage. List uncovered functions.
> 2. Test quality — meaningful assertions, not just compilation
> 3. Missing edge case tests
> 4. Missing negative tests
> 5. Test organization
> 6. Clippy compliance
> 7. Formatting compliance
> 8. Test naming
> 9. Missing roundtrip/property tests for serialization
> 10. Missing tests for error message content
> 11. Frontend type-check (`npm run check`) clean
> 12. Full release app compile (`cargo tauri build --no-bundle`) succeeds

### After reviews complete

Post a summary to the chat:
- Per-reviewer finding counts broken down by severity
- Total findings

If total is 0, skip to Phase 5.

Write findings to `tmp/review-findings.json`.

### Phase 3: Fix (sequential)

Run fixer agents **one at a time** (sequential, not parallel) to avoid file conflicts.
Post a status update before and after each fixer.

For each category (architecture, API surface, QA) that has findings:

1. Post: "[1/N] Starting Architecture fixer — X findings..."
2. Launch Agent with the findings list and instructions to fix them, run tests after.
3. Post: "[1/N] Architecture fixer done."
4. Repeat for API surface and QA fixers.

Each fixer agent must run `cargo test -p arm-rules` after fixing and ensure tests pass.
A fixer that touched ui/ must also run svelte-check via
`cd <repo>/ui && npm run check` (see Command hygiene) so TS type errors are caught; vitest
does not type-check. The QA fixer must also run clippy, fmt, and tarpaulin. All commands
follow the Command hygiene rules: allowlisted stages only, no `npm --prefix`, no
env-sourcing.

### Phase 4: Verify

Run a verification agent (or bash commands directly) to confirm:
1. `cargo test --workspace` — all pass
2. `cargo clippy --workspace -- -D warnings` — clean
3. `cargo fmt --check` — clean
4. `cargo tarpaulin -p arm-rules --out json --output-dir tmp/` — coverage percentage
5. If ui/ exists: `cd <repo>/ui && npm run check`, then `npm run test:unit`,
   `npm run lint`, `npm run format:check` — each a `cd <repo>/ui && npm run <script>`
   compound (see Command hygiene) — all clean
6. **FULL RELEASE APP COMPILE (mandatory): `cargo tauri build --no-bundle`** — must
   succeed. This is the only gate that type-checks the frontend and builds the
   production app/binary; `cargo test`/`clippy` and vitest do not. Convergence
   (Phase 5) MUST NOT be declared unless this passes.

Post results to chat.

### Loop

After Phase 4 (Verify), **always go back to Phase 2** and re-run all 3 reviewers from
scratch against the current state of the code. The reviewers must evaluate the code as it
stands now — not diff against previous findings.

Continue looping (Phase 2 → 3 → 4 → repeat) until **all 3 reviewers return zero findings
in the same iteration**. Only then proceed to Phase 5.

Maximum 10 iterations as a safety cap. Post between each iteration:
"Iteration N complete — X findings remain. Re-running reviewers..."

### Phase 5: Summary

Post a final report to the chat:
- Total iterations completed
- Final test count and coverage percentage
- Whether clippy and fmt pass
- Any remaining issues
- List of files modified

## PROJECT_CONTEXT

Include this context in every reviewer and fixer agent prompt:

```
## Project: Ars Magica 5e Character Generator
Workspace: the repository root (this project's checkout)

### Key architecture invariants (from CLAUDE.md)
- Engine purity: arm-rules has NO dependency on tauri, filesystem, or UI
- Entity-generic design: Characters and covenants share one buildable-entity abstraction
- Character types are data-driven profiles (JSON)
- One evaluation path: engine always computes validation, ValidationMode governs enforcement
- Prerequisites as recursive boolean expressions (Prereq enum with exhaustive match)
- Two-file separation per rules domain (core/ + i18n/)
- Strict separation of data kinds (no user-facing strings in Rust source)
- Canonical serialization (BTreeMap, sorted keys)
- TDD mandatory, YAGNI/KISS
- Rust: rustfmt + clippy (warnings as errors via #![deny(clippy::all)])
- Human-readable code: intention-revealing names, small functions, early returns

### Workspace layout
- crates/arm-rules/src/ — pure engine (types.rs, ruleset.rs, validation.rs, lib.rs)
- ui/ — Svelte 5 frontend (may not exist yet)
- rules/ — JSON rule data (may not exist yet)
- tmp/ — review artifacts (gitignored)
```
