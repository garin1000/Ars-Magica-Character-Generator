# Book-template conformance — Grog, Companion and Magus Templates

**Date:** 2026-09-22 (grogs), 2026-09-23 (companions and magi)
**Scope:** every character template the core rulebook prints — the six **Grog
Templates**, `ArMDE:1191-1404` (`### Grog Templates`), the five **Companion
Templates**, `ArMDE:1406-1597` (`### Companion Templates`), and the twelve
**Magus Templates**, `ArMDE:1599-2199` (`### Magus Templates`), one per House.
Twenty-three templates in all. The book prints no Mythic Companion template, so
that character type has no counterpart here.

Each template was transcribed into a save fixture under
`crates/arm-rules/tests/fixtures/book_templates/` and run through the engine's
real load path against the shipped `rules/core/*.json`. The assertions live in
`crates/arm-rules/tests/book_templates.rs`; every one of them is green, because a
disagreement is written there as an explicit expectation naming its entry below.

Each discrepancy is judged as one of:

- **(a)** the engine or the rules data is wrong,
- **(b)** the fixture transcription is wrong,
- **(c)** **the book itself** is internally inconsistent — reserved for a
  contradiction the book's own arithmetic demonstrates,
- **(d)** **unsettled or silent** — the rulebook does not say, and more than one
  reading is defensible; or the rule is fine but the engine has no vocabulary to
  express it (a capability gap).

**Citation form.** Entries added in the companion and magus passes carry the
owning `####` heading beside each `ArMDE:NNNN` line number —
`` ArMDE:3573 `#### Clerk` ``. The rulebook sources will be re-synced upstream and
every line number dies at once; the heading anchor survives that. The grog entries
predate the convention and still cite bare line numbers.

## Headline result — grogs

Four of the six templates — the Grizzled Veteran, the Hunter, the Specialist and
the Standard Soldier — reproduce **every** printed figure exactly: Soak, all five
Fatigue penalties, all five Wound bands, Encumbrance (both the penalty and the
Burden in parentheses), and every Initiative / Attack / Defense / Damage on every
Combat row. The Berserker and the Tough Guy differ, and both differences are
traced below.

**All six Characteristic point-buys cost exactly the 7 starting points**
("You start with seven points to spend", `ArMDE:2342`, and the cost table below
it at `ArMDE:2346-2354`; encoded in `rules/core/characteristics.json`), and
**all six Ability XP totals match the book's own age formula to the point**, with
the single exception recorded under S2. The formula is 75 XP native language +
45 XP childhood spread + 15 XP per year after age 5 (`ArMDE:2213`, `ArMDE:2214`,
`ArMDE:2378`, `ArMDE:2392`), plus 50 restricted Martial XP where Warrior is taken
(`ArMDE:5227`).

| Template | Age | Book grant (general) | Warrior (Martial) | Engine needs (general) |
|---|---|---|---|---|
| Berserker | 15 | 270 | — | 270 |
| Grizzled Veteran | 45 | 720 | 50 | 720 |
| Hunter | 20 | 345 | 50 | 345 |
| Specialist | 19 | 330 | 50 | **329** (see S2) |
| Standard Soldier | 25 | 420 | 50 | 420 |
| Tough Guy | 19 | 330 | 50 | 330 |

"Engine needs" is the `xp_pool` at which the engine reports neither
`not_enough_xp` nor `general_xp_unspent`; both directions were probed, so each
figure is exact rather than merely sufficient.

**Social Status Virtues are free.** Covenfolk (`virtue.covenfolk`) carries
magnitude `free` (0 points), so it never touches the 3/3 grog budget — correct
against `ArMDE:2825` ("You must take one Social Status") read with `ArMDE:2824`
("up to 3 points of Flaws, and an equal number of points of Virtues").
**Do not generalize this**: it is true of Covenfolk, not of the category. Four of
the five companion Social Statuses are Minor and cost a point — see "Social Status
Virtues are **not** uniformly free" below.

## Headline result — companions

One of the five — **the Rogue** — reproduces every printed figure exactly and
validates clean: no errors, no warnings. The other four disagree, and **every
mechanical disagreement but one has the same root cause**: a Virtue whose rulebook
text authorizes an Ability category, or grants experience, or grants a bonus, is
encoded in `rules/core/virtues_flaws.json` with that clause missing (F1, K1, P1,
P2, P3, W1) or over-broad (P4). Not one of them is a fixture error, and **not one
is the book contradicting itself on the mechanics.** Three further entries are
not defects at all: K3 (mounted combat) and W2 (Wise One's exclusive choice) are
capability gaps, and K5 is a model gap exposed by fixing K2. **K2 itself — the
Knight's Encumbrance — is the one place a template's own mechanics are wrong**:
its printed "2 (3)" omits the great sword's Load, contradicting both its Equipment
line and the whole-loadout method four other templates demonstrate (see *Per-row
loadout hypothesis, refuted* under K2). Four more (F2, F3, K4, plus the en-dash
note) are formatting in `rules/source/en/`.

**All five Characteristic point-buys cost exactly the 7 starting points**
(`ArMDE:2342` `### Characteristics`, cost table at `ArMDE:2346-2354`), once the
two post-purchase Virtues are accounted for: **Improved Characteristics** adds 3
points to the budget (`ArMDE:4103-4105` `#### Improved Characteristics`) and
**Great (Characteristic)** adds its +1 *after* the buy, so it never appears in the
cost (`ArMDE:3987-3989` `#### Great (Characteristic)`).

| Template | Printed | Bought | Buy cost | Budget |
|---|---|---|---|---|
| Female Scholar | Int +5 | Int +3 (Great Intelligence x2) | `6+1+1+3-1-1+0+1 = 10` | `7 + 3` |
| Knight | — | as printed | `0+0+1+1+1+1+3+3 = 10` | `7 + 3` |
| Priest | — | as printed | `1+3+1+3+0+0-1+0 = 7` | `7` |
| Rogue | Dex +4, Qik +4 | Dex +3, Qik +3 (Great Dexterity, Great Quickness) | `0+1+0+1-1+0+6+6 = 13` | `7 + 3 + 3` |
| Witch | — | as printed | `3+3+1+1-1+0+3+0 = 10` | `7 + 3` |

**All five Ability XP totals match the book's age formula to the point.** The
formula is 75 XP native language + 45 XP childhood spread (`ArMDE:2378`
`#### Early Childhood`) + 15 XP per year after age 5 (`ArMDE:2392`
`#### Later Life`), except that **Wealthy** pays 20 per year (`ArMDE:2394`, same
heading) — which is the Knight, and `ArMDE:1486` says so outright.

| Template | Age | Book grant (general) | Book restricted | Engine needs (general) |
|---|---|---|---|---|
| Female Scholar | 20 | 345 | — | 345 |
| Knight | 25 | 520 (Wealthy, 20/yr) | — | 520 |
| Priest | 33 | 540 | 50 (Well-Traveled) | **590** (see P2) |
| Rogue | 20 | 345 | — | 345 |
| Witch | 30 | 495 | 50 (Educated) | 495 |

"Engine needs" is the `xp_pool` at which validation reports neither
`not_enough_xp` (error) nor `general_xp_unspent` (warning); since both code sets
are asserted empty of those, each figure is exact in both directions rather than
merely sufficient.

**A Virtue-granted Supernatural Ability's first point is free**, and the engine
implements it: "you will not need to spend experience points for the first point
of those Abilities" (`ArMDE:2639` `## Mythic Companions`). That is what makes the
Priest's Sense Holiness and Unholiness 4 cost 45 rather than 50, and each of the
Witch's Premonitions 3 / Second Sight 3 / Wilderness Sense 3 cost 25 rather than
30 — and without it neither total balances.

### Social Status Virtues are **not** uniformly free

The grog pass could record "Social Status Virtues are free" because Covenfolk is.
Across the companions it is false and the arithmetic depends on it: **Clerk,
Knight, Priest and Wise One are Minor** (1 point each — `ArMDE:3572`
`#### Clerk`, `ArMDE:4196` `#### Knight`, `ArMDE:4797` `#### Priest`,
`ArMDE:5258` `#### Wise One`), while **Wanderer is Free** (`ArMDE:5224`
`#### Wanderer`). `rules/core/virtues_flaws.json` has all five right. Counting the
social status as free would leave four of the five templates one point short on
the Virtue side of a balance the book intends to be exact:

| Template | Virtues | Flaws | Budget |
|---|---|---|---|
| Female Scholar | Clerk 1 + 4 Minor + Great Intelligence x2 = 7 | Black Sheep 3 + Driven 3 + Social Handicap 1 = 7 | 10 / 10 |
| Knight | Knight 1 + Wealthy 3 + 3 Minor = 7 | Oath of Fealty 3 + Proud 3 + Overconfident 1 = 7 | 10 / 10 |
| Priest | Priest 1 + 7 Minor = 8 | Compassionate 3 + Plagued 3 + Clumsy 1 + Vow 1 = 8 | 10 / 10 |
| Rogue | Wanderer 0 + 8 Minor = 8 | Avaricious 3 + Dark Secret 3 + Compulsion 1 + Night Terrors 1 = 8 | 10 / 10 |
| Witch | Wise One 1 + 6 Minor = 7 | Compassionate 3 + Enemies 3 + Nocturnal 1 = 7 | 10 / 10 |

Every row balances, and three of the Customization Notes confirm the headroom
independently: the Knight "can still take three more points of Flaws"
(`ArMDE:1486`), the Priest "could take another two Minor Flaws and corresponding
Virtues" (`ArMDE:1523`), the Witch "can take another Major Flaw, or three more
Minor ones" (`ArMDE:1597`) — 7+3, 8+2 and 7+3 against the companion budget of 10
(`ArMDE:2834-2840` `#### Companions`). The per-category caps hold too: none
exceeds 5 Minor Flaws, one Story Flaw, two Personality Flaws or one Major
Personality Flaw.

## The Berserker (`ArMDE:1195-1227`)

Characteristics Int -2, Per -1, Pre -1, Com -1, Str +3, Sta +2, Dex +2, Qik +1
(`ArMDE:1197`) cost `-3 -1 -1 -1 +6 +3 +3 +1 = 7`. Exact.

Abilities (`ArMDE:1221`) cost `5 + 15 + 30 + 30 + 5 + 75 + 75 + 5 + 30 = 270`,
and the book grants `75 + 45 + 15 x 10 = 270` at age 15. Exact — including the
Native Language 5 that consumes the whole 75 XP childhood language grant.

### B1 — Berserk authorizes Martial Abilities, but the rules data does not say so

- **Book:** Virtues and Flaws are Covenfolk, Berserk, Large, Short Attention
  Span, Wrathful (Minor) — **no Warrior** (`ArMDE:1205`) — yet the character buys
  Great Weapons 5 and Single Weapon 1 (`ArMDE:1221`). Berserk's own description
  ends: "You may learn Martial Abilities at character creation." (`ArMDE:3502`).
- **Engine:** two `ability_category_requires_virtue` errors, one per Martial
  Ability, because Martial is a gated category (`ArMDE:2392`,
  `rules/core/abilities.json` → `categories_requiring_virtue`) and
  `virtue.berserk` in `rules/core/virtues_flaws.json` carries only its three
  combat/Soak modifiers — no `ability_authorization` effect and no restricted XP
  pool, which are the two things the engine accepts as authorization.
- **Wrong:** **(a) the rules data.** The book is self-consistent here; the
  extraction of Berserk dropped its authorization clause. Fixing it is a
  data-only change (add the Martial authorization to `virtue.berserk`), and it
  would turn this template's error set empty.

### B2 — Berserk's combat modifiers are applied unconditionally

- **Book:** Soak +9 "(Stamina, full metal scale armor)" = `2 + 7`
  (`ArMDE:1215`); Pole Axe Init +2, Attack +13, Defense +7, Damage +14
  (`ArMDE:1212`); Kick Init +0, Attack +6, Defense +4, Damage +6
  (`ArMDE:1213`). Berserk grants "+2 to Attack and Soak scores, but ... a -2
  penalty to Defense" **"While berserk"** (`ArMDE:3502`) — a condition, and the
  printed statblock is the not-berserk one.
- **Engine:** Soak 11, Pole Axe Attack +15 / Defense +5, Kick Attack +8 /
  Defense +2 — every figure shifted by exactly Berserk's `+2 / -2 / +2`.
  Initiative and Damage, which Berserk does not touch, match the book exactly, as
  do Encumbrance 0 (3) and the Size +1 wound bands.
- **Wrong:** **(a) the rules data.** `virtue.berserk` is classified
  `in_play_effect` with three unconditional modifiers; the engine has no way to
  express "only while berserk", so the character permanently fights as if raging.
  This is a wrong-rules-output defect on every Berserk character, not only this
  template.

### B3 — Two Personality Flaws, against the book's own grog checklist

- **Book:** Short Attention Span and Wrathful (Minor) (`ArMDE:1205`) are both
  Personality Flaws. The grog checklist says "You should not take more than one
  Personality Flaw" (`ArMDE:2827`).
- **Engine:** `too_many_personality_flaws` (count 2, max 1) — a **warning**,
  correctly, because the rule is a "should".
- **Wrong:** **(c) the book.** The template violates the checklist printed
  sixteen hundred lines later in the same volume. Harmless in play (the
  Customization Notes at `ArMDE:1227` even invite adding a fourth Flaw), but it
  is an internal inconsistency.

### B4 — The Size line is missing from the statblock

- **Book:** the template format promises "Size: The character's size."
  (`ArMDE:1153`), and the other five grog templates print it. The Berserker's
  entry omits it (`ArMDE:1197-1199`) although he takes Large.
- **Engine:** derives Size +1 from `virtue.large` and prints wound bands
  1-6 / 7-12 / 13-18 / 19-24, which is exactly what `ArMDE:1219` shows — so the
  template's own Wound Penalties row proves the missing line should read +1.
- **Wrong:** **(c) the book**, a typographical omission with no mechanical
  consequence. Recorded because the Wound row is the evidence.

### B5 — Virtue/Flaw budget is 2/2, not 3/3

Berserk + Large = 2 Virtue points; Short Attention Span + Wrathful (Minor) = 2
Flaw points. Legal (the budget is a ceiling, `ArMDE:2824`), balanced, and the
Customization Notes say so outright. **No discrepancy** — recorded so the table
below is complete.

## The Grizzled Veteran (`ArMDE:1229-1263`)

**Reproduces every printed figure.** Error set empty.

Characteristics are printed as Int 0, Per 0, Pre -1, Com -1, Str 0, Sta +1 (1),
Dex +2 (2), Qik +2 (2) (`ArMDE:1231`), which cost only 5 of the 7 starting
points — but the Customization Notes state "the years have already reduced his
Presence and Communication to -1 each" (`ArMDE:1263`). The fixture therefore buys
Pre 0 / Com 0 (cost `0+0+0+0+0+1+3+3 = 7`, **exact**) and records the aging
points that pushed them down. Two independent checks confirm that reading:

- A Characteristic drops when its aging points **exceed** the absolute value of
  the current score (`ArMDE:16579`, `ArMDE:16613`). At a bought 0, one point is
  enough — so Pre and Com each cost exactly 1 aging point, while Sta +1 (1),
  Dex +2 (2) and Qik +2 (2) sit one short of their own thresholds and do not
  drop. The engine derives Pre -1 and Com -1 from precisely that.
- Total aging points `1 + 1 + 1 + 2 + 2 = 7` gives Decrepitude 1 with 2 excess —
  exactly the "Decrepitude: 1 (2)" the book prints at `ArMDE:1237`. Reading the
  printed Pre/Com as bought scores would give 5 points and Decrepitude 1 (0),
  contradicting the book.

Abilities (`ArMDE:1257`) cost 770; the book grants `75 + 45 + 15 x 40 = 720`
general plus Warrior's 50 Martial. Exact. Single Weapon 8 is exactly the age-45
cap (`ArMDE:2368-2374`).

### V3 — Aging rolls advisory on a template that shows only their outcome

- **Book:** prints the resolved aging state (Decrepitude 1 (2), `ArMDE:1237`)
  with no per-year history — templates are "fully generated" (`ArMDE:1147`).
- **Engine:** `aging_rolls_pending` **warning**, because the character is past
  the first-roll age and carries an empty `aging_log`.
- **Wrong:** **none of the three.** The save format has no way to say "these
  rolls happened off-screen", so the advisory is correct and unavoidable for any
  imported pre-aged character. Noted, not a defect.

### V4 — Equipment omits weapons the statblock scores

Bows 5 (longbow) and Great Weapon 5 (pole axe) are bought (`ArMDE:1257`) but
neither weapon appears under Equipment (`ArMDE:1259`), so the book prints no
combat row for them. The fixture follows the book. Confirmed correct by
Encumbrance: axe 1 + heater shield 2 + full metal scale 7 = Load 10 = Burden 4,
and the book prints "Encumbrance: 4 (4)" (`ArMDE:1261`). Adding the missing
weapons would change nothing at Burden 4 but would contradict the book's own
equipment list. **No discrepancy.**

## The Hunter (`ArMDE:1265-1296`)

**Reproduces every printed figure.** Error set empty, warning set empty.

Characteristics (`ArMDE:1267`) cost `0 +6 -3 -3 +0 +1 +3 +3 = 7`. Exact.
Abilities (`ArMDE:1290`) cost 395 = 345 general (`75 + 45 + 15 x 15` at age 20)
+ Warrior's 50, and Bows 4 costs exactly 50 — matching the note "the character's
Bows score uses the bonus XP from Warrior" (`ArMDE:1296`) to the point.

Short Bow Attack +9 / Defense +6 (`ArMDE:1282`) reproduce only **without** the
Ability-specialization bonus, which is correct: the Bows specialty is "shooting
from cover", not the weapon, so the fixture leaves `specialization_applies`
false. This is a useful cross-check that the engine's specialization toggle
behaves as the book's arithmetic expects.

### H1 — Two statblock lines are merged in the Markdown source

- **Book:** `ArMDE:1277` reads "Virtues and Flaws: Covenfolk, Warrior,
  Pessimistic Personality Traits: Brave +3, Loyal +1, Pessimistic +3" — the
  Personality Traits heading has lost its line break, so the flaw *Pessimistic*
  and the trait *Pessimistic* run together.
- **Engine:** not affected (the rules JSON is not extracted from the template
  section).
- **Wrong:** **(c) the book**, a source-formatting defect in
  `rules/source/en/`. Worth a fix in the Markdown; no mechanical consequence.

## The Specialist (`ArMDE:1298-1332`)

**Reproduces every printed figure.** Error set empty, warning set empty.

Characteristics (`ArMDE:1300`) print Com -4, which is below the -3 buy floor:
the bought score is -3 (cost -6) and Poor (Characteristic: Com) supplies the
last point ("lower one which is already -3 or lower by one point",
`ArMDE:6598-6600`). Total `-1 +0 -1 -6 +3 +3 +6 +3 = 7`. Exact.

Virtues 3 / Flaws 3 — the only template that spends the grog budget in full, and
exactly one Personality Flaw (Obsessed); Afflicted Tongue and Poor
(Characteristic) are General.

Single Weapon 7 at age 19 exceeds the age cap of 5 (`ArMDE:2368-2374`) and is
accepted, because Affinity raises the cap by two for that Ability
(`ArMDE:3374`). The engine implements that exception correctly.

### S2 — Affinity overcharges by one experience point

- **Book:** the template's own arithmetic balances only if Single Weapon 7 costs
  **93** experience points. Affinity says creation XP "are increased by half,
  rounded up" (`ArMDE:3374`), and `ceil(93 x 3 / 2) = 140`, which is the table
  cost of score 7. The remaining budget then lands on the printed "Bows 1 (2)"
  (`ArMDE:1326`) — a score of 1 with **2** points banked toward the next, which
  is exactly the leftover: `330 - (30+30+50+15+5+75+5+75) - (93 - 50) = 2`.
- **Engine:** charges **94**. `charged_cost` computes `ceil(T x den / num)` =
  `ceil(140 x 2 / 3) = 94`, but the smallest charge satisfying
  `ceil(c x 3 / 2) >= T` is `floor(den x (T-1) / num) + 1 = floor(278/3) + 1 =
  93`. The two formulas agree on the rulebook's own worked example (Perdo 10,
  `T = 55`: both give 37, `ArMDE:2443`), which is why the off-by-one was not
  caught there. Probed empirically: at `xp_pool` 330 the engine reports 1 point
  unspent, not the book's 2.
