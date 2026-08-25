# Guided Creation — Manual Review Findings (2026-08-25)

Collected in a manual walkthrough of the guided wizard (magus, life-stage
funding) plus the editor it hands off to. **Collection only — nothing in this
document has been implemented.** Decisions marked **DECIDED** were settled with
the user during the review and are not open for re-litigation; items marked
**OPEN** still need a call.

Locations were verified against the working tree at commit `721224a`. Every
rules citation was read from `rules/source/en/` during the review, not recalled.

---

## How to read this

| Field | Meaning |
|---|---|
| Severity | Reviewer's judgement: high = unusable/data-loss, medium = wrong or misleading, low = polish |
| Where | Verified file:line |
| Cause | Root cause, where established |
| Fix | **DECIDED** = settled with the user; **OPEN** = needs a call; otherwise a proposal |

Nine of the 31 issues live in **shared** components and are therefore edit-mode
bugs too, not guided-mode bugs: #3, #4, #5, #6, #8, #9, #13, #17, #26.
Only #1, #2, #7, #10, #14, #16, #19, #24, #31 are wizard-specific.

---

## Cross-cutting themes

These recur across findings and are worth fixing as deliberate rules rather than
as N ad-hoc patches.

1. **An element enters or leaves the flow on first interaction, moving a control
   the user is actively clicking.** → #2, #3, #17, #20, #21.
   Rule to adopt: reserve the space (toggle visibility) or take the element out
   of flow. Never unmount something above an input surface.
2. **No base margin reset for `p` / headings.** `app.css` zeroes paragraph
   margins ad hoc in ~20 individual rules, so any unstyled `<p>` keeps the UA
   `margin: 1em 0`. In flex columns margins do not collapse, so this compounds
   with `gap`. → #3, #23. One fix, but an app-wide visual diff needing its own
   verification pass.
3. **One component, two mounts, silent divergence.** → #19, #27.
   Where a mount must differ, pass an explicit prop (the `barProps` seam in
   `WizardStep.svelte` already exists); never sniff the flow from the store.
4. **The editor should mirror the wizard's phases.** → #11, #28.
5. **Raw slugs / placeholder text leaking into user-facing labels.** → #4, #13.

---

## Issues

### #1 — Wizard step "Character type" is redundant
- **Severity** medium (UX/flow)
- **Where** the `type` entry in each profile's `creation_phases`
  (`rules/core/character_types.json`) → `ui/src/lib/components/TypeStep.svelte`
- **Cause** `StartScreen.svelte:73-82` enters the wizard *per type*
  (`store.startWizard(id)`) and the type is immutable afterwards. `TypeStep` has
  no input at all — it only prints the type name, the V/F budget and the Gift
  policy.
- **Fix** Remove the phase. The step carries two facts that live nowhere else
  (V/F budget numbers, Gift policy line) — relocate them to the banner or the
  head of the next step.
- **Interacts with** #11 (magus step count: 11 → 10 here, → 11 again after the
  #11 split), #31 (a persisted phase slug must degrade gracefully when `type`
  disappears).

### #2 — Incomplete-step hint unmounts on first edit, shifting the input surface
- **Severity** medium-high (causes mis-clicks)
- **Where** `ui/src/lib/components/WizardShell.svelte:78-82`; string
  `wizard-step-incomplete-hint` (`locales/en/main.ftl:66`)
- **Cause** `{#if store.wizardPhaseIncomplete}` conditionally *mounts* the
  hint `<p>`. The flag is the engine's `completeness.incomplete_phases`
  (`derive.ts:1085`), which flips on the very first recorded value — so the
  first `+` click is guaranteed to collapse the paragraph and move everything
  below it.
- **Reproduced on** step 3 (characteristics — worst, repeated stepper clicks),
  step 7 (arts), step 8 (spells, least harmful: first interaction is a list add).
  General case: any step whose first edit flips completeness.
- **Fix** Reserve the space — keep the element and toggle visibility, or move the
  note out of the vertical flow.

### #3 — Validation footer is taller when empty than when showing a violation
- **Severity** medium (layout shift)
- **Where** `ValidationPanel.svelte:42` vs `:47`; `app.css:540` (`.validation-bar`),
  `:1566` (`.validation-docked .issue-list`), `:1522` (`.issue`)
- **Cause** `ul` is globally margin-reset (`app.css:682-686`) but `p` is **not**
  — there is no base `p` rule. The empty-state `<p class="muted">` keeps the UA
  `margin: 1em 0` (≈3em total) while one `.issue` `<li>` is ≈1.75rem inside a
  zero-margin `ul`.
- **Fix** Give the panel a `min-height` sized for the "No issues" line and kill
  the stray `<p>` margin so both states start from the same box. Applies to the
  docked wizard footer and the editor's boxed `.panel` mount.
- **Same root cause as** #23.

### #4 — `ParameterPicker` has no branch for `form` / `technique` / `item`
- **Severity** **high** (field is effectively unfillable; leaks raw slugs)
- **Where** `ui/src/lib/components/ParameterPicker.svelte:154-217` — branches
  exist for `characteristic`, `ability`, `art` only; everything else falls to the
  `<input type="text">` at `:209`
- **Cause** `ParameterDomain` has seven variants
  (`crates/arm-rules/src/types.rs:319-342`, mirrored `ui/src/lib/types.ts:35-42`):
  `ability, art, technique, form, characteristic, item, text`. Only `text` is
  *meant* to be an input — its doc comment says so.
- **Why not cosmetic** `Form`/`Technique` are validated as Art ids **plus** the
  right `ArtType`, raising `unknown_param_value` otherwise (`types.rs:325-332`).
  The only strings that pass are internal slugs like `art.ignem`. Typing "Ignem"
  or a German label fails. Also breaks the CLAUDE.md rule that a raw ID must
  never be user-facing.
