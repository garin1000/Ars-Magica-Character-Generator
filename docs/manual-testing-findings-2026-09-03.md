# Manual-testing findings — 2026-09-03

Findings from a manual test session through guided creation, the
Arts/Abilities/Spells surfaces, the life-stage panel and the Aging tab. Each
entry records the symptom as observed, the root cause located in the code, the
rules citation where the finding is a rules question, and the decision taken.

Rules citations are `file basename:line` into `rules/source/en/`; the mechanics
behind them live in `crates/arm-rules/RULES.md`, which is the traceability map
and is not restated here.

Status values: **open**, **done**, **closed** (deliberately no change).

Findings are grouped by the work package that implements them. Two numbers are
absent by decision: 13 was withdrawn, 14 was closed after the rules check
contradicted its premise.

---

## Progress log

Appended as work lands, so a session that dies mid-flight can be resumed from
here. The plan this executes is `/home/norbert/.claude/plans/done-doocument-issues-in-jolly-quiche.md`
(machine-local, not in the repo). Implementation order: WP1 → WP2 → WP3 → WP4 →
WP5 → WP6 → WP7, one finding at a time.

| Date | Work package | What landed | Gate |
|---|---|---|---|
| 2026-09-03 | — | This findings document written; 25 findings recorded, 13 withdrawn, 10 and 14 closed with reasons | n/a (docs only) |
| 2026-09-03 | WP1 | Finding 8 done: eleven `source` refs re-pointed at the core rules, locked by `core_rules_virtues_cite_the_core_rules_file` in `tests/data_integrity.rs`; RULES.md re-cited throughout. Finding 28 opened as a by-product. | `cargo test --workspace`, `clippy --all-targets -D warnings`, `fmt --check` — all green |
| 2026-09-03 | WP1 | Findings 9 and 10 done: `tainted: true` added to `virtue.demonic_blood` **and** `flaw.tragic_life` (found by auditing all 21 tagged descriptors), locked by a new test; finding 10 closed with the Mythic-Companion-is-a-marker rationale recorded in RULES.md. **WP1 complete.** | `cargo test --workspace` (16 suites ok, 0 failed), `clippy`, `fmt --check` — all green |
| 2026-09-03 | WP2 | Findings 6 and 7 done: `categories: Vec<String>` end to end — engine, rules data, export, UI grouping/filtering/badges; legacy `category:` fails loudly; `$category` no longer renders a raw slug. **WP2 complete.** | Full gate green including `cargo tauri build --no-bundle`; e2e not yet run |
| 2026-09-03 | WP3 | Finding 4 done, engine untouched: `issuesForStep` + `phaseSelectedItemIds` let the docked panel show a foreign-phase finding whose `context` was chosen on this step, marked `data-elsewhere` with a dashed bar and the `issue-other-step` sentence naming the owning step (4a/4b); a `data-pending` rail marker with its own glyph and `.sr-only` label carries an open warning forward, gated to steps already reached after measuring 7 warnings over 4 of 11 unopened steps on a fresh magus (4c). | Full gate green including `cargo tauri build --no-bundle`; e2e not run |
| 2026-09-04 | WP3 | Finding 5 done: the ability parameter picker offers the whole catalogue plus an instance field for parameterized abilities, and `ability_bonus_dangling_target` moved to the `abilities` phase, so the wizard runs forward and the rail keeps flagging what is owed. Finding 12 done: the exemplar now leads the requirement ("below Latin 1 (any Dead Language)"), fixed alongside the recommended-ability warning, the scholarly-language finding and the checklist. **WP3 complete.** | Full gate green including `cargo tauri build --no-bundle`; e2e run at package end |
| 2026-09-04 | WP5 | Finding 16 done: `AppStore.#effectiveBasis` + `readSettled()` pin a badge's bought score to the generation its modifiers were computed for, so the Art/Ability/Characteristic badges make one transition per committed edit instead of rendering `newScore + oldBonus` for the length of the debounce. Three new `client` tests. | `npm run test:unit` (1259 tests), `check`, `lint`, `format:check`, `cargo test --workspace` — all green; release build and e2e **not** run, an e2e suite held `target/release` |
| 2026-09-04 | WP7 | Findings 21, 2 and 3 done: the project-wide prose sweep. 43 Fluent keys deleted from both locales (rendering site, EN, DE and the tests that pinned them), one sentence trimmed, one `familiar-powers-note` swept in beyond the brief; every `aria-describedby` that named a deleted node removed with it; `wizardGuidance` and the whole guidance machinery deleted from `derive.ts`; RULES.md re-written where it cited the removed keys. | `npm run test:unit` (1239 tests), `check`, `lint`, `format:check`, `cargo test --workspace`, `clippy --all-targets -D warnings`, `cargo fmt --check` — all green; release build and e2e **not** run, an e2e suite held `target/release` |
| 2026-09-04 | WP4/WP6/WP7 | Findings 11, 15, 19, 20, 22–27, 1 done and committed; statuses corrected across the document. **All 25 findings from the session are resolved** — 22 fixed, 13 withdrawn, 10 and 14 closed. (10 was reopened the same day and fixed; see the row below.) Only finding 28, raised here as a by-product, is still open and needs a decision. | Full gate green per package; e2e green after WP3 (43/43), WP4, WP5, WP6 |
| 2026-09-04 | WP1 correction | **Finding 10 reopened and done.** The 2026-09-03 closure was wrong: `Mythic Companion` is one of the `### <Category>, <Magnitude>` headings the `## List of Virtues` index groups by (:3329), not a `Tainted`-style marker. `mythic_companion` is now a real category on the four Virtues, permitted to the `mythic_companion` profile alone — which stops a grog taking Devil Child, as :2637 requires. `category-mythic_companion` added to both locales; RULES.md's marker note replaced by the corrected one. | `cargo test --workspace`, `clippy --all-targets -D warnings`, `fmt --check`, full UI gate, `cargo tauri build --no-bundle` — all green |
| 2026-09-04 | follow-up | Two real e2e failures from WP6/WP7 fixed: the Living Conditions checklist overflowed its column by 15px (rows inherit `nowrap`, widest label 358px against a 325px track — floor raised to 24rem, labels may wrap), and the new German tab-strip test read `''` for every tab because this WebKitGTK driver's `getText()` returns the empty string for any `overflow: hidden` element, clipped or not. Both locked with unit-level assertions. | `aging`, `aging-crisis`, `tab-area`, `i18n-german` specs green individually; full suite re-run after |

