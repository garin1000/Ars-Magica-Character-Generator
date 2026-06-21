export const meta = {
  name: 'full-review',
  description: 'Run architecture, UI, and QA review agents in parallel, fix all findings in a loop until zero remain. Enforces 95% test coverage.',
  whenToUse: 'When you want a comprehensive codebase review with automated fixing. Covers architecture invariants, API surface, and test quality/coverage.',
  phases: [
    { title: 'Setup', detail: 'Install coverage tooling and create tmp directory' },
    { title: 'Review', detail: 'Three parallel review agents analyze the codebase' },
    { title: 'Fix', detail: 'Fixer agents address all findings' },
    { title: 'Verify', detail: 'Re-run tests, clippy, and fmt to confirm fixes' },
    { title: 'Summary', detail: 'Final report' },
  ],
}

const FINDING_SCHEMA = {
  type: 'object',
  properties: {
    reviewer: { type: 'string' },
    findings: {
      type: 'array',
      items: {
        type: 'object',
        properties: {
          category: { type: 'string' },
          severity: { type: 'string', enum: ['critical', 'major', 'minor'] },
          file: { type: 'string' },
          line: { type: 'integer' },
          description: { type: 'string' },
          suggested_fix: { type: 'string' },
        },
        required: ['category', 'severity', 'file', 'description', 'suggested_fix'],
      },
    },
  },
  required: ['reviewer', 'findings'],
}

const MAX_ITERATIONS = 10

const PROJECT_CONTEXT = `
## Project: Ars Magica 5e Character Generator
Workspace at /home/norbert/Rolle/arm-char-gen

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
`

const ARCH_REVIEW_PROMPT = `${PROJECT_CONTEXT}

You are an Architecture Reviewer for this project. Read CLAUDE.md and every source file in crates/arm-rules/src/.

Check for ALL of the following — report every violation, including pre-existing ones:
1. Engine purity violations (any IO, filesystem, tauri, or UI dependency)
2. Separation of concerns issues (logic in wrong module)
3. Data-driven design violations (hardcoded character-type assumptions)
4. Entity-generic design violations (character-specific assumptions that should be generic)
5. Prerequisite enum exhaustiveness (no catch-all wildcards hiding new variants)
6. Canonical serialization compliance (BTreeMap, sorted keys)
7. Module structure problems (re-exports, circular deps)
8. Error handling quality (clear messages with offending IDs)
9. YAGNI violations (speculative code not needed yet)
10. Code readability (naming, function size, early returns vs deep nesting)
11. Consistency between CLAUDE.md and actual code

Return structured JSON. Empty findings array if nothing found.`

const API_REVIEW_PROMPT = `${PROJECT_CONTEXT}

You are an API Surface & UI-Readiness Reviewer. Read every source file in crates/arm-rules/src/.

Check for ALL of the following — report every issue, including pre-existing ones:
1. Public type ergonomics — easy to construct, query, display?
2. Missing pub items needed by a Tauri+Svelte consumer
3. Overly-public items leaking implementation details
4. Naming conventions — Rust conventions and domain language
5. Documentation — public types/functions need /// doc comments
6. Serde attributes — will JSON shapes work well for a Svelte frontend?
7. Display/Debug implementations — useful for UI and debugging?
8. Missing convenience constructors or builder patterns
9. Error type usability — can Tauri commands convert errors to user messages?
10. Missing From/Into/AsRef implementations for ergonomics

Return structured JSON. Empty findings array if nothing found.`

const QA_REVIEW_PROMPT = `${PROJECT_CONTEXT}

You are a QA Reviewer. Check BOTH Rust backend and frontend (if ui/ exists).

### Rust (always check):
Run these commands:
1. cargo test -p arm-rules
2. cargo clippy -p arm-rules -- -D warnings
3. cargo fmt -p arm-rules -- --check
4. cargo tarpaulin -p arm-rules --out json --output-dir tmp/ (if installed)

### Frontend (only if ui/ directory exists):
Check if ui/ exists. If it does:
1. cd ui && npm test (or npm run test)
2. cd ui && npm run lint
3. cd ui && npm run format:check
4. Check for coverage configuration and run it
If ui/ does not exist, skip frontend checks — do NOT report missing frontend as a finding.

### Check for ALL of the following — report every issue including pre-existing ones:
1. Test coverage must reach 95% (Rust, and frontend if it exists). Report exact percentage. List every uncovered function/branch with file and line.
2. Test quality — verify behavior, not just compilation. Specific assertions.
3. Missing edge case tests (empty inputs, boundaries, error paths, all enum variants)
4. Missing negative tests (invalid inputs that should fail)
5. Test organization and effective use of helpers
6. Clippy compliance
7. Formatting compliance
8. Test naming — names should describe the scenario
9. Missing roundtrip/property tests for serialization
10. Missing tests for error message content

If cargo-tarpaulin is not installed, report that as a critical finding with suggested_fix: "Run: cargo install cargo-tarpaulin"

Return structured JSON. Empty findings array if nothing found.`

// --- Setup ---
phase('Setup')
log('Ensuring tmp/ directory and coverage tooling exist')

await agent(
  `In the project at /home/norbert/Rolle/arm-char-gen:
1. Create tmp/ directory if it doesn't exist: mkdir -p tmp
2. Check if cargo-tarpaulin is installed: cargo tarpaulin --version
   If not installed, run: cargo install cargo-tarpaulin
3. Confirm tmp/ is in .gitignore (it should already be)
Report what you did.`,
  { label: 'setup', phase: 'Setup' }
)

let iteration = 0
let totalFindings = -1