- **Affected catalogue items** (exhaustive over `rules/core/`):
  - `virtue.deft_form` — `domain: form` (`virtues_flaws.json:3741`) ← the reported case
  - `flaw.deficient_form` — `domain: form` (`:600`)
  - `flaw.deficient_technique` — `domain: technique` (`:611`)
  - `item` domain: in the enum, unused by any catalogue entry today — latent only
- **Fix** Reuse the proven control: `SpellTab.svelte:391-407` already renders a
  Forms-only `<select>` for the meta-magic Vim spells (also `domain: form`) and
  gets it right.

### #5 — Five V/F declare a Form-only parameter as `domain: "art"`
- **Severity** medium (wrong choices offered; the engine cannot catch it, since
  `art` accepts either Art type)
- **Where** `rules/core/virtues_flaws.json`, `{ "key": "form", … "domain": "art" }`
  on `flaw.form_monstrosity` (`:1147`), `flaw.hunger_for_form_magic` (`:1409`),
  `virtue.extractor_of_form_vis`, `virtue.imbued_with_the_spirit_of_form`,
  `virtue.master_of_form_creatures`. `ParameterPicker.svelte:193-194` even names
  Master of (Form) Creatures as declaring `form` under the `art` domain.
- **Fix** Verify each item's cited line range in its source book first. Where the
  rules restrict it to a Form, change to `"domain": "form"` — a one-word data
  change that #4's new branch then renders correctly.

### #6 — Error rows render smaller and red-tinted vs warning rows
- **Severity** medium (visual inconsistency; severity partly conveyed by size)
- **Where** `app.css:1577-1580` vs `:1522-1537`; markup `ValidationPanel.svelte:50`
- **Cause** `<li class="issue {issue.severity}">` emits the bare class `error`,
  so the standalone `.error { color: var(--error); font-size: 0.85rem }` rule —
  written for banner text like `StartScreen.svelte:34` — also matches the row.
  There is no `.warning` counterpart. `.issue` never sets `font-size`/`color`, so
  `.error` wins by default, not by specificity.
- **Fix** Set font-size and colour explicitly on `.issue` so neither severity
  inherits from elsewhere; keep colour coding in the left border + background
  tint + uppercase badge, as designed. Also scope or rename the global `.error`
  rule — a bare one-word class carrying typography will keep colliding.