**Caution for the next session:** `npm run test:e2e` exited **0** with
`41 passed, 2 retries, 2 failed, 43 total`. The exit status is not a verdict —
read the `Spec Files:` summary line and grep for `✖`.

---

## WP1 — Rules data corrections

### 8 — Eleven core-rules Virtues cite sourcebooks they were not taken from

**Status:** done (2026-09-03)

Each of these has a real entry in *Ars Magica - Definitive Edition (Core
Rules).md*, but `rules/core/virtues_flaws.json` cites a Realms of Power book:

| ID | Correct core-rules line | Currently cited |
|---|---|---|
| `virtue.blood_of_the_nephilim` | :3504 | Divine |
| `virtue.curse_throwing` | :3625 | Faerie |
| `virtue.demonic_blood` | :3649 | Infernal |
| `virtue.demonic_might` | :3663 | Infernal |
| `virtue.demonic_powers` | :3667 | Infernal |
| `virtue.devil_child` | :3671 | Infernal |
| `virtue.faerie_doctor` | :3821 | Faerie |
| `virtue.nephilim` | :4594 | Divine |
| `virtue.spirit_votary` | :5006 | RoP: Magic |
| `virtue.spiritual_pact` | :5010 | RoP: Magic |
| `virtue.strong_angelic_heritage` | :5022 | Divine |

Magnitudes and categories otherwise match the core tags. Every replacement range
must be verified against the file before it is written:
`crates/arm-rules/tests/rules_source_provenance.rs:165` checks that a citation
brackets real content in the file it names.

**Resolution.** All eleven now cite the core rules, with end lines derived from
the file (heading through the line before the next `####`, the convention the
neighbouring entries already use). Locked by
`core_rules_virtues_cite_the_core_rules_file` in
`crates/arm-rules/tests/data_integrity.rs`. Worth recording: the sourcebook
citations were not fabricated — these Virtues really are reprinted in the Realms
of Power books — so this was a source-of-truth violation (English core wins), not
a dangling reference, which is why the existing provenance test never caught it.
Curse-Throwing is the Virtue at :3625, not the Supernatural Ability at :7396.
After the fix no file under `rules/core/` cites a Realms of Power book at all.

### 28 — `virtue.spirit_votary`'s +7 Flaw points is cited to a core passage that does not state it

**Status:** open — found while fixing 8, not part of the manual session

`rules/core/mythic_companion_types.json` gives Spirit Votary a +7 Flaw-point
budget and cites core `:2741-2764`. Those lines are silent on the number; only
*Realms of Power: Magic* `:5486` states it. `crates/arm-rules/RULES.md:1667`
already notes the discrepancy. Since English core is the source of truth for
values as well as ids, either the citation is wrong or the value is unsourced —
it needs a decision, not a silent fix.

### 9 — `virtue.demonic_blood` is missing `tainted: true`

**Status:** done (2026-09-03) — widened by one

The core rules tag it *Major, Supernatural, **Tainted*** (:3649-3650). Without
the flag the Tainted half-of-points cap (`validation/caps.rs`,
`validate_tainted_cap`) under-counts it, so a character can carry more Tainted
Virtue points than the rules allow — a wrong-rules-output defect, not cosmetic.

**Resolution.** An audit of all 21 core-rules descriptor lines carrying the
`Tainted` tag against the catalogue found a **second** entry missing it:
`flaw.tragic_life` (:6855-6856, *Major, Story, Tainted*), which under-counted the
Flaw side of the same cap. Both fixed. `flaw.tainted_with_evil` correctly has no
flag — its descriptor (:6844) is *Minor, General*, so the name is a false friend.
Locked by `core_rules_tainted_virtues_carry_the_tainted_flag` in
`crates/arm-rules/tests/data_integrity.rs`, which samples tagged entries and
keeps `flaw.tainted_with_evil` as a control.

### 10 — Four "Free, Mythic Companion" virtues are stored as `social_status`

**Status:** done (2026-09-04) — reopened after being closed on a wrong reading,
then fixed

Devil Child (:3671), Faerie Doctor (:3821), Nephilim (:4594) and Spirit Votary
(:5006) are tagged *Free, Mythic Companion* in the source.

**The first ruling was wrong.** It read "Mythic Companion" as a marker like
`Tainted`, on the grounds that the chapter's prose defines six categories
(Hermetic :2878, Social Status :2884, Supernatural :2958, Personality :2964,
Story :2980, General :2994) and this is not among them — so the four stayed
`social_status`. That argument mistook a set of *explanatory sections* for the
taxonomy. The taxonomy is the index: `## List of Virtues` (:3004) groups every
entry under a `### <Category>, <Magnitude>` heading, and one of those headings is
**`### Mythic Companion, Free`** (:3329), listing exactly these four
(:3331-3334). None of them appears under `### Social Status, Free`
(:3336-3354), whose seventeen entries are Apprentice, Bard, Covenfolk and the
rest. The descriptors settle it: `*Free, Mythic Companion*` (:3672, :3822,
:4595, :5007) is magnitude-then-category, the same shape as `*Minor, Social
Status*` — while `Tainted` never occupies that slot, appearing only as a third
token *after* a real category (`*Major, Supernatural, Tainted*`, :3650). That
asymmetry is what the earlier argument missed.

