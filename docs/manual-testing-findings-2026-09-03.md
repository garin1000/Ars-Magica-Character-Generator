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

---

## WP1 — Rules data corrections

### 8 — Eleven core-rules Virtues cite sourcebooks they were not taken from

**Status:** open

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

### 9 — `virtue.demonic_blood` is missing `tainted: true`

**Status:** open

The core rules tag it *Major, Supernatural, **Tainted*** (:3649-3650). Without
the flag the Tainted half-of-points cap (`validation/caps.rs`,
`validate_tainted_cap`) under-counts it, so a character can carry more Tainted
Virtue points than the rules allow — a wrong-rules-output defect, not cosmetic.

### 10 — Four "Free, Mythic Companion" virtues are stored as `social_status`

**Status:** closed — deliberate mapping, documented

Devil Child (:3671), Faerie Doctor (:3821), Nephilim (:4594) and Spirit Votary
(:5006) are tagged *Free, Mythic Companion* in the source. "Mythic Companion" is
not one of the six category headings the rules define (Hermetic :2878, Social
Status :2884, Supernatural :2958, Personality :2964, Story :2980, General
:2994) — it is a marker, exactly like `Tainted`. No rule keys off it, so
inventing a category (or a flag) for it would be speculative. The
`social_status` mapping stands and is recorded in `RULES.md`.

---

## WP2 — Multi-category Virtues and Flaws

### 7 — The engine models one category per V/F; the rules give some two

**Status:** open

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
counting against the Story flaw cap and becomes permitted for companions (whose
permitted categories include `story`), and Sufi becomes findable as a
Supernatural virtue.

### 6 — Visions is categorised Story only

**Status:** open — subsumed by 7

Reported from *Definitive Edition* p. 150 (`:6985-6986`, *Minor, Story,
Supernatural*). A symptom of 7, not a standalone data typo; corrected as part of
it.

---

## WP3 — Wizard flow and validation messages

### 5 — Puissant Ability deadlocks the wizard's Next button

**Status:** open

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

### 4 — Characteristic-affecting Virtues give no feedback where the user is

**Status:** open

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

### 12 — "below Dead Language (e.g. Latin) 1" reads wrong

**Status:** open

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

**Status:** open

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

**Status:** open

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

**Status:** open

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

**Status:** open

Changing an elemental Form makes the other three Forms' effective badges cycle
through intermediate values before settling. The score is written synchronously
(`ui/src/lib/state.svelte.ts:988-1000`) while the bonus arrives from a 150 ms
debounced IPC round-trip (`:62, 2206-2263`), and the badge renders
`currentScore + staleBonus`. Most visible with Elemental Magic because it is the
only nonlinear cross-Art recompute, but the same shape affects the ability and
characteristic badges.

**Decision.** Hold the badge's previous value while a revalidation is in flight,
so there is one transition per committed edit.

### 18 — Non-takable rows are effectively invisible

**Status:** open

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

**Status:** open — data-corruption defect

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

### 19 — One take-back button per recorded year

**Status:** open

Fine at three recorded years, unusable at thirty or fifty: the button list
(`AgingRollCalculator.svelte:285-302`) renders one per recorded year, wrapping
across the panel with no ordering cue. Removing arbitrary years is legitimate —
`revert_year` addresses entries by age and is exact rather than recomputed — so
the capability is kept, in the log where the years are already listed.

### 22, 20 — Layout is awkward and wastes a screen-third

**Status:** open

The panel is an `auto-fit` grid (`ui/src/app.css:1308-1313`) with a full-width
row below it, which reads as a header band plus an orphan row. The large vertical
gap above the "Aging" heading is not an empty element: `align-items: start` means
a short block leaves the rest of its row's height bare, and the tall roll
calculator sets that height.

### 24, 25 — The aging log is below the fold; longevity is below that

**Status:** open

Even on a large monitor the log scrolls out of view, so its existence is not
apparent, and the longevity-potion section sits past it. Reading order becomes
schedule → living conditions → roll → log → longevity.

### 26 — The aging-effects list grows one row per year, unbounded

**Status:** open

A magus aged forty years renders forty stacked rows, which is what pushes
everything else off screen. A bounded scrollport already exists
(`app.css:1359-1377`); it contains the growth once the block is not last.

### 27 — The log is not prefilled from the stress roll

**Status:** open

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

**Status:** open

The tabs eat too much horizontal space and their titles ellipsize; German is
worse, its labels being longer. `.tab` sets no `font-size` (it inherits 15px) and
pads `0.6rem 0.75rem` (`ui/src/app.css:252-273`). Any change must keep the strip
one line tall with every tab hit-testable (`ui/e2e/specs/tab-area.e2e.js:125-137`)
and must be checked against the German locale, not English.

### 21 — Explanatory prose, project-wide

**Status:** open

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