### #7 — V/F guidance states only the point budget, not the per-type recommendations
- **Severity** medium (guided mode's purpose is to say what the rules advise)
- **Where** `locales/en/main.ftl:82` (`wizard-guidance-virtues_flaws`) +
  `ui/src/lib/derive.ts:1154-1160` (passes only `virtues`/`flaws`)
- **Source** Core Rules `:2822-2862` (per-type list), plus `:2818-2820`:
  - Grog (`:2826-2827`): no Story Flaws; not more than one Personality Flaw
  - Companion (`:2837-2838`): ≤1 Story Flaw; ≤2 Personality Flaws
  - Mythic Companion (`:2850-2851`): as Companion
  - Magus (`:2860-2862`): **should take at least one Hermetic Flaw**; ≤1 Story
    Flaw; ≤2 Personality Flaws
- **Nothing new to author in the rules layer** — the profile already carries
  these as `flaw_category_caps` with a `hard` flag separating *may not* from
  *should not* (`character_types.json:10-14`, `:44-48`), and the magus
  Hermetic-Flaw guideline is already the `missing_hermetic_flaw` warning
  (`RULES.md:227`, `:1467`). Generate the sentence from the profile the way the
  budget numbers already are.
- **Correction to the original request** The user's example was "at least one
  Story Flaw should be taken". The rules say the opposite direction: Story Flaws
  have a recommended *ceiling* of one (`:2818`, `:2982`) and no minimum. The only
  "at least one" is the magus's **Hermetic** Flaw.

### #8 — No separator between the last row of a group and a header-less group
- **Severity** low-medium (visual)
- **Where** `app.css:705-712` — `.selection-list li { border-bottom }` +
  `.selection-list li:last-child { border-bottom: none }`
- **Cause** `:last-child` is scoped per `<ul>`, and `SelectionList.svelte:79`
  opens a new `<ul>` per group. Headed groups hide it (the next
  `<h3 class="category">` draws its own boundary); a **header-less** group does
  not.
- **Affected** V/F granted group (fixed for free by #9);
  `SpellTab.svelte:82` and `EquipmentTab.svelte:76` both emit conditionally
  header-less groups and keep the bug. `AbilityTab.svelte:148` always sets a
  header — unaffected.
- **Fix** Draw the separator as a `border-top` on rows after the first, or on the
  `<ul>` boundary, instead of per-list `:last-child`.

### #9 — Granted V/F render under the previous category's heading — **DECIDED**
- **Severity** medium (actively misleading)
- **Where** `VirtueFlawTab.svelte:153-165` — the header-less granted group
- **Symptom** A `Hermetic`-badged granted Virtue (Faerie Magic) appeared beneath
  the **SUPERNATURAL** heading. Any granted V/F inherits whatever category
  happened to sort last.
- **DECIDED** Merge granted V/F into the normal category-grouped list, ordered
  like any other row. No separate group.
- **Notes** Grouping must key off each granted item's **own** category. Keep the
  row union (`{kind:'bought'|'granted'}`) — granted rows have no entity index and
  no remove button, so the "Granted" marker at `:340-343` stays the only
  distinction, matching how the in-list `Required` rows already behave. Needs a
  combined grouping path: `groupSelectionsByCategory` currently takes only bought
  selections, and the within-group localized-name sort must cover granted rows.

### #10 — Remove the "not started" text from the rail step names
- **Severity** low (noise), requested change
- **Where** `WizardShell.svelte:55-59`; `wizard-step-incomplete-label`
  (`locales/en/main.ftl:64`); `app.css:477-482`
- **Note** The flag is not "unopened" — it is `completeness.incomplete_phases`,
  so it marks any step with nothing recorded, including the current one. On a
  fresh character every step carries it at once.
- **Fix** Keep the span for assistive tech via the existing `.sr-only` utility
  (`app.css:1449`), drop it visually. The span sits *inside* the button
  deliberately so the marker is part of its accessible name (`:52-54`); deleting
  it outright would leave `data-incomplete` as a style hook only, risking a
  colour-only channel (WCAG 1.4.1).
- **Not resolved by this** the on-step hint (#2) and the Review step's
  `wizard-review-incomplete` list both stay.

### #11 — Abilities step packs three concerns into one bounded column — **DECIDED**
- **Severity** **high** ("unusable from a UI/UX perspective")
- **Where** `AbilityTab.svelte:207-217`; floors at `app.css:571-581`
  (`.region-row` `min-height: 12rem`) and `:614-618` (`.list-scroll`, ~3 rows)
- **Cause** Three concerns in one bounded flex column, only the third needing
  height: `LifeStagePanel` (funding model, age, gauntlet, lab seasons, spell
  levels, native language, childhood picker + preview + slot inputs, **six
  explanatory paragraphs**) ≈620px; `MagusMinimumAbilities` ≈200px; then
  `.region-row` gets what is left → its 12rem floor. Both preamble blocks *must*
  be auto-height siblings or the row collapses (`:201-206`, `:209-215`).
- **The floors are prior symptom-patches for this exact bug.** `app.css:571-581`
  records: *"on the magus's Abilities step — the funding panel plus the
  Hermetic-minimums checklist — the row measured 0 in an 800px window, so both
  lists vanished and their controls were not even clickable."*
- **DECIDED** Split the `abilities` phase in two. A new phase before it carries
  funding model + age/gauntlet/lab seasons/spell levels/native language +
  childhood picker/preview; `abilities` keeps only the Available/Selected lists
  at full height.
- **Fix-pass checklist** `CreationPhase` in `crates/arm-rules/src/types.rs`
  (closed enum — compile errors find the rest); TS union in `ui/src/lib/types.ts`;
  `STEPS` in `WizardStep.svelte:44-57`; `GUIDANCE_ARGS` in `derive.ts:1149-1169`;
  `phase-<id>` + `wizard-guidance-<id>` in both locales; the validation
  phase-ownership map `issuesForPhase` reads; `creation_phases` for all four
  profiles in `rules/core/character_types.json`. Then revisit whether the two
  `min-height` floors are still needed.
- **Editor** in scope via #28 (the editor's Abilities tab mounts the same panels).

### #12 — Magus minimums checklist duplicates the Validation panel — **DECIDED**
- **Severity** medium
- **Where** `MagusMinimumAbilities.svelte` (comment `:15-18` confirms the rows
  come from the same engine findings); `app.css:1328-1356`
- **DECIDED** Collapse to the `magus-minimums-summary` line ("N of M still
  unmet"), expandable on demand. Validation stays the authoritative surface.
- **Also fix while in there** the summary counts all rows but renders under the
  *Minimum Abilities* heading, above only the required ones (`:65-74`) — it reads
  as "7 of 7" for a list of 3.

### #13 — Parameterized ability requirements render as a doubled placeholder
- **Severity** medium (confusing; placeholder leak into user-facing text)
- **Symptom** "(Language) (Dead Language) 1 is not met" where the rules say
  plainly "Latin 1" (Core Rules `:2437`)
- **Where** `MagusMinimumAbilities.svelte:49-56` →
  `abilityDisplayName(…, instanceOf(row), paramHint(store.t))`. With no instance
  held, `instanceOf` returns `null` and the label falls back to the parenthetical
  hint, stacked on the ability's own parenthesized name.
- **Also appears in** the validation message ("No magus is admitted to the Order
  below (Language) (Dead Language) 1"), so the fix belongs in the shared label
  path, not the component.
- **OPEN** what an unfilled instance should read as.

### #14 — XP pool line: non-chronological order and duplicated block labels — **DECIDED**
- **Severity** medium-high (the pools were unreadable)
- **Where** `XpBar.svelte` + `xp-pool-*` Fluent keys; block figures from
  `LifeStageBudget` (`crates/arm-rules/src/life_stage.rs:453-490`)
- **Symptom** Two chips both labelled "Later life" (`Later life: 5 × 15 = 75 XP`
  the derivation, `Later life (Abilities only): 0 / 75` the restricted pool), and
  a left-to-right order that made "Later life" read as *post*-Gauntlet.
- **Which pool is which** "Later life (Abilities only)" is the
  early-childhood-end → apprenticeship-start pool. Source `:2214` — *"Later Life.
  15 experience points per year (until apprenticeship for magi)"* — implemented
  `life_stage.rs:527-550`. Gauntlet 25 − childhood 5 − apprenticeship 15 = 5
  years × 15 = 75, matching the Darius example at `:2402`. "Abilities only"
  because a magus cannot spend pre-apprenticeship experience on Arts and the two
  share one pool (`:533-538`).
- **DECIDED**
  1. **Chronological order** per `:2213-2216` / `:2364`: Early childhood → Later
     life → Apprenticeship → After the Gauntlet.
  2. **One chip per block** — merge each block's derivation figure with its
     restricted-pool spent/total, so no label appears twice.
  3. **"Later life (ages N-M)"** — keep the rules term (preserves the German
     glossary mapping) and add the per-character span from `later_life_years`.
  4. **"After the Gauntlet"** replaces "As a magus" — ties the label to the
     `Gauntlet age` field that drives it. Source-backed via `:2216`.
  5. Native language + the 45-point spread group under one **Early childhood**
     heading, per `:2378`.
- **Target shape**
  ```
  XP pool 0 / [380]   Available: 380
  Early childhood — Native language 0 / 75 · Other 0 / 45
  Later life (ages 5-10): 5 × 15 = 75 XP — 0 / 75
  Apprenticeship: 15 years = 240 XP
  After the Gauntlet: 10 × 30 - 130 for lab work = 170 points, 140 XP
  ```
- **Also in scope** `spell-levels-post-gauntlet = As a magus: { $levels }`
  (`locales/en/main.ftl:387`, `de:390`) gets the same rename.
- **Notes** New/renamed keys in **both** locales; the DE term for "Later Life"
  must come from `rules/source/de/translation-tables/`, not be invented. The
  span may need surfacing to the UI if `later_life_years` is not already in the
  payload. Do together with #16 — a taller multi-line bar changes what sticky
  costs.

### #15 — Apprenticeship duration is not editable — **BLOCKED ON SOURCE**
- **Severity** medium
- **Where** `apprenticeship.years: 15` is a **ruleset** value
  (`rules/core/life_stages.json:16`), read at `life_stage.rs:456`; nothing in
  `LifeStagePlan` can override it.
- **Why it is blocked** The rules price apprenticeship as a **flat lump**, not a
  rate: *"The fifteen years of apprenticeship give the character 240 experience
  points, and 120 levels of spells"* (Core Rules `:2435`). There is **no**
  XP-per-apprenticeship-year figure anywhere in the source. `:11018` acknowledges
  variation (*"normally administered after fifteen years … traditionally for
  another year"*) but never prices it.
- **Options** (a) leave fixed — the existing lever is **Gauntlet age**, which
  already moves how many later-life years precede apprenticeship; (b) make the
  ruleset value editable (a data-level house rule, honest about it); (c)
  per-character duration with scaled XP — **forbidden** by the provenance rule,
  since it requires inventing 240/15 = 16/yr.
- **Default unless told otherwise** (a), plus making it *visible* that 15 is a
  ruleset constant rather than an oversight.

### #16 — The XP pool line scrolls out of view while spending XP
- **Severity** **high** (the user had to "change them blindfold")
- **Where** `XpBar` is mounted by the step (`WizardStep.svelte:49`,
  `STEPS.abilities.bar`) as an ordinary flow child of `.vf-tab`; the scrollport is
  the ancestor `.tab-content { overflow-y: auto }` (`app.css:337-349`)
- **Fix** `position: sticky; top: 0` inside that scrollport, with a background so
  it does not overlay rows transparently. Same treatment for `BalanceBar` (V/F)
  and `SpellBudgetBar` (spells) — all three are budgets you edit against and all
  three are mounted the same way. Needed on the editor tabs too.
- **Interacts with** #11 (the split shortens the abilities step, so the outer
  scroll may stop triggering there — sticky is still the robust fix and still
  needed for V/F, spells and edit mode) and #14.

### #17 — Mastery spinner shifts horizontally when mastery goes 0 → 1
- **Severity** medium (mis-clicks on a repeated-click control)
- **Where** `SpellTab.svelte:429-530`; `app.css:992-999` (`.spell-mastery-block`),
  `:1001-1007` (`.mastery-abilities`)
- **Cause** `.spell-mastery-block` is a `flex-direction: column` with
  `flex: 0 1 auto`, so its width is that of its widest child. At mastery 1 the
  `.mastery-abilities` row appears (`:480`) carrying a label plus the "Add special
  ability" `<select>` — much wider. The block widens, `.item-name` (which takes
  the row's free width, `:982`) gives ground, and the spinner is repositioned.
  The `×` stays pinned at the row's right edge.
- **Fix** Give `.mastery-abilities` its own full-width wrap line
  (`flex-basis: 100%`) so it sits below without widening the block. A `min-width`
  on the block would permanently indent every unmastered row.

### #18 — "150 / [120]" reads as an overspend when the budget is exactly balanced
- **Severity** medium-high (the primary budget readout misreports its own state)
- **Where** `SpellBudgetBar.svelte:68-90`; allocation `derive.ts:940-961`
- **Cause** `baseUsed = 150` (all levels charged to the base side), the bracketed
  input shows `base = 120`, and `available = base + lifeStage − baseUsed =
  120 + 30 − 150 = 0`. Engine and Available are right, but the **displayed pair
  does not close**: `baseUsed` includes post-Gauntlet-funded levels while the
  denominator beside it excludes them. The comment at `:65-67` claims they
  *"always close against the bracketed base"* — true only when `lifeStage == 0`.
  `derive.ts:932-933` is the honest version.
- **OPEN** either show `150 / 150` (base + lifeStage as denominator, the field
  still editing only the base), or charge post-Gauntlet levels to their own
  used/amount chip so `baseUsed` drops to 120. First is less disruptive.

### #19 — Spell-levels base must be read-only in the wizard — **DECIDED**
- **Where** `SpellBudgetBar.svelte:78-90`
- **DECIDED** Read-only when mounted by the wizard, editable when mounted by the
  editor.
- **Implementation** Pass an explicit prop (e.g. `readonlyBase`) via
  `WizardStep.svelte`'s `barProps` — which already exists for exactly this kind
  of per-mount difference (`STEPS.arts` uses it for `XpBar`'s prefix). Do **not**
  sniff the flow from the store.
- **Consequence to document** This is the first behavioral divergence between the
  two mounts. `WizardStep.svelte:38-43` currently states the reuse rule as *"the
  wizard reuses the direct-entry components as they are, so its steps and the
  editor's tabs can never drift apart"* — update that comment or the prop will
  read as a violation.
- **Rationale** The 120 is a fixed rules grant (`:2215`), and every legitimate
  in-rules variation already arrives elsewhere: Skilled/Weak Parens as the
  `bonus` chip, post-Gauntlet levels as the `lifeStage` chip. Precedent in the
  same component: the post-Gauntlet input is deliberately absent here because
  *"the Abilities step owns the choice"* (`:23-28`).
- **OPEN, out of scope unless requested** the XP pool's editable total on the
  Abilities step is the same shape of field. Left as-is; the inconsistency is
  noted.

### #20 — Aging step reorders whenever any block's height changes — **DECIDED**
- **Severity** medium-high (whole-panel reflow on a single checkbox)
- **Where** `app.css:1037-1040` — `.character-details { columns: 2 }`
- **Cause** CSS **multi-column** *flows* content between columns, so any height
  change shifts the column break and blocks migrate. Inherent to `columns`, not a
  bug in the picker. Evidence: `AgingRecordPanel` is currently **split across the
  break** — "Apparent age" at the bottom of the left column while its own
  "Aging / Aging points per Characteristic" section is at the top of the right.
- **Key fact** CSS **grid** auto-placement is order-stable under height changes;
  multi-column is not. Each grid item owns its cell, so a block growing changes
  only row heights.
- **DECIDED** `display: grid`, auto-placed,
  `grid-template-columns: repeat(auto-fit, minmax(~22rem, 1fr))`,
  `align-items: start`. Replaces the existing 720px `columns: 1` media query with
  natural 1/2/3-column response.
- **DECIDED** The **aging log gets `grid-column: 1 / -1`** — its own full-width
  row with a bounded `max-height` and its own scrollport. It is the only
  unbounded-growth block, and the narrow column is also truncating its fields
  ("Describe the aging roll's e…").
- **Blocks involved** `AgingStep.svelte` → `AgingPanel.svelte:17-22`, with
  `display: contents` at `app.css:1056-1059` making nested blocks individual
  items: `AgeFields`, `AgingSchedulePanel`, `LivingConditionsPicker`,
  `AgingRollCalculator`, `AgingRecordPanel` (several items), `LongevityPanel`.
  The `display: contents` wrappers keep working — children become grid items
  exactly as they became column items.
- **Sequence** do #23 first, then re-measure, then set the `minmax` floor against
  real block heights rather than inflated ones.

### #21 — "No Living Conditions chosen" is redundant with the line beneath it
- **Severity** low
- **Where** `LivingConditionsPicker.svelte:96-106`
- **Cause** The two blocks are not alternatives: `living-conditions-none` renders
  when nothing is ticked, and `living-conditions-total` renders whenever
  `total != null`. With nothing chosen you get *"No Living Conditions chosen: the
  character counts as an average peasant (0)"* immediately followed by *"Living
  Conditions modifier: +0"*.
- **Fix** Drop the `none` line — the total already states 0, and one stable line
  replaces two-then-one (also removing a height change). If the "average peasant"
  framing is worth keeping, move it into the always-present
  `living-conditions-hint`.

### #22 — The aging-total formula omits the Virtue/Flaw term, so it does not add up
- **Severity** medium-high (visible arithmetic contradiction in a rules readout)
- **Symptom** *"Stress die +4 (age) 0 (living conditions) 0 (Longevity Ritual) =
  stress die +3"* — 4 + 0 + 0 ≠ 3
- **Where** `locales/en/main.ftl:497` (`aging-total-formula`) has exactly three
  named terms, but `AgingSchedulePanel.svelte:32` fills `fixed` from
  `aging.fixed_total`, which per `:23-25` is *"the engine's own sum of all of them
  (including the Virtue/Flaw aging-roll modifiers the book's three lines do not
  name)"*. The character's Faerie Blood contributes −1 with no term in the
  sentence.
- **Proof it is an oversight** the sibling string `aging-total-parts` (`:525`)
  **does** carry a `{ $traits } (Virtues and Flaws)` term.
- **Fix** Add the traits term to `aging-total-formula` in both locales, hidden or
  shown-as-0 when zero, matching the other terms.

### #23 — Huge vertical whitespace on the aging step
- **Severity** high (largest contributor to the step overflowing)
- **Cause** Those blocks are flex columns (`.detail-section`, `app.css:1083-1087`,
  `gap: 0.4rem`) and **flex gaps do not collapse margins**. Between two
  single-line paragraphs: 1em bottom + 0.4rem gap + 1em top ≈ **2.4rem** where
  the design intended 0.4rem — ~6× the intended spacing, repeated for every
  paragraph and heading. Unstyled `h3` does the same. Root cause is the missing
  base `p`/heading reset (see #3): `app.css` zeroes margins ad hoc in ~20
  individual rules, so anything unstyled keeps the UA default.
- **Fix** Add the base reset. **Careful** — ~20 existing rules currently
  compensate for the UA default and some may have been tuned against it. This is
  an app-wide visual diff and wants its own verification pass, not a drive-by.
- **Closes** #3 as well.

### #24 — Age has no canonical home in the guided flow — **DECIDED**
- **Severity** medium (confusing surface duplication of one value)
- **Facts** The Concept step mounts `IdentityFields`, which has **birth year**
  but deliberately no age (`IdentityFields.svelte:12`). The **editor** mounts
  `IdentityFields` at `CharacterDetails.svelte:85` and `AgeFields` immediately
  after at `:87` — so in edit mode age already sits beside identity. Step 6 has a
  *separate* input (`LifeStagePanel.svelte:117-127`, `life-stage-age-input`),
  rendered only under life-stage funding. Step 10 has `AgeFields` (`age-input`),
  added because *"under flat (pool) funding there is no age field anywhere in the
  wizard"* (`AgeFields.svelte:14-19`). All three write the one `entity.age` —
  duplicated *surface*, never duplicated data.
- **DECIDED** Concept owns it: mount `AgeFields` on the Concept step beside
  `IdentityFields`, mirroring `CharacterDetails.svelte:85-87`. Step 6 shows the
  age **read-only** next to the still-editable Gauntlet age (*"a magus carries
  two ages, not one"*, `LifeStagePanel.svelte:19-20`). Step 10 drops its copy.
- **Notes** Removing step 10's copy is only safe because Concept now supplies one
  under both funding models — add an explicit test that a pool-funded character
  has exactly one age input. `AgeFields` needs a read-only mode via the same
  explicit-prop seam as #19. The `age-cap-note` ("Max Ability score: 6") renders
  in both `AgeFields` and `LifeStagePanel:130-133` — decide its home once #11
  splits the step, since it is most useful beside the ability lists. Update tests
  asserting `age-input` on the aging step; `LifeStagePanel.test.ts:322` stays
  valid. Editor unaffected.

### #25 — Age and birth year are independent and mutually unchecked — **DECIDED**
- **Severity** low-medium (feature; prevents a class of inconsistency)
- **Today** the relationship runs one way only: the engine computes calendar year
  as `birth_year + age` (`crates/arm-rules/src/aging.rs:148`, `:158-165`). There
  is **no** current-year or saga-start field anywhere in the entity or ruleset.
- **DECIDED** An app-level current-year setting (default **1220**, source: Core
  Rules `:597`, also `:364`, `:440`), with age ↔ birth year as two views of one
  fact on the Concept step — edit either, the other follows.
- **Key simplification** `entity.age` and `entity.birth_year` are *both* already
  stored, so the setting is a pure **editing aid**, not new entity state. **No
  `SCHEMA_VERSION` bump, no migration, no save-portability problem.** A character
  built in a 1220 saga and opened under a 1230 setting keeps both stored values;
  the setting only governs what the derivation fills in while typing.
- **OPEN** where the setting surfaces and whether it persists across sessions;
  validation for a current year earlier than the birth year (`birth_year` is
  `i32`, `age` is `u32`, so the derived age would underflow — needs a guard and
  probably an advisory finding); whether the derivation is one-shot (fills the
  empty field) or continuously linked.
- **Note** The approximation is already baked in and consistent: `birth_year +
  age` ignores birthdays within the year, exactly as `saga_year − birth_year`
  would. This is also the field the aging-for-loaded-characters backlog item will
  need.

### #26 — `decrepitude_effect` is a single-line input where its analogue is a textarea
- **Severity** low-medium (wrong control for the data)
- **Where** `AgingRecordPanel.svelte:96-106` — `<input type="text">`
- **Confirmation** `RULES.md:1243-1246`: the field is *"free-text overall
  aging/decrepitude narrative. Pure annotation"*, sourced to Core Rules
  `:16563-16577`. It is the cumulative description, distinct from `aging_log`'s
  per-year one-liners, so it does grow over a character's life.
- **Precedent** `warping_effect` is the same thing for Warping and is already
  `<textarea class="warping-effect" rows="3">` (`CharacterDetails.svelte:128-134`).
  `IdentityFields` uses a textarea for the description too.
- **Fix** `<textarea rows="3">` matching the warping field. `setDecrepitudeEffect`
  is unchanged; only the event cast becomes `HTMLTextAreaElement`.
- **Checked** the remaining single-line free-text fields are per-row/short by
  nature (aging-log `effect`, twilight scars, ability specialty, parameter
  values) and are correctly single-line. This was the only mismatch.

### #27 — Characteristics panel is centered in the editor, full-width in the wizard
- **Severity** low (visual divergence between two mounts of one component)
- **Cause** `.tab-content` is `display: flex; flex-direction: column;
  align-items: center` (`app.css:351-353`). The editor mounts
  `CharacteristicPicker` as a **direct child** (`App.svelte:306-307`), so its
  `<section class="panel char-panel">` is centered. The wizard always wraps the
  step body in `<div class="vf-tab">` (`WizardStep.svelte:78`), which is
  `width: 100%` and stretches its children.
- **Why the wrapper exists** `WizardStep.svelte:73-77` — it reproduces the
  editor's height chain, without which `.region-row`'s Available/Selected columns
  collapse. The characteristics step has no `.region-row`, so it gets the side
  effects without needing them.
- **Fix** Make the panel center *itself* (`margin-inline: auto` plus its existing
  max-width) so both mounts agree by construction. Conditionally dropping the
  wrapper also works but is more fragile — the wrapper also supplies the
  `gap: 1rem` between guidance, bar and body.

### #28 — Editor tabs do not mirror the wizard's steps — **DECIDED**
- **Severity** medium (structural; the crowding in #11/#27 is a symptom)
- **DECIDED** The editor's tab structure mirrors the wizard's phases:
  - **New "Experience" tab** — the life-stage/funding panel. The Abilities tab
    becomes only the Available/Selected lists.
  - **Details splits** — identity + age (+ Warping/Twilight, which have no wizard
    phase) stay on Details; **Personality & Reputations** becomes its own tab;
    **Aging** becomes its own tab.

  | Wizard phase | Editor tab |
  |---|---|
  | concept | Details |
  | *(new)* experience | Experience |
  | abilities / arts / spells | Abilities / Arts / Spells |
  | personality_reputations | Personality & Reputations |
  | aging | Aging |
  | — | Possessions, Equipment, Totals *(no phase)* |

- **A simplification this unlocks** `LongevityPanel` is currently mounted by
  `AgingStep` rather than inside `AgingPanel`, specifically to avoid giving a
  magus two on-screen homes for one ritual — its editor home is the magus-gated
  Possessions tab (`AgingStep.svelte:15-25`). With a real Aging tab it can move
  *into* `AgingPanel` and off Possessions, deleting the special case.
- **Notes** The tab list is gated per character type, so each new tab needs the
  same gating (Experience only where life-stage funding applies, Aging wherever
  the aging subsystem is live). Tab order and `aria-controls`/`tabpanel` ids
  follow. Do this **after** #11, so the phase list is settled first.

### #29 — Switching funding mode silently destroys typed input, both ways — **DECIDED**
- **Severity** **high** (unconfirmed, unrecoverable loss of user input)
- **Where** `state.svelte.ts:761-773` (`setAbilityFunding`)
  - pool → life stages: `this.entity.xp_pool = 0` — a hand-typed total is zeroed
  - life stages → pool: `delete this.entity.life_stages` — the whole plan goes
    (native language, Gauntlet age, lab seasons, spell levels, childhood
    package), plus `childhoodDraft`, `childhoodRejections`, the aging draft
- A round trip loses everything typed on either side, with no prompt and no undo.
  The unsaved-changes guard governs quitting, not destructive in-app edits.
- The code is explicit that this is intended (`:741-743`), while also arguing
  correctly that bought `ability_scores` must survive because losing those would
  be worse — so the principle is already "don't discard the player's work"; the
  plan and pool just weren't held to it.
- **Structural reason it is hard**
  `abilityFunding = $derived(this.entity.life_stages ? 'life_stages' : 'pool')`
  (`:420`) — the *presence* of the plan **is** the mode.
- **DECIDED** Add an explicit funding discriminator to the entity.
  `setAbilityFunding` becomes a pure mode set: no deletion, no zeroing.
- **Fix-pass notes** (largest item on the list — engine + schema + UI, its own
  TDD slice)
  - New `Entity` field (e.g. `ability_funding`), serde-defaulted so old saves
    parse. `SCHEMA_VERSION` bump + migration inferring the value from
    `life_stages` presence — today's rule, applied once at load.
  - `state.svelte.ts:420` reads the field instead of inferring.
  - **Retired invariant** the code states *"the engine makes a plan and a typed
    pool mutually exclusive"* (`:740-741`). After this they coexist. Every site
    inferring the mode from plan presence must be found and switched to the
    field, starting with `life_stage.rs:453` (`entity.life_stages.as_ref()?`,
    where `None` means "pool"), plus the `not_enough_xp` / restricted-pool
    validation paths, completeness and export. **The closed-enum discipline will
    not catch these** — they are `Option` checks, not `match`es — so this needs a
    deliberate grep, not just a compile.
  - Any validation asserting mutual exclusivity must be removed or inverted
    (note `life_stage_xp_pool_conflict`, `validation/mod.rs:144`).
  - **The sparse-save principle is deliberately departed from** — the save now
    keeps data the active mode ignores. Note it in `RULES.md` so nobody "fixes"
    it back.
  - Keep pruning **drafts** (`childhoodDraft`, `childhoodRejections`, aging
    draft) per the existing blanket rule that an un-submitted draft never
    outlives a change to how the document is built (`:756-759`). Only the plan
    and pool stop being destroyed.

### #30 — No warning for unspent general XP or unspent spell levels
- **Severity** medium (feature gap; asymmetric with existing findings)
- **Today** (`crates/arm-rules/src/validation/mod.rs:126-176`) overspending is an
  error on both (`not_enough_xp` `:139`, `over_spell_levels` `:176`), but
  underspending is warned only for characteristics
  (`characteristic_points_unspent` `:136`) and for life-stage **restricted**
  blocks (`restricted_xp_unspent` `:141`). Leaving 200 general XP or 60 spell
  levels unspent produces **silence** while one unspent characteristic point
  produces a warning.
- **Fix** Two new warning codes, owned by the `abilities` and `spells` phases.
  Because `ValidationPanel` renders unfiltered on the terminal step
  (`WizardShell.svelte:91`), phase-owned warnings appear **both** on their own
  step and on Review — which is desirable. (User asked for "in the review step";
  this satisfies it and more. Flag if they want Review-only.)
- **Design details**
  - Count only the **general** remainder, not the restricted blocks — those
    already have `restricted_xp_unspent`, and summing them again double-reports
    the same points on a life-stage character.
  - **Wording must stay factual.** `restricted_xp_unspent` says "will be wasted",
    which is backed for childhood. The Core Rules contain **no** equivalent
    statement for apprenticeship XP or the 120 spell levels (searched), so those
    messages should say "N of M unspent" and stop there rather than asserting a
    rule the source does not make.
  - Warnings, so `canFinish` (errors only) is unaffected.
  - They will fire from the start on a fresh character, as
    `characteristic_points_unspent` already does — consistent, but it adds to the
    noise #10 addresses.

### #31 — Open a saved character into the guided wizard, with persisted progress — **DECIDED**
- **Severity** medium (feature gap; blocks resuming an interrupted wizard run)
- **Feasibility: small** (≈a day with tests). No engine change beyond the new
  field. `startWizard` is only three things (`state.svelte.ts:1924-1929`):
  instantiate blank, `view = 'wizard'`, reset the rail — and only the first is
  new-character-specific. It is cheap because the wizard mounts the editor's own
  components, reads its phases from `creation_phases` as data, and shares the one
  evaluation path.
- **"Saved during the wizard" is not a special case.** Wizard progress is
  currently not persisted at all — *"moving through the flow is not an edit, so it
  never dirties the document and a save records no progress through it"*
  (`wizard-navigation.svelte.ts:31-45`). A file saved on step 6 is
  indistinguishable from one built in the editor, so implementing the mid-wizard
  case **is** implementing the general case.
- **DECIDED — persist the furthest phase as a slug, not an index.** New optional
  entity field (e.g. `wizard_furthest_phase: Option<CreationPhase>`),
  skip-if-none, `SCHEMA_VERSION` bump. An index would resolve against
  `creation_phases`, which is data and about to change twice (#1 removes a phase,
  #11 inserts one). A slug survives both and matches the project rule that saves
  store choices, not resolved values, with slug-style non-positional IDs.
- **DECIDED — restore behavior**
  - **Field present** → wizard-saved. Restore as `furthest`, land there, and
    **clamp as during the original run** (forward jumps stop at the first
    blocking phase).
  - **Field absent, or slug not in the current profile's phase list** →
    manually-created or migrated. All steps reachable, **exempt from the clamp**,
    findings shown but never gating.
- **Why persisting `furthest` is safe for the dirty flag** it only ever rises in
  `next()` (`wizard-navigation.svelte.ts:112-117`); `back()` and `goTo()` never
  touch it. So browsing the rail stays free and only a deliberate Next marks the
  document changed.
- **Also in scope** entry point (`view = 'wizard'` after `open()`); a guard so the
  action is not offered for a `type_id` the loaded ruleset has no profile for.
- **Notes** `next()` now dirties the document even on an unedited step (a step
  can be legally empty — `canAdvance` gates on errors only). Defensible, but add
  a case to the unsaved-changes guard tests in `state.svelte.test.ts` so it is
  deliberate. The field is UI/document state on an entity defined in the pure
  engine crate — document it in `RULES.md` as such so nobody wires validation to
  it.
- **Rejected** exact-step resume by persisting the *current* step as well: that
  would make every rail click a document edit. The landing heuristic (stored
  phase, else first incomplete) is preferred.

---

## Backlog (not part of this fix list)

### Aging for loaded characters
The step-10 aging mechanism should also serve characters loaded from a save —
advancing an existing character rather than only resolving pre-play years at
creation. **Details to be discussed.** Note it will need #25's current-year
field, so #25 is groundwork rather than duplicate effort.

---

## Confirmed non-issues

- **The 10 blank aging-log rows** in the step-10 screenshot were a deliberate
  demonstration of unbounded growth, not auto-generated. They come from
  `entity.aging_log` via the Add button (`AgingRecordPanel.svelte:132`).
- **Native language in the life-stage panel is load-bearing**, not decoration.
  The childhood rules give 75 XP in the native language plus 45 spread over a
  list that explicitly reads *"Living Language (other than the character's native
  language)"* (`RULES.md:2545-2563`). The engine needs the name to create the
  right parameterized `ability.living_language` instance, to enforce the "other
  than" clause (`childhood.rs:997`), and because every package lists
  `Native Language 5` (`RULES.md:2597-2601`, `childhood.rs:954`). Whether it
  belongs *visually* in that panel is folded into #11.
- **Wizard and editor share their input components.** Verified
  `WizardStep.svelte:44-57` against `App.svelte`. Identical components:
  `CharacteristicPicker`, `VirtueFlawTab`, `AbilityTab`, `ArtGrid`, `SpellTab`,
  `HouseSelector`, `MythicCompanionTypeSelector`, `IdentityFields`. Wizard-only
  *compositions* of editor leaves: `PersonalityReputationsStep` (=
  `PersonalityTraits` + `Reputations`), `AgingStep` (= `AgeFields` +
  `AgingPanel` + `LongevityPanel`). Genuinely wizard-only: `TypeStep`,
  `WizardReview`, `WizardShell`/`WizardStep`. Editor surfaces no phase reaches:
  `MagicPossessions`, `SupernaturalBeing`, `EquipmentTab`, `DerivedTotalsPanel`.

---

## Suggested sequencing (for the plan, not decided)

1. **#23 + #3** — the base margin reset, with its own verification pass. Do it
   first: it changes every block's height, and #20's `minmax` floor and #11's
   space budget should be measured against real heights.
2. **#11** — split the `abilities` phase (`CreationPhase` change ripples widely).
3. **#28** — mirror the editor's tabs onto the settled phase list. Depends on #11.
4. **#1** — remove the `type` phase (another `creation_phases` change; batch with
   #11 if convenient, since #31's slug lookup must tolerate both).
5. **#29** — the funding discriminator: engine + schema + migration, own slice.
6. **#31** — open-into-wizard + persisted phase slug (second schema change;
   consider batching the bump with #29).
7. **#20 + #21 + #22 + #26** — the aging-step cluster.
8. **#4 + #5 + #13** — the parameter-domain cluster (data + picker + label path).
9. **#14 + #16 + #18 + #19** — the budget-bar cluster.
10. **#2 + #17 + #6 + #8 + #9 + #10 + #27** — layout-shift and styling cluster.
11. **#7 + #12 + #30** — guidance and findings cluster.
12. **#24 + #25** — age ownership and the current-year setting.

## Reminders for the implementation plan

- **TDD is strictly test-first**: write a failing test and see it RED before any
  implementation, even for pure helpers.
- **The full gate** must pass before any commit or "done" claim:
  ```
  cargo test --workspace
  cargo clippy --workspace --all-targets -- -D warnings
  cargo fmt --check
  cd ui && npm run test:unit && npm run lint && npm run format:check && cd ..
  cargo tauri build --no-bundle      # non-negotiable final gate
  ```
  Run the gate directly — never report green from a subagent's summary.
- **Every user-facing string** goes in `.ftl`, both locales. German rules
  terminology must come from `rules/source/de/translation-tables/`, never be
  invented (#14 especially).
- **Every rule** must be cited at the implementation site by source file basename
  + verified line range, and recorded in `crates/arm-rules/RULES.md`.
- **Negative signs** are ASCII hyphen `-` (U+002D) via `formatSigned`.
- Two `SCHEMA_VERSION` bumps are implied (#29, #31) — consider whether they batch.