**Resolution.** `mythic_companion` is a real category. The four carry it alone —
each is listed under one heading only, so none of them is dual-category — with
`category-mythic_companion` in both locales (DE *Mythischer Gefährte*, the German
index heading at `Basisregeln.md:3329` and `translation-tables/grundbegriffe.md:83`,
the same string `type-mythic_companion` already used).

The move is not cosmetic: it changes who may take them, correctly. Only the
`mythic_companion` profile lists the category in `permitted_categories`, because
":2637 All Mythic Companions take a Free Virtue which specifies their status.
These Virtues are incompatible with each other, and with The Gift, and are **not
available to grogs**", and each descriptor says the Virtue *makes* its bearer a
Mythic Companion (:3673 "can only be taken for a Mythic Companion", :3823, :4596,
:5008). Under the old mapping a **grog** could legally take Devil Child, since
grogs may take Social Status Virtues — a wrong-rules-output defect the mis-ruling
carried with it. A companion could too, acquiring a Mythic Companion's status
without its type; both are now `category_not_permitted`. A magus was already
blocked by the `incompatible_with: virtue.the_gift` its profile requires, and now
fails the category check as well.

No other consumer moved: the shipped category caps name only `personality`,
`story` and `hermetic`; `gift_categories` is `["hermetic"]`; the House grant
constraints in `houses.json` name only `hermetic` (and match on Minor/Major, never
Free); and the mythic-type constraints in `mythic_companion_types.json` name
`supernatural`, `story` and `personality`. The UI needed no change — its category
headings, badges, filter dropdown and the export's Type cell are all built from
the catalogue's own categories through `category-<id>`.

Locked by `the_mythic_companion_virtues_carry_the_mythic_companion_category`,
`only_the_mythic_companion_profile_permits_the_mythic_companion_category` and
`a_grog_may_not_take_a_mythic_companion_virtue` in
`crates/arm-rules/tests/data_integrity.rs`, plus the locale check in
`ui/src/lib/i18n.test.ts`. `crates/arm-rules/RULES.md` records the corrected
reasoning where it previously recorded the wrong one.

---

## WP2 — Multi-category Virtues and Flaws

### 7 — The engine models one category per V/F; the rules give some two

**Status:** done (2026-09-03)

`PointItem.category` is a single `String` (`crates/arm-rules/src/types.rs:1541`),
and every consumer assumes exactly one: category caps, type-profile
permit/forbid lists, grant constraints, gift-category checks,
`items_by_category`, the export Type cell, and the UI's grouping and filtering.
The extraction convention recorded at `crates/arm-rules/RULES.md:252-256` —
"compound labels … take the earliest-listed category" — is where the second
category was dropped.

Four items in the English corpus carry two categories. There are none with
three:

| Name | Source line | Tag | Stored today |
|---|---|---|---|
| Sufi | :5077 | Minor, Social Status, Supernatural | `social_status` |
| Raised from the Dead | :6646 | Major, Story, Supernatural | `story` |
| Suppressed Gift | :6803 | Major, Hermetic, Story | `hermetic` |
| Visions | :6985 | Minor, Story, Supernatural | `story` |

`Tainted` is **not** affected: it is already a separate `tainted: bool`, which is
correct. It is a cross-cutting property — every Tainted item also carries a real
category, it has its own independent cap ("no more than half a character's
Virtues should be tainted", :2998-3002), and it carries a rider unrelated to
categorisation (a Supernatural Ability it grants is an Infernal power). Modelling
it as a category would make it compete for a category slot and give the Tainted
cap and the category caps a shared mechanism the rules do not share.

Consequences of the fix, which are the point of it: Suppressed Gift starts
counting against the Story flaw cap, and Sufi becomes findable as a Supernatural
virtue.

**Resolution.** `PointItem.category: String` became `categories: Vec<String>`
with the descriptor's earliest-listed category first. Membership tests (permit,
forbid, caps, grant constraints, gift categories, `items_by_category`) use the
whole set; display and grouping use the primary, so a bought row still appears
under exactly one heading and the index-addressed remove buttons stay valid. A
legacy singular `category:` key now fails to load with an error naming the
offending item rather than being silently coerced — an old `rules/` directory
beside a new binary is the portable layout's real hazard. No save migration and
no `SCHEMA_VERSION` bump: saves store `Selection { ref, params }`, never a
category.

Two corrections to the finding as written above:

- Suppressed Gift does **not** become legal for a companion. The permitted check
  now passes via `story`, but the companion profile also forbids `hermetic`, so
  `forbidden_category` still fires — correctly, it being a Hermetic flaw.
- The Markdown export's Type cell now lists every category, joined through the
  localized separator, each still resolved via `category-<id>` — no raw slug.