while (totalFindings !== 0 && iteration < MAX_ITERATIONS) {
  iteration++
  log(`--- Iteration ${iteration} of ${MAX_ITERATIONS} ---`)

  // --- Review phase ---
  phase('Review')
  log(`Running 3 review agents in parallel (iteration ${iteration})`)

  const reviews = await parallel([
    () =>
      agent(ARCH_REVIEW_PROMPT, {
        label: `arch-review-${iteration}`,
        phase: 'Review',
        schema: FINDING_SCHEMA,
      }),
    () =>
      agent(API_REVIEW_PROMPT, {
        label: `api-review-${iteration}`,
        phase: 'Review',
        schema: FINDING_SCHEMA,
      }),
    () =>
      agent(QA_REVIEW_PROMPT, {
        label: `qa-review-${iteration}`,
        phase: 'Review',
        schema: FINDING_SCHEMA,
      }),
  ])

  // --- Merge findings ---
  const allFindings = []
  const reviewerNames = ['architecture', 'api_surface', 'qa']

  for (let i = 0; i < reviews.length; i++) {
    const review = reviews[i]
    if (review && review.findings) {
      for (const finding of review.findings) {
        allFindings.push({ ...finding, reviewer: reviewerNames[i] })
      }
    }
  }

  totalFindings = allFindings.length
  log(`Found ${totalFindings} total findings in iteration ${iteration}`)

  if (totalFindings === 0) {
    log('All reviewers report zero findings. Done!')
    break
  }

  // Write findings to tmp file
  const findingsJson = JSON.stringify(allFindings, null, 2)
  await agent(
    `Write the following JSON content to /home/norbert/Rolle/arm-char-gen/tmp/review-findings.json, overwriting any existing content:

\`\`\`json
${findingsJson}
\`\`\``,
    { label: `write-findings-${iteration}`, phase: 'Review' }
  )

  // --- Fix phase ---
  phase('Fix')

  // Categorize findings
  const archFindings = allFindings.filter((f) => f.reviewer === 'architecture')
  const apiFindings = allFindings.filter((f) => f.reviewer === 'api_surface')
  const qaFindings = allFindings.filter((f) => f.reviewer === 'qa')

  // Run fixers sequentially to avoid file conflicts
  if (archFindings.length > 0) {
    log(`Fixing ${archFindings.length} architecture findings`)
    await agent(
      `${PROJECT_CONTEXT}

You are a Fixer agent for architecture and structural issues in this project.

Fix ALL of the following findings. Make minimum changes needed. Do not break existing tests.
After fixing, run: cargo test -p arm-rules
If any test fails, fix it until all tests pass.

Findings to fix:
${JSON.stringify(archFindings, null, 2)}`,
      { label: `fix-arch-${iteration}`, phase: 'Fix' }
    )
  }

  if (apiFindings.length > 0) {
    log(`Fixing ${apiFindings.length} API surface findings`)
    await agent(
      `${PROJECT_CONTEXT}

You are a Fixer agent for API surface, documentation, and UI-readiness issues.

Fix ALL of the following findings. Add doc comments, fix naming, improve ergonomics.
Do not break existing tests. After fixing, run: cargo test -p arm-rules

Findings to fix:
${JSON.stringify(apiFindings, null, 2)}`,
      { label: `fix-api-${iteration}`, phase: 'Fix' }
    )
  }

  if (qaFindings.length > 0) {
    log(`Fixing ${qaFindings.length} QA findings`)
    await agent(
      `${PROJECT_CONTEXT}

You are a Fixer agent for test coverage, test quality, and code quality issues.

Fix ALL of the following findings. Add missing tests to reach 95% coverage. Fix clippy warnings and formatting.
After fixing, run:
1. cargo test -p arm-rules
2. cargo clippy -p arm-rules -- -D warnings
3. cargo fmt -p arm-rules
4. cargo tarpaulin -p arm-rules --out json --output-dir tmp/

All must pass. If coverage is below 95%, add more tests until it reaches 95%.
If ui/ exists and has test/lint issues, fix those too.

Findings to fix:
${JSON.stringify(qaFindings, null, 2)}`,
      { label: `fix-qa-${iteration}`, phase: 'Fix' }
    )
  }

  // --- Verify phase ---
  phase('Verify')
  log(`Verifying fixes (iteration ${iteration})`)

  await agent(
    `In /home/norbert/Rolle/arm-char-gen, run these commands and report results:
1. cargo test -p arm-rules
2. cargo clippy -p arm-rules -- -D warnings
3. cargo fmt -p arm-rules -- --check
4. cargo tarpaulin -p arm-rules --out json --output-dir tmp/

If ui/ exists, also run:
5. cd ui && npm test
6. cd ui && npm run lint

Report: did all pass? What is the exact coverage percentage?
If any command fails, describe the failure.`,
    { label: `verify-${iteration}`, phase: 'Verify' }
  )
}

if (iteration >= MAX_ITERATIONS) {
  log(`Reached maximum iterations (${MAX_ITERATIONS}). Some findings may remain.`)
}

// --- Final summary ---
phase('Summary')
const summary = await agent(
  `In /home/norbert/Rolle/arm-char-gen:

1. Read tmp/review-findings.json if it exists
2. Run: cargo test -p arm-rules 2>&1
3. Run: cargo clippy -p arm-rules -- -D warnings 2>&1
4. Run: cargo fmt -p arm-rules -- --check 2>&1
5. Run: cargo tarpaulin -p arm-rules --out json --output-dir tmp/ 2>&1
6. If ui/ exists, run its tests too

Provide a final summary:
- Total iterations completed: ${iteration}
- Final test count and coverage percentage
- Whether clippy and fmt pass
- Any remaining issues
- List of files modified during the review`,
  { label: 'final-summary', phase: 'Summary' }
)

log('Full review workflow complete.')
