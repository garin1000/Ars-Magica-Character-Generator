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

You are a world-class Architecture Reviewer with 30+ years of experience designing mission-critical systems. You have seen every anti-pattern, every subtle design rot, every shortcut that became a permanent liability. You are legendary for catching issues that other reviewers miss.

You are EXTREMELY picky. You do not give the benefit of the doubt. If something smells wrong, it IS wrong. If a design choice is merely "okay" rather than "right," you flag it. You hold this codebase to the standard of production software that must be maintained for a decade.

Read CLAUDE.md and every source file in crates/arm-rules/src/. Scrutinize every line, every type, every module boundary.

Check for ALL of the following — report every violation, including pre-existing ones, no matter how small:
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

You are a legendary API Surface & UI-Readiness Reviewer with decades of experience designing public crate APIs and frontend-backend contracts. You have shipped dozens of production libraries and know exactly what makes an API a joy or a nightmare to consume. You are ruthlessly picky about ergonomics, naming, documentation, and type safety.

You treat every public type as a contract that will be used by thousands of downstream consumers. If a name is slightly misleading, if a doc comment is missing or vague, if a constructor is clunky, if a serde shape will cause frontend pain — you flag it without hesitation. "Good enough" is never good enough for you.

Read every source file in crates/arm-rules/src/. Examine every pub item, every derive, every field.

Check for ALL of the following — report every issue, including pre-existing ones, no matter how small:
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

You are an elite QA Reviewer with 25+ years of experience in test engineering and code quality. You have caught production outages that passed entire QA teams. You believe untested code is broken code, and poorly-tested code is a time bomb. You are obsessively thorough — you check every branch, every edge case, every error path.

You are MERCILESS about test coverage, test quality, and code hygiene. A test that merely compiles and runs green without meaningful assertions is worse than no test — it gives false confidence. You demand that every public function has tests covering happy path, edge cases, and error conditions. You flag vague test names, missing boundary tests, and inadequate assertions.

Check BOTH Rust backend and frontend (if ui/ exists).

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

// Helper to summarize findings by severity
function summarizeFindings(findings, reviewerName) {
  const critical = findings.filter(f => f.severity === 'critical').length
  const major = findings.filter(f => f.severity === 'major').length
  const minor = findings.filter(f => f.severity === 'minor').length
  const parts = []
  if (critical) parts.push(`${critical} critical`)
  if (major) parts.push(`${major} major`)
  if (minor) parts.push(`${minor} minor`)
  return `${reviewerName}: ${findings.length} findings (${parts.join(', ') || 'none'})`
}

// --- Setup ---
phase('Setup')
log('Setting up: ensuring tmp/ directory and coverage tooling exist...')

await agent(
  `In the project at /home/norbert/Rolle/arm-char-gen:
1. Create tmp/ directory if it doesn't exist: mkdir -p tmp
2. Check if cargo-tarpaulin is installed: cargo tarpaulin --version
   If not installed, run: cargo install cargo-tarpaulin
3. Confirm tmp/ is in .gitignore (it should already be)
Report what you did.`,
  { label: 'setup', phase: 'Setup' }
)

log('Setup complete. Starting review loop.')

let iteration = 0
let totalFindings = -1

while (totalFindings !== 0 && iteration < MAX_ITERATIONS) {
  iteration++
  log(`━━━ ITERATION ${iteration}/${MAX_ITERATIONS} ━━━`)

  // --- Review phase ---
  phase('Review')
  log(`Launching 3 review agents in parallel: Architecture, API Surface, QA...`)

  const reviewers = [
    { name: 'Architecture', key: 'architecture', prompt: ARCH_REVIEW_PROMPT },
    { name: 'API Surface', key: 'api_surface', prompt: API_REVIEW_PROMPT },
    { name: 'QA', key: 'qa', prompt: QA_REVIEW_PROMPT },
  ]

  const reviews = await parallel(
    reviewers.map((r) => () =>
      agent(r.prompt, {
        label: `${r.key}-review-${iteration}`,
        phase: 'Review',
        schema: FINDING_SCHEMA,
      }).then((result) => {
        const count = result && result.findings ? result.findings.length : 0
        log(`✓ ${r.name} reviewer finished — ${count} findings reported`)
        return result
      })
    )
  )

  // --- Merge findings ---
  const allFindings = []

  for (let i = 0; i < reviews.length; i++) {
    const review = reviews[i]
    if (review && review.findings) {
      for (const finding of review.findings) {
        allFindings.push({ ...finding, reviewer: reviewers[i].key })
      }
    }
  }

  totalFindings = allFindings.length

  // Log per-reviewer breakdown
  for (const r of reviewers) {
    const rFindings = allFindings.filter(f => f.reviewer === r.key)
    if (rFindings.length > 0) {
      log(summarizeFindings(rFindings, r.name))
    }
  }
  log(`TOTAL: ${totalFindings} findings in iteration ${iteration}`)

  if (totalFindings === 0) {
    log('All 3 reviewers report zero findings — codebase is clean!')
    break
  }

  // Write findings to tmp file
  log('Saving findings to tmp/review-findings.json...')
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

  const fixQueue = []
  if (archFindings.length > 0) fixQueue.push({ name: 'Architecture', findings: archFindings })
  if (apiFindings.length > 0) fixQueue.push({ name: 'API Surface', findings: apiFindings })
  if (qaFindings.length > 0) fixQueue.push({ name: 'QA', findings: qaFindings })

  log(`Fix phase: ${fixQueue.length} fixer agents will run sequentially to avoid conflicts`)

  // Run fixers sequentially to avoid file conflicts
  if (archFindings.length > 0) {
    log(`[1/${fixQueue.length}] Starting Architecture fixer — ${archFindings.length} findings to address...`)
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
    log(`[1/${fixQueue.length}] Architecture fixer done`)
  }

  if (apiFindings.length > 0) {
    const idx = archFindings.length > 0 ? 2 : 1
    log(`[${idx}/${fixQueue.length}] Starting API Surface fixer — ${apiFindings.length} findings to address...`)
    await agent(
      `${PROJECT_CONTEXT}

You are a Fixer agent for API surface, documentation, and UI-readiness issues.

Fix ALL of the following findings. Add doc comments, fix naming, improve ergonomics.
Do not break existing tests. After fixing, run: cargo test -p arm-rules

Findings to fix:
${JSON.stringify(apiFindings, null, 2)}`,
      { label: `fix-api-${iteration}`, phase: 'Fix' }
    )
    log(`[${idx}/${fixQueue.length}] API Surface fixer done`)
  }

  if (qaFindings.length > 0) {
    log(`[${fixQueue.length}/${fixQueue.length}] Starting QA fixer — ${qaFindings.length} findings to address...`)
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
    log(`[${fixQueue.length}/${fixQueue.length}] QA fixer done`)
  }

  log('All fixers complete. Moving to verification...')

  // --- Verify phase ---
  phase('Verify')
  log(`Running verification suite: tests, clippy, fmt, coverage...`)

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

  log(`Iteration ${iteration} complete. ${iteration < MAX_ITERATIONS ? 'Re-running reviewers to check for remaining issues...' : ''}`)
}

if (iteration >= MAX_ITERATIONS) {
  log(`Reached maximum iterations (${MAX_ITERATIONS}). Some findings may remain.`)
}

// --- Final summary ---
phase('Summary')
log('Generating final report...')
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

log(`Full review complete after ${iteration} iteration(s).`)