A pre-existing raw-slug bug was fixed alongside: `issue-forbidden_category`
rendered its `$category` argument as the bare slug ("belongs to a forbidden
category (supernatural)") because `ENUM_ARG_FLUENT_PREFIX` had no `category`
entry.

### 6 — Visions is categorised Story only

**Status:** done (2026-09-03) — subsumed by 7

Reported from *Definitive Edition* p. 150 (`:6985-6986`, *Minor, Story,
Supernatural*). A symptom of 7, not a standalone data typo; corrected as part of
it.

---

## WP3 — Wizard flow and validation messages

### 5 — Puissant Ability deadlocks the wizard's Next button

**Status:** done (2026-09-03)

Taking Puissant Ability on the Virtues/Flaws step blocks Next until its ability
parameter is filled, but the parameter picker offers only abilities the
character already has (`ui/src/lib/components/ParameterPicker.svelte:223-242`) —
and abilities are bought on a *later* step. There are two blocking errors, not
one: `missing_param`, and `ability_bonus_dangling_target`
(`crates/arm-rules/src/validation/selections.rs:355-367`), an error filed on the
`virtues_flaws` phase reading "targets an Ability the character does not have;
add it first".

**Decision.** The parameter picker offers the whole ability catalogue, as the
`art` domain already does (`ParameterPicker.svelte:105-113`) — Puissant (Ability)
is "choose one Ability" with no requirement that a score already exists.
`ability_bonus_dangling_target` stays an error but is re-filed to the `abilities`
phase, which follows `virtues_flaws` in every profile: the wizard then flows
forward, and the block lands on the step where the work actually is. A pending
item is carried forward on the wizard rail so it is not silently forgotten.

**Implemented as decided.** The `ability` domain now lists the whole catalogue,
keeping the character's own instances of a parameterized ability alongside the
generic entry — and, because the engine expects the instance discriminator
whenever the target is parameterized, the picker grew a text input for it, so the
generic entry is not a `missing_param` dead end of its own.
`ability_bonus_dangling_target` is filed on `CreationPhase::Abilities`
(`validation/selections.rs`), pinned by
`every_shipped_profile_buys_abilities_after_virtues_flaws` in
`crates/arm-rules/tests/core_type_conformance.rs`, and its message now names the
work ("Add … to the character's Abilities") instead of the Virtue. The rail's
carry-forward needed no new mechanism: the existing `data-blocked` marker plus its
`.sr-only` hint already marks any phase holding an error, and a regression test in
`WizardShell.test.ts` now locks that for this case.

### 4 — Characteristic-affecting Virtues give no feedback where the user is

**Status:** done (2026-09-03)

Three cases, one cause. Great Characteristic and Poor Characteristic already
produce precondition errors, and Improved Characteristics' three extra points
already produce a `characteristic_points_unspent` warning — but all of them are
filed under `CreationPhase::Characteristics`
(`crates/arm-rules/src/validation/scores.rs:156-163, 213-241`), while the user is
standing on the Virtues/Flaws step. The docked validation panel filters to the
current phase, so the step where the virtue was taken says nothing.

- **4a** Great Characteristic: no hint that the chosen characteristic must be
  raised on the Characteristics step.
- **4b** Poor Characteristic: mirrored.
- **4c** Improved Characteristics grants 3 points with no warning that they are
  unspent.

**Decision.** Do not move the engine's phase attribution — the offending *value*
really is the characteristic score. Instead the docked panel also admits issues
whose `context` is an item selected on the current step, rendered with a distinct
marker naming the step that owns the fix. `characteristic_points_unspent` has no
`context`, so 4c is served by a "still to do" marker on the wizard rail, which is
also what finding 5 needs.

**Implemented as decided.** The engine is untouched.

- **4a/4b.** `issuesForStep` (`ui/src/lib/derive.ts`) replaces `issuesForPhase` in
  the docked panel: it keeps the step's own findings and admits a finding filed on
  another phase when its `context` names an item that step holds
  (`phaseSelectedItemIds`, which is the entity's `selections` and only on
  `virtues_flaws`). An admitted row carries `data-elsewhere="<phase>"`, a **dashed**
  severity bar instead of a solid one (a shape change, so it survives greyscale) and
  the visible sentence `issue-other-step` — "Resolve on the Characteristics step",
  built from the existing `phase-<slug>` label. That sentence is the part that
  matters: `canAdvance` still keys strictly on the owning phase, so the row reads as
  an error while Next stays enabled, and without it the gate would look broken.
- **4c.** A second, lower-weight rail marker: `data-pending="true"` plus an
  `.sr-only` `wizard-step-pending-label` ("has open warnings"), glyph `*` in the
  warning hue against the blocked marker's bold `!`. It is **gated to steps already
  reached** (`index <= wizardFurthest`, the same set the rail lets you click) and
  suppressed on a blocked step, so a step says one thing and the strongest one.

  The gate is not a guess. `a_fresh_wizard_magus_already_carries_warnings_on_several_unreached_phases`
  (`crates/arm-rules/tests/core_type_conformance.rs`) measures the alternative: a
  magus the wizard has only just created already emits 7 warnings —
  `missing_hermetic_flaw`, `house_unset`, `spell_levels_unspent` and four
  `magus_recommended_ability` — across 4 of its 11 steps, every one of them a step
  never opened. An ungated marker would light over a third of the rail on step one.

One side effect, judged an improvement: `ability_bonus_dangling_target` (finding 5)
also carries an item `context` and is filed on `abilities`, so the V/F step now
shows it too, marked "Resolve on the Abilities step". Next stays enabled exactly as
finding 5 requires; the step simply stops being silent about it.

### 12 — "below Dead Language (e.g. Latin) 1" reads wrong

**Status:** done (2026-09-04) — reads "below Latin 1 (any Dead Language)"; the
recommended-ability warning, the scholarly-language finding and the
minimum-abilities checklist carried the same shape and were fixed with it

`issue-magus_minimum_ability` composes `requirement-exemplar`
(`locales/en/main.ftl:930`) into its `$ability` argument and then appends the
score, so the parenthetical lands between ability and score and the actual
requirement — Latin 1 — is buried. Wording only, in EN and DE. The check itself
stays deliberately wider than the rules' letter (any Dead Language instance
counts), because language instances are free-text player input with no
localization path (`crates/arm-rules/src/life_stage.rs:112-118`).