- **Wrong:** **(a) the engine.** The formula is off by one whenever
  `T x den` is not a multiple of `num`. It always errs **against** the player, by
  at most one point per Affinity-bought score, on both Abilities and Arts. The
  fixture uses 329 (the engine's exact requirement) and this entry records the
  book's 330.

## The Standard Soldier (`ArMDE:1334-1368`)

**Reproduces every printed figure.** Error set empty, warning set empty.
No discrepancies.

Characteristics (`ArMDE:1336`) cost `-1 +0 +0 +0 +1 +1 +3 +3 = 7`. Abilities
(`ArMDE:1362`) cost 470 = 420 general (`75 + 45 + 15 x 20` at age 25) + Warrior's
50. Both exact. Virtues 1 / Flaws 1, one Personality Flaw (Weakness).

Fist Attack +7 and Defense +7 (`ArMDE:1354`) reproduce only **with** the
specialization bonus (Brawl "punching" does align with the fist), the mirror of
the Hunter's bow — so the two templates between them pin both settings of the
toggle.

## The Tough Guy (`ArMDE:1370-1404`)

Characteristics (`ArMDE:1372`) cost `-1 +0 +1 -1 +1 +6 +0 +1 = 7`. Abilities
(`ArMDE:1398`) cost 380 = 330 general (`75 + 45 + 15 x 14` at age 19) + Warrior's
50. Both exact. Every derived figure reproduces, including Soak +13 (Sta +3 +
armor 7 + Tough +3, `ArMDE:1392`) and the Size +1 wound bands (`ArMDE:1396`).
Error set empty.

### T1 — Two Personality Flaws, against the book's own grog checklist

- **Book:** Overconfident and Weakness (drinking) (`ArMDE:1382`) are both
  Personality Flaws; No Sense of Direction is General. The checklist allows one
  (`ArMDE:2827`).
- **Engine:** `too_many_personality_flaws` (count 2, max 1) — warning.
- **Wrong:** **(c) the book**, the same shape as B3. Note that the Customization
  Notes say "the Flaws can be changed freely" (`ArMDE:1404`), so the fix is the
  troupe's.

## The Female Scholar (`ArMDE:1410-1445` `#### The Female Scholar`)

Abilities (`ArMDE:1439`) cost `5+15+30+15+15+5+15+15+75+75+5+30+30+15 = 345`, and
the book grants `75 + 45 + 15 x 15 = 345` at age 20. Exact. Every derived figure
reproduces: Soak -1, the five Fatigue penalties, the Size 0 wound bands,
Encumbrance 0 (0), and Dodging Init +1 / Def +1 with Attack and Damage absent —
the book's "Atk n/a … Dam n/a" (`ArMDE:1431`).

**Her Reputation is covered.** "Selfish shrew 2 (local)" (`ArMDE:1428`) needs a
granting Virtue or Flaw — "Characters only start with a Reputation if they choose
a Virtue or Flaw that grants one" (`ArMDE:2514` `### Reputations`) — and
`flaw.black_sheep` carries exactly `grants_reputation` local, score 2. No
`reputation_not_granted`, and the score matches the grant to the point.

### F1 — Clerk authorizes Academic Abilities, but the rules data does not say so

- **Book:** "Due to your training, you may take Academic Abilities during
  character generation" (`ArMDE:3573` `#### Clerk`), and Clerk is her only Social
  Status. She buys six Academic Abilities (`ArMDE:1439`).
- **Engine:** six `ability_category_requires_virtue` errors —
  `ability.artes_liberales`, `ability.civil_and_canon_law`,
  `ability.dead_language` (Latin), `ability.medicine`, `ability.philosophiae`,
  `ability.theology_christian`. Academic is a gated category (`ArMDE:2392`
  `#### Later Life`, `rules/core/abilities.json` →
  `categories_requiring_virtue`), and `virtue.clerk` in
  `rules/core/virtues_flaws.json` carries **no effects at all**.
- **Wrong:** **(a) the rules data**, the exact shape of the Berserker's B1. A
  data-only fix (add an academic `ability_authorization` to `virtue.clerk`) empties
  this template's error set.

### F2 — A period where the Abilities list needs a comma

- **Book:** `ArMDE:1439` opens "Arabic 1 (medical terms). Area Lore 2
  (nunneries), …" — a sentence-ending period inside a comma-separated list, so a
  naive split loses the boundary between the first two Abilities.
- **Engine:** not affected; the rules JSON is not extracted from the template
  section.
- **Wrong:** **(c) the book**, a source-formatting defect in `rules/source/en/`.

### F3 — The Virtues and Flaws line groups by semicolon only once

- **Book:** the format promises Gift, then Social Class, then Major Virtues,
  Minor Virtues, Major Flaws, Minor Flaws (`ArMDE:1163` `### Format`).
  `ArMDE:1424` runs Clerk, five Minor Virtues and two Major Flaws together with
  commas and then puts its single semicolon before the one Minor Flaw. The
  intended grouping is still recoverable — each class is alphabetical, which is
  what the format also promises — but the separator is in one place out of four.
- **Wrong:** **(c) the book**, formatting. Recorded because reading a name into
  the wrong group is the fastest way to mis-total the budget; the Priest
  (`ArMDE:1502`) uses the separator correctly and is the contrast case.

## The Knight (`ArMDE:1447-1486` `#### The Knight`)

Abilities (`ArMDE:1480`) cost
`15+30+15+30+15+5+30+75+15+5+50+5+75+75+75+5 = 520`, and Wealthy's 20 XP per year
grants `75 + 45 + 20 x 20 = 520` at age 25. Exact — and the Customization Notes
name the dependency: "The Knight's Wealthy Virtue gives him a lot of experience
points, and so cannot be changed without effectively recreating the character"
(`ArMDE:1486`). Soak +10 (chain mail 9 + Stamina +1, `ArMDE:1474`), the Fatigue
row and the Size 0 wound bands all reproduce, as do **Attack, Defense and Damage
on every on-foot Combat row**. Single Weapon 5+2 resolves to 7, so Puissant
Ability's fixed bonus (`ArMDE:4814-4816` `#### Puissant Ability`) is applied
correctly.

### K1 — Knight authorizes Martial Abilities, but the rules data does not say so

- **Book:** "You may take Martial Abilities during character generation"
  (`ArMDE:4197` `#### Knight`), and he buys Great Weapon 5 and Single Weapon 5
  (`ArMDE:1480`).
- **Engine:** two `ability_category_requires_virtue` errors, on
  `ability.great_weapon` and `ability.single_weapon`; `virtue.knight` carries no
  effects.
- **Wrong:** **(a) the rules data.** Third instance of the same gap, after B1
  (Berserk) and F1 (Clerk).

### K2 — RESOLVED 2026-09-23: only equipped gear counts toward Load

**Fixed in the engine on Norbert's decision ("K2 align to RAW. Use the Equipped
flag"). `derived/combat.rs::encumbrance` now sums only slots whose `equipped`
flag is set, and the Knight reproduces the book's Encumbrance and all three of
his on-foot Initiative figures exactly.** The analysis that led there is kept
below, because it is the evidence for the change; what it describes as the
engine's behaviour is now historical. The change surfaced a new gap, **K5**.

Guards: `derived.rs::only_equipped_gear_counts_toward_load` (unit) and
`book_templates.rs::the_knight_matches_the_book` (this template). Recorded in
`crates/arm-rules/RULES.md` under Encumbrance.

#### The original analysis

- **Book:** Equipment is "Full chain mail, long sword, heater shield, great
  sword" (`ArMDE:1482`) and Encumbrance is "2 (3)" (`ArMDE:1484`). Burden 3
  corresponds to Load 6-9 (`ArMDE:17109-17121` `## Encumbrance`), and the Knight's
  Str +1 then gives Encumbrance `3 - 1 = 2`.
- **Engine:** Load `chain 6 + long sword 1 + heater 2 + great sword 2 = 11`,
  Burden 4, Encumbrance `4 - 1 = 3`.
- **Wrong:** **(c) the book** — see the *Per-row loadout hypothesis, refuted*
  section below, which settled this. This entry passed through a wrong verdict of
  "(d) unsettled" on the way; the reasoning that produced it is kept here because
  the refutation is only legible against it.

  **The superseded argument was:** the book's figure is correct under a
  per-loadout reading, and it is correct for *either* loadout — taken at the time
  as evidence that it was computed that way rather than carelessly:

  - long sword 1 + heater shield 2 + chain mail 6 = **9** → Burden 3
  - great sword 2 + chain mail 6 = **8** → Burden 3 (the great sword is
    two-handed, so it is never wielded with the shield)

  The engine instead sums **everything carried, equipped or not** — a deliberate
  and documented choice (`derived/combat.rs::encumbrance`) resting on the literal
  wording "Add up the total Load that a character is carrying" (`ArMDE:17107`).
  On that reading the Knight is carrying all four items at once and Burden 4 is
  right.

  **The rulebook never says which reading applies.** It gives no notion of a
  stowed or sheathed item, and no template other than the Knight lists a second
  primary weapon, so nothing else in the book discriminates between the two. The
  Grizzled Veteran is suggestive but not decisive: he has Bows 5 (longbow) and
  Great Weapon 5 (pole axe) yet his Equipment line names neither
  (`ArMDE:1257`, `ArMDE:1259`), so his "4 (4)" cannot distinguish the readings
  either.

  **What is at stake** is exactly **one point of Initiative on every Combat row** —
  the book's +2 / +2 / +0 become +1 / +1 / -1 — and nothing else. Attack and
  Defense are exempt from Encumbrance "as long as the Encumbrance is largely due
  to weapons and armor" (`ArMDE:17105`), but **Initiative is not**:
  `INITIATIVE TOTAL: Quickness + Weapon Initiative Modifier - Encumbrance +
  Stress Die` (`ArMDE:16658`). That the Attack, Defense and Damage figures
  reproduce untouched while only Initiative moves is the confirmation that Load
  is the single term in dispute.

  **Decided and implemented.** The engine's old reading penalised a player for
  recording gear the character owns — a magus who listed a spare dagger lost
  Initiative for it. Load now counts only `equipped` items, which is what
  `EquipmentSlot::equipped` already recorded and what `encumbrance` had been
  ignoring, and the Knight reproduces exactly. One pre-existing unit test,
  `derived.rs::casting_total_with_encumbrance_and_focus`, had to be corrected: it
  set `equipped: false` on the armour whose Encumbrance it then subtracted from a
  Casting Score, relying on the old semantics incidentally rather than asserting
  them, so the armour is now worn and the test's intent is unchanged.

#### Per-row loadout hypothesis, refuted

Norbert proposed that the book computes each Combat row against **that row's own
equipped set** — so the great sword row would drop the shield's Load, the fist row
would drop both weapons', and the Knight would need no erratum. The hypothesis is
**testable and false**, and the test also settles K2's verdict.

Each printed row's Initiative exposes the Encumbrance used to compute it, because
`INITIATIVE TOTAL = Quickness + Weapon Initiative Modifier - Encumbrance`
(`ArMDE:16658` `### Initiative`) and the weapon modifiers are fixed data. Solving
for Encumbrance on the **unarmed** row of every template that carries a weapon —
the row where a per-row loadout would drop the most Load — gives:

| Template | Row | Printed Init | Implied Enc | Full loadout | Per-row |
|---|---|---|---|---|---|
| Standard Soldier (`ArMDE:1354`) | Fist | -1 | **3** | 3 ✓ | 2 ✗ |
| Grizzled Veteran (`ArMDE:1249`) | Kick | -3 | **4** | 4 ✓ | 3 ✗ |
| Tough Guy (`ArMDE:1390`) | Fist | -2 | **3** | 3 ✓ | 2 ✗ |
| Specialist (`ArMDE:1318`) | Fist | 0 | **2** | 2 ✓ | 1 ✗ |

All four use the **whole carried loadout** on every row: each fist or kick row is
still paying Encumbrance for the axe and shield the character is not striking
with. Four independent refutations, so the book applies **one** Encumbrance
figure to every row of a statblock.

The Knight cannot discriminate the two readings — Loads 6, 8 and 9 all fall in
Burden 3 (`ArMDE:17109-17121`), so every loadout of his yields Encumbrance 2 —
which is why the coincidence was mistaken for evidence. Against the book's actual
method, his Equipment line (`ArMDE:1482`) totals Load 11 → Burden 4 → Encumbrance
3, and his printed "2 (3)" is an arithmetic slip that omits the great sword's
Load 2. All three of his Combat rows then inherit it, which is exactly why they
are self-consistent with the wrong figure.

**This does not disturb the engine change.** Only `equipped` gear counting toward
Load was Norbert's decision and stands on its own: it is the more defensible model
(a stowed weapon should not slow you down), and it corrected a live defect on the
shipped export path — the golden magus in `tests/fixtures/magus_export.md` carries
**unequipped** partial leather scale and was being charged Load 3, Burden 3,
Encumbrance 2 and a point of Initiative on both Combat rows for it. The engine now
reproduces the Knight's printed figures; it simply arrives there by a sounder
route than the book did.

### K5 — `equipped` is overloaded, so a stowed alternate weapon loses its Combat rows

- **Book:** the Knight prints **five** Combat rows (`ArMDE:1468-1472`), two of
  them for the great sword — while printing an Encumbrance that counts only the
  wielded set (K2). So the book wants the stowed great sword to *yield Combat
  rows* and *not contribute Load*.
- **Engine:** `EquipmentSlot::equipped` gates both behaviours with one flag.
  Marking the great sword equipped restores its rows and breaks the Encumbrance;
  marking it unequipped fixes the Encumbrance and drops the rows. The fixture
  takes the second, so the engine emits four on-foot rows (fist ×2, long sword
  ×2) and no great-sword row.
- **Wrong:** **(d) unsettled — a model gap exposed by fixing K2.** The two
  behaviours are independent in the rules and coupled in the model. Resolving it
  needs a third state — carried and wieldable but not currently wielded — or two
  separate flags. Judged the better trade for now: a **wrong number** (Encumbrance
  and three Initiatives) is a product-integrity defect, while a **missing row** is
  a visibly absent feature, and K3 already records that this template's Combat
  block is incompletely modelled.

### K3 — The two mounted Combat rows have no engine counterpart

- **Book:** four weapon rows, two of them mounted (`ArMDE:1468`, `ArMDE:1470`).
  "A mounted character adds his Ride score, to a maximum of +3, to his Attack and
  Defense Totals" (`ArMDE:16839` `#### Mounted Combat`), and at Ride 5 the printed
  mounted lines are exactly the on-foot ones plus +3 Attack and +3 Defense:
  `+14/+14 → +17/+17`, `+13/+10 → +16/+13`.
- **Engine:** emits one line per equipped weapon (two for a one-handed weapon
  carried with a shield) and no mounted variant. Nothing in the save format
  records being mounted, and no `CombatMod` in the catalogue is conditional on it.
- **Wrong:** **none of the three — a capability gap.** The rule is simple and
  the data already has the Ride score; what is missing is a place to say "mounted"
  and a `min(Ride, 3)` term on the Attack/Defense totals. The test pins the exact
  set of lines the engine does emit, so adding the variant will fail loudly here.

### K4 — `Etiquette (noble) 3` puts the parameter before the score

- **Book:** `ArMDE:1480` prints "Etiquette (noble) 3" where the declared format is
  "Ability X(Z) (specialization)" (`ArMDE:1177` `### Format`) and every other
  entry in the same list — including the Knight's own "Area Lore 3 (nobles)" two
  items earlier — puts the score first.
- **Wrong:** **(c) the book**, formatting. The fixture reads it as Etiquette 3
  with specialty "noble", which is the only reading that fits the 520 XP total.

## The Priest (`ArMDE:1488-1523` `#### The Priest`)

Abilities (`ArMDE:1517`) cost
`75+15+5+75+30+5+30+30+105+5+50+30+75+45+15 = 590` — note Sense Holiness and
Unholiness 4 at **45**, not 50, because its granting Virtue pays the first point
(`ArMDE:2639`). Soak +0, the Fatigue row, the Size 0 wound bands, Encumbrance
0 (0) and Dodging Init +0 / Def +2 (`ArMDE:1509`) all reproduce; the Defense
figure needs Brawl 1 **plus** its "dodging" specialty, which is the fixture's one
specialization toggle.

### P1 — Priest authorizes Academic Abilities, but the rules data does not say so

- **Book:** "You may purchase Academic Abilities during character generation"
  (`ArMDE:4804` `#### Priest`). The same paragraph also names his Vow: "you would
  normally take the Minor Personality Flaw Vow … for your vow of celibacy", which
  settles `flaw.vow_minor` over `flaw.vow_major`.
- **Engine:** four `ability_category_requires_virtue` errors —
  `ability.artes_liberales`, `ability.civil_and_canon_law`,
  `ability.dead_language`, `ability.theology_christian`. `virtue.priest` carries no
  effects.
- **Wrong:** **(a) the rules data.** Fourth instance. His *Arcane* Ability,
  Dominion Lore, is accepted, because `virtue.student_of_realm` does carry an
  authorization — see P4 for how much.

### P2 — Well-Traveled's 50 experience points are missing from the rules data

- **Book:** "You have fifty bonus experience points to spend on living languages,
  Area Lores, and Bargain, Carouse, Charm, Etiquette, Folk Ken, or Guile"
  (`ArMDE:5241` `#### Well-Traveled`). He holds 360 XP of eligible Abilities
  (Area Lore 5 at 75, Charm 5 at 75, Etiquette 3 at 30, Folk Ken 6 at 105, Living
  Language 5 at 75), so the pool is fully spendable, and `540 general + 50 = 590`
  is his total.
- **Engine:** `virtue.well_traveled` carries no effects, so there is no restricted
  pool and the whole 590 must come out of the general one. The fixture therefore
  records `xp_pool: 590` where the book's age formula grants 540.
- **Wrong:** **(a) the rules data.** A `restricted_ability_xp` effect of 50 over
  those Abilities is all that is missing; `virtue.educated` and `virtue.warrior`
  already encode the identical shape. Note the side effect: because an earmarked
  pool is *also* read as authorization, adding it would additionally authorize
  those Abilities — none of which is gated here, so it changes nothing else.

### P3 — Student of (Realm)'s +2 Lore bonus is missing from the rules data

- **Book:** "you have a +2 bonus on all uses of the appropriate Lore"
  (`ArMDE:5054` `#### Student of (Realm)`), which is why the statblock prints
  **Dominion Lore 3+2** (`ArMDE:1517`) — the `X+Y` format being "a fixed bonus
  from a Virtue" (`ArMDE:1177`). The Witch's **Magic Lore 3+2** (`ArMDE:1591`) is
  the same Virtue on the other realm.
- **Engine:** effective score 3 in both cases. `virtue.student_of_realm` carries
  `ability_authorization` but no `ability_bonus`.
- **Wrong:** **(a) the rules data.** Wrong rules output on every Student of
  (Realm) character, not just these two. `virtue.puissant_ability` shows the
  effect shape the entry needs; the complication is that the bonus must follow the
  Virtue's `realm` parameter to the matching Lore, where Puissant Ability's
  parameter *is* the ability. The same passage also forbids taking Student of
  (Realm) and Puissant Ability for the same Lore — an incompatibility the data
  does not express either.

### P4 — Student of (Realm) authorizes all four Lores, whatever realm it names

- **Book:** "You may take that Lore at character generation even if you cannot
  learn other Arcane Abilities" (`ArMDE:5054`) — **that** Lore, the one the
  parameter names.
- **Engine:** `virtue.student_of_realm` lists all four of `ability.dominion_lore`,
  `ability.faerie_lore`, `ability.infernal_lore` and `ability.magic_lore` in a
  single unparameterized `ability_authorization`. The Witch demonstrates it: she
  holds Student of **Magic** alone yet her Divine, Faerie and Infernal Lores are
  all accepted (`ArMDE:1591`).
- **Wrong:** **(a) the rules data**, and this one is *permissive* — it lets
  through characters the rules forbid, where F1/K1/P1 block characters the rules
  allow. Recorded rather than fixed; note that fixing it would turn three of the
  Witch's Abilities into errors unless her Wise One Arcane access is modelled too
  (W2).

## The Rogue (`ArMDE:1525-1560` `#### The Rogue`)

**Reproduces every printed figure.** Error set empty, warning set empty — the only
companion template that validates clean.

Abilities (`ArMDE:1554`) cost `5+30+30+30+15+5+5+75+75+75 = 345` against the
`75 + 45 + 15 x 15 = 345` the book grants at age 20. Exact. Fist Init +4 /
Attack +7 / Defense +7 / Damage -1 (`ArMDE:1546`) reproduces only **without** the
Brawl specialization, correctly: the specialty is "getting away", not punching —
the mirror of the Standard Soldier, whose "punching" specialty must be applied.

He is also the densest test of the post-purchase Characteristic Virtues:
Improved Characteristics twice (+6 to the buy budget) and Great Dexterity and
Great Quickness lifting a bought +3 to the printed +4 each. The engine derives
both +4s, and the Initiative and Attack totals that depend on them.

## The Witch (`ArMDE:1562-1597` `#### The Witch`)

Abilities (`ArMDE:1591`) cost
`30+5+5+5+30+50+5+15+30+30+5+50+30+75+75+25+25+30+25 = 545` — Premonitions,
Second Sight and Wilderness Sense at **25** each rather than 30, their first point
paid by the granting Virtue (`ArMDE:4790` `#### Premonitions`, `ArMDE:4890`
`#### Second Sight`, `ArMDE:5249` `#### Wilderness Sense`). Educated's 50
earmarked points cover Artes Liberales 1 + Latin 4 = 55 of that, leaving
`545 - 50 = 495` general, which is exactly `75 + 45 + 15 x 25` at age 30. Exact.
Soak +0, Fatigue, the Size 0 wound bands, Encumbrance 0 (0) and Dodging
Init +0 / Def +0 (`ArMDE:1583`) all reproduce.

**All four Supernatural Abilities are covered.** The companion profile grants no
free Supernatural slot without The Gift, so each of Premonitions, Second Sight and
Wilderness Sense needs its own Virtue, and each has one. (Magic Lore is Arcane,
not Supernatural, and goes through P4.) No
`supernatural_ability_requires_virtue`.

### W1 — Educated authorizes only the two Abilities its XP grant names

- **Book:** "You may purchase Academic Abilities during character generation.
  During character generation you get an additional 50 experience points, which
  must be spent on Latin and Artes Liberales" (`ArMDE:3713` `#### Educated`) —
  **two** clauses, a category authorization and a narrower XP earmark.
- **Engine:** one `ability_category_requires_virtue` error, on
  `ability.medicine`. `virtue.educated` encodes only the earmark
  (`restricted_ability_xp` over `ability.artes_liberales` and
  `ability.dead_language`), and the engine reads an earmarked pool as permission
  for exactly what it names — so her Artes Liberales and Latin pass and her
  Medicine 5 does not.
- **Wrong:** **(a) the rules data.** Subtler than F1/K1/P1: the entry is not empty,
  it is half-transcribed, and the half that survived happens to authorize enough
  Abilities to look right. Adding an academic category authorization alongside the
  existing earmark fixes it.

### W2 — Wise One's "either Arcane or Academic, but not both" is unmodelled

- **Book:** "You may take either Arcane or Academic Abilities, but not both, at
  character creation" (`ArMDE:5259` `#### Wise One`). The Witch holds both —
  Academic through Educated, Arcane through Student of Magic — which is legal
  only if each is read as coming from its own Virtue rather than from Wise One.
- **Engine:** `virtue.wise_one` carries no effects, so it neither grants nor
  restricts anything; the exclusive-choice clause has no representation in the
  effect vocabulary at all (authorizations are additive, never subtractive).
- **Wrong:** **none of the three — a capability gap**, and a low-value one: the
  rule only ever bites a character whose *only* access to a gated category is Wise
  One. Recorded because it is the other half of P4: whoever fixes Student of
  (Realm)'s over-broad authorization has to decide what Wise One grants.

## Headline result — magi

**Five of the twelve reproduce every printed figure exactly and validate clean** —
Bonisagus, Flambeau, Guernicus, Tremere and Tytalus: no errors, no warnings, and
not one number out of place. A sixth, Merinita, differs only on **P3**, a defect
the companion pass had already found twice. The remaining six differ on the
entries listed below, and **every one of those differences is either a known
defect shape or the book contradicting itself** — none is a new engine bug.

Across all twelve the engine reproduces Soak, all five Fatigue penalties, all five
Wound bands (at Size -2, 0 and +2), Encumbrance, the Dodging Combat row, every Art
score including Puissant and Elemental-Magic bonuses, the one non-default
Confidence score, and — the richest arithmetic in the book — **the Casting Total
printed beside 79 of the 80 spells** (the eightieth is MAG2, a spell the book
never defines).

The magi are by far the strongest confirmation in this exercise, because nothing
about their arithmetic is forgiving: a single wrong Art, Characteristic, Virtue
magnitude or experience-point cost moves a Casting Total, and 79 of them had to
land. Where the numbers do part company the cause is one of exactly three things:

- **a conditional modifier the data applies unconditionally** (MAG1 — Bjornaer,
  Mercere), the same defect shape as the Berserker's B2;
- **a free-text Magical Focus the engine cannot match to a spell** (MAG8 —
  Flambeau, Ex Miscellanea, Mercere, Tremere), a capability gap, not an error;
- **the book disagreeing with itself** (MAG3, MAG4, MAG5, MAG6, MAG7, MAG11 and
  the Criamon `Ag 0` typo), each demonstrable from its own printed Arts line.

Only one new **(a)** defect is a wrong *number* on every character who has the
Virtue (MAG1). Two more are narrow data omissions (MAG10, MAG11). Nothing here
needed the engine changed.

### All twelve Virtue/Flaw budgets balance exactly

The magus budget is 10 Virtue / 10 Flaw points (`rules/core/character_types.json`,
`ArMDE:2853-2862` `#### Magi`). Reading it right depends entirely on the semicolon
grouping the format declares — "The first Virtue listed is The Gift … The next is
the character's Social Class. Then comes all other Major Virtues, Minor Virtues,
Major Flaws, and Minor Flaws" (`ArMDE:1163` `### Format`) — and on knowing that a
Virtue can go unpaid in **two** ways that never show on the line itself: magnitude
`free` (The Gift, Hermetic Magus) and **free by House grant**, which is what the
asterisk marks (`ArMDE:1601`).

| Template | Virtues (bought) | Flaws (bought) | Free by House grant |
|---|---|---|---|
| Bjornaer | Ways of the Forest 3 + 4 Minor (Quiet Magic twice) = 7 | Blatant Gift 3 + Tormenting Master 3 + Deficient Form 1 = 7 | Heartbeast |
| Bonisagus | 8 Minor (Great Intelligence twice) = 8 | Favors 3 + Painful Magic 3 + Driven 1 + Weak Enchanter 1 = 8 | Puissant Magic Theory |
| Criamon | Flexible Formulaic Magic 3 + 4 Minor = 7 | Magic Addiction 3 + Twilight Prone 3 + Incomprehensible 1 = 7 | The Enigma |
| Ex Miscellanea | Major Magical Focus 3 + 4 Minor = 7 | Generous 3 + Plagued by Supernatural Entity 3 + Deficient Auram 1 = 7 | Giant Blood, Affinity with Terram, Necessary Condition |
| Flambeau | Greater Immunity 3 + Major Magical Focus 3 + 3 Minor = 9 | Enemies 3 + Necessary Condition 3 + Wrathful 3 = 9 | Puissant Ignem |
| Guernicus | 6 Minor = 6 | Curse of Venus 3 + Restriction 3 = 6 | Hermetic Prestige |
| Jerbiton | Gentle Gift 3 + 4 Minor = 7 | Deficient Technique 3 + Necessary Condition 3 + Susceptibility to Infernal Power 1 = 7 | Puissant Music |
| Mercere | Major Magical Focus 3 + 4 Minor = 7 | Ambitious 3 + Difficult Longevity Ritual 3 + Cyclic Magic (Negative) 1 = 7 | Puissant Creo |
| Merinita | Strong Faerie Blood 3 + 4 Minor = 7 | Chaotic Magic 3 + Plagued by Faerie 3 + Faerie Upbringing 1 = 7 | Faerie Magic |
| Tremere | Elemental Magic 3 + 4 Minor = 7 | Ambitious 3 + Weak Magic Resistance 3 + Susceptibility to Divine Power 1 = 7 | Minor Magical Focus (certamen) |
| Tytalus | Life-Linked Spontaneous Magic 3 + 4 Minor = 7 | Painful Magic 3 + Tormenting Master 3 + Weak Parens 1 = 7 | Self-Confident |
| Verditius | 7 Minor = 7 | Dwarf 3 + Weak Spontaneous Magic 3 + Difficult Spontaneous Magic 1 = 7 | Verditius Magic |

Every row balances and every row is under budget or on it, so each template's
Customization Notes ("can take three more points of Flaws, and a similar number
of Virtues", `ArMDE:2149`) check out too. The per-category caps hold throughout:
at most one Major Hermetic Virtue (`virtue_category_caps`), at most one Story
Flaw, at most two Personality Flaws of which at most one is Major, at most five
Minor Flaws. Three templates are worth calling out because they are the ones that
could have gone wrong:

- **Bjornaer's Ways of the Forest is Major** (`ArMDE:5232` `#### Ways Of The
  (Land)`) and sits in the Major-Virtue group before the first semicolon of
  `ArMDE:1617`; read as Minor the budget comes to 5, not 7.
- **Bonisagus's Favors is a Major Story Flaw** (`ArMDE:1668`), which is what makes
  his Flaw side 8 rather than 5.
- **Ex Miscellanea carries three grants, not one** — a Minor Hermetic Virtue, a
  Major non-Hermetic Virtue **and a Major Hermetic Flaw** (`rules/core/houses.json`),
  which is why `ArMDE:1766` carries three asterisks, one of them on a Flaw. Count
  Giant Blood or Necessary Condition as bought and both sides break.

### All twelve Characteristic point-buys cost exactly 7

The buy budget is 7 (`ArMDE:2342` `### Characteristics`, cost table at
`ArMDE:2346-2354`), raised by 3 per **Improved Characteristics** (`ArMDE:4103-4105`).
Three Virtues and one Flaw move a printed score away from its bought one:
**Great (Characteristic)** adds +1 after the buy (`ArMDE:3987-3989`), **Giant
Blood** adds Size +2 with +1 Str and +1 Sta (`ArMDE:3977` `#### Giant Blood`), and
**Dwarf** subtracts Size 2 with -1 Str and -1 Sta (`ArMDE:5998` `#### Dwarf`).

| Template | Printed | Bought | Buy cost | Budget |
|---|---|---|---|---|
| Bjornaer | — | as printed | `6+0+0-1+0+1+0+1 = 7` | `7` |
| Bonisagus | Int +5 | Int +3 (Great Intelligence x2) | `6+0+0+1+0+0+0+0 = 7` | `7` |
| Criamon | — | as printed | `6+0+3-6+0+3+0+1 = 7` | `7` |
| Ex Miscellanea | Str +4, Sta +4 | Str +3, Sta +3 (Giant Blood) | `6-1-1+0+6+6-3-3 = 10` | `7 + 3` |
| Flambeau | — | as printed | `3+1+0-1+0+3+0+1 = 7` | `7` |
| Guernicus | Per +4 | Per +3 (Great Perception) | `6+6+1+0+0+0-3+0 = 10` | `7 + 3` |
| Jerbiton | — | as printed | `6+1+1+1+0+0+1+0 = 10` | `7 + 3` |
| Mercere | — | as printed | `3+0+0-1+0+3+1+1 = 7` | `7` |
| Merinita | — | as printed | `6+1+1+3-1-1-1-1 = 7` | `7` |
| Tremere | — | as printed | `6-3+0+0+0+3+0+1 = 7` | `7` |
| Tytalus | Int +4 | Int +3 (Great Intelligence) | `6-1+0+0+0+3+1+1 = 10` | `7 + 3` |
| Verditius | Str -3, Sta +1 | Str -2, Sta +2 (Dwarf) | `6+0+0+0-3+3+1+0 = 7` | `7` |

The engine derives every printed score from the bought one, and the tests assert
the four that differ.

### Experience: 435 points at Gauntlet, and every template spends them to the point

A magus of 25 straight out of apprenticeship has `75` XP of native language +
`45` early childhood (`ArMDE:2378` `#### Early Childhood`) + `15 x 5 = 75` for ages
5 to 10 (`ArMDE:2392` `#### Later Life`) + **240 apprenticeship**
(`ArMDE:2435` `#### Magus Only — Apprenticeship`) = **435**, spendable on
Abilities *and* Arts
from one pool ("These experience points can be spent on Arts or Abilities",
`ArMDE:2435`). Two Virtues move the figure and the engine applies both as a signed
bonus on top of the typed `xp_pool`: **Skilled Parens** +60 (`ArMDE:4966`) and
**Weak Parens** -60.

| Template | Abilities | Arts | Total | Engine needs (`xp_pool`) |
|---|---|---|---|---|
| Bjornaer | 300 | 135 | 435 | 435 |
| Bonisagus | 315 | 120 | 435 | 435 |
| Criamon | 315 | 120 | 435 | 435 |
| Ex Miscellanea | 315 | 120 | 435 | 435 |
| Flambeau | 305 | 130 | 435 | 435 |
| Guernicus | 320 | 115 | 435 | **432** (see MAG12) |
| Jerbiton | 350 | 135 | 485 | 435 + Privileged Upbringing's 50 restricted |
| Mercere | 315 | 120 | 435 | 435 |
| Merinita | 315 | 120 | 435 | 435 |
| Tremere | 350 | 145 | 495 | 435 + Skilled Parens 60 |
| Tytalus | 285 | 90 | 375 | 435 - Weak Parens 60 |
| Verditius | 313 | 122 | 435 | **436** (see S2) |

"Engine needs" is the `xp_pool` at which validation reports neither
`not_enough_xp` (error) nor `general_xp_unspent` (warning); both code sets are
asserted empty, so each figure is exact in both directions.

Three of these balance **only** because a Virtue-granted Ability's first point is
free (`ArMDE:2639`): Bjornaer's Heartbeast 2 costs 10 rather than 15, Criamon's
Enigmatic Wisdom 3 costs 25 and Magic Sensitivity 2 costs 10, and Merinita's
Faerie Magic 1 costs nothing at all. Without that rule none of the three reaches
435, which is an independent confirmation that the engine implements it.

**Affinity is charged, not free.** Bonisagus, Flambeau, Mercere and Verditius each
buy an Art at 12 with an Affinity, and `78 x 2 / 3 = 52` exactly — so the engine's
`charged_cost` off-by-one (S2) does **not** bite on Arts, whose table costs are
triangular numbers and therefore never leave the residue that trips it. It bites
on exactly one figure in this whole pass: Verditius's Affinity-bought Craft
(stonemason) 4 (see MAG13).

### Spell levels: ten of twelve are exactly 120

The magus profile's budget is 120 levels (`rules/core/character_types.json`,
`ArMDE:2435` `#### Magus Only — Apprenticeship`: "240 experience points, and 120
levels of spells"), and the two Parens Virtues move it: **Skilled Parens** +30
(`ArMDE:4966` `#### Skilled Parens`) and **Weak Parens** -30, the latter stated as
a total — "60 fewer experience points and 30 fewer spell levels from
apprenticeship, for a total of 180 experience points and 90 levels of spells"
(`ArMDE:7074` `#### Weak Parens`). Both are encoded as `Effect::SpellLevels` and
both are applied.

| Template | Spells | Levels | Budget |
|---|---|---|---|
| Bjornaer | 8 | 120 | 120 |
| Bonisagus | 5 | 120 | 120 |
| Criamon | 6 of 7 | **100** | 120 (see MAG2) |
| Ex Miscellanea | 7 | 120 | 120 |
| Flambeau | 5 | 120 | 120 |
| Guernicus | 6 | 120 | 120 |
| Jerbiton | 10 | 120 | 120 |
| Mercere | 5 | 120 | 120 |
| Merinita | 7 | 120 | 120 |
| Tremere | 8 | 150 | 120 + 30 |
| Tytalus | 5 | 90 | 120 - 30 |
| Verditius | 7 | 120 | 120 |

## The Bjornaer (`ArMDE:1603-1652` `#### Bjornaer`)

Every printed figure reproduces except the Casting Totals, and that one difference
is MAG1. Soak +1, the Fatigue row, the Size 0 wound bands, Encumbrance 0 (0), and
Dodging Init +1 / Def +4 (`ArMDE:1624`) — the Defense needing Brawl 2 **plus** its
"dodge" specialty — all come back exactly. Error set empty.

**Quiet Magic twice is legal, and only the rules data says so.** The template takes
"Quiet Magic (x2)" (`ArMDE:1617`) and its 7/7 budget needs both copies counted, but
the Virtue's own entry (`ArMDE:4822-4824` `#### Quiet Magic`) never says it may be
taken more than once — unlike Puissant Art, Puissant Ability and Ways of the
(Land), all of which say so explicitly. `virtue.quiet_magic` carries
`max_per_target: 2`, so the engine accepts it. **(d) — the book is silent**; the
data's reading is the only one under which this template is legal, and it is
almost certainly right, but nothing in the printed entry supports it.

### MAG1 — a conditional Casting-Total modifier applied unconditionally

- **Book:** the Casting Totals at `ArMDE:1643-1650` are Technique + Form + Stamina
  and nothing else: MuAn +19 = `Mu 10 + An 8 + Sta 1`, PeAn +12, ReAn +10, and the
  Corpus mirror of each. Ways of the (Land) grants "+3 … to all rolls, including
  combat and Casting Scores, **that directly involve that area and its
  inhabitants**" (`ArMDE:5233` `#### Ways Of The (Land)`) — a condition, and none of
  these eight spells is a forest spell.
- **Engine:** +22 / +15 / +13 throughout, every figure exactly 3 high.
  `virtue.ways_of_the_land` carries its bonus as a `casting_total_mod` with
  `scope: "all"`, which the engine adds to every cell of the grid.
- **Wrong:** **(a) the rules data**, the exact shape of the Berserker's B2. The
  effect vocabulary has no way to say "only in your terrain", so the magus casts
  as if permanently at home. Wrong rules output on every Ways of the (Land)
  character, not just this template.
- **The Mercere is the second instance, and worse**: Cyclic Magic (Positive) +3,
  Cyclic Magic (Negative) -3 and Special Circumstances +3 are *all* `scope: "all"`,
  so a magus whose Virtues describe day, night and storms applies all three at
  once, day or night, storm or none, for a permanent net +3 (`ArMDE:1992-1996`
  prints +26 and +35; the engine returns +29 and +38). Whoever fixes this should
  sweep `rules/core/virtues_flaws.json` for `casting_total_mod` with a conditional
  clause in its rulebook text rather than patch the two templates that show it.

## The Bonisagus (`ArMDE:1654-1700` `#### Bonisagus`)

**Reproduces every printed figure.** Error set empty, warning set empty — and it
is the most load-bearing clean sheet in the magus pass, because almost everything
that could go wrong is present: two Affinities, two Great Intelligences, a House
choice-grant, and five Casting Totals.

Abilities cost `5+15+15+30+15+50+5+50+75+15+5+5+30 = 315` and Arts
`Cr 12 → 52 + Re 3 → 6 + Au 12 → 52 + Co 4 → 10 = 120` — the two twelves each
Affinity-charged at `ceil(78 x 2 / 3) = 52` — for exactly the 435 the age formula
grants. Int +5 (`ArMDE:1656`) is a bought +3 plus two Great Intelligences, and
"Magic Theory 4+2" (`ArMDE:1683`) is the House's free Puissant Magic Theory on a
bought 4. Every Casting Total lands: `Cr 12 + Au 12 + Sta 0 = 24` on all four
Auram spells and 16 on the Corpus one.

## The Criamon (`ArMDE:1702-1750` `#### Criamon`)

Reproduces every printed figure — Soak +2, Fatigue, the Size 0 wound bands,
Encumbrance 0 (0), Dodging Init +1 / Def +1, all eight Arts, "Enigmatic Wisdom
3+2" and all six available Casting Totals. Error set empty; one warning, MAG2.

Abilities cost 315 (with the Enigma's and Magic Sensitivity's free first points)
and Arts 120, for 435 exactly.

### MAG2 — a spell the template spends levels on has no entry in the book

- **Book:** the spell list at `ArMDE:1741-1748` runs to seven spells totalling the
  magus budget of 120 levels. The fourth, **"Piercing the Magical Veil" (InVi 20)**
  (`ArMDE:1745`), links to `#piercing-the-faerie-veil` and adds "(see Piercing the
  Faerie Veil)". The Spells chapter prints only the Faerie one
  (`ArMDE:15707` `##### Piercing the Faerie Veil`), whose text ends "There are
  separate but related spells for Divine, Magical and Infernal regiones"
  (`ArMDE:15709`) — an acknowledgement that the variants exist, not an entry for
  them.
