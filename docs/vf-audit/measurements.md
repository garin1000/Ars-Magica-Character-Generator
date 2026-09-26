# Phase 2 — M0 measurements

Slice **M0** of `phase-2-plan.md` § 3 ("Phase 0 — measure"): the **counts** owed by
`corrections.md` § 8. Measured 2026-09-26 at `cffdbd7` (clean tree). Every figure
carries the command that produced it (run from the repo root unless it says
`cd rules/core`). **Ids are durable; counts are snapshots** — re-run before acting.

No rulebook line numbers are written here on purpose (this file sits inside the
`rulebook_citations.rs` guard).

**Moved:** S1's before-offender set needs code to compute and is measured as the
first step of **S1**, not here.

---

## § 8 row 2 — D10: entries riding the unlimited multiplicity default

Default is `max_total = u8::MAX` (`types.rs::default_max_total`), serialized as an
absent key; `max_per_target` defaults to 1 (`types.rs::default_max_per_target`).

```sh
jq -r '.[] | select(.parameters != null and .max_total == null) | .id' rules/core/virtues_flaws.json
```

**42** (audit figure: ~42 — agrees).

`flaw.anchored_to_the_land` `flaw.bound_to_realm` `flaw.bound_to_role_role`
`flaw.careless_with_ability` `flaw.deficient_form` `flaw.deficient_technique`
`flaw.false_power_minor` `flaw.flawed_parma_magica` `flaw.form_monstrosity`
`flaw.hunger_for_form_magic` `flaw.limited_magic_resistance`
`flaw.magical_being_companion` `flaw.necessary_realm_aura_for_ability`
`flaw.poor_characteristic` `flaw.realm_stigmatic` `flaw.restricted_power`
`flaw.servant_of_the_land` `flaw.slow_power` `virtue.academic_concentration_subject`
`virtue.affinity_ability` `virtue.alluring_to_beings` `virtue.aptitude_for_sin`
`virtue.cautious_with_ability` `virtue.deft_form` `virtue.doctor_in_faculty`
`virtue.enchanting_ability` `virtue.extractor_of_form_vis` `virtue.folk_magic`
`virtue.great_characteristic` `virtue.imbued_with_the_spirit_of_form`
`virtue.land_regio_network` `virtue.learn_ability_from_mistakes`
`virtue.major_magical_focus` `virtue.master_of_form_creatures`
`virtue.minor_magical_focus` `virtue.mythic_blood` `virtue.perfect_eye_for_commodity`
`virtue.puissant_ability` `virtue.student_of_realm` `virtue.variable_power`
`virtue.voice_of_the_land` `virtue.ways_of_the_land`

For contrast, the 9 parameterized entries that *do* state `max_total`:
`flaw.curse_of_slander` (1), `flaw.false_power` (1), `flaw.fish_out_of_water_terrain` (1),
`flaw.offensive_to_beings` (1), `flaw.unbearable_to_beings` (1), `virtue.affinity_art` (2),
`virtue.inoffensive_to_beings` (1), `virtue.puissant_art` (2), `virtue.sufi` (1).

```sh
jq -c '.[] | select(.max_total != null) | {id, max_total, p: (.parameters != null)}' rules/core/virtues_flaws.json
```

### Non-default `max_per_target` with no `parameters` (corrections § 3.7 sweep input)

```sh
jq -c '.[] | select(.max_per_target != null and .parameters == null) | {id, max_per_target}' rules/core/virtues_flaws.json
```

**22.** `max_per_target: 2` — `flaw.weak_characteristics`, `virtue.quiet_magic`.
`max_per_target: 255` (20) — `flaw.deteriorating_power` `flaw.vulnerable_casting`
`flaw.vulnerable_magic` `virtue.demonic_might` `virtue.demonic_powers`
`virtue.focus_power` `virtue.greater_immunity` `virtue.greater_power`
`virtue.improved_characteristics` `virtue.lesser_power` `virtue.magic_items`
`virtue.mastered_spells` `virtue.mentored_by_demons` `virtue.minor_enchantments`
`virtue.personal_power` `virtue.ritual_power` `virtue.social_contacts`
`virtue.special_circumstances` `virtue.strong_angelic_heritage` `virtue.withstand_casting`.

(The other two non-default carriers are parameterized: `flaw.poor_characteristic`
and `virtue.great_characteristic`, both 2. Census: 631 default, 4 at 2, 20 at 255.)

---

## § 8 row 3 — D20: the 19 surfaced-only entries

Transcribed from gitignored `tmp/q136-number-check.md`, **re-derived** against
today's data and engine. The two literal-`amount: 0` arms in
`derived.rs::in_play_mods` are unchanged (`MagicResistanceMod` for four kinds,
`SpecialCastingMod` for eight):

```sh
grep -rn "amount: 0" crates/arm-rules/src/
jq -r '.[] | select(any(.effects[]?; (.type=="special_casting_mod" and (.kind|IN("diedne","faerie_raised","life_linked_spontaneous","spell_improvisation","mercurian","life_boost","circumstantial","doubled_aura_penalty"))) or (.type=="magic_resistance_mod" and (.kind|IN("aura_bonus","susceptible_faerie","susceptible_infernal","conditional_penetration_waiver"))))) | .id' rules/core/virtues_flaws.json
```

**19 — all ids still exist, set identical to the scratch file.**