### 14 — Order-minimum severity

**Status:** closed — no change

Originally logged as "demote the hard ERROR to a warning, since the book calls
these *Recommended* Minimum Abilities". That was a mis-attribution. Two distinct
sets exist and the app already models both correctly:

- **Hard minimum, error:** Parma Magica 1, Magic Theory 1, Latin 1 — ":2437
  Magi must have the following minimum Abilities … Characters with lower scores
  would not be admitted to the Order." An explicit bar.
- **Recommended, warning:** Artes Liberales 1, Latin 4, Magic Theory 3 (:2451,
  "Total Cost: 90 experience points") — already emitted as a warning
  (`crates/arm-rules/src/validation/magus.rs:244-249`).

Only finding 12's wording fix remains.

### 13 — (withdrawn)

Proposed adding "Puissant does not count toward the minimum" to the Order-minimum
message. Cut by the user: do not overcomplicate messages. The underlying
behaviour is correct and unchanged — the minimum is judged on the *bought* score,
because Puissant adds 2 "whenever you use it" (:4816) and meeting an
apprenticeship threshold is not a use, and because the book's own "Total Cost: 90
experience points" (:2461) only adds up for purchased scores.

---

## WP4 — Life stage: Gauntlet age and lab seasons

### 15 — Gauntlet age defaults to the character's age

**Status:** done (2026-09-04) — `apprenticeship.default_gauntlet_age: 25` in
`rules/core/life_stages.json`, clamped to the age, nothing written to the save

Leaving the Gauntlet-age field blank makes the engine read the character's own
age as the Gauntlet age (`crates/arm-rules/src/life_stage.rs:505-509`), i.e. zero
years as a magus. The book's baseline is a magus "25 years old and just out of
apprenticeship" (:1601), and the Darius example runs post-Gauntlet years from 26
(:2486).

**Decision.** The default is data, not a frontend prefill:
`apprenticeship.default_gauntlet_age: 25` in `rules/core/life_stages.json`, with
the absent-value fallback reading `min(default, age)`. A magus younger than 25
still stands at its Gauntlet, so no new error becomes reachable, nothing is
written into the save, and the unsaved-changes flag is untouched. A frontend
write was considered and rejected: `setGauntletAge` no-ops without a life-stage
plan (`ui/src/lib/state.svelte.ts:847-852`), the age is usually unset when the
type is chosen, and a stored 25 against a later-typed age of 22 would raise
`life_stage_gauntlet_age_after_age` — an error the blank field never produced.

### 11 — The lab-seasons warning is incomprehensible at zero post-Gauntlet years

**Status:** done (2026-09-04) — own code and message for the zero-year case, the
general one shortened, and both per-year fields read-only while the span is empty

Observed: "5 lab seasons are charged against 0 year(s) as a magus, more than the
0 those years can be charged for: only three seasons a year cost anything,
because the third already takes the whole 30 points that year — a fourth is
free." Technically correct and useless. At `max == 0` the cause is that the magus
has no years as a magus at all, which the message never says, while the
three-seasons-a-year clause it does say has no bearing on that case.

**Decision.** A distinct code and message for the `max == 0` case naming the real
cause, a plainer wording for the general case, and the lab-seasons input disabled
while post-Gauntlet years are 0. Finding 15 makes this state rare rather than the
default.

---

## WP5 — Effective-score surfaces

### 17 — Puissant Magic Theory does not appear in the Abilities section

**Status:** done (2026-09-04) — bonuses walk the catalogue and the bought
instances as a deduped union, and the tab renders a read-only "not bought" row;
granted floors such as Bjornaer's Heartbeast surface the same way