- **Engine:** `rules/core/spells.json` therefore holds one
  `spell.piercing_the_faerie_veil` and no Magical counterpart, so the fixture can
  carry six spells and 100 levels, and validation reports
  `spell_levels_unspent` (used 100, budget 120).
- **Wrong:** **(d) — the book leaves it undefined and the extraction was faithful.**
  Two fixes are defensible and nobody has chosen between them: add a
  `spell.piercing_the_magical_veil` (and the Divine and Infernal siblings) on the
  strength of `ArMDE:15709`, or model one `Piercing the (Realm) Veil`
  parameterized by realm, the shape `spell.unravelling_the_fabric_of_form` already
  uses for its `(Form)`. Until then no Criamon can be built to the printed 120.

### MAG3 — `Ag 0` is not an Art

- **Book:** the Arts line reads "Cr 4, In 6, Mu 4, Pe 4, Re 4, An 0, **Ag 0**, Au 0,
  Co 0, He 0, Ig 0, Im 2, Me 1, Te 0, Vi 10" (`ArMDE:1733`). There is no Art
  abbreviated `Ag`: each Art "is listed with its common two-letter abbreviation"
  from `ArMDE:8843` onwards (`ArMDE:8847` `#### Creo (Cr) "I Create"` and the
  fourteen entries after it), and the slot between An and Au is **Aq**, Aquam —
  which is otherwise missing from the line entirely.