| id | family / kind | Q-136 verdict |
|---|---|---|
| `virtue.commanding_aura` | MR / `aura_bonus` | **number missing** |
| `virtue.life_boost` | casting / `life_boost` | **number missing** |
| `virtue.leper_magus` | casting / `life_boost` | **number missing** |
| `virtue.special_circumstances` | MR / `aura_bonus` | **number missing** (summary quotes it) |
| `flaw.corrupted_spells` | casting / `circumstantial` | **number missing** |
| `flaw.deleterious_circumstances` | casting / `circumstantial` | right |
| `flaw.disjointed_magic` | casting / `circumstantial` | right |
| `flaw.environmental_magic_condition` | casting / `circumstantial` | right |
| `flaw.short_ranged_magic` | casting / `circumstantial` | right |
| `flaw.susceptibility_to_divine_power` | casting / `doubled_aura_penalty` | right |
| `flaw.the_constant_expression` | casting / `circumstantial` | right (D5 summary gap) |
| `virtue.diedne_magic` | casting / `diedne` | right (D5 summary gap) |
| `virtue.faerie_raised_magic` | casting / `faerie_raised` | right (D5 summary gap) |
| `virtue.life_linked_spontaneous_magic` | casting / `life_linked_spontaneous` | right (D5 summary gap) |
| `virtue.mercurian_magic` | casting / `mercurian` | right (D5 summary gap) |
| `virtue.spell_improvisation` | casting / `spell_improvisation` | right |
| `flaw.susceptibility_to_faerie_power` | MR / `susceptible_faerie` | right |
| `flaw.susceptibility_to_infernal_power` | MR / `susceptible_infernal` | right |
| `flaw.weak_magic_resistance` | MR / `conditional_penetration_waiver` | right |

**The five swallowed numbers** (book states a figure; neither the surfaced row nor
the summary carries it, except where noted):

1. `virtue.commanding_aura` — Magic Resistance 25/20/15/10 and Soak +5/+4/+3/+2 by
   Church rank; summary truncated before the mechanics. (F-39/F-40 cover Soak and
   the rank parameter, not the MR figure or the truncation.)
2. `virtue.life_boost` — +5 per Fatigue level spent.
3. `virtue.leper_magus` — inherits Life Boost's +5; own vis table 3/6/9/12/15 pawns
   by wound, extra Aging roll past three uses a year.
4. `virtue.special_circumstances` — +3 to Magic Resistance (the casting +3 is
   computed; the MR row is blank; summary quotes the sentence).
5. `flaw.corrupted_spells` — +3/-3 Casting and Penetration Total (no
   `casting_total_mod` at all), plus the 30-level and 5-xp clauses.

Adjacent, excluded by Q-136 (data amount 0 on a computed family): `flaw.age_quickly`,
`flaw.baneful_circumstances`, `flaw.bound_to_role_role`, `flaw.leprosy`,
`flaw.incomprehensible`, `flaw.loose_magic`, `virtue.bee_king`, `virtue.unaging`.

---

## § 8 row 6 — D30: anchors and range-end form

```sh
cd rules/core && jq -r '[.. | objects | select(has("source") and (.source|type)=="object") | .source] as $s | "\(input_filename) total=\($s|length) no_anchor=\([$s[] | select(.anchor == null)] | length)"' *.json
cd rules/core && jq -r --rawfile book "../source/en/Ars Magica - Definitive Edition (Core Rules).md" '($book | split("\n")) as $L | [.. | objects | select(has("source") and (.source|type)=="object") | .source.lines] as $r | "\(input_filename) ranges=\($r|length) end_blank=\([$r[] | select($L[.[1]-1] | test("^\\s*$"))] | length) end_nonblank=\([$r[] | select($L[.[1]-1] | test("\\S"))] | length) start_blank=\([$r[] | select($L[.[0]-1] | test("^\\s*$"))] | length)"' *.json
```

All 1,218 refs cite the core book (`… | .source.file' *.json | sort | uniq -c`). The 15
string-valued `source` keys in `virtues_flaws.json` are `advancement_mod` sources,
not refs, and are excluded by the object filter.

| file | refs | no `anchor` | end on blank | end on content |
|---|---|---|---|---|
| `abilities.json` | 78 | 78 | **73** | 5 |
| `aging.json` | 30 | 30 | 0 | 30 |
| `arts.json` | 15 | 15 | **1** | 14 |
| `childhoods.json` | 5 | 5 | 0 | 5 |
| `equipment.json` | 45 | 45 | 0 | 45 |
| `houses.json` | 12 | 12 | 0 | 12 |
| `mythic_companion_types.json` | 4 | 4 | 0 | 4 |
| `spell_mastery_abilities.json` | 14 | 14 | 0 | 14 |
| `spells.json` | 360 | 360 | 0 | 360 |
| `virtues_flaws.json` | 655 | 0 | **627** | 28 |
| **total** | **1,218** | **563** | **701** | **517** |

`characteristics`, `character_types`, `life_stages`, `ruleset`: no refs. No range
*starts* on a blank line.

**Surprise vs D30:** anchors agree (563). The range form does not scale from B16's
span the way D30 implies: the blank-line form is concentrated in **`virtues_flaws.json`
(627/655) and `abilities.json` (73/78)** plus one Art; every other file already ends
on content. So normalising is **701 ranges**, not "most of 1,218".

