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

### Phase 1: Setup

Run these bash commands (no agent needed):
1. `mkdir -p tmp`
2. Check `cargo tarpaulin --version` — if not installed, run `cargo install cargo-tarpaulin`
3. Confirm `tmp/` is in `.gitignore`

Post: "Setup complete."

### Phase 2: Review (parallel)

Launch **3 Agent calls in a single message** (so they run in parallel). Each agent must
return structured JSON with this schema:

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
> If ui/ exists, also run its type-check, tests, lint, and format:
> `cd ui && npm run check && npm run test:unit && npm run lint && npm run format:check`.
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
A fixer that touched ui/ must also run `cd ui && npm run check` (svelte-check) so TS
type errors are caught — vitest does not type-check. The QA fixer must also run clippy,
fmt, and tarpaulin.

### Phase 4: Verify

Run a verification agent (or bash commands directly) to confirm:
1. `cargo test --workspace` — all pass
2. `cargo clippy --workspace -- -D warnings` — clean
3. `cargo fmt --check` — clean
4. `cargo tarpaulin -p arm-rules --out json --output-dir tmp/` — coverage percentage
5. If ui/ exists: `cd ui && npm run check && npm run test:unit && npm run lint && npm run format:check` — all clean
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