- **Engine:** unaffected; `rules/core/arts.json` is not extracted from the template
  section, and the fixture records Aquam 0 by simply omitting it.
- **Wrong:** **(c) the book**, a typo with no mechanical consequence (the score is
  0 either way). Recorded because the line's own shape is the proof: fifteen slots,
  fourteen real abbreviations and one that names nothing, with exactly one Art
  unaccounted for.

## The Ex Miscellanea (`ArMDE:1752-1801` `#### Ex Miscellanea`)

The only Size +2 template in the book, and the only one exercising three House
grants at once. Str +4 / Sta +4 (bought +3 each, lifted by Giant Blood), Soak +7
(Stamina +4 + Tough +3), the Size +2 wound bands 1-7 / 8-14 / 15-21 / 22-28,
Encumbrance 0 (0), Dodging Init -2 / Def +1 and Te 12+3 = 15 all reproduce. Error
set empty, warning set empty. Abilities 315 + Arts 120 = 435 exactly, with Terram
12 Affinity-charged at 52.

### MAG4 — two identical spells, two different Casting Totals, neither reachable

- **Book:** The Earth's Carbuncle prints `Re(Mu)Te 15/+27` and Hands of the
  Grasping Earth `Re(Mu)Te 15/+23` (`ArMDE:1798-1799`). Same Technique, same Form,
  same magus. The statblock's own Arts line (`ArMDE:1784`) gives
  `Re 5 + Te 15 + Sta +4 = 24`, and the Major Magical Focus (stone) would add
  `min(5, 15) = 5` for 29. Neither 27 nor 23 is reachable, and the two cannot both
  be right whatever the focus does. A requisite adds nothing to a Casting Total
  (`ArMDE:9089` `## Casting Spells`), so the `(Mu)` explains neither.