---

## § 8 row 7 — D34: what `flaw.false_power`'s domain admits

Its (and `flaw.false_power_minor`'s) one parameter: `domain: item`,
`require_categories: [hermetic, special, supernatural]`, `require_possessed: true`,
`forbid_tainted: true`. Resolution is `selections.rs::param_value_resolves` →
`item_matches_required_categories` (**any** listed category) and not tainted; the
UI mirrors it in `ParameterPicker.svelte::itemOptionsFor`. **Neither filters on
`kind`.**

```sh
jq -r '.[] | select(.kind=="virtue" and (.categories|any(.=="hermetic")) and .tainted != true) | .id' rules/core/virtues_flaws.json
jq -r '[.[] | select((.categories | any(IN("hermetic","special","supernatural"))) and (.tainted != true))] | group_by([.kind, (.categories|map(select(IN("hermetic","special","supernatural")))|join("+"))]) | map("\(.[0].kind) \(.[0].categories|map(select(IN("hermetic","special","supernatural")))|join("+")) \(length)") | .[]' rules/core/virtues_flaws.json
```

**Hermetic Virtues admitted: 56** (audit: 56 — agrees; no Hermetic Virtue is tainted):

`virtue.adept_laboratory_student` `virtue.affinity_art` `virtue.atlantean_magic`
`virtue.boosted_magic` `virtue.cautious_sorcerer` `virtue.clan_ilfetu`
`virtue.cyclic_magic_positive` `virtue.deft_form` `virtue.diedne_magic`
`virtue.elemental_magic` `virtue.enduring_magic` `virtue.exotic_casting`
`virtue.extractor_of_form_vis` `virtue.faerie_magic` `virtue.faerie_raised_magic`
`virtue.fast_caster` `virtue.flawless_magic` `virtue.flexible_formulaic_magic`
`virtue.free_study` `virtue.gentle_gift` `virtue.gorgiastic`
`virtue.guest_of_house_criamon` `virtue.harnessed_magic` `virtue.heartbeast`
`virtue.hermetic_prestige` `virtue.imbued_with_the_spirit_of_form`
`virtue.inventive_genius` `virtue.leper_magus` `virtue.life_boost`
`virtue.life_linked_spontaneous_magic` `virtue.magical_memory`
`virtue.major_magical_focus` `virtue.mastered_spells` `virtue.masterpiece`
`virtue.mercurian_magic` `virtue.method_caster` `virtue.minor_magical_focus`
`virtue.mystical_choreography` `virtue.mythic_blood` `virtue.performance_magic`
`virtue.personal_vis_source` `virtue.potent_magic_major` `virtue.potent_magic_minor`
`virtue.puissant_art` `virtue.quiet_magic` `virtue.secondary_insight`
`virtue.side_effect` `virtue.skilled_parens` `virtue.special_circumstances`
`virtue.spell_improvisation` `virtue.study_bonus` `virtue.subtle_magic`
`virtue.tethered_magic` `virtue.the_enigma` `virtue.verditius_magic`
`virtue.withstand_casting`

**Whole domain: 227 items** — virtues: 56 hermetic + 1 special (`virtue.the_gift`)
+ 71 supernatural; **flaws: 64 hermetic + 35 supernatural = 99.**
**Surprise:** the domain admits **Flaws** (any untainted Hermetic/Supernatural Flaw
the character holds), because no layer checks `kind`. Excluded as tainted:
`flaw.corrupted_abilities` `flaw.corrupted_arts` `flaw.corrupted_spells`
`flaw.false_power` `flaw.false_power_minor` `virtue.amorphous_major`
`virtue.amorphous_minor` `virtue.command_animals` `virtue.demonic_blood`
`virtue.immune_to_disease` `virtue.infernal_heirloom`.

---

## § 8 row 9 — D43: `restricted_ability_xp` carriers

```sh
jq -r '.[] | . as $e | .effects[]? | select(.type=="restricted_ability_xp") | "| `\($e.id)` | \(.amount // "-") | \([(.abilities // [])[] | sub("^ability\\.";"")] | join(", ")) | \((.categories // []) | join(", ")) |"' rules/core/virtues_flaws.json
```

**28 carriers, 28 pools** (one pool each; audit: 28 — agrees). Pool keys used:
`abilities`, `amount`, `categories` only. Abilities shown without the `ability.` prefix.

| carrier | xp | abilities | categories |
|---|---|---|---|
| `flaw.feral_upbringing` | 120 | animal_handling, area_lore, athletics, awareness, brawl, hunt, stealth, survival, swim | |
| `virtue.arcane_lore` | 50 | | arcane |
| `virtue.baccalaureus` | 90 | artes_liberales, dead_language | |
| `virtue.cathedral_school_master` | 240 | teaching | academic |
| `virtue.clan_ilfetu` | 50 | dead_language, magic_lore, organization_lore | |
| `virtue.craft_guild_training` | 50 | bargain, craft, organization_lore, profession | |
| `virtue.doctor_in_faculty` | 300 | | academic |
| `virtue.educated` | 50 | artes_liberales, dead_language | |
| `virtue.falconer` | 50 | animal_handling, area_lore, dead_language, etiquette, hunt, profession, ride | |
| `virtue.forge_companion` | 50 | craft | |
| `virtue.hermetic_experience` | 50 | dead_language, magic_lore, organization_lore | |
| `virtue.ineslemen` | 50 | dominion_lore, faerie_lore, infernal_lore, islamic_law, magic_lore, theology_islam | |
| `virtue.lone_redcap` | 300 | | academic, arcane, general, martial, supernatural |
| `virtue.magister_in_artibus` | 240 | teaching | academic |
| `virtue.magister_in_medicina` | 300 | | academic |
| `virtue.marshal` | 50 | animal_handling, dead_language, etiquette, hunt, profession, ride | |
| `virtue.master_bard` | 240 | area_lore, art_of_memory, faerie_lore, magic_lore, organization_lore, profession | |
| `virtue.master_of_kennels` | 50 | animal_handling, dead_language, etiquette, hunt, profession, ride | |
| `virtue.mentored_by_demons` | 50 | | academic, arcane, general, martial, supernatural |
| `virtue.physician_of_salerno` | 50 | medicine, philosophiae | |
| `virtue.privileged_upbringing` | 50 | | general, academic, martial |
| `virtue.rosh_beth_din` | 50 | dead_language, rabbinic_law, theology_judaism | |
| `virtue.schooled_in_crime` | 50 | area_lore, athletics, awareness, bargain, brawl, charm, guile, legerdemain, stealth | |
| `virtue.senior_bard` | 90 | area_lore, art_of_memory, faerie_lore, magic_lore, organization_lore, profession | |
| `virtue.shadchan` | 50 | area_lore, bargain, charm, etiquette, folk_ken, guile, intrigue, leadership | |
| `virtue.trained_assassin` | 50 | athletics, guile, stealth | martial |
| `virtue.venditor` | 50 | bargain, charm, folk_ken, guile, intrigue, living_language | |
| `virtue.warrior` | 50 | | martial |

---

## § 8 row 10 — D44: `incompatible_with` declarations

```sh
cd rules/core && jq -r '"\(input_filename) \([.. | objects | select(has("incompatible_with"))] | length)"' *.json
jq '[.[] | .incompatible_with // [] | length] | add' rules/core/virtues_flaws.json
jq -r '[.[] | select(.incompatible_with) | .id as $a | .incompatible_with[] | [$a, .] | sort | join(" -- ")] | unique | .[]' rules/core/virtues_flaws.json
jq -r '[.[] | select(.incompatible_with) | .id as $a | .incompatible_with[] | [$a, .] | sort | join(" ")] | group_by(.) | map(select(length != 2)) | length' rules/core/virtues_flaws.json
```

| measure | figure |
|---|---|
| entries carrying the field (only `virtues_flaws.json` has any) | **81** (= D44's "81") |
| directed declarations (list elements summed) | **106** (the brief's "106") |
| distinct unordered pairs | **53** |
| asymmetric pairs | **0** (every pair declared from both ends) |

So "81" and "106" are the same data counted two ways; the work unit is **53 pairs**.
Of those, **33** are a Virtue/Flaw's own Major/Minor twin (32 `X_major -- X_minor`
plus `virtue.major_magical_focus -- virtue.minor_magical_focus`):

`flaw.ambitious` `flaw.avaricious` `flaw.beloved_rival` `flaw.compassionate`
`flaw.compulsion` `flaw.compulsive_lying` `flaw.depraved` `flaw.driven`
`flaw.envious` `flaw.gender_nonconforming` `flaw.generous` `flaw.greedy`
`flaw.hatred` `flaw.higher_purpose` `flaw.lecherous` `flaw.meddler` `flaw.obsessed`
`flaw.optimistic` `flaw.outsider` `flaw.overconfident` `flaw.oversensitive`
`flaw.pious` `flaw.proud` `flaw.rebellious` `flaw.reckless` `flaw.true_love`
`flaw.vow` `flaw.weakness` `flaw.wrathful` `virtue.amorphous`
`virtue.magian_lineage` `virtue.potent_magic` (each `_major -- _minor`).

The other **20**:

| pair | pair |
|---|---|
| `flaw.blatant_gift` -- `flaw.unbearable_to_beings` | `flaw.blatant_gift` -- `virtue.gentle_gift` |
| `flaw.dwarf` -- `flaw.small_frame` | `flaw.dwarf` -- `virtue.giant_blood` |
| `flaw.dwarf` -- `virtue.large` | `flaw.magical_air` -- `flaw.offensive_to_beings` |
| `flaw.small_frame` -- `virtue.giant_blood` | `flaw.small_frame` -- `virtue.large` |
| `virtue.giant_blood` -- `virtue.large` | `virtue.failed_apprentice` -- `virtue.the_gift` |
| `virtue.devil_child` -- `virtue.faerie_doctor` | `virtue.devil_child` -- `virtue.nephilim` |
| `virtue.devil_child` -- `virtue.spirit_votary` | `virtue.devil_child` -- `virtue.the_gift` |
| `virtue.faerie_doctor` -- `virtue.nephilim` | `virtue.faerie_doctor` -- `virtue.spirit_votary` |
| `virtue.faerie_doctor` -- `virtue.the_gift` | `virtue.nephilim` -- `virtue.spirit_votary` |
| `virtue.nephilim` -- `virtue.the_gift` | `virtue.spirit_votary` -- `virtue.the_gift` |

(The last ten are the full 5-clique over `devil_child`, `faerie_doctor`, `nephilim`,
`spirit_votary`, `the_gift`.)

---

## § 8 row 11 — D46: computed classes with no `effects`

```sh
jq -r '[.[] | .classification] | group_by(.) | map("\(.[0]) \(length)") | .[]' rules/core/virtues_flaws.json
jq -c '.[] | select(.classification=="creation_effect" and ((.effects // []) | length) == 0) | {id, fields: (keys - ["id","kind","magnitude","categories","classification","entity_kinds","source"])}' rules/core/virtues_flaws.json
jq -r '.[] | select(.classification=="in_play_effect" and ((.effects // []) | length) == 0) | .id' rules/core/virtues_flaws.json
```

Class census: `creation_effect` 125, `in_play_effect` 93, `narrative` 335,
`uncomputed_rule` 102.

**`creation_effect` with no `effects`: 5** (audit: five — agrees). Other mechanical
fields they carry, if any:

| id | other fields |
|---|---|
| `flaw.corrupted_arts` | `tainted` |
| `flaw.savantism` | — |
| `virtue.devil_child` | `incompatible_with` |
| `virtue.nephilim` | `incompatible_with` |
| `virtue.simple_student` | — |

**`in_play_effect` with no `effects`: 0.** No entry has an empty `effects: []`, and
no `narrative`/`uncomputed_rule` entry carries `effects`.

---

## § 8 row 12 — D48: pools that name an Ability *instance*

Parameterized Abilities (`rules/core/abilities.json`, key `parameter`): `area_lore`,
`craft`, `dead_language`, `living_language`, `mystery_cult_lore`,
`organization_lore`, `profession`. Latin is `dead_language` with exemplar `latin`
(`.scholarly_language`). Method: of the 28 pools (row 9), take those naming a
parameterized Ability (**16**), then read each passage's xp sentence for a fixed
instance.

```sh
jq -r '.. | objects | select(has("parameter")) | "\(.id) \(.parameter)"' rules/core/abilities.json
cd rules/core && jq -r --rawfile book "../source/en/Ars Magica - Definitive Edition (Core Rules).md" '($book | split("\n")) as $L | ["ability.area_lore","ability.craft","ability.dead_language","ability.living_language","ability.mystery_cult_lore","ability.organization_lore","ability.profession"] as $P | .[] | . as $e | .effects[]? | select(.type=="restricted_ability_xp") | [(.abilities // [])[] | select(IN($P[]))] as $hit | select($hit|length>0) | "\($e.id) [\($hit|join(","))] :: \([$L[$e.source.lines[0]-1:$e.source.lines[1]][] | [match("[^.]*(experience points|xp)[^.]*"; "gi") | .string]] | flatten | join(" / "))"' virtues_flaws.json
```

**12 instance-naming pools — three times B06's count.** Known four first:

| carrier | instance(s) the passage names | status |
|---|---|---|
| `virtue.forge_companion` | Craft: the master's particular Crafts | known (F-74) |
| `virtue.marshal` | Profession: Marshal (+ Latin) | known (B06) |
| `virtue.master_of_kennels` | Profession: Master of Kennels (+ Latin) | known (B06) |
| `virtue.master_bard` | Profession: Storyteller, Profession: Poet | known (B06) |
| `virtue.senior_bard` | Profession: Poet / Storyteller (same shape as Master Bard) | **new** |
| `virtue.falconer` | Profession: Falconer, Latin | **new** |
| `virtue.craft_guild_training` | Organization Lore: Guild (Craft/Profession are "any") | **new** |
| `virtue.educated` | Latin | **new** |
| `virtue.baccalaureus` | Latin | **new** |
| `virtue.hermetic_experience` | Order of Hermes Lore, Latin | **new** |
| `virtue.clan_ilfetu` | House Bjornaer Lore, Gothic | **new** |
| `virtue.rosh_beth_din` | Hebrew | **new** |

Not instance-scoped (the passage says "any"/"an … appropriate"): `flaw.feral_upbringing`
("(Area) Lore"), `virtue.schooled_in_crime`, `virtue.shadchan`, `virtue.venditor`.
The "new" rows are a reading of one sentence each and are **candidates** for the
D48 slice to confirm; the Latin cases matter doubly, since `hermetic_experience`'s
passage also forbids other xp on Latin.

---

## § 8 row 12a — D49: entries that change free seasons per year

`effects` carries no season data (`grep -c season rules/core/virtues_flaws.json` → 0).
Scan of every V/F passage:

```sh
cd rules/core && jq -r --rawfile book "../source/en/Ars Magica - Definitive Edition (Core Rules).md" '($book | split("\n")) as $L | .[] | . as $e | [$L[.source.lines[0]-1:.source.lines[1]][] | [match("[^.]*\\bseasons?\\b[^.]*"; "gi") | .string]] | flatten | select(length>0) | "\($e.id) :: \(join(" / "))"' virtues_flaws.json
```

34 entries mention a season; read, they split:

**Change the free-season count (7):** `virtue.wealthy` (three free),
`flaw.poor` (works three), `virtue.license_of_absence` (+1, hard cap four),
`virtue.landed_noble` (every season managing), `flaw.regular` (spends one free season
on worship), `flaw.fluctuating_fortune` (works one season, then three, alternating),
`flaw.servant_of_the_land` (one complete season a year on the task).

**State a per-year working obligation of two seasons (7)** — whether that differs
from the default depends on the character type, so D49 must decide:
`virtue.redcap`, `virtue.lone_redcap` (with Poor/Wealthy interactions),
`virtue.doctor_in_faculty`, `virtue.magister_in_artibus` (one with Wealthy),
`virtue.master_bard`, `virtue.town_magistrate`, `virtue.university_grammar_teacher`.

**Mention "season" without changing the count (20):** `flaw.bound_to_realm`,
`flaw.craving_for_travel`, `flaw.curse_of_slander`, `flaw.cyclic_magic_negative`,
`flaw.hermetic_patron`, `flaw.hunger_for_form_magic`, `flaw.master_of_none`,
`flaw.necessary_realm_aura_for_ability`, `flaw.wanderlust` (location, not time),
`virtue.blood_of_the_nephilim`, `virtue.cyclic_magic_positive`,
`virtue.faerie_raised_magic`, `virtue.imbued_with_the_spirit_of_form`,
`virtue.leper_magus`, `virtue.maker_of_textured_vessels`,
`virtue.maker_of_water_vessels`, `virtue.potent_magic_major`,
`virtue.potent_magic_minor`, `virtue.secondary_insight`, `virtue.skinchanger`.

**Surprise (row 12a):** up to 14 entries, not B05's three. `virtue.wealthy`/`flaw.poor` are
modelled only as `later_life_xp_rate` (20/10); their season counts are not.

---

## § 8 row 12c — D56: every `is_magus` / `IsMagus` / `isMagus` site

```sh
grep -rlE "is_magus|IsMagus|isMagus" crates/arm-rules/src crates/arm-app/src ui/src | wc -l
grep -roE "is_magus|IsMagus|isMagus" crates/arm-rules/src crates/arm-app/src ui/src | cut -d: -f1 | sort | uniq -c | sort -k1 -nr
grep -roE "is_magus|IsMagus|isMagus" crates/arm-rules/tests crates/arm-app/tests ui/e2e | cut -d: -f1 | sort | uniq -c | sort -k1 -nr
```

**52 files** under the three `src` roots (audit: 52 — agrees) — 33 source files plus
19 co-located `ui/src/**/*.test.ts`. **217 occurrences** in all: Rust 151
(`… crates/arm-rules/src crates/arm-app/src | wc -l`), UI source 27, UI tests 39. Rust `src` files carry inline `#[cfg(test)] mod tests`; the
prod/test split below counts occurrences above/below that module's line (via
`grep -rnoE … | jq -R -s` against each file's `#[cfg(test)]` line). 38 of the Rust
hit lines are `//` comments.

| file | total | prod | inline test |
|---|---|---|---|
| `crates/arm-rules/src/validation/mod.rs` | 41 | **1** | 40 |
| `crates/arm-rules/src/types.rs` | 23 | 9 | 14 |
| `crates/arm-rules/src/derived.rs` | 21 | 13 | 8 |
| `crates/arm-rules/src/validation/prereq.rs` | 13 | 9 | 4 |
| `crates/arm-rules/src/validation/magus.rs` | 11 | 10 | 1 |
| `crates/arm-rules/src/ruleset.rs` | 8 | 0 | 8 |
| `crates/arm-rules/src/ruleset/integrity.rs` | 8 | 8 | 0 |
| `crates/arm-rules/src/effective/xp.rs` | 6 | 6 | 0 |
| `crates/arm-rules/src/life_stage.rs` | 4 | 2 | 2 |
| `crates/arm-rules/src/validation/authorization.rs` | 3 | 2 | 1 |
| `crates/arm-rules/src/effective.rs` | 3 | 0 | 3 |
| `crates/arm-rules/src/validation/life_stage.rs` | 2 | 1 | 1 |
| `crates/arm-rules/src/effective/warping.rs` | 2 | 2 | 0 |
| `crates/arm-app/src/ruleset_io.rs` | 2 | 2 | 0 |
| `crates/arm-rules/src/validation/warping.rs` | 1 | 1 | 0 |
| `crates/arm-rules/src/export.rs` | 1 | 0 | 1 |
| `crates/arm-rules/src/effective/reputation_and_caps.rs` | 1 | 1 | 0 |
| `crates/arm-rules/src/completeness.rs` | 1 | 0 | 1 |
| `ui/src/lib/types.ts` | 5 | 5 | |
| `ui/src/App.svelte` | 5 | 5 | |
| `ui/src/lib/components/SupernaturalBeing.svelte` | 3 | 3 | |
| `ui/src/lib/components/LifeStagePanel.svelte` | 3 | 3 | |
| `ui/src/lib/derive.ts` | 1 | 1 | |
| `ui/src/lib/components/SpellBudgetBar.svelte` | 1 | 1 | |
| `ui/src/lib/components/MagusMinimumAbilities.svelte` | 1 | 1 | |
| `ui/src/lib/components/DerivedTotalsPanel.svelte` | 1 | 1 | |
| `ui/src/lib/components/DerivedPenetrationSection.svelte` | 1 | 1 | |
| `ui/src/lib/components/DerivedMasterpieceSection.svelte` | 1 | 1 | |
| `ui/src/lib/components/DerivedMagicResistanceSection.svelte` | 1 | 1 | |
| `ui/src/lib/components/DerivedLongevitySection.svelte` | 1 | 1 | |
| `ui/src/lib/components/DerivedLabCastingSection.svelte` | 1 | 1 | |
| `ui/src/lib/components/DerivedFamiliarSection.svelte` | 1 | 1 | |
| `ui/src/lib/components/DerivedAuraField.svelte` | 1 | 1 | |
| **33 source files** (18 Rust, 15 UI) | **178** | **94** (67 Rust + 27 UI) | **84** (Rust) |

Co-located UI tests (19 files, 39): `DerivedTotalsPanel.test.ts` 8,
`LifeStagePanel.test.ts` 4, `VirtueFlawTab.test.ts` 3, `CharacterDetails.test.ts` 3,
`state.svelte.test.ts` 2, `VirtueFlawTab.client.test.ts` 2, `AgingPanel.test.ts` 2,
`AbilityTab.test.ts` 2, `App.test.ts` 2, `App.client.test.ts` 2, and 1 each in
`WizardStep`, `WizardShell`, `SupernaturalBeing`, `SpellBudgetBar`,
`ParameterPicker`, `MagusMinimumAbilities`, `ExperienceStep`,
`DerivedTotalsPanel.client`, `CharacterBanner` (`.test.ts`).

Test directories: `crates/arm-rules/tests/data_integrity.rs` 9,
`crates/arm-rules/tests/uncomputed_clauses.rs` 1,
`crates/arm-rules/tests/roundtrip_proptest.rs` 1, `ui/e2e/specs/grog-wizard-aging.e2e.js` 1
(`crates/arm-app/tests`: 0).

**Surprise:** § 8's "~33 in `validation/mod.rs`" is **1 production site** — the other
40 are test fixtures. The real prod weight is `derived.rs` (13), `validation/magus.rs`
(10), `types.rs` (9), `validation/prereq.rs` (9), `ruleset/integrity.rs` (8),
`effective/xp.rs` (6) and the UI (27). Occurrences, not re-read sites: one line
can hold two (`grep -c` gives 38 *lines* in `validation/mod.rs`).

---

## § 8 row 12d — D57: German `name` templates with a mid-phrase `{token}`

```sh
cd rules/i18n/de && jq -r 'to_entries[] | select(.value|type=="object") | select((.value.name // "") | test("\\{")) | "\(input_filename)\t\(.key)\t\(.value.name)"' *.json
cd rules/i18n/de && jq -r 'to_entries[] | select(.value|type=="object") | select((.value.name // "") | gsub("(, |: )\\{[a-z_]+\\}$"; "") | gsub(" \\(\\{[a-z_]+\\}\\)$"; "") | gsub("^\\{[a-z_]+\\} \\("; "(") | test("\\{")) | "\(.key)\t\(.value.name)"' abilities.json spells.json virtues_flaws.json
```

**53** German templates carry a token (only in `abilities`, `spells`,
`virtues_flaws`). Stripping the grammatically inert forms — trailing `, {x}`
apposition, trailing `: {x}` label, trailing `({x})`, and head-plus-parenthesis
`{x} (…)` — leaves **38 mid-phrase** (RULES.md: ~36).

**Free-standing, in a position German inflects for (30)** — the D57 work list:

| id | template |
|---|---|
| `flaw.anchored_to_the_land` | `Verwurzelt im {land}` |
| `flaw.bound_to_role_role` | `Gebunden an {role}` |
| `flaw.careless_with_ability` | `Nachlässig mit {ability}` |
| `flaw.deficient_technique` | `Defizitäre {technique}` |
| `flaw.fish_out_of_water_terrain` | `Fremd im {terrain}` |
| `flaw.form_monstrosity` | `{form} Missgeburt` |
| `flaw.necessary_realm_aura_for_ability` | `Notwendige Aura für {ability}, {realm}` |
| `flaw.offensive_to_beings` | `Abstoßend für {being}` |
| `flaw.poor_characteristic` | `Schlechte {characteristic}` |
| `flaw.servant_of_the_land` | `Diener des {land}` |
| `flaw.unbearable_to_beings` | `Unerträglich für {being}` |
| `virtue.academic_concentration_subject` | `Akademische Vertiefung {subject}` |
| `virtue.affinity_ability` | `Affinität zu {ability}` |
| `virtue.affinity_art` | `Affinität zu {art}` |
| `virtue.alluring_to_beings` | `Anziehend auf {being}` |
| `virtue.aptitude_for_sin` | `Begabung für {sin}` |
| `virtue.cautious_with_ability` | `Vorsichtig mit {ability}` |
| `virtue.doctor_in_faculty` | `Doktor der {faculty}` |
| `virtue.enchanting_ability` | `Bezaubernde {ability}` |
| `virtue.extractor_of_form_vis` | `Vis-Gewinner der {form}` |
| `virtue.folk_magic` | `Volksmagie {category}, {realm}` |
| `virtue.great_characteristic` | `Hervorragende {characteristic}` |
| `virtue.imbued_with_the_spirit_of_form` | `Durchdrungen vom Geist der {form}` |
| `virtue.inoffensive_to_beings` | `Unauffällig für {being}` |
| `virtue.learn_ability_from_mistakes` | `{ability} durch Fehler lernen` |
| `virtue.perfect_eye_for_commodity` | `Untrüglicher Blick für {commodity}` |
| `virtue.puissant_ability` | `Begabung in {ability}` |
| `virtue.puissant_art` | `Begabung in {art}` |
| `virtue.voice_of_the_land` | `Stimme des {land}` |
| `virtue.ways_of_the_land` | `Wege des {land}` |

**Inside a compound (8)** — no case agreement, but the fill must compound cleanly:
`ability.area_lore` `{area}-Kunde`, `ability.mystery_cult_lore` `{mystery_cult}-Kunde`,
`ability.organization_lore` `{organization}-Kunde`, `flaw.hunger_for_form_magic`
`Hunger nach {form}-Magie`, `flaw.magical_being_companion` `Magischer {being}-Gefährte`,
`virtue.master_of_form_creatures` `Meister der {form}-Kreaturen`,
`spell.unravelling_the_fabric_of_form` `Das ({form})-Gefüge auflösen`,
`virtue.land_regio_network` `{land}Regio-Netz` (**no hyphen** — likely a template defect).

Excluded (15): comma apposition `flaw.bound_to_realm`, `flaw.realm_stigmatic`,
`virtue.student_of_realm`; colon label `ability.craft`, `ability.profession`,
`flaw.false_power`, `flaw.false_power_minor`; head + parenthesis
`ability.dead_language`, `ability.living_language`; trailing parenthesis
`flaw.restricted_power`, `flaw.slow_power`, `virtue.variable_power`,
`spell.mirror_of_opposition_form`, `spell.wizards_boost_form`, `spell.wizards_reach_form`.

---

## § 8 row 13 — D23 / Q-137: Flaws carrying `grants_reputation`

```sh
jq -r '.[] | select(.kind=="flaw") | . as $e | .effects[]? | select(.type=="grants_reputation") | "\($e.id) \(del(.type)|tostring)"' rules/core/virtues_flaws.json
```

**16 ids, 17 rows** (B18: 16 across 17, `flaw.failed_monk` two — agrees). Virtues
carry 16 further rows. The effect has no sign field; "bad" is implied by `kind: flaw`.

| id | kind | score |
|---|---|---|
| `flaw.apostate` | ecclesiastical | 4 |
| `flaw.black_sheep` | local | 2 |
| `flaw.failed_journeyman` | local | 2 |
| `flaw.failed_master` | local | 4 |
| `flaw.failed_monk` | local | 2 |
| `flaw.failed_monk` | ecclesiastical | 2 |
| `flaw.failed_student` | academic | 2 |
| `flaw.feral_scent` | local | 2 |
| `flaw.gabai` | local | 2 |
| `flaw.hedge_wizard` | hermetic | 3 |
| `flaw.infamous` | local | 4 |
| `flaw.infamous_master` | hermetic | 3 |
| `flaw.outlaw` | local | 2 |
| `flaw.outlaw_leader` | local | 3 |
| `flaw.outsider_major` | local | 3 |
| `flaw.outsider_minor` | local | 1 |
| `flaw.usurer` | local | 4 |

---

## Handover § 8 — entry-less `ArMDE:` citations

A citation is **entry-less** when its (first) line falls inside no V/F entry's
`source.lines` range. Exact, not approximated — `jq --slurpfile` tests every
citation against all 655 ranges. Per file (swap the path):

```sh
grep -oE "ArMDE:[0-9]+" docs/vf-audit/corrections.md | jq -R -s --slurpfile vf rules/core/virtues_flaws.json -r '[$vf[0][] | .source.lines] as $R | ($R | map(.[0]) | min) as $lo | ($R | map(.[1]) | max) as $hi | [split("\n")[] | select(length>0) | ltrimstr("ArMDE:") | tonumber] as $c | "total=\($c|length) in_entry=\([$c[] | . as $n | select(any($R[]; .[0] <= $n and $n <= .[1]))] | length) entry_less=\([$c[] | . as $n | select(all($R[]; .[0] > $n or $n > .[1]))] | length) in_block_span=\([$c[] | select(. >= $lo and . <= $hi)] | length) outside_span=\([$c[] | select(. < $lo or . > $hi)] | length)"'
```

| file | citations | in a V/F entry | **entry-less** | distinct entry-less lines | inside V/F block span | outside it |
|---|---|---|---|---|---|---|
| `corrections.md` | 314 | 223 | **91** | 51 | 227 | 87 |
| `decisions.md` | 152 | 110 | **42** | 23 | 113 | 39 |
| `phase-2-handover.md` | 8 | 2 | **6** | 5 | 2 | 6 |
| **three working files** | **474** | 335 | **139** | **56** | 342 | 132 |
| `book_templates.rs` | 208 | 17 | **191** | 181 | 17 | 191 |

The V/F block span is the min start to max end over all 655 ranges. In the three
working files only **7** entry-less citations sit inside that span (between entries);
the rest are outside it. Handover said 470 on 2026-09-24; today 474.

Not counted: continuation citations (`, :NNNN` after an `ArMDE:` — 24 in
`corrections.md`, 11 in `decisions.md`, 8 in `book_templates.rs`) and other books'
acronyms (4 and 8 in the two audit files). A citation inside a V/F range is
"recoverable via `source.anchor`" only in the sense D30 means; one citing a
different entry's rule from inside another's range still resolves to that range.