`ability_bonuses` iterates `entity.ability_scores`
(`crates/arm-rules/src/effective/ability.rs:133-146`), so a Puissant target with
no bought score produces no bonus entry and no row to show it on. Arts fixed
exactly this and recorded why (`effective/art.rs:117-122`, "a Puissant Art
applies even at 0 bought points"); Abilities never got the same treatment, and
the current behaviour is pinned by a test named
`ability_bonuses_still_omit_zero_entries`.

The fix must iterate the union of catalogue entries and bought instances:
`ability_bonus` matches `(ability, parameter)` exactly, so iterating catalogue
definitions alone would silently zero the bonus on a parameterized ability like
Puissant "(Area) Lore: Brandenburg" that *is* bought.

### 16 — Effective badges flicker while editing

**Status:** done

Changing an elemental Form makes the other three Forms' effective badges cycle
through intermediate values before settling. The score is written synchronously
(`ui/src/lib/state.svelte.ts:988-1000`) while the bonus arrives from a 150 ms
debounced IPC round-trip (`:62, 2206-2263`), and the badge renders
`currentScore + staleBonus`. Most visible with Elemental Magic because it is the
only nonlinear cross-Art recompute, but the same shape affects the ability and
characteristic badges.

**Decision.** Hold the badge's previous value while a revalidation is in flight,
so there is one transition per committed edit.

**Landed.** `AppStore` now retains `#effectiveBasis` — the very
`$state.snapshot(entity)` that produced the currently-published
`effective`/`derived`, assigned inside `revalidate`'s existing `#seq` guard so
basis and payload can never drift apart, out-of-order responses included. The
badges read their bought score through `store.readSettled(read)` instead of off
the live entity, which pins both halves of the `bought + modifier` pair to one
generation; the spinners keep reading the live entity, so direct feedback is
unchanged. Applied in `ArtGrid.svelte`, `AbilityTab.svelte` (matched by
ability + parameter, so the pairing survives a row removed above it) and
`CharacteristicPicker.svelte` (where the stale pair also flashed a phantom
"→ +0" badge, since its gate compares effective against bought). Covered by
`ArtGrid.client.test.ts`, `AbilityTab.client.test.ts` and
`CharacteristicPicker.client.test.ts` — `client` tests because the defect is a
mid-flight frame the SSR renderer can never show.

### 18 — Non-takable rows are effectively invisible

**Status:** done (2026-09-04) — the row's contents dim (brightness, not hue),
the recolour is demoted to a redundant cue, the focus ring is deliberately left
unfaded, and the dead `.pick-row:disabled` rule is gone. The blocked reason was
already announced through `use:tooltip`'s `aria-describedby`, not a `title`

Rows that cannot be taken are no longer greyed; the only cue is the "+" turning
from gold (`--accent`) to `--muted`, which is close to the normal ink colour.
Affects Abilities, Spells and Virtues/Flaws — all three use `SourcePicker`
(`ui/src/lib/components/SourcePicker.svelte:109-111`). The rows carry
`aria-disabled` rather than the native `disabled` attribute (deliberately, to
keep the reason tooltip reachable), so the generic `button:disabled { opacity:
.45 }` never applies, and the `.pick-row:disabled` rule in `app.css:969-971` is
dead code. Nothing tests this today.

---

## WP6 — Aging tab

### 23 — The log's × and the take-back button are not the same function

**Status:** done (2026-09-04) — data-corruption defect

Reported as a duplicate control. It is worse than that: the log row's ×
(`ui/src/lib/components/AgingRecordPanel.svelte:174`) calls
`removeAgingLogEntryAt(i)` (`ui/src/lib/state.svelte.ts:1739-1742`), a plain
array filter that deletes the record while leaving the aging points and the
apparent-age increase applied to the character. The take-back button calls
`revertAgingRoll(age)` → `aging::revert_year`
(`crates/arm-rules/src/aging.rs:1560-1583`), which subtracts that entry's points,
steps apparent age back, and then removes the entry.

**Decision.** Route the × through the engine revert for engine-recorded rows,
keeping plain deletion only for legacy free-text rows that carry no age (which
`revert_year` documents as out of its reach). Only then remove the duplicate
control (finding 19).

**Fixed as decided.** `AppStore.removeAgingLogEntryAt` now reads the row at the
clicked index and dispatches on it: a row carrying an `age` goes to
`revertAgingRoll(entry.age)` — the calculator's own path — and a row without one
is dropped as before. The index is the UI's address and the age is the engine's,
so reading the entry is what translates between them; `age` is exactly the key
`revert_year` addresses and `retain`s on, which is why nothing narrower would be
correct. Refusals behave as the calculator's do: a rejected revert leaves both the
character and the row untouched and lands in `agingRejections`, a failed IPC call
in the error banner — the row is never dropped on a revert that did not happen.
The ×'s accessible name follows the row: `aging-revert` ("Take back age N") for a
recorded row, `remove-item` for a hand-written one. Covered by five cases in
`state.svelte.test.ts` (undo, age-not-index addressing across a hand-written row,
the hand-written row itself, rejection, IPC failure) and one in
`AgingRecordPanel.client.test.ts` that presses the real button.

### 19 — One take-back button per recorded year

**Status:** done (2026-09-04)

Fine at three recorded years, unusable at thirty or fifty: the button list
(`AgingRollCalculator.svelte:285-302`) renders one per recorded year, wrapping
across the panel with no ordering cue. Removing arbitrary years is legitimate —
`revert_year` addresses entries by age and is exact rather than recomputed — so
the capability is kept, in the log where the years are already listed.

**Fixed as decided.** The list and its `recorded` derived value are gone from
`AgingRollCalculator.svelte`. Nothing is lost: finding 23 had already re-pointed
the log row's × at `revert_year` through `AppStore.removeAgingLogEntryAt`, under
the same `aging-revert` accessible name, and `revert_year` addresses any recorded
year rather than only the latest — so per-year undo survives with one control per
year instead of two. The e2e specs that clicked `aging-revert-36`
(`aging.e2e.js`, `aging-crisis.e2e.js`) now click `aging-log-remove-0`, which is
that same year's row and the same engine call.

### 22, 20 — Layout is awkward and wastes a screen-third

**Status:** done (2026-09-04)

The panel is an `auto-fit` grid (`ui/src/app.css:1308-1313`) with a full-width
row below it, which reads as a header band plus an orphan row. The large vertical
gap above the "Aging" heading is not an empty element: `align-items: start` means
a short block leaves the rest of its row's height bare, and the tall roll
calculator sets that height.

**Fixed at the cause.** A grid row is as tall as its tallest item, so *any*
pairing of a short block with the roll calculator reproduces the gap somewhere;
capping the calculator or stretching the short block would only move it. So the
aging surface no longer pairs at all: every one of its blocks takes a full-width
row (`.character-details .aging-panel > *`, `.character-details .aging-record > *`
in `app.css`), which is the one arrangement in which the gap cannot arise at any
width or content length. The width is reclaimed *inside* the stages instead of
between them — the Living Conditions checklist flows into as many readable columns
as fit (`.living-conditions-list`), and the four accumulated read-outs are one
stage laid out across its own width (`.aging-state`) rather than four items
scattered around the calculator, which was the other half of the "band of
columns". The surface is therefore shorter than the columns it replaces, not
taller. Reading order is now literally DOM order — one column, nothing moved by
CSS — so eye, caret and screen reader agree. The rest of `.character-details`
(the Details and Personality tabs) keeps the auto-fit columns unchanged; their
blocks are of comparable height and never had this problem.

### 24, 25 — The aging log is below the fold; longevity is below that

**Status:** done (2026-09-04)

Even on a large monitor the log scrolls out of view, so its existence is not
apparent, and the longevity-potion section sits past it. Reading order becomes
schedule → living conditions → roll → log → longevity.

**Fixed as decided.** The log now leads `AgingRecordPanel` instead of trailing it,
so the surface reads schedule → living conditions → roll → log → accumulated
totals → ritual, and the three stages above the log are all shorter than they were
(no per-year revert list, a multi-column conditions checklist). Guarded by
`AgingPanel.test.ts`'s reading-order test, which asserts the order over the real
composition rather than over any one child.

### 26 — The aging-effects list grows one row per year, unbounded

**Status:** done (2026-09-04)

A magus aged forty years renders forty stacked rows, which is what pushes
everything else off screen. A bounded scrollport already exists
(`app.css:1359-1377`); it contains the growth once the block is not last.

**Fixed as decided.** The block is no longer last — the log leads the record (see
24/25) — so its bounded scrollport is what absorbs the growth: a fortieth year
scrolls in place and moves nothing. The bound itself was also unqualified: it read
`.character-details .aging-log-scroll`, which made a correctness guarantee
conditional on a layout class, while `AgingPanel`'s own comment contemplates a
mount without that wrapper. It is now the log's own property.

### 27 — The log is not prefilled from the stress roll

**Status:** done (2026-09-04) — a recorded row reads back total, die, points per
Characteristic and apparent age, localized at render from the stored fields;
nothing generated is written to the save, and the free text stays as a note

`resolve_year` writes `effect: String::new()` deliberately
(`crates/arm-rules/src/aging.rs:1471-1488`, doc `:1396-1402`: "the structured
fields *are* the record, and the prose is the player's to add later") — while
recording `die`, `total`, the resolved per-Characteristic `points` and
`apparent_age_increased`, none of which the log row displays.

**Decision.** Render a localized read-only summary from those structured fields,
exactly as the crisis read-out already does, and leave `effect` as an optional
player note. Generated prose is deliberately *not* written into the save: saves
store choices, not rendered values, and stored prose would freeze one language
into the file.

---

## WP7 — Chrome: tabs and explanatory prose

### 1 — Edit-mode tab titles ellipsize at the default window size

**Status:** done (2026-09-04) — `.tab` gets its own 0.75rem type, tighter side
padding, and a narrower strip gap, sized so the widest set (a magus in German,
146 characters over thirteen tabs) fits the default window. The `title` fallback
stays: at the 900px minimum width German still truncates

The tabs eat too much horizontal space and their titles ellipsize; German is
worse, its labels being longer. `.tab` sets no `font-size` (it inherits 15px) and
pads `0.6rem 0.75rem` (`ui/src/app.css:252-273`). Any change must keep the strip
one line tall with every tab hit-testable (`ui/e2e/specs/tab-area.e2e.js:125-137`)
and must be checked against the German locale, not English.

### 21 — Explanatory prose, project-wide

**Status:** done (2026-09-04), with findings 2 and 3

The Aging panel spends a whole column on explanation instead of content, and the
same pattern recurs across guided creation. The player has the rulebook open; the
app does not need to teach the rules, and the prose pushes real content off
screen.

**Decision.** Remove it — not shrink, collapse, or move to tooltips — in EN and
DE together, along with the tests that pin the sentences. Keep field labels,
validation messages, and short warnings. Findings 2 and 3 are instances:

- **2** — the "Nothing has been recorded on this step yet. It does not block: you
  can continue and come back to it." notice
  (`ui/src/lib/components/WizardShell.svelte:120-127`), which is always mounted
  and reserves a line above every step body.
- **3** — the character-type explanation banner (fixed type / Flaw budget / The
  Gift prose, `ui/src/lib/components/CharacterBanner.svelte:35-60`).

The sweep also covers the per-step `wizard-guidance-*` family, the in-step hints
on the concept, experience and abilities steps, and the Aging/longevity notes.
Two hazards: deleting a hint must also remove the `aria-describedby` that points
at it, and `crates/arm-rules/RULES.md` cites two of the removed keys.

**What was removed.** 43 Fluent keys in both locales, plus one sentence trimmed:

- Wizard chrome: `wizard-step-incomplete-hint`; the whole `wizard-guidance-*`
  family (twelve per-phase lines, the two `-flaw-cap` selectors and
  `wizard-guidance-hermetic-flaw`); `character-type-explainer`,
  `character-type-budget`, `character-type-gift-{required,forbidden,optional}`.
- In-step hints: `saga-year-hint`, `ability-funding-{pool,life_stages}-hint`,
  `life-stage-gauntlet-note`, `life-stage-{gauntlet-age,lab-seasons,spell-levels}-hint`,
  `childhood-apply-hint`, `magus-recommended-hint`, `age-cap-note`,
  `warping-owed-hint`, `familiar-powers-note`.
- Aging/longevity: `living-conditions-hint`, `living-conditions-cumulative-note`,
  `aging-longevity-clamp`, `aging-die-hint`, `aging-outcome-crisis-note`,
  `aging-calculator-note`, `aging-points-note`, `longevity-sterility-note`,
  `crisis-note`, `crisis-die-hint`.
- Trimmed: `life-stage-post-gauntlet-no-years-note` lost its second sentence; the
  first stays because it explains a read-only state and is the `aria-describedby`
  target of both read-only fields.

**Kept deliberately.** The `start-*-hint` copy on the start screen, the
`derived-*-note` "guidance only / vis costs ignored" disclosures,
`familiar-{characteristics,bond}-note`, `crisis-bedridden`,
`crisis-die-unrolled`, `aging-die-capped` and `aging-note-*` — each states what
*this application* does or does not do, or a read-only state a control cannot
state for itself, rather than teaching a rule.

**Code that went with the copy.** `wizardGuidance`, `GUIDANCE_ARGS`,
`guidanceNotes`, `flawCapNotes`, `GuidanceContext`, `GuidanceNote` and
`WizardGuidance` in `ui/src/lib/derive.ts`; `WizardNavigation.phaseIncomplete`
and `AppStore.wizardPhaseIncomplete`; the `.wizard-incomplete-hint`,
`.wizard-guidance`, `.age-cap`, `.gauntlet-note`, `.magus-recommended-hint` and
`.childhood-hint` rules in `app.css`. The `.hidden-reserved` utility is kept
though nothing uses it now: it encodes cross-cutting theme 1's house rule and is
pinned by `app.css.test.ts`.

---

## Second pass — 2026-09-05

Findings from a further session against the Windows build. **Collected only;
nothing here is implemented, and no approach is decided.** Numbering continues
the first pass.

### 29 — Granted Reputations are added by a button instead of being listed

**Status:** open

A character whose Virtue or Flaw grants a Reputation gets a button reading
"Add Ecclesiastical Reputation (level 4)". Pressing it twice produces two
identical rows, and the duplicate is only reported afterwards as a validation
error.

The user's objection is the flow, not the error: the grant is already known, so
the Reputations list should simply *contain* the granted rows with their
description empty and waiting to be filled, rather than making the player press
a button to conjure something the rules already gave them. That would also make
the duplicate unreachable rather than merely invalid.

To settle at fix time: whether any V/F can grant the same Reputation kind twice
legitimately (two separate reputations of one kind), and what a save holds for a
granted-but-undescribed reputation.

### 30 — A granted Reputation never says which Virtue or Flaw granted it

**Status:** open

Asked as "Why Ecclesiastical at all??" — the app offers an Ecclesiastical
Reputation with no indication of where it came from. Three shipped items grant
that kind: `flaw.apostate` (Ecclesiastical 4), `virtue.senior_master`
(Ecclesiastical 4) and `flaw.failed_monk` (Ecclesiastical 2). The level 4 in the
screenshot narrows it to the first two, but the character's full Virtue/Flaw list
is needed to say which — and that is precisely the finding: the player should not
have to work it out. `virtue.hermetic_magus` grants no reputation, so the
character's Social Status is not the source.

Related to 29: if the list were prefilled from the grants, the row could name its
source directly.

### 31 — Alignment breaks on the not-bought Ability rows

**Status:** open

The read-only "not bought" rows added for finding 17 (an Ability carrying a
Puissant bonus or a granted floor but no bought score) do not line up with the
bought rows around them. Reported after the type rebase, so check it against the
current scale: the marker occupies the specialty column
(`.ability-selection .ability-unbought`) and the row omits the specialty input,
the parameter field and the remove button, any of which may be what shifts the
columns.

### 33 — The Aging tab lost its columns; it should keep three, done properly

**Status:** open — corrects findings 22/20/24/25 as delivered

Findings 22 and 20 asked for the awkward layout and its screen-third of white
space to be fixed. What landed instead **removed the multi-column layout**: every
aging block now spans the full width (`.character-details .aging-panel > *` /
`.aging-record > *` at `grid-column: 1 / -1`), one stage per row. That was chosen
because a grid row is as tall as its tallest item, so any pairing of a one-line
read-out with the tall roll calculator leaves the leftover as blank space — one
item per row is the only arrangement where the gap *cannot* arise.

That reasoning was sound but answered the wrong question. The three-column layout
is wanted; the gap is what was not.

Approaches to weigh at fix time — not yet decided:

- **CSS multi-column** (`column-count: 3`, `break-inside: avoid`) for the stages
  above the log, with the log kept full-width below. Blocks flow down one column
  and into the next, so short and tall neighbours pack without leftover rows and
  no block's height is set by another's. Reading order stays DOM order. The cost
  is that column *balance* is the browser's decision, not ours.
- **Explicit grid placement** — assign each block a column deliberately, grouping
  by height (e.g. schedule and living conditions in one column, the roll
  calculator alone in another, the read-outs in a third). Fully deterministic and
  keeps the log's full-width row, but a content change can reintroduce a gap, so
  it needs a geometry test that fails when one appears.
- Whatever is chosen must keep what the last pass bought: the log visible without
  scrolling, its growth bounded, the panel degrading to one column on a narrow
  window, DOM order equal to visual order, and no horizontal clipping (the
  Living Conditions row overflowed by 15px and is now held by a 360px column
  floor plus wrapping).

### 32 — Heartbeast is offered to a magus of any House

**Status:** open

A Bonisagus magus can select `virtue.heartbeast`. The entry carries no
`prerequisites` (`rules/core/virtues_flaws.json:4421-4429`), while the source
says: "You have been initiated into the Outer Mystery of the Heartbeast … and
thus are a member of House Bjornaer. You start with the Ability Heartbeast 1.
Note that all Bjornaer magi gain this Virtue for free at character creation."
(`Ars Magica - Definitive Edition (Core Rules).md:4059-4061`).

So the Virtue is not merely *associated* with Bjornaer — taking it makes the
character Bjornaer, which contradicts a House already chosen as Bonisagus. The
engine has a `Prereq::House` variant, so the mechanism exists; what needs
deciding is whether the correct model is a House prerequisite, a
mutual-exclusion with the other Houses' free Virtues, or something that changes
the House. Worth checking the other Houses' Outer Mystery Virtues for the same
gap while fixing it.