- **Engine:** 24 base and 29 within focus, for both rows.
- **Wrong:** **(c) the book.** The rest of the list is internally consistent —
  CrTe +35 is `27 + 8` within the focus, MuTe +27 is `23 + 4`, Pe(Re)Te +22 is the
  base with the focus not applied (metal, not stone) — so these two rows are the
  only ones that do not follow from the printed Arts.

### MAG5 — the Grappling Combat row has no engine counterpart

- **Book:** two Combat rows, "Dodging: Init -2 … Defense +1" and "Grappling:
  Init -2, Attack +2, Defense +2, Damage n/a" (`ArMDE:1773-1774`). Grappling is a
  mode of unarmed combat, not a weapon, and this is the only template in the book
  that scores it.
- **Engine:** `rules/core/equipment.json` has `weapon.fist`, `weapon.kick` and
  `weapon.dodge` but no grapple entry, and `combat_totals` derives a row only from
  an equipped weapon, so only the Dodging row is emitted.
- **Wrong:** **(d) — a capability gap**, the same shape as the Knight's K3. Adding
  a zero-Load `weapon.grapple` with the right Attack/Defense modifiers would be a
  data-only change; whether Grappling belongs in the weapons table at all is the
  decision nobody has made.

## The Flambeau (`ArMDE:1803-1849` `#### Flambeau`)

**Reproduces every printed figure**, Casting Totals included. Error set empty,
warning set empty.

Abilities 305 + Arts 130 = 435 (Creo and Ignem both at 12 with an Affinity, 52
each). Ig 12+3 = 15 from the House's free Puissant Ignem. Soak +2, Dodging Init +1
/ Def +4 (Brawl 2 plus its "dodging" specialty), Encumbrance 0 (0).

**All five Casting Totals are +41, and that is the within-focus figure**:
`Cr 12 + Ig 15 + Sta 2 = 29`, plus `min(12, 15) = 12` inside the Major Magical
Focus (Flames) = 41. Every spell on the list is a flame, so the book prints the
focused figure throughout and never the base — which is what makes this template
the cleanest proof that the focus doubling is implemented correctly. See MAG8 for
why the engine reports both.

### MAG8 — a Magical Focus is free text, so the engine cannot say which spells are inside it

- **Book:** "you get to add the lower of your Technique or Form score twice"
  for spells within the focus (`ArMDE:4399-4422` `#### Major Magical Focus`), and
  each statblock prints, per spell, whichever figure applies: Flambeau +41
  throughout (all flames), Ex Miscellanea +35 / +27 / +22 (stone, stone, metal),
  Mercere +26 / +35 (not weather, weather), Tremere the base throughout (nothing
  on his list is certamen).
- **Engine:** the focus parameter is `domain: "text"` — "flames", "stone",
  "weather", and nothing relates it to a spell — so `derived/casting.rs` computes
  **both** figures for every cell and leaves the choice to the reader:
  `CastingTotal::within_focus` beside `CastingTotal::formulaic`.
- **Wrong:** **(d) — a capability gap, and arguably the right design.** Deciding
  membership would mean a taxonomy the rulebook does not supply; `ArMDE:4399-4422`
  settles it by example and troupe agreement, not by rule. The tests assert both
  numbers for all four focus magi, so a future membership model will fail loudly
  here.

## The Guernicus (`ArMDE:1851-1897` `#### Guernicus`)

**Reproduces every printed figure**: Per +4 (bought +3 plus Great Perception),
In 12+3 = 15, Soak +0, Dodging Init +0 / Def +2, Encumbrance 0 (0), and all six
Casting Totals (`InCo +20`, `InIm +21`, `PeIm +8`, `InMe +21`). Error set empty,
warning set empty.

### MAG11 — Hermetic Prestige grants a Reputation of 4, and the book uses 3 twice

- **Book:** the Virtue's own entry says "You gain a Reputation of level 4 within
  the Order" (`ArMDE:4073` `#### Hermetic Prestige`). This template prints
  "Reputations: Quaesitor (Hermetic) **3**" (`ArMDE:1869`), and the worked example
  in Detailed Character Creation prints the same: "Darius does have a Reputation,
  thanks to his Hermetic Prestige Virtue … it has a level of **3**"
  (`ArMDE:2518` `### Reputations`).
- **Engine:** `virtue.hermetic_prestige` carries `grants_reputation` with
  `score: 4`, following the Virtue's entry, and nothing objects to a fixture
  recording 3 — the grant is a permission to hold a Reputation of that kind, not a
  floor on its score.
- **Wrong:** **(c) the book**, two witnesses to 3 against one to 4. The rules data
  follows the Virtue's own definition, which is the right precedence (a definition
  outranks a statblock), so no change is proposed — but the disagreement is real
  and a saga that follows the templates will be one point off the Virtue as
  written.

### MAG12 — an Art's leftover experience has nowhere to go

- **Book:** the Arts line prints "In 12+3 **(5)**" (`ArMDE:1881`) — score 12 with
  5 experience points banked toward 13, the `X (Z)` format declared at
  `ArMDE:1179` `### Format`. Those 5 points are Affinity-bought, so they cost 3 of
  his 435: `Abilities 320 + (In 83 → 55) + Pe 3 + Co 15 + Im 21 + Me 21 = 435`
  exactly, and he is the only magus whose total needs the banked points to balance.
- **Engine:** `types.rs::ArtScore` stores a whole bought score and nothing else, so
  In 12 costs the Affinity-charged 52 and the remaining 3 points have no home. The
  fixture uses `xp_pool: 432`.
- **Wrong:** **(d) — a storage-model gap**, the Art-side twin of the grog pass's
  "Bows 1 (2)" note. Deliberate and documented (`effective/art.rs` says so of the
  Elemental Magic redistribution too), and it costs at most 4 experience points per
  Art. Recorded because this is the only template in the book that makes it
  visible.

## The Jerbiton (`ArMDE:1899-1950` `#### Jerbiton`)

Reproduces every printed figure but one, and the exception is MAG6. Seven Arts,
"Music 4+2" from the House's free Minor Virtue spent on Puissant Music
(`ArMDE:1950`), Soak +0, Dodging Init +0 / Def +0, Encumbrance 0 (0), and nine of
ten Casting Totals. Error set empty, warning set empty.

Abilities cost 350 and Arts 135, for 485 — which balances only with **Privileged
Upbringing**'s 50 extra experience points (`ArMDE:4806-4808`
`#### Privileged Upbringing`; `rules/core/virtues_flaws.json` →
`restricted_ability_xp` over general, academic and martial). He holds far more
than 50 XP of eligible Abilities, so the pool is
fully spendable and `435 + 50 = 485` is exact.

### MAG6 — a halved Casting Total rounded up, against the book's own rounding rule

- **Book:** Illusion of Cool Flames prints `PeIm 10/+6` (`ArMDE:1946`). The magus
  has Deficient Technique (Perdo), and "All totals, including Lab and Casting
  totals, including a particular Technique are halved" (`ArMDE:5915`
  `#### Deficient Technique`). His Perdo Imaginem Casting Score is
  `Pe 1 + Im 10 + Sta 0 = 11`, and `11 / 2 = 5.5`. The book prints 6.
- **Engine:** 5. `derived.rs::halve` is `div_euclid(2)` — it rounds **down**.
- **Wrong:** **(c) the book**, and the rulebook convicts itself in one sentence:
  "The rules for Ars Magica sometimes involve division. In most cases, a rule
  specifies whether you should round up or down, but if it does not, **round
  down**" (`ArMDE:547`). `ArMDE:5915` does not specify, so the default applies and
  the engine is right. Every other Jerbiton row reproduces exactly, which is what
  isolates the halving as the sole term in dispute.

## The Mercere (`ArMDE:1952-1998` `#### Mercere`)

Soak +2, the Fatigue row, the Size 0 wound bands, Encumbrance 0 (0), Dodging
Init +1 / Def +1 and all eight Art scores (Cr 6+3 = 9, Au 12+3 = 15) reproduce.
Abilities 315 + Arts 120 = 435 exactly. Error set empty, warning set empty. The
Casting Totals are MAG1's second instance, above, plus one row of their own.

### MAG7 — a Creo Auram total the Arts line cannot produce

- **Book:** Wings of the Soaring Wind prints `Cr(Re)Au 30/**+27**` (`ArMDE:1996`)
  where the four other Creo Auram rows on the same statblock read +26 (base) or
  +35 (within the Major Magical Focus on Weather). `Cr 9 + Au 15 + Sta +2 = 26`,
  and the focus would give 35; 27 is neither, and the Rego requisite contributes
  nothing to a Casting Total (`ArMDE:9089`).
- **Engine:** 29 base / 38 focused, i.e. the book's 26 / 35 plus MAG1's spurious
  +3 — so the engine reproduces the book's *relationship* between the rows exactly
  and disagrees only by the one constant, on this row as on the other four.
- **Wrong:** **(c) the book**, an isolated arithmetic slip. Sibling of MAG9
  (Verditius's Touch of Midas), which has the same shape on the same Form family.

## The Merinita (`ArMDE:2000-2048` `#### Merinita`)

Reproduces every printed figure except Faerie Lore, which is **P3** again — the
third template in this document to show Student of (Realm)'s missing +2, after the
Priest's Dominion Lore and the Witch's Magic Lore. Soak -1, Dodging Init -1 /
Def -1, Im 10+3 = 13, Encumbrance 0 (0) and all seven Casting Totals come back
exactly. Error set empty, warning set empty.

Abilities 315 + Arts 120 = 435, and it balances only because Faerie Magic 1 is
free — the House grant seeds it (`rules/core/houses.json` → `virtue.faerie_magic`
→ `ability_score_grant`), exactly as the House table describes it: "Faerie Magic
…, beginning score of 1 in Faerie Magic" (`ArMDE:2280`
`#### Hermetic Houses Summary`).

**Strong Faerie Blood's own grant is folded in silently and correctly.**
`virtue.strong_faerie_blood` carries `grants_selection` over `virtue.second_sight`
(`ArMDE:2014` takes it as "Strong Faerie Blood (Undine)"), so the magus holds
Second Sight without paying for it. The book prints no Second Sight score for her,
the fixture buys none, and the free floor costs nothing — so the grant is
invisible in the totals, which is the correct behaviour rather than an absence of
one.

## The Tremere (`ArMDE:2050-2101` `#### Tremere`)

**Reproduces every printed figure**, and is the single most valuable template in
this pass, because its Customization Notes hand over an explicit experience-point
audit trail no other template offers.

**The Elemental Magic worked example checks out to the point.** `ArMDE:2099-2101`
states the bought scores — "Initially, he was assigned scores of Aq 3, Au 6, Ig 6,
and Te 6" — and then the redistribution: "Aquam received an additional 33
experience points (from Auram, Ignem and Terram), for a final total of 39 and a
score of 8 (3), and each of the other Arts received an additional 25 experience
points, for a final total of 46 and a score of 9 (1)." The engine's
`effective/art.rs::elemental_form_bonus` reconstructs each pooled Form's table-XP
from its bought score and gives every other pooled Form half of it rounded up
(`ArMDE:3733` `#### Elemental Magic`, whose own worked example is 21 → 11):

- Aquam `6 + ceil(21/2) x 3 = 6 + 33 = 39` → score **8**, 3 over — the book's
  "8 (3)".
- Auram `21 + ceil(6/2) + ceil(21/2) + ceil(21/2) = 21 + 25 = 46` → score **9**,
  1 over — the book's "9 (1)". Ignem and Terram likewise.

The fixture stores the bought 3 / 6 / 6 / 6 and the test asserts the derived
8 / 9 / 9 / 9, so both halves are pinned. The experience cost is the bought scores
only — `6 + 21 + 21 + 21 = 69` — which is what makes `Abilities 350 + Arts 145 =
495` land on `435 + Skilled Parens 60` exactly, and the 8 spells on
`120 + 30 = 150`. Both Skilled Parens effects, `general_xp` and `spell_levels`,
are therefore confirmed in the same template.

Every Casting Total is +16 = `Technique 5 + Form 9 + Sta +2`, which is a second,
independent confirmation of the elemental scores: the Forms feed the totals at 9,
not at the bought 6.

### MAG10 — the Tremere House grant does not say what its Magical Focus is on

- **Book:** "Minor Magical Focus(certamen)\*" (`ArMDE:2064`), the asterisk marking
  it as the free House Virtue; the House table says the same —
  "Minor Magical Focus (certamen)." (`ArMDE:2281`
  `#### Hermetic Houses Summary`).
- **Engine:** `rules/core/houses.json` grants `house.tremere` a
  `{ "kind": "fixed", "item": "virtue.minor_magical_focus" }` with **no `params`**,
  although `virtue.minor_magical_focus` declares a required `focus` parameter and
  `grant.rs`'s own documentation uses this very case as its example of a fixed
  grant that carries one (`params: { "focus": "certamen" }`). The focus still
  registers — the magus gets his within-focus Casting Totals — but it is a focus on
  nothing, and `validation/selections.rs::validate_magical_focus` only counts foci,
  so nothing complains.
- **Wrong:** **(a) the rules data**, a one-line fix in `houses.json`. Low impact
  (the parameter is display-only; MAG8 explains why nothing computes with it) but
  it is a printed rule the data simply does not carry.

## The Tytalus (`ArMDE:2103-2149` `#### Tytalus`)

**Reproduces every printed figure**, Confidence included. Int +4 is a bought +3
plus Great Intelligence; Soak +2; Dodging Init +1 / Def +4 (Brawl 3, its
"grappling" specialty correctly **not** applied to a dodge); Encumbrance 0 (0);
Cr 5 / In 5 / Re 5 / Me 9; and Casting Totals +16 / +16 / +11 / +16 / +16.
Error set empty, warning set empty.

**Confidence Score: 2 (5)** (`ArMDE:2115`) against the magus profile's 1 (3), and
the engine derives it: `virtue.self_confident` — free from the House
(`ArMDE:2117`) — carries `confidence_bonus` with `score: 1, points: 2`, and
`effective/gift_confidence.rs::confidence` sums it onto the profile's base. This is
the only template in the book with a non-default Confidence, so it is the only
place `ConfidenceBonus` is exercised against a printed figure.

**Weak Parens is confirmed on both of its effects at once**: Abilities 285 +
Arts 90 = 375 = `435 - 60`, and the five spells total 90 = `120 - 30`. Together
with the Tremere's Skilled Parens, the pair pins `Effect::GeneralXp` and
`Effect::SpellLevels` in both directions.

## The Verditius (`ArMDE:2151-2199` `#### Verditius`)

The only Size -2 template, and the best stress test of per-instance Ability
parameters in the book. Str -3 / Sta +1 (bought -2 / +2, shifted by Dwarf),
Soak +1, the Size -2 wound bands 1-3 / 4-6 / 7-9 / 10-12, Dodging Init +0 / Def +0,
Encumbrance 0 (0), Te 12+3 = 15 and six of seven Casting Totals reproduce. Error
set empty, warning set empty.

**Two instances of one Ability, each with its own Affinity and its own Puissant,
and the engine keeps them apart.** Craft (metalsmith) and Craft (stonemason) are
both `ability.craft` distinguished by the `craft` parameter, and each carries
`virtue.affinity_ability` and `virtue.puissant_ability` naming that instance. The
bonus lands on the named instance and on no other, and the Affinity charges that
instance's experience and no other — which is exactly what
`effective/ability.rs::ability_bonus` and `effective/xp.rs::ability_affinity`
promise, tested here against a printed statblock rather than a synthetic fixture.

### MAG9 — a Creo Terram total the Arts line cannot produce

- **Book:** Touch of Midas prints `CrTe 20/**+25**` (`ArMDE:2192`) where its two
  Creo Terram neighbours, Seal the Earth and Wall of Protecting Stone, both read
  +23 (`ArMDE:2191`, `ArMDE:2193`). `Cr 7 + Te 15 + Sta +1 = 23`. This magus holds
  no Magical Focus, so unlike the Ex Miscellanea and the Mercere there is not even
  a second reading to reach a higher number by.
- **Engine:** 23, like its neighbours.
- **Wrong:** **(c) the book**, an isolated slip; sibling of MAG7.

### MAG13 — Affinity overcharges Craft (stonemason) by one point

This is **S2** again — `crates/arm-rules` `charged_cost` computes
`ceil(T x den / num)` where the smallest charge satisfying `ceil(c x num / den) >= T`
is `floor(den x (T-1) / num) + 1` — recorded here only because this pass locates
exactly where it bites and where it does not.

- **Book:** Craft (stonemason) 4 costs 50 table experience points; with an
  Affinity the charge is the smallest `c` with `ceil(3c/2) >= 50`, i.e.
  `c = 33` (`ceil(49.5) = 50`). The template's total is then
  `Abilities 313 + Arts 122 = 435`, exactly the age formula's grant.
- **Engine:** `ceil(50 x 2 / 3) = 34`, so the fixture needs `xp_pool: 436`.
- **Wrong:** **(a) the engine**, already filed as open-todo row 47 / V/F-audit
  F-547 and as S2 above; **not re-filed here.** What this pass adds is the
  boundary: the defect trips whenever `(T x den) mod num` lands in `(0, den)`,
  which for the 3/2 Affinity means `T ≡ 2 (mod 3)`. Ability table costs hit that
  at scores 1, 4, 7, 10, 13, 16 and 19; **Art** table costs are triangular numbers
  and are never `≡ 2 (mod 3)`, so no Affinity-bought Art in any of these
  twenty-three templates is affected. Craft (stonemason) 4 is the single instance
  in the whole book-template corpus.

### MAG14 — the statblock prints Puissant Ability's bonus as +3

- **Book:** "Craft (metalsmith) 5+3" and "Craft (stonemason) 4+3" (`ArMDE:2180`).
  Puissant **Ability** adds 2 — "add 2 to its value whenever you use it"
  (`ArMDE:4816` `#### Puissant Ability`); 3 is Puissant **Art**'s figure
  (`ArMDE:4820` `#### Puissant Art`), and this magus's Te 12+3 on the line below
  uses it correctly.
- **Engine:** 7 and 6. `virtue.puissant_ability` carries `ability_bonus` amount 2.
- **Wrong:** **(c) the book.** Four other templates print the +2 correctly — the
  Knight's "Single Weapon 5+2" (`ArMDE:1480`), Bonisagus's "Magic Theory 4+2"
  (`ArMDE:1683`), Criamon's "Enigmatic Wisdom 3+2" (`ArMDE:1731`) and Jerbiton's
  "Music 4+2" (`ArMDE:1928`) — so this statblock is alone, and its own Terram row
  shows it knows the difference.

## Source formatting in the Magus Templates section

Five more instances of the shape already recorded as F2, F3, H1 and K4 — defects
in `rules/source/en/` with no mechanical consequence, all **(c) the book**:

- **`ArMDE:1766`** runs "Giant Blood\* Major Magical Focus (stone)" with no
  separator at all between two Virtues, so a naive split reads one Virtue named
  "Giant Blood Major Magical Focus". This is the worst of the five, because the V/F
  line is where the 10/10 budget is read from.
- **`ArMDE:1928`** ends "Code of Hermes 1 (dealing with mundanes)**.** Etiquette 2
  (nobility)" — a period inside a comma-separated list, exactly F2 — and later runs
  "Music 4+2 (singing) Native Language 5 (noble style)" with no separator, exactly
  the `ArMDE:1766` shape.
- **`ArMDE:1954`** prints "Int +2, Per 0 Pre 0, Com -1" and **`ArMDE:2002`** "Int
  +3, Per +1 Pre +1, Com +2" — a missing comma between two Characteristics in each,
  which matters because the Characteristic line is the point-buy audit.
- **`ArMDE:1704`** prints "Sta + 2" and "Qik + 1" and **`ArMDE:2153`** "Sta + 1",
  "Dex + 1" — a space between the sign and the digit, where every other template
  writes "+2".
- **En dash for minus** continues into this section: `ArMDE:1605` "Com –1",
  `ArMDE:1729` "Incapacitated (16–20)", `ArMDE:1774` "Init –2", `ArMDE:1805`,
  `ArMDE:1828`, `ArMDE:1830`, `ArMDE:2016`, `ArMDE:2021`, `ArMDE:2105`,
  `ArMDE:2119` all use U+2013 where the surrounding lines use ASCII.

## Transcription notes (not discrepancies)

These are places where the fixture had to supply something the book leaves
implicit. They are **(b)-adjacent** — deliberate transcription choices, recorded
so the fixtures can be audited.

- **Unnamed Area Lore.** `(Area) Lore` is parameterized by the area, and five of
  the six templates name only the *specialty* ("streams", "game trails",
  "warriors", "taverns", "brewers"). The fixtures use the placeholder
  `"Local area"`. The Grizzled Veteran is the exception and names its areas
  itself, "Area A" and "Area B" (`ArMDE:1257`), which the fixture copies
  verbatim.
- **"Native Language".** There is no `ability.native_language`; the catalogue
  models it as `ability.living_language` parameterized by the language
  (`rules/core/life_stages.json` points `native_language_ability` at exactly
  that). The templates never name the tongue, so the fixtures use the
  placeholder `"Native language"`.
- **"Order of Hermes Lore"** maps to `ability.organization_lore` with the
  parameter "Order of Hermes" — the book does name this one.
- **"Great Weapons"** (plural, `ArMDE:1221`) is the singular
  `ability.great_weapon` everywhere else in the book and in the catalogue.
- **Equipment with no catalogue id:** "pack containing gear to care for weapons
  and armor and establish camps when traveling" (all four axe-and-shield
  templates), and "arrows, survival kit" (the Hunter, `ArMDE:1292`). These are
  left out of the fixtures. They carry no Load in the book's Armor and Weapons
  tables (`ArMDE:17107`), and every Encumbrance figure reproduces without
  them — which is the book's own confirmation that they weigh nothing
  mechanically.
- **Fist and Kick as equipment.** The engine derives a combat row only from an
  equipped weapon, so the templates that print a Fist or Kick row carry
  `weapon.fist` / `weapon.kick` (Load 0) in the fixture.
- **"Bows 1 (2)"** (`ArMDE:1326`). The save format stores whole Ability scores
  and banks the remainder in the pool rather than against a single Ability
  (`types.rs::AbilityScore`), so the 2 points toward the next level cannot be
  attached to Bows. See S2 for where they went.
- **"Church Lore"** (the Female Scholar and the Priest) maps to
  `ability.organization_lore` with the parameter "The Church", the same shape as
  the grogs' "Order of Hermes Lore". There is no `ability.church_lore`.
- **"Latin"** maps to `ability.dead_language` parameterized "Latin"; **"Arabic"**
  (the Female Scholar) to `ability.living_language` parameterized "Arabic", since
  Arabic is spoken and Latin is not. This matters mechanically: Latin is Academic
  and therefore gated, Arabic is General and is not.
- **"Divine Lore"** (the Witch, `ArMDE:1591`) is `ability.dominion_lore` — the
  catalogue's name for it, and the one the Priest's own "Dominion Lore" uses.
- **"Theology"** maps to `ability.theology_christian`; the catalogue splits the
  Ability by faith and both companions who take it are Christian clergy.
- **Dodging as equipment.** Three companions print a Dodging row rather than a
  weapon; `weapon.dodge` (Load 0, no Attack, no Damage) is the catalogue entry for
  it, which is why those rows come back with Attack and Damage absent rather than
  zero.
- **Unused entitlements are not errors.** The Priest's Lesser Immunity, Relic,
  Social Contacts and Inspirational, and the Knight's Relic, are all narrative:
  they change nothing the statblock prints, and the fixtures carry them because
  the V/F budget does not balance without them.
- **"Living Language 5" vs "Native Language 5".** Both label the same thing — the
  75-XP mother tongue every template buys at 5 — and the book uses them
  interchangeably: the Bjornaer (`ArMDE:1632`) and the Flambeau (`ArMDE:1832`) say
  "Living Language", the other ten magi say "Native Language". Both map to
  `ability.living_language` with the placeholder parameter `"Native language"`, so
  the fixtures are identical and only the printed label differs. **(c) the book**,
  inconsistent labelling, no mechanical consequence.
- **"Second Area Lore 1 (forests)".** The Bjornaer buys two `(Area) Lore`
  instances (`ArMDE:1632`), the first named only by its specialty and the second
  not named at all. The fixture uses `"Local area"` and `"Second area"`. The 435
  XP total needs both, at 5 points each.
- **"Hermes Lore 1 (House Flambeau)"** (`ArMDE:1832`) is the same Ability the other
  eleven magi call "Order of Hermes Lore" — `ability.organization_lore`
  parameterized "Order of Hermes".
- **Wizardly robes.** Every magus's Equipment line is "Wizardly robes"
  (`ArMDE:1638` and eleven more), which has no id in `rules/core/equipment.json`
  and no Load in the book's own Armor table. Every template prints "Encumbrance:
  0 (0)" and every fixture reproduces it, which is the book's confirmation that
  robes weigh nothing mechanically. The fixtures carry only `weapon.dodge`
  (Load 0), which is what produces the Dodging Combat row.
- **General-level spells.** Three of the Criamon's spells — Wind of Mundane
  Silence, Circular Ward Against Demons and Unravelling the Fabric of (Form) —
  are **General** spells, so the catalogue carries no level and the learned level
  lives on the selection (`types.rs::SpellSelection`). The book prints 20 for each
  (`ArMDE:1746-1748`), and the fixture records it. Unravelling the Fabric of (Form)
  additionally takes the `form` parameter, `art.imaginem`, which is how the book's
  "Unraveling the Fabric of Imaginem" is expressed.
- **Elemental Magic stores the bought score, not the derived one.** The Tremere's
  fixture records Aq 3 / Au 6 / Ig 6 / Te 6 — the scores `ArMDE:2099` says he was
  assigned — and the engine derives the printed 8 / 9 / 9 / 9. Recording the
  printed scores instead would charge him 36+45+45+45 = 171 XP for the four Forms
  rather than 69, and nothing would balance.
- **En dash for minus in the template section.** `ArMDE:1426` prints
  "Trusting –2" and `ArMDE:1552` "Incapacitated (16–20)" with U+2013, where the
  surrounding lines use the ASCII hyphen — as do `ArMDE:1300` and `ArMDE:1354` in
  the grog section. **(c) the book**, source formatting, no mechanical
  consequence; recorded because this repo treats the hyphen/minus distinction as
  load-bearing on its own output.

## Darius of Flambeau — the worked example (`ArMDE:2285-2615`)

The book's twenty-fourth example character, and the only **worked** one: the
Detailed Character Creation chapter builds him step by step, showing its
arithmetic at each stage instead of printing finished totals. That makes him a
far better oracle than the 23 statblocks, because every figure can be checked
against a stated intermediate rather than inferred from a sum.

He exists in two states. **At Gauntlet** (age 25) he is fully specified by
`ArMDE:2285-2449` and is built as a fixture here. **At 87** (`ArMDE:2538-2615`)
he is the same character after 62 years of advancement, aging, lab work and
Twilight; that state is out of scope — see D2 below.

### D1 — At Gauntlet: reproduces exactly, with nothing to report

`darius_of_flambeau_at_gauntlet_matches_the_book` validates with an **empty error
set and an empty warning set**, and every stage of the book's narrated build
comes out to the point:

| Stage | Source | Book | Engine |
|---|---|---|---|
| Flaws | `ArMDE:2336` | "ten points of Flaws" — Driven + Enemies + Blatant Gift (3 x 3) + Disfigured (1) | 10 ✓ |
| Virtues | `ArMDE:2338` | 5 Minor, then Flawless Magic (3), then "two Minor Virtues" | 10 ✓ |
| Characteristics | `ArMDE:2358-2360` | Int +3 (6), Per +1 (1), Pre -3 (gain 6), Com -1 (gain 1), Qik +2 (3), Str +2 (3), Dex +1 (1) | **exactly 7** ✓ |
| Early childhood | `ArMDE:2400` | German 5 = 75; Bavaria Lore 2 + Awareness 2 + Folk Ken 2 = 45 | ✓ |
| Ages 5-10 | `ArMDE:2402` | 70 spent, "leaves him with 5", spent on Order of Hermes Lore 1 = 75 | ✓ |
| Apprenticeship Abilities | `ArMDE:2441` | eleven Abilities = 180, "He has 60 experience points left" of 240 | ✓ |
| Apprenticeship Arts | `ArMDE:2443` | Perdo 37 -> 56 with Affinity = Pe 10 (1); Creo 5 = 15; Corpus 2 = 3 | ✓ |
| Final spend | `ArMDE:2449` | "his last 5 exp on Parma Magica 1" | ✓ |
| Spells | `ArMDE:2445` | seven Perdo spells, 15+15+20+20+20+15+15 | **120** ✓ |
| Free Abilities | `ArMDE:2398` | "Premonitions 1, Second Sight 1" granted by Virtues, unbought | ✓ |

The XP total is **exact, not merely sufficient**: probed in both directions, 436
yields `general_xp_unspent` and 434 yields `not_enough_xp`.

Two things worth noting even though neither is a disagreement. His **Affinity with
Perdo** does *not* trip the `charged_cost` off-by-one (S2 / open-todo row 47):
Perdo 10 costs `T = 55` on the Art table, `55 mod 3 = 1`, and Art costs are
triangular numbers which are never `2 mod 3` — the same proof that clears Arts
generally. And his **Reputation** is the second witness to MAG11: Hermetic
Prestige grants level **4** (`ArMDE:4071-4073` `#### Hermetic Prestige`), while
the worked example says "it has a level of 3" (`ArMDE:2518`), exactly as the
Guernicus template prints 3 (`ArMDE:1869`). The fixture records the Virtue's 4.

### D2 — At 87: out of scope, and one genuine arithmetic error on the way there

The age-87 sheet cannot be built as a starting character and is not attempted.
It is reached by `ArMDE:2484-2492`, which spends 240 points across ages 26-33,
then a year of lab work (talisman, Longevity Ritual), then year-by-year
advancement to 87 with aging and a Twilight — none of which the creation path
models. Its printed Soak, Combat, Fatigue and Wound rows (`ArMDE:2556-2565`)
therefore describe a state this exercise does not cover, and the test
deliberately asserts none of them.

**The post-Gauntlet spend contains a real error**, and it is the only arithmetic
defect found anywhere in Darius. `ArMDE:2486` reads:

> ...spending **30 exp to raise Creo to 10**...

He leaves apprenticeship with Creo 5, bought for 15 XP (`ArMDE:2443`). On the Art
advancement table (`ArMDE:2404-2427`, reproduced in `rules/core/arts.json`) Creo
10 costs 55 and Creo 9 costs 45:

- `15 + 30 = 45` -> **Creo 9**, not 10
- Creo 10 costs `55 - 15 = ` **40 exp**, not 30

Both cannot be right. He holds Affinity with **Perdo**, not Creo, so no affinity
discount applies. **Wrong: (c) the book.** Note the year's total only balances
*because* of the bad figure — `25 + 5 + 88 + 30 + 60 + 2 + 30 = 240` — so
correcting the cost to 40 would push the spend to 250 against a 240 budget, which
suggests the intended fix is "to 9" rather than "40 exp". That is the author's
call, not ours. The age-87 sheet does not constrain it either way: `Cr 10`
(`ArMDE:2569`) is reached after fifty further years of advancement.

Everything else in the post-Gauntlet passage checks out — Parma Magica 1 to 3 is
`30 - 5 = 25` ✓, Corpus 2 to 13 is `91 - 3 = 88` ✓, six Arts to 4 is `6 x 10 = 60`
✓, and Rego and Intellego at 1 exp each ✓.

### D3 — `Puissant Art Perdo)` has an unmatched parenthesis

`ArMDE:2548` prints "Puissant Art Perdo) (free Virtue)". **(c) the book**,
formatting. Worth noting that this is the one place the book spells out a free
House Virtue in words rather than with the asterisk the templates use
(`ArMDE:1601`), which is why the annotation cannot be relied on and the grant has
to be read from `rules/core/houses.json`.

## Summary

| # | Template | Book says | Engine says | Wrong |
|---|---|---|---|---|
| B1 | Berserker | Berserk grants Martial access (`ArMDE:3502`) | `ability_category_requires_virtue` x2 | (a) rules data |
| B2 | Berserker | Soak +9, Pole Axe +13/+7, Kick +6/+4 (`ArMDE:1212-1215`) | Soak 11, +15/+5, +8/+2 | (a) rules data |
| B3 | Berserker | 2 Personality Flaws (`ArMDE:1205`) | `too_many_personality_flaws` (max 1, `ArMDE:2827`) | (c) the book |
| B4 | Berserker | no Size line (`ArMDE:1197-1199`) | Size +1, wound bands 1-6 … (matches `ArMDE:1219`) | (c) the book |
| V3 | Grizzled Veteran | Decrepitude 1 (2), no roll log (`ArMDE:1237`) | `aging_rolls_pending` | none |
| H1 | Hunter | V/F and Personality Traits merged on one line (`ArMDE:1277`) | n/a | (c) the book (source formatting) |
| S2 | Specialist | Affinity: Single Weapon 7 costs 93 XP (`ArMDE:3374`) | costs 94 XP | (a) the engine |
| T1 | Tough Guy | 2 Personality Flaws (`ArMDE:1382`) | `too_many_personality_flaws` (max 1, `ArMDE:2827`) | (c) the book |
| F1 | Female Scholar | Clerk grants Academic access (`ArMDE:3573` `#### Clerk`) | `ability_category_requires_virtue` x6 | (a) rules data |
| F2 | Female Scholar | period for comma in the Abilities list (`ArMDE:1439`) | n/a | (c) the book (source formatting) |
| F3 | Female Scholar | one semicolon for four V/F group breaks (`ArMDE:1424` vs `ArMDE:1163` `### Format`) | n/a | (c) the book (source formatting) |
| K1 | Knight | Knight grants Martial access (`ArMDE:4197` `#### Knight`) | `ability_category_requires_virtue` x2 | (a) rules data |
| K2 | Knight | Encumbrance 2 (3) (`ArMDE:1484`) contradicts its own Equipment line (Load 11 → Burden 4) and the whole-loadout method four other templates prove | **RESOLVED 2026-09-23** — `encumbrance` now counts only `equipped` gear; reproduces the printed figures exactly, as do all three on-foot Initiatives | **(c) the book** — great sword's Load 2 omitted; engine now right by a sounder route |
| K5 | Knight | 5 Combat rows (`ArMDE:1468-1472`) *and* a wielded-set Encumbrance | `equipped` gates both Combat rows and Load, so the stowed great sword cannot do one without the other; its 2 rows are dropped | (d) model gap, exposed by fixing K2 |
| K3 | Knight | 2 mounted Combat rows, Ride up to +3 (`ArMDE:1468`, `ArMDE:1470`, `ArMDE:16839` `#### Mounted Combat`) | no mounted variant emitted | none (capability gap) |
| K4 | Knight | `Etiquette (noble) 3` reverses the declared format (`ArMDE:1480` vs `ArMDE:1177`) | n/a | (c) the book (source formatting) |
| P1 | Priest | Priest grants Academic access (`ArMDE:4804` `#### Priest`) | `ability_category_requires_virtue` x4 | (a) rules data |
| P2 | Priest | Well-Traveled grants 50 restricted XP (`ArMDE:5241` `#### Well-Traveled`) | no pool; all 590 must come from the general one | (a) rules data |
| P3 | Priest, Witch | Student of (Realm) gives +2 on its Lore (`ArMDE:5054` `#### Student of (Realm)`) | Dominion Lore 3, Magic Lore 3 (no bonus) | (a) rules data |
| P4 | Priest, Witch | Student of (Realm) opens **that** Lore only (`ArMDE:5054`) | authorizes all four Lores | (a) rules data (permissive) |
| W1 | Witch | Educated grants Academic access *and* 50 XP (`ArMDE:3713` `#### Educated`) | XP only → `ability_category_requires_virtue` on Medicine | (a) rules data |
| W2 | Witch | Wise One: Arcane **or** Academic, not both (`ArMDE:5259` `#### Wise One`) | not represented | none (capability gap) |
| MAG1 | Bjornaer, Mercere | Casting Totals with no conditional bonus applied (`ArMDE:1643-1650`, `ArMDE:1992-1996`) | +3 on every cell: Ways of the (Land), Cyclic Magic (both) and Special Circumstances are all `scope: all` | (a) rules data |
| MAG2 | Criamon | 7 spells, 120 levels, one of them "Piercing the Magical Veil" (`ArMDE:1745`) | the spell has no entry in the book (`ArMDE:15709`) and none in the catalogue → `spell_levels_unspent` at 100/120 | (d) the book leaves it undefined |
| MAG3 | Criamon | `Ag 0` in the Arts line (`ArMDE:1733`) | no such Art; the missing slot is Aq, Aquam (`ArMDE:8843`) | (c) the book |
| MAG4 | Ex Miscellanea | two Re(Mu)Te 15 rows at +27 and +23 (`ArMDE:1798-1799`) | 24 base / 29 focused for both; neither printed figure is reachable from its own Arts line | (c) the book |
| MAG5 | Ex Miscellanea | a Grappling Combat row (`ArMDE:1774`) | no grapple entry in `rules/core/equipment.json`; only the Dodging row is emitted | (d) capability gap |
| MAG6 | Jerbiton | PeIm +6 under Deficient Technique (`ArMDE:1946`) — 11 halved, rounded up | 5, rounding down per `ArMDE:547` ("if it does not [say], round down") | (c) the book |
| MAG7 | Mercere | Cr(Re)Au +27 (`ArMDE:1996`) where its four CrAu neighbours read +26 / +35 | 26 / 35 on the same footing as the others (29 / 38 with MAG1's +3) | (c) the book |
| MAG8 | Flambeau, Ex Miscellanea, Mercere, Tremere | one Casting Total per spell, focused or not (`ArMDE:4399-4422` `#### Major Magical Focus`) | both figures reported; a focus is free text and nothing relates it to a spell | (d) capability gap |
| MAG9 | Verditius | CrTe +25 for Touch of Midas (`ArMDE:2192`) where its two CrTe neighbours read +23 | 23; this magus holds no focus, so no second reading exists | (c) the book |
| MAG10 | Tremere | "Minor Magical Focus(certamen)" (`ArMDE:2064`, `ArMDE:2281`) | `houses.json` grants the Virtue with **no `focus` param**; a focus on nothing, and `validate_magical_focus` only counts foci | (a) rules data |
| MAG11 | Guernicus | Hermetic Prestige gives Reputation 4 (`ArMDE:4073`), but two statblocks print 3 (`ArMDE:1869`, `ArMDE:2518`) | `grants_reputation` score 4, following the Virtue's own entry | (c) the book |
| MAG12 | Guernicus | "In 12+3 (5)" (`ArMDE:1881`) — 5 XP banked toward the next score | `types.rs::ArtScore` stores whole scores only; 3 of his 435 XP are unspendable, fixture uses 432 | (d) storage-model gap |
| MAG13 | Verditius | Affinity-bought Craft (stonemason) 4 costs 33 XP (`ArMDE:3374`) | costs 34; **S2 again**, and the only instance in all 23 templates | (a) the engine — already filed, not re-filed |
| MAG14 | Verditius | "Craft (metalsmith) 5+3", "Craft (stonemason) 4+3" (`ArMDE:2180`) | +2, per `ArMDE:4816`; +3 is Puissant *Art* (`ArMDE:4820`), and four other templates print +2 | (c) the book |
| — | Magus section | five source-formatting defects (`ArMDE:1766`, `ArMDE:1928`, `ArMDE:1954`, `ArMDE:2002`, `ArMDE:1704`/`ArMDE:2153`) | n/a | (c) the book (source formatting) |
| D1 | Darius, at Gauntlet | every narrated stage — 10/10 V/F, 7 Characteristic points, 435 XP, 120 spell levels (`ArMDE:2336-2449`) | **reproduces exactly**, empty error and warning sets; 435 proved exact in both directions | none |
| D2 | Darius, post-Gauntlet | "30 exp to raise Creo to 10" (`ArMDE:2486`) | Creo 5 costs 15, Creo 10 costs 55, so the step is **40 exp**; 30 exp buys **Creo 9** | **(c) the book** |
| D3 | Darius | "Puissant Art Perdo)" (`ArMDE:2548`) | n/a | (c) the book (formatting) |

Nothing in the magus pass required a change to production code or rules data;
every entry is recorded, not fixed. (The one resolved entry, K2, was decided and
implemented separately — see its section.)

**Where the companion findings point.** Five of the eleven companion-pass entries
(F1, K1, P1, W1, and B1 from the grog pass) are one defect: a Social Status or
education Virtue whose "you may take *category* Abilities" clause was dropped when
`rules/core/virtues_flaws.json` was extracted. Whoever fixes it should sweep the
catalogue for the clause rather than patch the five templates happen to name —
`virtue.clerk`, `virtue.knight`, `virtue.priest` and `virtue.wise_one` all carry
*no effects whatsoever*, which is the signature to search for among
`social_status` entries. P2 and P3 are the same extraction missing an XP grant
and a bonus; P4 is it over-reaching. None of it is reachable by editing code.

**Where the magus findings point.** The magi add exactly **one** new defect that
produces a wrong number on a real character: **MAG1**, a Virtue whose rulebook
text states a *condition* encoded as an unconditional `casting_total_mod`. It is
the same shape as the grog pass's B2 (Berserk's "while berserk" modifiers), which
makes it the second sighting of a general problem rather than two local bugs: the
effect vocabulary has no way to mark a modifier conditional, so every such Virtue
is either over-applied or dropped. Four Virtues are already implicated —
`virtue.ways_of_the_land`, `virtue.cyclic_magic_positive`,
`flaw.cyclic_magic_negative`, `virtue.special_circumstances` — plus `virtue.berserk`
from B2, and the right fix is a conditional marker on the effect (surfaced to the
player as a toggle), not five data patches.

Everything else the magi turned up is either a narrow data omission (MAG10's
missing `focus` parameter, MAG11's Reputation level), a gap the rules themselves
leave open (MAG2, MAG5, MAG8, MAG12), or the book disagreeing with its own
arithmetic (MAG3, MAG4, MAG6, MAG7, MAG9, MAG14). **Six (c) entries out of twelve
templates is the highest rate in this document**, and the reason is simply that
magi print far more derived numbers than grogs and companions do: 80 Casting
Totals and 12 full fifteen-Art lines on top of everything a companion prints, each
one an independent chance for the statblock to disagree with the rules that
generated it.
