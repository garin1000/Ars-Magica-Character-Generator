# UI chrome for the Ars Magica character generator.
# Rules text (virtue/flaw names, summaries) is NOT here — it comes from
# rules/i18n/<lang>/ via the load_ruleset command.

app-title = Ars Magica Character Generator
app-logo-alt = Ars Magica Open License logo

language-label = Language
# Option labels for the language picker itself: each language's OWN endonym
# (never a translation into the currently active language — a language picker
# always shows "Deutsch", not "German", regardless of which language is active).
language-name-en = English
language-name-de = Deutsch
mode-label = Validation

mode-enforced = Enforced
mode-advisory = Advisory
mode-silent = Silent

# Which palette the app paints with. `auto` follows the desktop and is the
# default. Option labels, never the raw `auto`/`light`/`dark` ids — a slug on
# screen is the same violation as a hardcoded English string.
theme-label = Appearance
theme-auto = Match the desktop
theme-light = Light
theme-dark = Dark

# The settings dialog (C4): language, appearance, validation strictness and — since
# C8 — the saga year new documents start at, all persisted across restarts.
# Reachable from the native menu's Settings item (U2/P2 retired the header's own
# button — the menu is the only way in now, on every screen).
settings-title = Settings
settings-close = Close
# The saga year a NEWLY created character or covenant is stamped with. Deliberately
# not "Saga year": the saga year itself belongs to each document (`saga-year-label`),
# and this seeds the next one. The label says what it seeds so the dialog needs no
# explanatory sentence under it.
settings-default-saga-year-label = Saga year for new documents

type-label = Character type
type-grog = Grog
type-companion = Companion
type-mythic_companion = Mythic Companion
type-magus = Magus
# Shown in place of `type-<id>` when a loaded save names a character type the
# active ruleset has no profile for — the raw id must never reach the screen.
type-unknown = Unknown character type

# Startup screen: the app opens here, with no character loaded yet. The character
# type is picked once, by creating a character of that type.
start-title = Create or open a character
start-open-title = Open an existing character
# The second way in through a file: the same open, landing on the guided flow
# instead of the editor.
action-open-into-wizard = Open in guided creation
start-open-wizard-hint = A character saved part-way through guided creation resumes at the step it was left on. One built in the editor opens with every step reachable.
start-create-title = Create a new character
start-create-hint = The character type is chosen here, once — it cannot be changed later.
# S28 (full-audit UX): names the validation-mode axis (Enforced/Advisory/Silent)
# that distinguishes direct-validated from direct-unchecked creation — invisible
# on this screen otherwise. C4 moved the Validation control into the settings
# dialog, which is reachable from this screen too, so the hint names where it is
# rather than pointing ahead to a toolbar that no longer exists (C3c).
start-create-mode-hint = How strictly the rules are checked (Validation) can be changed at any time, under Settings.
start-wizard-title = Guided creation
start-wizard-hint = Step by step through this type's creation phases, in order. The character type is chosen here too, once — it cannot be changed later.

# Creation-phase labels, keyed by the engine's `CreationPhase` slug. They name the
# guided wizard's steps and are the only rendering of a phase — the slug itself
# never reaches the screen. `review` is the wizard's own terminal step, appended
# after whatever phases the character type declares.
phase-concept = Concept
phase-characteristics = Characteristics
phase-virtues_flaws = Virtues & Flaws
phase-experience = Experience
phase-abilities = Abilities
phase-arts = Arts
phase-spells = Spells
phase-house_specialisation = House
phase-mythic_type = Mythic companion type
phase-personality_reputations = Personality & Reputations
phase-aging = Aging
phase-review = Review

# Guided wizard chrome. The rail lists the steps; the step body is the same input
# surface the editor's tabs use.
wizard-rail-label = Creation steps
wizard-step-progress = Step { $current } of { $total }
wizard-back = Back
wizard-next = Next
wizard-finish = Finish
# Why Next is disabled: the current step has an error. Errors only — an advisory
# never blocks — and switching the validation mode to Advisory lifts the gate.
wizard-blocked-hint = Fix this step's errors to continue, or switch the validation mode to Advisory.
# Marks a step in the rail that still holds an error.
wizard-step-blocked-label = has errors
# Marks a step in the rail that still holds a WARNING — outstanding work that
# gates nothing, so the step is left open rather than held shut. Deliberately a
# weaker statement than `wizard-step-blocked-label`, and only ever shown for a
# step already reached.
wizard-step-pending-label = has open warnings
# Marks a step in the rail nothing has been recorded for yet. Legal is not the
# same as finished: an empty step is marked, never blocked.
wizard-step-incomplete-label = not started
# Shown while the validation mode is Advisory or Silent: nothing is enforced, so
# no step gates and Finish is always available.
wizard-unchecked-hint = Validation is not enforced, so no step blocks progress.

# The wizard's closing step.
wizard-review-title = Review
wizard-review-clean = No errors or warnings — this character is legal.
# Honest about what the gating does and does not check: the steps block on errors
# only, so a legal character can still be an unfinished one. Introduces the list of
# the steps that were left empty; nothing here holds Finish shut.
wizard-review-incomplete = A legal character is not necessarily a finished one: steps only block on errors, so these were left empty. You may finish anyway.
# Shown instead of that list once every step of the flow holds a choice.
wizard-review-complete = Every step of this flow has choices recorded.
wizard-review-hint = Equipment, magic items, Might and Warping are not part of the guided flow — they are edited after finishing.

available-title = Available
items-virtues-title = Virtues
items-flaws-title = Flaws
selections-title = Selected
# Marker on a selected row the character type requires (e.g. a magus's The Gift
# and Hermetic Magus) — auto-selected and not removable.
selection-required-label = Required
validation-title = Validation
characteristics-title = Characteristics
abilities-title = Abilities

# Virtue/Flaw category (type) labels, keyed by the engine's category id.
category-general = General
category-hermetic = Hermetic
category-mythic_companion = Mythic Companion
category-personality = Personality
category-social_status = Social Status
category-special = Special
category-story = Story
category-supernatural = Supernatural

# Magnitude (level) labels, keyed by the engine's magnitude value.
magnitude-free = Free
magnitude-minor = Minor
magnitude-major = Major

# Virtue/Flaw "Type" tag: a Tainted (Infernal-associated) Virtue or Flaw.
vf-tag-tainted = Tainted

# Reason shown on a Virtue/Flaw the enforced mode greys out because an already
# selected item excludes it (Major vs Minor of the same V/F, Gentle vs Blatant
# Gift, …). { $other } is the selected item that blocks it.
vf-blocked-incompatible = Incompatible with { $other }

# Reason shown on a Virtue/Flaw the Available list greys out because its
# bought + granted copies have already reached its total ceiling (e.g.
# Puissant Art, capped at two total across every Art target). Unlike
# vf-blocked-incompatible this applies in every ValidationMode. { $max } is the
# item's stated maximum.
vf-blocked-max-total = Maximum of { $max } already reached

# Filter/search controls for long selectable lists.
filter-search-placeholder = Search…
filter-magnitude-all = All levels
filter-category-all = All types
# Accessible names for filter selects that carry no visible label of their own.
ability-category-filter-label = Filter by ability category
# { $side } is the localized side title ("Virtues"/"Flaws").
vf-category-filter-label = Filter { $side } by category
vf-magnitude-filter-label = Filter { $side } by magnitude

# The eight Characteristics, keyed by the engine's characteristic value.
characteristic-int = Intelligence
characteristic-per = Perception
characteristic-str = Strength
characteristic-sta = Stamina
characteristic-pre = Presence
characteristic-com = Communication
characteristic-dex = Dexterity
characteristic-qik = Quickness
# Short tooltip descriptions, condensed from the Core Rules Characteristics chapter.
characteristic-desc-int = The power to analyze and synthesize concepts, plus simple memory; paramount for the Hermetic Arts.
characteristic-desc-per = The ability to notice things and powers of intuition; key to Awareness, Hunt, and Folk Ken.
characteristic-desc-str = Physical power — lifting, pushing, and moving — and the force behind a melee weapon.
characteristic-desc-sta = Staying power of body and mind; spellcasting, carrying loads, and withstanding wounds all rely on it.
characteristic-desc-pre = Appearance, demeanor, and charisma — making an impression, leading, and intimidating.
characteristic-desc-com = The aptitude for self-expression — influencing and communicating with others.
characteristic-desc-dex = Agility and skillful, accurate handling of objects; hand-eye coordination and bodily grace.
characteristic-desc-qik = Reaction speed and reflexes — who acts first in haste; modified by Encumbrance.

# Ability category labels, keyed by the engine's category value.
ability-category-general = General
ability-category-academic = Academic
ability-category-arcane = Arcane
ability-category-martial = Martial
ability-category-supernatural = Supernatural
# Trailing marker shown after the name of an "asterisked" Ability — one that
# cannot be used without at least one experience point in it (no untrained roll),
# per the rulebook. Spans General, Academic, Arcane, and Supernatural abilities.
ability-requires-training-marker = *

# Characteristic point-buy readout.
characteristic-points = Points: { $used } / { $budget }
characteristic-size = Size: { $size }
# Tooltip on the Characteristics tab explaining the effective score (the bought
# score after aging drops and free Virtue deltas). Shown only when the effective
# score differs from the bought score. $bought/$effective are pre-formatted
# signed numbers; $drops is a positive drop count; $bonus is a signed delta.
characteristic-effective-tooltip-summary = Bought { $bought }, effective { $effective }.
characteristic-effective-tooltip-breakdown-label = Includes
characteristic-effective-tooltip-aging = aging -{ $drops }
characteristic-effective-tooltip-virtue = Virtue { $bonus }
# Tab labels for the editor's main area. Each tab mirroring a wizard phase reads
# the same as that phase's rail entry; the keys hyphenate where the phase slug
# uses an underscore (tab-personality-reputations vs phase-personality_reputations).
tab-characteristics = Characteristics
tab-virtues-flaws = Virtues & Flaws
tab-experience = Experience
tab-abilities = Abilities
tab-personality-reputations = Personality & Reputations
tab-aging = Aging
tab-arts = Arts
tab-house-specialisation = House
tab-mythic-type = Type
tab-supernatural = Supernatural
tab-spells = Spells
tab-possessions = Magic Items
tab-equipment = Equipment
tab-details = Details
# Shared XP summary (Abilities + Arts). One row: the label, the read-only used
# figure, the editable general-pool total (bracketed), then Available and any
# restricted sub-budgets. `xp-pool` labels the whole general-pool group.
xp-pool = XP pool
xp-available = Available: { $available }
# The Virtue/Flaw contribution to the general pool: Skilled Parens grants "an
# additional 60 experience points … during apprenticeship" (Core Rules.md:4966),
# Weak Parens takes 60 away. A POSITIVE one is a pool of its own, spent before the
# base exactly as a restricted pool is; a NEGATIVE one has nothing to spend and is
# charged to the base, so it is reported as the signed modifier that explains the
# charge. Same split, same wording as the spell-levels bar's own bonus lines.
xp-bonus-pool = Virtues/Flaws: { $used } / { $amount }
xp-bonus = Virtues/Flaws: { $bonus }
# Restricted experience pools (Educated/Warrior/Privileged): extra XP spendable
# only on the listed Abilities/categories. `$eligibility` is a localized list.
restricted-xp-pool = { $eligibility }: { $used } / { $amount }
restricted-xp-list-separator = ,
# Life-stage experience blocks, keyed by the engine's `LifeStageBlock` slug. Each is
# a restricted pool of its own: the first buys the native language and nothing else,
# the second the childhood Abilities but never that language. Later life appears for
# a magus, whose general pool is apprenticeship instead — its years earn Abilities
# only, never an Art, so the label says so.
#
# These name a block INSIDE A SENTENCE, not on the bar: `resolveIssueArgValue`
# resolves an unspent-experience warning's `origin` arg through them. The XP bar's own
# chips are the `xp-pool-block-*` family below, which merges each block's derivation
# with its pool — do not point the bar back at these keys, or one block gets two
# labels again (guided-creation-review-2026-08 #14).
xp-pool-childhood_native_language = Native language
xp-pool-childhood_spread = Early childhood
xp-pool-later_life = Later life (Abilities only)
# D40/D2: an apprenticeship-SHAPED replacement pool (Redcap, Lone Redcap) —
# distinct from a real magus's own apprenticeship, which funds the general
# pool and carries no chip of its own (`xp-pool-block-apprenticeship` names a
# DIFFERENT thing: that pool's own bar row, not this restricted-pool label).
xp-pool-apprenticeship = Apprenticeship (Abilities only)
# The Abilities funding switch. Experience either comes from one pool the player
# enters, or the character's life stages earn it. The switch is not a stored flag:
# a life-stage plan on the character IS guided funding, so a loaded save lands in
# the mode its own data implies.
ability-funding-label = Source of experience
ability-funding-pool = Experience pool
ability-funding-life_stages = Life stages
life-stage-no-budget = No life-stage experience yet.
# THE LIFE-STAGE BLOCK CHIPS — one chip per block, in the order the character lived
# them (guided-creation-review-2026-08 #14). The rules state the blocks as an ordered
# sequence — "Early Childhood … Later Life … Apprenticeship … Years after
# apprenticeship" (Core Rules.md:2213-2216) and again as a chronology of periods
# (`:2364`) — so the bar reads top-to-bottom as that chronology.
#
# Each key merges the block's DERIVATION with the spent/total of the restricted pool
# it forms, because two chips carrying one block's name (the old
# `life-stage-later-life` beside the old `xp-pool-later_life` row) made the same
# label appear twice and read as two different pools.
#
# Childhood's 75 for the native language and 45 for the spread are one block, not two
# (`:2378`), so they share one heading. The spread-only variant is the state before a
# native language is named: the engine forms that pool only once it is
# (`effective/xp.rs`), and a chip must not claim a pool that does not exist.
xp-pool-block-early-childhood = Early childhood — Native language { $nativeUsed } / { $nativeAmount } · Other Abilities { $spreadUsed } / { $spreadAmount }
xp-pool-block-early-childhood-spread-only = Early childhood — Other Abilities { $spreadUsed } / { $spreadAmount }
# Later life: "15 experience points per year (until apprenticeship for magi)"
# (`:2214`). The age span is the character's own — childhood's years to the start of
# apprenticeship — and it is what tells the player which years these points are
# from: the Darius example spends exactly this block over ages 5 to 10 (`:2402`).
# The rate is this character's own too; the Wealthy Virtue and the Poor Flaw change
# it (`:2394`). Two variants because later life is a pool of its own only for a
# magus, whose general pool is apprenticeship; for anyone else later life IS the
# general pool already shown as the bar's own total, so repeating it would be a
# second spent/total for one pool.
xp-pool-block-later-life = Later life (ages { $from }-{ $to }): { $years } × { $rate } = { $xp } XP
xp-pool-block-later-life-restricted = Later life (ages { $from }-{ $to }): { $years } × { $rate } = { $xp } XP — { $used } / { $amount }
# Apprenticeship, for a magus alone: fifteen fixed years whose experience "can be
# spent on Arts or Abilities" (Core Rules.md:2435), which makes it the general pool —
# so this chip names the block the pool total comes from and needs no spent/total of
# its own. Absent for anyone who serves no apprenticeship, which is what keeps the
# non-magus bar unchanged.
xp-pool-block-apprenticeship = Apprenticeship: { $years } years = { $xp } XP
# The years after apprenticeship: "For every year, the magus gets 30 points"
# (Core Rules.md:2471), less 10 for every charged season of lab work (`:2482`). Each
# point is an experience point or one level of a spell, so the points and the
# experience left after the spell levels are both named.
#
# "After the Gauntlet", not "As a magus" (#14.4): the block is driven by the
# `Gauntlet age` field, and naming it for the Gauntlet ties label to field. It also
# stops the chip reading as a *state* the character is in — which, sitting where it
# used to sit, made "Later life" below it look like life past the Gauntlet.
xp-pool-block-after-gauntlet = After the Gauntlet: { $years } × { $rate } - { $lab } for lab work = { $points } points, { $xp } XP
# Age is repeated inside the panel because later life is measured in years, so it
# is edited here as well as on the Details tab.
life-stage-age-label = Age
life-stage-gauntlet-age-label = Gauntlet age
life-stage-lab-seasons-label = Lab seasons
life-stage-spell-levels-label = Levels of spells
# Why both fields above are read-only: it states a read-only state the controls
# cannot state for themselves, and it is the `aria-describedby` target of both.
life-stage-post-gauntlet-no-years-note = No years as a magus yet, so lab seasons and levels of spells can take nothing.
life-stage-post-gauntlet-summary = Years as a magus: { $years }; { $points } points = { $xp } XP + { $levels } level(s) of spells
# Escape hatch for a hand-edited save: a character funded by its life stages must
# not also carry an entered pool, and guided mode offers no field to correct one,
# so this empties it.
# The native language: childhood's first block buys this Ability and nothing else,
# so there is no childhood budget until it is named.
native-language-label = Native language
native-language-placeholder = e.g. German
# Sample Childhoods: ready-made Ability packages for early childhood's experience.
# Package names are rules text (rules/i18n/<lang>/childhoods.json), never keys here.
# Spending the points yourself is an equal choice rather than an opt-out, so it is
# the picker's first option and not an empty selection.
childhood-label = Sample Childhood
childhood-taken = Childhood taken: { $name }
childhood-choose-prompt = — Spend childhood's experience yourself —
childhood-preview-label = Package preview
childhood-entry = { $name } { $score }
# A slot the package leaves open (the Area of an Area Lore, the Language of a Living
# Language). Its label is the entry's own Ability name — slot ids are per-package
# data and must never be rendered — with a 1-based ordinal only where one package
# slots the same Ability twice (Traveling Childhood's two Area Lores).
childhood-slot-label = { $name }
childhood-slot-label-nth = { $name } ({ $index })
childhood-apply = Take this childhood
# Why taking the package is blocked. Two slots of one Ability sharing a value would
# merge into a single row and waste the other's experience; a childhood language
# must differ from the native language, which has a block of its own.
childhood-slot-empty-reason = Fill in every Ability the package leaves open.
childhood-slot-duplicate-reason = Two slots of the same Ability need different values, or they merge into one row and experience is wasted.
childhood-slot-native-reason = A childhood language must differ from the native language.
# The Hermetic minimum Abilities a magus owes. The first group is admission to the
# Order — "Characters with lower scores would not be admitted"
# (Core Rules.md:2437) — the second the rulebook's recommended package
# (`:2451-2461`), which is advice and therefore a warning. Each row states its status
# as a whole sentence rather than a shared template plus a "met"/"unmet" word, so
# nothing is carried by colour and the German reads as German.
magus-minimums-label = Minimum Abilities
# The label of the collapsed checklist (guided-creation-review-2026-08 #12), so it
# must name its own subject: it is now the disclosure's summary rather than a line
# under the "Minimum Abilities" heading, and { $total } counts every row it heads —
# the demanded ones and the recommended ones alike.
magus-minimums-summary = Ability requirements: { $unmet } of { $total } still unmet
magus-minimum-met = { $ability } { $min }{ $qualifier } is met: this character has { $score }.
magus-minimum-unmet = { $ability } { $min }{ $qualifier } is not met: this character has { $score }.
magus-recommended-label = Recommended minimum Abilities
ability-score-label = Score
# X10b: the number input for the "Z" of the book's own "X (Z)" notation
# (ArMDE:1177) — XP already banked toward the next score.
ability-banked-xp-label = Banked XP
# UI review 2026-09-30b #2: the always-visible unit caption beside the
# banked-XP input on Arts/Abilities rows (the input's own aria-label already
# says "Banked XP" in full). Shared by both tabs, which already share one XP
# pool (see the Arts comment below).
xp-unit-abbr = XP
ability-specialty-label = Specialty
# Heading for the rulebook's list of example specialties shown in the picker.
ability-specialties-label = Specialties
ability-add = Add ability
# Named per row ($name is the ability's own display name), so a screen reader
# tabbing a long Abilities list hears which score each stepper adjusts instead
# of an identical bare "Raise"/"Lower" on every row.
ability-increment = Raise { $name }
ability-decrement = Lower { $name }
# Hermetic Arts: the two classes and the spinner controls. Arts share the
# Abilities XP pool (the `xp-pool` key), so no Art-specific pool label.
art-type-technique = Techniques
art-type-form = Forms
# X10b: the number input for the "Z" of the book's own "X (Z)" notation
# (ArMDE:1179) — XP already banked toward the next score.
art-banked-xp-label = Banked XP
art-add = Add Art
# Named per row ($name is the Art's own display name) — same reasoning as
# ability-increment/-decrement above.
art-increment = Raise { $name }
art-decrement = Lower { $name }
# Spell picker (magi only). Spell display names come from the rules i18n (keyed
# by spell id), not from these chrome keys. Filter a spell by Technique + Form,
# then add it; a General spell prompts for its learned level.
spell-technique-label = Technique
spell-form-label = Form
spell-level-label = Level
# X10c: shown only for a known spell whose (Technique, Form) cell has a
# within-focus figure — i.e. the character holds a Magical Focus that could
# cover it. The player's own claim the spell falls within that focus.
spell-within-focus-label = Within focus
# D79: shown only for a known spell whose (Technique, Form) cell has a
# within-potent-field figure — i.e. the character holds a Potent Magic
# Virtue that could cover it. Independent of spell-within-focus-label: a
# spell may be within a Magical Focus, a Potent Magic field, both, or
# neither, since the two free-text themes need not coincide.
spell-within-potent-field-label = Within Potent Magic field
# X10c (D73.2): the in-app per-spell Casting Total beside a known spell.
# UI review 2026-09-30b #5: the value is baked into this one message rather
# than concatenated with a literal ": " in the template.
spell-casting-total = Casting Total: { $total }
# The min/max level range filter (two inputs, inclusive bounds; empty = open).
spell-level-min-label = Min level
spell-level-max-label = Max level
# Source spells are grouped by Technique + Form; the header composes the two
# localized Art names (never a raw id).
spell-group-header = { $technique } { $form }
# Compact "General" tag on a spell with no fixed catalogue level (shown after the
# Technique/Form abbreviations, e.g. "ReVi Gen").
spell-level-general = Gen
# Why a spell's add control is greyed: its level is above the magus's per-spell
# casting cap, the remaining spell-levels budget cannot afford it, or the spell is
# already in the selected list (an ordinary fixed-level spell is taken only once).
spell-cap-reason = Above your casting cap ({ $cap })
# D81.5: the plain Add control's reason when the spell's level exceeds the
# plain per-spell cap but still fits the Magical-Focus-doubled one — distinct
# from spell-cap-reason so the tooltip points at the separate "add within
# focus" action instead of calling the spell simply out of reach.
spell-cap-within-focus-reason = Above your casting cap ({ $cap }); fits within your Magical Focus
spell-budget-reason = Not enough spell levels remaining
spell-already-taken-reason = Already selected
# D81.5: a spell whose level exceeds the plain per-spell cap but fits the
# Magical-Focus-doubled one — a separate action that adds it already marked
# as within focus (`SpellSelection.within_focus`), since the engine cannot
# match a spell to a player's own free-text Magical Focus by itself.
spell-add-within-focus = Add within focus
spell-add-within-focus-label = Add { $name } within focus
spell-add-within-focus-tooltip = Fits within your Magical Focus (cap { $cap })
# The Technique/Form baseline casting cap (no spell-specific requisites
# folded in), shown as a hover hint on a source group's header — a quick
# at-a-glance figure; the per-spell cap above is what actually gates a row.
spell-group-cap-tooltip = Spell-level cap: { $cap }
# Accessible label for the inline level field on a chosen General spell (its
# level is not fixed by the catalogue, so it is edited per row).
spell-general-level-label = General level
spell-add = Add spell
spell-none = — Select a spell —
# The spell-levels budget bar, laid out like the XP summary: the label, the used
# figure over the whole unconditional side, then Available, the base and any V/F
# bonus.
spell-levels-pool = Spell levels
spell-levels-available = Available: { $available }
# The base budget as its own entry, because it is NO LONGER the denominator beside
# the used figure (guided-creation-review-2026-08 #18): a magus past its Gauntlet is
# charged against base + post-Gauntlet levels, so pairing the used figure with the
# base alone displayed "150 / 120  Available: 0" — an overspend the engine never
# raised. The pair now closes as "150 / 150" and the base stands beside it, editable
# in the editor and read-only in the wizard (#19).
spell-levels-base-entry = Base
# A POSITIVE Virtue/Flaw contribution: an extra pool of levels spent before the
# base, so it reads used/amount exactly like a restricted XP pool.
spell-levels-bonus-pool = Virtues/Flaws: { $used } / { $amount }
# A NEGATIVE Virtue/Flaw modifier (Weak Parens): no pool to spend, so it is charged
# to the base and reported as the signed modifier. `$bonus` arrives already signed.
spell-levels-bonus = Virtues/Flaws: { $bonus }
# The levels of spells the magus's years past its Gauntlet bought: its chosen
# slice of the fungible 30 points a year, where each point "can be an experience
# point in an Art or Ability or one level of spell" (Core Rules.md:2471). Named
# the way the XP bar names the same block (`xp-pool-block-after-gauntlet`), so one
# block reads alike in both bars — which is why this label was renamed with it
# (guided-creation-review-2026-08 #14.4). READ-ONLY here: the split is chosen once on
# the Abilities step, because the magus phase order runs abilities, arts, spells, so
# moving a point back to experience here would retroactively shrink a pool spent
# two steps earlier. Unlike the V/F modifier these levels are already earned, so
# they raise Available instead of forming a pool of their own. Shown only when
# there are any, which is what keeps a magus at its Gauntlet unchanged.
spell-levels-post-gauntlet = After the Gauntlet: { $levels }
# Accessible name for the editable BASE spell-levels field; empty = use the type
# profile's default (shown as the field's placeholder). It overrides the profile
# base ALONE — the V/F modifier and the post-Gauntlet levels stay additive on top.
spell-levels-base-label = Spell-levels budget
spell-mastery-xp = Mastery XP: { $xp }
# The per-spell Spell-Mastery XP pool bar: how much of the pool is spent.
spell-mastery-pool = Mastery XP: { $used } / { $pool }
spell-mastery-floor = All spells mastered at { $score }
# The per-spell Spell-Mastery stepper.
spell-mastery-label = Mastery
# Named per row ($name is the spell's own display name) — same reasoning as
# ability-increment/-decrement above.
spell-mastery-increment = Increase spell mastery for { $name }
spell-mastery-decrement = Decrease spell mastery for { $name }
spell-mastery-abilities-label = Special abilities
spell-mastery-ability-add = Add special ability
# Generic per-row remove control across the item lists (Abilities, Virtues &
# Flaws, aging log, Personality Traits, Twilight Scars, Familiar traits/powers,
# Magic Devices, Talisman attunements/effects, Reputations, Spells and their
# Mastery special abilities): names the row's own content so a screen reader
# does not hear the same bare "Remove" repeated on every row of a long list.
remove-item = Remove { $name }
# Details tab: age, Confidence (read-only, derived), Personality Traits, Reputations.
age-label = Age
apparent-age-label = Apparent age
confidence-label = Confidence
confidence-readout = Score { $score }, Points { $points }
warping-label = Warping
warping-readout = Score { $score }, Points { $points }
warping-effect-label = Warping effect
warping-owed-label = Warping Virtues & Flaws
warping-owed-minor-flaws = { $count ->
    [one] { $count } Minor Flaw
   *[other] { $count } Minor Flaws
}
warping-owed-supernatural-virtues = { $count ->
    [one] { $count } supernatural Minor Virtue
   *[other] { $count } supernatural Minor Virtues
}
warping-owed-major-flaws = { $count ->
    [one] { $count } Major Flaw
   *[other] { $count } Major Flaws
}
warping-slot-minor-flaw = Minor Flaw
warping-slot-supernatural-virtue = Supernatural Minor Virtue
warping-slot-major-flaw = Major Flaw
warping-choose-prompt = Choose…
true-faith-label = True Faith
true-faith-readout = Score { $score }
decrepitude-label = Decrepitude
decrepitude-readout = Score { $score }
decrepitude-effect-label = Decrepitude effect (overall)
item-levels-label = Enchanted Devices
item-levels-readout = { $levels } levels
# Identity / flavor fields (free-text, no mechanical effect).
identity-label = Identity
identity-name = Name
identity-name-placeholder = Character name
identity-description = Short description
identity-description-placeholder = e.g. Knight of the Teutonic Order, Crusader in the IVth Crusade
identity-concept = Concept
identity-concept-placeholder = Describe the character concept
identity-gender = Gender
identity-birth-year = Birth year
# D42: the concept's default realm — a default SOURCE for every Supernatural
# Virtue/Flaw's realm (ArMDE:2960), never the character's own realm.
identity-concept-realm = Default realm
identity-concept-realm-none = — None —
# The saga year (guided-creation-review-2026-08 #25). Part of the CHARACTER since
# C8 — the year the saga this one was built for stands in, and the year against
# which its age and birth year are two views of one fact. It was a machine-global
# app setting until then, which made it wrong for every saga but one.
saga-year-label = Saga year
identity-sigil = Wizard's sigil
identity-covenant = Covenant
identity-parens = Parens
# Directly-entered aged / warped state. The Decrepitude and Warping scores shown
# are computed by the engine from these points, never recomputed here.
aging-label = Aging
aging-points-heading = Aging points per Characteristic
# Shown once, after opening a save an older schema version wrote. The upgrade is
# LOSSY: the loader rebuilds the smallest Aging-Point total that still produces
# each recorded Characteristic score, so the original totals are gone, and the
# next Save writes the reconstruction back as the document's own figures. The
# user is told because it cannot be undone and the next keystroke makes it
# permanent. Separator as restricted-xp-list-separator, via Fluent rather than a
# hardcoded ", ".
aging-migration-notice = This character was saved in an older format. The Aging points for { $characteristics } were rebuilt as the smallest total that still produces the recorded scores, so the original figures could not be recovered. Saving will keep the rebuilt values.
aging-migration-list-separator = ,
# Shown after opening a save whose parameterized Ability value (an Area, a
# language, …) did not spell out any catalogue entry's name in either locale
# (CV4b, design-cv-catalogued-values.md § 5.5). A Literal instance is satisfied
# only by a recognized catalogue value, so an unrecognized one can silently
# stop authorizing or funding what it used to. Separator as
# restricted-xp-list-separator, via Fluent rather than a hardcoded ", ".
unresolved-catalogued-parameter-notice = This character was saved with a value the rules catalogue does not recognize: { $items }. It was kept exactly as typed, but check that anything relying on it still works.
unresolved-catalogued-parameter-item = { $ability } ("{ $text }")
unresolved-catalogued-parameter-list-separator = ,
# The positive counterpart: a value WAS recognized and linked to its catalogue
# entry, so a rename or a locale switch still resolves correctly from here on.
migrated-catalogued-parameter-notice = { $items } were recognized from what you typed and are now linked to their catalogue entry.
migrated-catalogued-parameter-item = { $ability }: "{ $text }" → { $resolved }
migrated-catalogued-parameter-list-separator = ,
warping-points-label = Warping points
twilight-scars-label = Twilight Scars
twilight-scar-placeholder = Describe the scar
twilight-scar-add = Add Twilight Scar
twilight-scars-empty = No Twilight Scars yet.
aging-log-heading = Aging log (per year)
aging-log-year-label = Year
aging-log-effect-placeholder = Describe the aging roll's effect
aging-log-add = Add aging entry
aging-log-empty = No aging entries yet.
# What a resolved year did, read back off the fields the engine recorded — the die
# the player typed, the total it made, the points it awarded and the year of
# apparent age it cost. Rendered, never stored: a save keeps choices, and stored
# prose would freeze one language into the file. The points and the apparent-age
# sentence are the calculator's own (`aging-outcome-*`), so a roll and its record
# read the same. On such a row the free-text box stops asking for what this line
# already says and offers itself as a note.
aging-log-roll = Aging total { $total } on a stress die of { $die }.
aging-log-points-none = No Aging Points.
aging-log-note-placeholder = Note (optional)
# A logged Crisis, read back off the entry that recorded it. Three states are
# tellable apart: no Crisis at all, one the table demanded that nobody has rolled,
# and one resolved against the Crisis Table (Core Rules.md:16619-16632).
aging-log-crisis-unrolled = Crisis: owed, and the simple die has not been rolled.
aging-log-crisis = Crisis: { $row } — crisis total { $total } on a simple die of { $die }.
aging-log-crisis-severity = Crisis: { $row } ({ $severity }) — crisis total { $total } on a simple die of { $die }.
# The aging schedule read-out: which years a character owes a roll for, how many
# are already recorded, and the standing (die-independent) half of the AGING
# TOTAL. Every figure comes from the engine's own readout — the threshold is read
# out of the rules, never printed here as a literal.
aging-schedule-label = Aging rolls
aging-rolls-none = No aging rolls are owed yet.
aging-first-roll-age = Aging begins after age { $begins }; the first roll falls at { $first }.
aging-rolls-owed = { $count ->
    [one] { $count } aging roll owed, at age { $from }.
   *[other] { $count } aging rolls owed, for ages { $from } to { $to }.
}
aging-rolls-years = { $count ->
    [one] Calendar year { $from }.
   *[other] Calendar years { $from } to { $to }.
}
aging-rolls-recorded = { $recorded } of { $owed } recorded
# AGING TOTAL = stress die (no botch) + age/10 (round up) - Living Conditions
# modifier - Longevity Ritual modifier (Core Rules.md:16567-16569), so a high
# modifier means a longer life. Each term arrives already signed.
# The fourth term is the Virtue/Flaw aging-roll modifier (Faerie Blood's -1,
# Core Rules.md:3801), which those three lines do not name but which the total on
# the right of the `=` includes — without it the sentence did not add up. Worded
# exactly as `aging-total-parts` below words it, so the two read-outs agree.
aging-total-formula = Stress die { $age } (age) { $conditions } (living conditions) { $longevity } (Longevity Ritual) { $traits } (Virtues and Flaws) = stress die { $fixed }
# The Living Conditions checklist (Core Rules.md:16581-16594). The total shown is
# the engine's own resolved modifier — it also carries the Virtue/Flaw
# contributions, which are not rows in this list.
living-conditions-label = Living Conditions
living-conditions-total = Living Conditions modifier: { $modifier }
living-conditions-cumulative-label = cumulative
# The aging roll calculator (Core Rules.md:16567-16615). The player rolls a stress
# die at the table and types it here — the app never rolls, and the die is never
# stored on the character. Every number shown is the engine's, and nothing is
# recorded until Apply.
aging-roll-label = Aging roll
aging-year-label = Year to roll for
aging-year-option = { $recorded ->
    [yes] Age { $age } — already recorded
   *[no] Age { $age }
}
aging-year-option-dated = { $recorded ->
    [yes] Age { $age } ({ $year }) — already recorded
   *[no] Age { $age } ({ $year })
}
aging-die-label = Stress die
aging-total-readout = Aging total: { $total }
aging-total-parts = { $die } (stress die) { $age } (age) { $conditions } (living conditions) { $longevity } (Longevity Ritual) { $traits } (Virtues and Flaws)
aging-die-capped = The Longevity Ritual caps this roll: { $uncapped } counts as { $total }.
aging-outcome-apparent_age = Apparent age increases by one year.
aging-outcome-no_apparent_aging = Apparent age does not advance.
aging-outcome-points_any = { $points ->
    [one] 1 Aging Point, in any Characteristic you choose.
   *[other] { $points } Aging Points, in any Characteristics you choose.
}
aging-outcome-points_fixed = { $points ->
    [one] 1 Aging Point in { $characteristic }.
   *[other] { $points } Aging Points in { $characteristic }.
}
aging-outcome-decrepitude_and_crisis = { $points } Aging Point(s) — enough to reach the next level of Decrepitude — and a Crisis.
aging-outcome-decrepitude_unpriceable = Enough Aging Points to reach the next level of Decrepitude, and a Crisis. The advancement table does not reach that level, so agree the number at the table.
# The Crisis (Core Rules.md:16619-16638). Two dice, both the player's: the stress
# die above sent the year here, and a simple die is thrown at the Crisis Table.
# The app never rolls either, never throws the survival roll, and never decides
# whether the character lives.
crisis-label = Crisis
crisis-die-label = Simple die
crisis-die-unrolled = Until the simple die is entered, applying this year records the Crisis as owed and unrolled.
crisis-total-readout = Crisis total: { $total }
# The same quantity as a bare column label, for the exported sheet.
crisis-total-label = Crisis total
crisis-total-parts = { $die } (simple die) { $age } (age) { $decrepitude } (Decrepitude Score)
crisis-row-readout = Crisis Table: { $row }
crisis-row-readout-severity = Crisis Table: { $row } ({ $severity })
# The five illness ranks of Core Rules.md:16628-16632, as standalone labels.
crisis-severity-minor = Minor
crisis-severity-serious = Serious
crisis-severity-major = Major
crisis-severity-critical = Critical
crisis-severity-terminal = Terminal
crisis-bedridden = Bedridden is time rather than a roll: there is no survival roll to make and no spell level to reach.
crisis-survival-label = Surviving the crisis
crisis-survival-ease-factor = Stamina stress roll against an Ease Factor of { $ease }.
crisis-survival-no-roll = There is no survival roll for this crisis.
crisis-survival-ritual = A Momentary Creo Corpus Ritual of level { $level } resolves it instead.
crisis-modifier-row = { $source } { $amount }
crisis-modifier-bronze_cord = Bronze cord
crisis-modifier-total = Modifiers to the survival roll: { $total }
crisis-allowance-attendant = An attending doctor may roll { $characteristic } + { $ability } against an Ease Factor of { $ease }; on a success their { $ability } score is added to the survival roll, and on a botch { $botch } applies. Only one doctor may usefully attend.
# What an applied year has to tell the player, as opposed to what it wrote.
aging-note-longevity_ritual_spent = The Crisis spends the Longevity Ritual: it assures the character survives, but its power is gone and the focal ritual must be performed again. The entry is left as it stands — record the new ritual yourself.
aging-note-heavy_wound = This character sustains a Heavy Wound from the Crisis, in addition to any other result. The app records no wound — mark it on the health track yourself.
aging-distribute = { $points ->
    [one] Place 1 Aging Point in a Characteristic of your choice.
   *[other] Place { $points } Aging Points across any Characteristics you choose.
}
aging-distribute-remaining = { $placed } of { $owed } placed
aging-apply = Apply this year
aging-revert = Take back age { $age }
aging-calculator-clear = Clear this roll
personality-label = Personality Traits
personality-name-placeholder = Trait
# Accessible name for the bound value input (S29, full-audit UX): each row's
# input shares no visible label of its own, so it must name which trait it
# scores, mirroring characteristic-description-for's shape.
personality-value-label = Value of { $name }
personality-add = Add trait
personality-empty = No Personality Traits yet.
reputations-label = Reputations
reputation-content-placeholder = What it is for
# A granted Reputation names the Virtue or Flaw that opened the slot, so
# "Ecclesiastical 4" is never an unexplained row. There is no add control any
# more: the grant IS the row.
reputation-granted-by = level { $score } from { $source }
reputation-granted-by-ranged = from { $source }
reputation-level-increment = Raise the { $source } Reputation
reputation-level-decrement = Lower the { $source } Reputation
# A grant that fixes no type (Famous) leaves the type to the player.
reputation-kind-label = Type
reputation-kind-choose = Choose a type
reputation-empty = No Reputation is granted (take a Virtue or Flaw that grants one).
reputation-type-local = Local
reputation-type-ecclesiastical = Ecclesiastical
reputation-type-hermetic = Hermetic
reputation-type-academic = Academic
# Magic Items tab (magi): aura, enchanted devices, familiar bond cords, the
# talisman and the Longevity Ritual. The item-level budget used/remaining comes
# from the engine, never recomputed here.
aura-label = Assumed lab/covenant aura
aura-out-of-range = Outside the rules range ({ $min } to { $max }); the value was adjusted to the nearest legal one.
possessions-devices-label = Enchanted Devices
device-name-placeholder = Device name
device-level-label = Level
device-add = Add device
devices-empty = No enchanted devices yet.
item-level-used = Item levels: { $used } / { $budget }
# Supernatural being (Might Score + powers): budget and effective values are
# engine-authoritative, never recomputed here.
supernatural-might-label = Might Score
might-realm-label = Realm
might-score-label = Base Might Score
might-add = Add Might Score
might-clear = Remove Might Score
might-empty = No Might Score yet (a Might Virtue grants one).
might-effective = Effective Might: { $realm } { $score }
might-mr = Magic Resistance (from Might): { $total }
supernatural-powers-label = Supernatural Powers
power-levels-used = Power levels: { $used } / { $budget }
power-name-placeholder = Power name
power-level-label = Level
# Levels spent one-for-one on Penetration, out of the SAME budget as the level
# (Core Rules.md:4019), which is why the bar above counts both.
power-penetration-label = Penetration
power-add = Add power
powers-empty = No supernatural powers yet.
# Focus Power (ArMDE:3895-3903): a SECOND power currency. Its 25 points buy a
# maximum level of effect at 2 points each and Penetration at 1 each, so its
# level column is a ceiling, not a level that was spent — hence its own list,
# its own bar and its own word for the level.
focus-powers-label = Focus Powers
focus-points-used = Focus points: { $used } / { $budget }
focus-power-name-placeholder = Focus power scope
focus-power-max-level-label = Max. level of effect
focus-power-initiative-label = Initiative
focus-power-fatigue-label = Fatigue levels
# Above level 75 the rulebook states no Fatigue cost (ArMDE:3901).
focus-power-fatigue-unstated = n/a
focus-power-derived = Magnitude { $magnitude }, Initiative { $initiative }, Fatigue { $fatigue }
focus-power-add = Add focus power
focus-powers-empty = No focus powers yet.
realm-magic = Magic
realm-faerie = Faerie
realm-divine = Divine
realm-infernal = Infernal
# D42: the V/F row's resolved-realm label (read-only for a Fixed entry) and
# its override control's unset option.
vf-realm-label = Realm
vf-realm-override-none = — Unset —
familiar-label = Familiar
familiar-name-placeholder = Familiar name
familiar-cord-gold = Gold cord
familiar-cord-silver = Silver cord
familiar-cord-bronze = Bronze cord
familiar-add = Add familiar
familiar-remove = Remove familiar
# Confirmation shown before removing the familiar (S18): the whole statblock —
# name, Might, Characteristics, personality traits, cords, powers — is
# discarded in one step, with no undo.
familiar-remove-confirm-title = Remove familiar?
familiar-remove-confirm-message = This deletes the familiar's entire statblock — name, Magic Might, Characteristics, Personality Traits, cords, and invested powers. This cannot be undone.
familiar-remove-confirm-confirm = Remove familiar
familiar-remove-confirm-cancel = Cancel
# The familiar's own creature statblock. Its Characteristics are the beast's, not
# bought from the magus's points, and its Might takes no Virtue grants on top —
# which is why it has its own score label rather than reusing might-score-label
# ("Base Might Score"). The invested-powers list deliberately has NO budget bar.
familiar-animal-label = Animal
familiar-animal-placeholder = e.g. a raven
familiar-size-label = Size
familiar-might-label = Magic Might
familiar-might-score-label = Might Score
familiar-might-add = Add Magic Might
familiar-might-clear = Remove Magic Might
familiar-might-empty = No Magic Might entered.
familiar-characteristics-note = The beast's own scores — not bought from the magus's Characteristic points.
familiar-powers-label = Invested Powers
familiar-bond-note = The bond grants both partners the Minor Virtue True Friend and the Personality Trait Loyal (partner) +3. A familiar lacking human intelligence gains it at Intelligence -3. These are not applied automatically.
# Talisman: the magus's personal enchanted item. Its capacity is engine-derived
# guidance (highest Technique + highest Form, in pawns of Vim vis) and is never
# recomputed here; the instilled effects are charged against no budget.
talisman-label = Talisman
talisman-add = Add talisman
talisman-remove-item = Remove talisman
# Confirmation shown before removing the talisman (S30): its identity,
# attunements, and instilled effects are discarded in one step, with no undo.
talisman-remove-confirm-title = Remove talisman?
talisman-remove-confirm-message = This deletes the talisman's shape and material, its attunements, and its instilled effects. This cannot be undone.
talisman-remove-confirm-confirm = Remove talisman
talisman-remove-confirm-cancel = Cancel
talisman-empty-item = No talisman yet.
talisman-description-label = Shape and material
talisman-description-placeholder = e.g. an ash staff shod with silver
talisman-capacity = { $pawns ->
    [one] Capacity: { $pawns } pawn of Vim vis
   *[other] Capacity: { $pawns } pawns of Vim vis
}
# Both Arts are named beside their scores so the derivation can be checked against
# the character sheet; the names come from the rules i18n, never as a raw slug.
talisman-capacity-note = Highest Technique { $technique } { $techniqueScore } + highest Form { $form } { $formScore }
talisman-attunements-label = Talisman Attunements
talisman-desc-placeholder = What it enhances
talisman-bonus-label = Bonus
talisman-attunement-add = Add attunement
talisman-empty = No talisman attunements yet.
talisman-effects-label = Instilled Effects
talisman-effect-name-placeholder = Effect name
talisman-effect-level-label = Level
talisman-effect-add = Add effect
talisman-effects-empty = No instilled effects yet.
longevity-label = Longevity Ritual
longevity-source-label = Source
longevity-source-self_made = Self-made
longevity-source-external = External
longevity-bonus-label = Aging bonus
longevity-add = Add Longevity Ritual
longevity-remove = Remove Longevity Ritual
# The stored bonus is empty: the ritual was made in a past season, so the value is
# entered, never derived. Distinguishes an unfilled field from a deliberate 0.
longevity-not-entered = Not entered
# The live suggestion beside the input: what a ritual made now would be worth.
# { $bonus } is already signed; { $total } is today's Creo Corpus Lab Total. The
# quantity is named ("aging bonus") because it is the STORED magnitude to type into
# the field — the totals panel shows the same number as an aging-roll modifier (-7).
longevity-hint = A ritual made now: aging bonus { $bonus } (Creo Corpus Lab Total { $total })
longevity-hint-halved = halved
longevity-focus-label = Focus
longevity-focus-placeholder = How the ritual culminates
# Shown as the reason a Supernatural Ability is greyed in the picker.
ability-requires-virtue = Requires a granting Virtue (or the Gift's one free Ability)
# Screen-reader-only text on a selected Ability row an error-severity issue
# points at (S7, full-audit a11y) — pairs with a visible glyph so the row's
# invalidity is never colour-only (WCAG 1.4.1).
ability-invalid-selection = Invalid selection
# Marks a display-only Ability row (#17): a Virtue gives this Ability a bonus or a
# free starting score, but no score has been bought, so the row shows the bonus and
# offers no controls. Says why in words — the greyed stepper alone is not a reason
# (WCAG 1.4.1). Add the Ability from the Available list to start buying it.
ability-unbought-marker = Not bought
# Named per row ($name is the characteristic's, or personality trait's, own
# display name — this key is shared by CharacteristicPicker, FamiliarPanel's
# personality-trait spinner, and PersonalityTraits) — same reasoning as
# ability-increment/-decrement above.
characteristic-increment = Raise { $name }
characteristic-decrement = Lower { $name }
characteristic-description-label = Description
# Accessible name for a description input: the eight per-Characteristic fields
# share the placeholder above, so each needs its own name naming which
# Characteristic it describes.
characteristic-description-for = Description of { $name }

# Hermetic House selector + per-grant specialisation pickers. House display
# names come from the rules i18n (keyed by house id), not from these chrome keys.
house-label = House
house-none = — None —
# A fixed grant shown read-only (e.g. Bjornaer's Heartbeast).
house-granted-label = Granted
# Prompt shown as the empty option of a choice/open specialisation picker.
house-choose-prompt = Choose…
# Title of the right column (the selected House's description + its grants).
house-grants-title = House details
# Shown in the right column when no House has been chosen yet.
house-none-selected = Select a House to see its details.

# Mythic Companion type selector (mythic-companion-only). Type names come from
# the rules i18n (keyed by mythic_type id), not from these chrome keys.
mythic-type-label = Mythic Companion type
mythic-type-none = — None —
# A free status/Minor Virtue the type grants, shown read-only.
mythic-granted-label = Granted
# Prompt shown as the empty option of the free-Minor choice picker.
mythic-choose-prompt = Choose…
# Label beside a required Flaw whose default may be swapped for a substitute.
mythic-required-flaw-label = Required Flaw

balance-virtues = Virtues: { $used } / { $budget }
balance-flaws = Flaws: { $used } / { $budget }

# Header actions. New/Save/Save As/Export lost their keys with the toolbar C3c
# removed: those five are the native menu's now, and it has its own `menu-*` keys
# below. `action-open` stays because the STARTUP SCREEN offers Open in its own
# right, on a screen that has no menu-shaped equivalent to lean on.
action-open = Open
# Takes the character already on screen into the guided flow, resuming from the
# furthest step its file recorded. Offered only where that flow can be walked.
action-continue-in-wizard = Continue in guided creation

# Native application menu. The menu's shape lives in Rust
# (`crates/arm-app/src/menu.rs`); every word of it comes from here, resolved by
# `ui/src/lib/menu.ts` and pushed across the IPC boundary, so nothing about the
# menu is authored in Rust. A trailing ellipsis marks an item that opens a
# dialog rather than acting at once. The macOS application menu is titled with
# `app-title` and needs no key of its own.
menu-file = File
menu-edit = Edit
menu-window = Window
menu-new = New
menu-open = Open…
menu-save = Save
menu-save-as = Save As…
menu-export = Export as Markdown…
menu-settings = Settings…
menu-quit = Quit
menu-services = Services
menu-hide = Hide
menu-hide-others = Hide Others
menu-show-all = Show All
menu-undo = Undo
menu-redo = Redo
menu-cut = Cut
menu-copy = Copy
menu-paste = Paste
menu-select-all = Select All
menu-minimize = Minimize
menu-fullscreen = Fullscreen
menu-close-window = Close Window

# Confirmation shown when closing or quitting the app with unsaved changes.
close-unsaved-title = Unsaved changes
close-unsaved-message = This character has unsaved changes. If you close now, they will be lost.
close-unsaved-discard = Discard and close
close-unsaved-cancel = Cancel

# Confirmation shown when starting a new document or opening another file while
# the current one has unsaved changes.
discard-changes-title = Unsaved changes
discard-changes-message = This character has unsaved changes. If you continue, they will be lost.
discard-changes-confirm = Discard changes
discard-changes-cancel = Cancel

# Window title. { $name } is the character's own name, falling back to the
# open file's name, falling back to `app-title-untitled` below when neither is
# set; { $app } is the application name. The dirty variant prepends an ASCII
# marker for a document with unsaved edits — including a brand-new, never-saved
# one (P3/U3, `docs/open-todos.md`).
app-title-document = { $name } — { $app }
app-title-document-dirty = *{ $name } — { $app }

# The window title's own placeholder name, for a character with no name of its
# own and no save file yet.
app-title-untitled = Untitled

# Hint shown for an unfilled parameter slot in an item name, e.g. "Puissant (Ability)".
param-hint = ({ $label })
# review-ui-today finding 2: an accessible separator label ParameterPicker shows
# before each of an item's `unordered_param_groups` groups (Incompatible Arts'
# two Technique+Form combinations) — generic and never names the item itself.
param-group-label = Combination { $n }
# a11y-d81 finding 2: a parameter option a sibling copy of the same item blocks.
# The reason sits in the option's own text, since an <option>'s title is
# mouse-hover only. { $label } is the option's name, { $reason } the
# vf-blocked-incompatible text.
param-option-blocked = { $label } ({ $reason })
# A requirement the engine enforces more widely than the rules word it. The rules'
# own example heads the requirement so its score follows it directly ("Latin 1", as
# the rulebook states it) and THIS note trails the score, saying what the engine
# really checks: "below Latin 1 (any Dead Language)". It leads with a space of its
# own because the messages carrying it interpolate it with no separator — it is
# empty for a requirement that names no example.
requirement-exemplar = { " " }(any { $ability })
# Localized parameter labels, keyed by the engine's parameter key. Also used as the
# type-aware placeholder/prompt for an empty parameter input.
param-label-ability = Ability
param-label-technique = Technique
# Incompatible Arts' two Technique+Form combinations (D81.8, ArMDE:6290-6292).
param-label-technique_1 = Technique 1
param-label-technique_2 = Technique 2
param-label-art = Art
param-label-focus = Focus
param-label-area = Area
param-label-language = Language
param-label-characteristic = Characteristic
# Cyclic Magic (Negative)'s cycle type — solar/lunar/seasonal (ArMDE:5893-5896,
# X7a/D52).
param-label-cycle = Cycle
# Magical Blood's four bloodline sub-types — Magic Animal/Human/Spirit/Thing
# (ArMDE:4359-4372).
param-label-bloodline = Bloodline
param-label-organization = Organization
param-label-mystery_cult = Mystery Cult
param-label-craft = Craft
param-label-guild = Guild
param-label-profession = Profession
param-label-form = Form
# Incompatible Arts' two Technique+Form combinations (D81.8, ArMDE:6290-6292).
param-label-form_1 = Form 1
param-label-form_2 = Form 2
# For the `item` domain: no shipped catalogue entry declares one yet, but the domain
# is part of the engine's closed enum and its picker branch names its control here
# rather than falling back to the raw key.
param-label-item = Item
param-label-realm = Realm
param-label-land = Land
param-label-being = Beings
param-label-terrain = Terrain
param-label-subject = Subject
param-label-sin = Sin
param-label-faculty = Faculty
param-label-hermetic_flaw = Hermetic Flaw
param-label-commodity = Commodity
param-label-company = Company
param-label-role = Role
param-label-power = Power
# "Vulnerable Magic" — "so long as a different condition is specified for
# each" (Ars Magica - Definitive Edition (Core Rules).md:7009).
param-label-condition = Condition
# "Greater Immunity" — "with a different immunity each time" (Ars Magica -
# Definitive Edition (Core Rules).md:4015).
param-label-hazard = Hazard
# "Social Contacts" — "each time specifying a different social group" (Ars
# Magica - Definitive Edition (Core Rules).md:4990).
param-label-social_group = Social Group
# The Supernatural Virtue a False Power taints (Ars Magica - Definitive Edition
# (Core Rules).md:6096).
param-label-virtue = Virtue
# Folk Magic's narrow area of spells — one of the four the book prints.
param-label-category = Category
# Which of an item's own listed categories it was taken as — Sufi, "either as
# a Minor Social Status Virtue or a Minor Supernatural Virtue" (Ars Magica -
# Definitive Edition (Core Rules).md:5083).
param-label-taken_as = Taken as
# Which restricted group of Abilities Custos/Templar Specialist/Wise One
# studied — the exclusive choice their `ability_authorization` gate reads.
param-label-study = Study
# Simple Student's finished-years count (ArMDE:4960); the cap (2 years, 60 xp)
# is the parameter's own min/max, enforced by the engine.
param-label-years = Years
# Abandoned Apprentice's years of apprenticeship completed before abandonment
# (ArMDE:5641-5650, D56/D62/D3); the cap (apprenticeship.years - 1) is the
# parameter's own min/max, enforced by the engine.
param-label-years_completed = Years completed
param-label-years_since_resurrection = Years since resurrection
# The three Corrupted entries' multi-valued choice (D9 part 3, D15,
# ArMDE:5847-5864) — which Abilities/Arts/spells the Flaw affects.
param-label-targets = Targets
# Enchanting (Ability)'s player-chosen artistic medium (ArMDE:3747-3750, F-63)
# — "music, dance, drawing, storytelling, even craftwork".
param-label-medium = Medium

# X6b's 16 rule-driving parameters (design-x6-parameters.md § 2).
# Commanding Aura's rank (ArMDE:3585-3591, :17651).
param-label-rank = Rank
# Savantism's favored Ability (ArMDE:6703-6708).
param-label-favored = Favored Ability
# Restricted Learning's five named Abilities, and Magian Lineage (Major)'s
# three connected ones (ArMDE:6685, :4345).
param-label-abilities = Abilities
# Faerie Blood / Strong Faerie Blood's type of fay heritage (ArMDE:3805-3819).
param-label-heritage = Heritage
# Strong Faerie Blood's unconditional physical quirk (ArMDE:5042).
param-label-quirk = Physical Quirk
# Ability Block's either/or discriminator between a whole category and a
# custom, narrower set (ArMDE:5651-5654).
param-label-scope = Scope
param-label-class = Ability Category
# Shared "or create/describe your own" free-text sibling — Faerie Blood's
# heritage, Monstrous Blood's bloodline, Ability Block's limited set, and
# Repellent's feature all reuse this one key (D9's E+ idiom).
param-label-custom = Custom
# Repellent's minor advantage (ArMDE:6681, Q-X6-3).
param-label-feature = Feature
# Warped Senses' sense/environmental affliction (ArMDE:7027-7051).
param-label-affliction = Affliction
# Potent Magic's field (ArMDE:4742-4746).
param-label-field = Field
# Special Circumstances' circumstance (ArMDE:4998-5001, F-541/F-287).
param-label-circumstance = Circumstance

# X6c's label-only parameters (`tmp/x6c-verdicts.md`) — none drive a
# computed rule; D9 still records the choice for the sheet/export.
# Greater Purifying Touch's cured disease (ArMDE:4027-4030).
param-label-disease = Disease
# Lesser Purifying Touch's healed illness (ArMDE:4287-4290).
param-label-illness = Illness
# Lesser Benediction's blessing, one of four named examples or custom
# (ArMDE:4253-4274).
param-label-benediction = Benediction
# Necessary Condition's required action while casting (ArMDE:6476-6479).
param-label-action = Action
# Supernatural Nuisance's kind of interfering entity, and Poor Memory's kind
# of forgotten thing (ArMDE:6799-6802, :6622-6625).
param-label-kind = Kind
# Curse of Slander's targeted section of mundane society, additive to its
# existing `taken_as` category choice (ArMDE:5881-5884).
param-label-section = Section
# Lycanthrope's predator form (ArMDE:6370-6377).
param-label-predator = Predator
# Paid Rights' purchased right (ArMDE:4606-4615).
param-label-right = Right
# Fida'i/Lasiq's optional cover social status — the ONE parameter D80 makes
# never required (ArMDE:3877-3882, :4233-4236).
param-label-cover = Cover Social Status
# Templar Office Holder's held position (ArMDE:5121-5124).
param-label-position = Position

# C5b: the multi-select checklist (a `multi_ref` parameter, D9 part 3) shown
# instead of a bare, unexplained empty list when there is nothing to choose
# from — e.g. Corrupted Spells before the character has learned any spells.
param-multi-ref-empty = No options available yet.

# CV7: the Ability parameter picker's combo box (design-cv-catalogued-values.md
# § 6.1/§ 6.3/§ 6.4). The free-text escape when neither a catalogue value nor a
# linked Virtue names the player's answer.
ability-param-other = Other…
# A linked value's indicator, distinguishing "this follows a Virtue" from a
# plain typed value that happens to currently agree with it — $item is the
# source item's own localized name, $value its current resolved text.
ability-param-follows = Follows { $item }: { $value }
# The SAME indicator, when the link's source cannot currently be resolved
# (removed, or held more than once) — visibly distinct from the "fine" chip
# above so a broken link is never confused with a working one. The
# `issue-ambiguous_bound_parameter` validation issue explains why.
ability-param-unresolved = Follows { $item } (unresolved)
# Conditional hint (design § 11 item 2): shown only when the bought free-text
# value leaves a Literal or Bound instance from one of the character's own
# items unmet — the free-text choice is costing the player an authorization a
# catalogue value or a link would grant instead.
ability-param-hint = A catalogue value or a linked Virtue may authorize something this free text does not.

# Localized names for the engine's `ParameterDomain` variants — the kind of value a
# parameter slot accepts. Read only by `unknown_param_value`, which names the domain a
# stored value failed to resolve in; the engine emits the enum's serialized name, and
# a slug must never reach the screen. One key per variant, so a new engine variant is
# a missing string here rather than a leaked `realm`.
param-domain-ability = Ability
param-domain-art = Art
param-domain-technique = Technique
param-domain-form = Form
param-domain-characteristic = Characteristic
# The `item` domain resolves against the point-item registry, which is the Virtue and
# Flaw catalogue — so this names what the player would recognize, not the internal term.
param-domain-item = Virtue or Flaw
# The parameter carries its own closed list of legal values.
param-domain-enumerated = listed value
param-domain-category = Category
param-domain-realm = Realm
param-domain-text = Text
param-domain-number = Number
param-domain-spell = Spell
param-domain-ability_category = Ability Category

# Localized names for the engine's `ItemPredicate` variants — a property-based
# test over a point item (D23/D33/D68.4/D69.5), read only by
# `excluded_by_predicate`'s `$predicate` arg. The engine emits the enum's
# serialized name, and a slug must never reach the screen.
predicate-trained = Requires Hermetic Training
predicate-grants_reputation = Grants a Reputation
predicate-grants_personality_trait = Grants a Personality Trait
predicate-requires_hermetic_arts = Requires Hermetic Arts
predicate-affects_size = Affects Size

# Effective score shown beside a base score when a virtue bonus applies.
effective-score = { $score }
# The same badge where the base score is shown right beside it, so the pair reads
# as a change rather than as two unrelated numbers. The arrow is user-facing text
# standing in for a word ("becomes"), exactly like the `&` in derived-combat-and,
# so it is translatable: a locale may replace it with a word or another mark.
effective-score-from = → { $score }

empty-selections-side = None yet.
no-issues = No issues.
loading = Loading…
# Shown by SourcePicker when the search/filter combination excludes every row
# of the catalogue (S25, full-audit UX) — otherwise the list area renders
# completely blank, indistinguishable from a slow-loading or broken panel.
filter-no-results = No matches for the current filter.

# Screen-reader-only severity prefix on each validation issue
# (ValidationPanel.svelte): color/border alone must not be the only signal
# (WCAG 1.4.1).
issue-severity-error = Error
issue-severity-warning = Warning

# Appended to a finding a wizard step shows because the item it names was chosen
# HERE, while the engine files it on the step that owns the offending value (a
# Great/Poor Characteristic Virtue, whose value is a Characteristic score). Such a
# finding reads as an error yet does not disable Next, so it must say where the
# fix lives. `$step` is always a `phase-<slug>` label, never the slug itself.
issue-other-step = Resolve on the { $step } step.

# One key per validation issue code emitted by the engine. Each message
# interpolates the engine's `args` (see crates/arm-rules/src/validation.rs):
# arg names are stable per code and passed through verbatim by the UI.
issue-over_budget_virtues = Virtue points ({ $points }) exceed budget ({ $budget }).
issue-over_budget_flaws = Flaw points ({ $points }) exceed budget ({ $budget }).
issue-unbalanced_virtues = Virtue points ({ $virtue_points }) exceed Flaw points ({ $flaw_points }); Virtues must be funded by Flaws.
issue-too_many_major_virtues = Too many Major Virtues ({ $count } of max { $max }).
issue-too_many_major_hermetic_virtues = Too many Major Hermetic Virtues ({ $count } of max { $max }).
issue-too_few_social_status_virtues = Too few Social Status Virtues or Flaws ({ $count } of min { $min }).
issue-too_many_social_status_virtues = More than one Social Status Virtue or Flaw taken ({ $item }, { $other }).
issue-too_many_major_flaws = Too many Major Flaws ({ $count } of max { $max }).
issue-too_many_minor_flaws = Too many Minor Flaws ({ $count } of max { $max }).
issue-too_many_major_personality_flaws = Too many Major Personality Flaws ({ $count } of max { $max }).
issue-too_many_personality_flaws = More Personality Flaws than recommended ({ $count } of { $max }).
issue-too_many_story_flaws = More Story Flaws than recommended ({ $count } of { $max }).
issue-too_many_tainted_virtues = More than half your Virtue points are Tainted ({ $tainted } of { $total }).
issue-too_many_tainted_flaws = More than half your Flaw points are Tainted ({ $tainted } of { $total }).
issue-too_large_share = { $item } accounts for more of your points than the rules allow it to ({ $points } of { $total }).
issue-prereq_not_met = Prerequisite not met for { $item }.
issue-prereq_unevaluated = Prerequisite for { $item } could not be checked yet.
issue-advisory_prereq_not_met = Prerequisite for { $item } is not normally met.
issue-incompatible = { $item } is incompatible with { $other }.
issue-forbidden_category = { $item } belongs to a forbidden category ({ $category }).
issue-category_not_permitted = { $item } is not in a permitted category ({ $category }).
issue-category_forbidden_by_effect = { $item } is forbidden while { $other } is in effect (category { $category }).
issue-ability_forbidden_by_effect = { $ability } is forbidden while { $other } is in effect.
issue-excluded_by_predicate = { $item } is forbidden while { $other } is in effect ({ $predicate }).
issue-same_choice_conflict = { $item } and { $other } may not both target { $target }.
issue-wrong_entity_kind = { $item } cannot be taken by a { $entity_kind }.
issue-duplicate_selection = { $item } is selected { $count } times, but may be taken at most { $max } time(s) for the same target.
issue-too_many_selections = { $item } is selected { $count } times in total across all targets, but may be taken at most { $max } time(s) altogether.
issue-param_groups_not_distinct = { $item } names the same group twice; a copy's groups must be pairwise distinct.
issue-missing_required_trait = A required trait is missing: { $item }.
issue-forbidden_trait = A forbidden trait is present: { $item }.
issue-missing_param = { $item } is missing the parameter { $key }.
issue-unexpected_param = { $item } has an unexpected parameter { $key }.
# The stored value is quoted back verbatim: after E2 this is what an older save's
# free-text realm word surfaces as, and showing it is the whole mechanism by which
# the player's choice is not lost — they read what they had typed and pick the
# matching value. `$domain` is now a word (`param-domain-<id>`), never the enum slug.
issue-unknown_param_value = { $item } parameter { $key }: unknown value { $value } (expected: { $domain }).
issue-param_wrong_shape = { $item } parameter { $key } names a single value, but this parameter now accepts a set — choose again.
# Core Rules.md:3919 — "a character cannot have access to both the Divine and
# Infernal Realms". Which values exclude each other is rules data
# (`ParameterDef.at_most_one_of`), never a hardcoded pair, so this message names
# the item and the slot and counts the copies rather than naming the values: a
# value's label depends on its domain, and a per-domain argument would make
# Fluent throw wherever that domain does not supply it.
issue-exclusive_param_values = { $item } names { $count } values for { $key } that the rules allow only one of.
# Core Rules.md:6482 — "A character may take this Flaw once for any particular
# Ability". The third and narrowest multiplicity axis, beside
# `duplicate_selection` (one identical target) and `too_many_selections` (copies
# in total): a cap on ONE parameter key's value. This one DOES name the value —
# there is exactly one of it, and "twice for something" is not a finding anyone
# can act on. "per { $key }" rather than "for the same { $key }" so the sentence
# needs no article before a word that comes from the data.
issue-too_many_for_param_value = { $item } is selected { $count } times for { $value }, but may be taken at most { $max } time(s) per { $key }.
# Core Rules.md:6096 — False Power is taken "once for each appropriate
# Supernatural Virtue that the character possesses". The first message is the
# unheld target, the second a Virtue two copies both claim.
issue-param_target_not_possessed = { $item }: { $key } names { $value }, which the character does not have.
issue-param_target_already_claimed = { $item }: { $key } names { $value }, which { $other } already names.
issue-multiple_magical_foci = A magus may have only one Magical Focus, but { $count } are selected.
issue-gift_required = This type requires The Gift.
issue-gift_forbidden = This type cannot have The Gift.
issue-unknown_ref = Unknown item reference: { $item }.
issue-unknown_type = Unknown entity type: { $type_id }.
# A warning, not an error: the save opens either way. It names both identities
# because the whole point is that the ids inside it still resolve — only the
# numbers behind them moved — so nothing else in the list can show the drift.
issue-ruleset_mismatch = Saved under ruleset { $saved_ruleset } { $saved_version }, but { $loaded_ruleset } { $loaded_version } is loaded. The values behind the same names may differ.
issue-characteristic_out_of_range = Characteristic { $characteristic } score { $score } is outside the allowed range ({ $min } to { $max }).
issue-characteristic_overspent = Characteristics cost { $cost } points, over the { $points } available.
issue-characteristic_points_unspent = Only { $cost } of { $points } Characteristic points spent.
issue-characteristic_above_cap = Characteristic { $characteristic } score { $score } exceeds its maximum of { $cap }; raise the cap with Great Characteristic.
issue-characteristic_below_floor = Characteristic { $characteristic } score { $score } is below its minimum of { $floor }; lower the floor with Poor Characteristic.
issue-characteristic_max_base_too_low = { $item } requires { $characteristic } to be at least { $min } (currently { $base }).
issue-characteristic_min_base_too_high = { $item } requires { $characteristic } to be at most { $max } (currently { $base }).
issue-unknown_ability = Unknown ability: { $ability }.
issue-duplicate_ability = { $ability } is listed { $count } times with the same specialty.
issue-not_enough_xp = Abilities cost { $spent } XP, more than the { $pool } in the pool.
issue-xp_solve_bound_exceeded = This character has too many Ability and Art scores and mastered spells ({ $spends } bought scores across { $pools } experience pools, { $nodes } in total) for experience to be allocated — the limit is { $limit }. This usually means the save file is damaged.
issue-restricted_xp_unspent = { $origin }: { $unspent } of { $amount } restricted experience points are unspent and will be wasted.
# guided-creation-review-2026-08 #30. Purely a count, and deliberately so: the
# restricted sibling above may say the points are wasted because childhood's blocks
# are spend-or-lose, but no passage in the Core Rules says the same of the general
# pool or of the 120 levels of spells. So these two state what is left and stop.
issue-general_xp_unspent = { $unspent } of { $pool } experience points are still unspent.
issue-spell_levels_unspent = { $unspent } of { $budget } levels of spells are still unspent.
issue-ability_category_requires_virtue = { $ability } is { $ability_category }, which needs a Virtue granting access at character creation.
issue-academic_ability_without_scholarly_language = An Academic Ability normally requires { $ability }{ $qualifier } at { $min } or better.
issue-life_stage_age_unset = Enter the character's age: later life earns experience per year, so with no age only childhood's blocks can be counted.
issue-life_stage_age_before_childhood = Age { $age } falls inside childhood, which lasts { $min } years — there are no later-life years to earn experience in.
issue-life_stage_age_before_gauntlet = No magus is gauntleted at { $age }: the Gauntlet comes no earlier than { $min } — childhood plus fifteen years of apprenticeship.
issue-life_stage_age_before_truncation = Age { $age } is too young to have completed { $min } years of childhood and apprenticeship-shaped training.
issue-life_stage_gauntlet_age_after_age = The Gauntlet at { $gauntlet_age } is still ahead of this magus, who is { $age }; the years as a magus are counted forward from the Gauntlet.
issue-life_stage_lab_seasons_out_of_range = { $seasons } lab seasons is more than the { $max } that { $years } year(s) as a magus can be charged for.
issue-life_stage_lab_seasons_without_years = { $seasons } lab season(s) recorded, but this character has no years as a magus to work them in.
issue-life_stage_spell_level_split_exceeds_points = Taking { $levels } level(s) of spells out of the years as a magus is more than those years grant: they are worth { $points } points, to be divided between experience and levels of spells.
issue-life_stage_native_language_unset = Choose a native language: childhood's largest block of experience can be spent on nothing else.
issue-life_stage_native_language_missing_score = No { $language } score is bought, so childhood's native-language experience is unspent.
issue-magus_minimum_ability = No magus is admitted to the Order below { $ability } { $min }{ $qualifier }; this character has { $score }.
issue-magus_recommended_ability = { $ability } { $min }{ $qualifier } is recommended for a magus just out of apprenticeship; this character has { $score }.
issue-childhood_package_unknown = Unknown childhood package: { $package }.
issue-childhood_slot_unfilled = Fill in the { $key } for { $ability } before applying the childhood package.
issue-childhood_slot_is_native_language = The { $key } for { $ability } must differ from the native language { $language }.
issue-childhood_slot_duplicate_value = The { $key } { $value } for { $ability } is already used by another entry of the childhood package; choose a different one.
issue-ability_parameter_required = { $ability } needs a value (e.g. the specific Area or Language).
issue-ability_score_out_of_range = Ability { $ability } score { $score } is outside the allowed range (0 to { $max }).
# `$ability` is the WHOLE Ability name, instance included ("Craft: Carpentry",
# "Brandenburg-Kunde"): the engine's `parameter` arg is folded into it by
# `resolveIssueArgs` before this message sees it, because where the instance sits
# inside the name is the Ability's own template and differs per language.
issue-ability_bonus_dangling_target = Add { $ability } to the character's Abilities; { $item } targets it.
issue-unknown_art = Unknown Art: { $art }.
issue-duplicate_art = { $art } is listed { $count } times.
issue-art_score_out_of_range = Art { $art } score { $score } is outside the allowed range (0 to { $max }).
# X10b. Shared by an Ability and an Art score, whose args carry `ability` or
# `art` respectively — never both — so the message interpolates neither
# directly and names only the two figures both cases share.
issue-banked_xp_at_or_above_next_level = Banked experience points ({ $banked }) are already enough to raise this score — the next level needs only { $needed }.
issue-house_choice_unresolved = House { $house } has an unresolved specialisation choice ({ $choice_key }).
issue-house_grant_constraint = The House { $house } grant { $choice_key } picks { $item }, which does not meet its constraint.
issue-warping_owed_minor_flaws = You still owe { $count } Minor Flaw(s) from Warping.
issue-warping_owed_supernatural_virtues = You still owe { $count } supernatural Minor Virtue(s) from Warping.
issue-warping_owed_major_flaws = You still owe { $count } Major Flaw(s) from Warping.
issue-warping_fill_constraint = The Warping fill { $choice_key } picks { $item }, which does not match the owed slot.
issue-warping_fill_ineligible = The Warping fill { $choice_key } picks { $item }, which itself grants Warping and cannot fill a Warping slot.
issue-warping_fill_excess = The Warping fill { $choice_key } is not owed and should be removed.
issue-realm_changed_default = { $item } is changed away from its usual realm; confirm this is intended.
issue-realm_unset_subset = { $item } needs a realm chosen from its restricted list; it cannot default to Magic.
issue-realm_override_invalid = { $item }'s realm must be one of its allowed realms, not { $value }.
issue-house_unset = A magus should belong to a Hermetic House.
issue-missing_hermetic_flaw = A magus should take at least one Hermetic Flaw.
issue-mythic_type_unset = A Mythic Companion should choose a type.
issue-mythic_choice_unresolved = The Mythic Companion type { $mythic_type } has an unresolved choice ({ $choice_key }).
issue-mythic_grant_constraint = The Mythic Companion type { $mythic_type } grant { $choice_key } picks { $item }, which does not meet its constraint.
issue-mythic_required_trait_missing = A required Virtue or Flaw (or a suitable substitute) is missing: { $item }.
issue-unknown_spell = Unknown spell: { $spell }.
issue-duplicate_spell = { $spell } is listed { $count } times.
issue-spell_level_unresolved = General spell { $spell } has no chosen level yet.
issue-over_spell_levels = Spells total { $used } levels, over the budget of { $budget } (by { $over }).
issue-spell_level_exceeds_cap = Spell { $spell } is level { $level }, above the maximum you can learn ({ $cap }).
issue-spell_ritual_legality = Spell { $spell } is learned at level { $level }, which breaks the ritual level bounds (rituals at least 20, non-rituals at most 50).
issue-ritual_casting_restricted = { $spell } is a Ritual, and Rigid Magic forbids using vis to cast it.
issue-spell_uses_incompatible_arts = { $spell } draws on two Techniques and Forms that Incompatible Arts forbids using together.
issue-spell_within_focus_without_magical_focus = { $spell } is marked within focus, but no Magical Focus is held; the marking has no effect.
issue-unknown_mastery_ability = Unknown Spell Mastery ability { $ability } chosen for { $spell }.
issue-too_many_mastery_abilities = { $spell } has more Mastery special abilities ({ $chosen }) than its Mastery score of { $mastery } allows (one per level).
issue-duplicate_mastery_ability = Mastery ability { $ability } is chosen { $count } times for { $spell }, but may be taken only once.
issue-too_many_of_mastery_ability = Mastery ability { $ability } is chosen { $count } times for { $spell }, above its maximum of { $max }.
issue-mastery_ability_forbidden_for_ritual = Mastery ability { $ability } may not be chosen for { $spell }, a Ritual spell.
issue-ability_above_age_cap = { $ability } score { $score } exceeds the age-{ $age } maximum of { $cap }.
issue-specialty_forbidden = { $ability } may not have a specialty ({ $specialty }) — Unspecialized forbids any Ability specialty.
issue-ambiguous_bound_parameter = { $item } is held more than once; { $ability }'s linked value cannot be resolved until the duplicate is removed.
issue-wrong_param_count = { $item } names { $count } value(s) for { $key }, but exactly { $expected } are required.
issue-ability_outside_restricted_scope = { $item } restricts experience to { $allowed }; { $ability } is outside that list.
issue-supernatural_ability_requires_virtue = { $ability } is a Supernatural Ability and requires a granting Virtue (or the Gift's one free Ability).
issue-personality_trait_out_of_range = Personality Trait { $name } ({ $value }) is outside the allowed range (±{ $max }).
issue-fickle_nature_trait_pair_missing = { $item } requires at least two Personality Traits at exactly { $value } (a matched pair).
issue-reputation_not_granted = A { $kind } Reputation ({ $content }) needs a Virtue or Flaw that grants one.
issue-reputation_score_out_of_range = A { $kind } Reputation ({ $content }) at level { $score } is outside the granted range ({ $min } to { $max }).
issue-over_item_level = Enchanted devices total { $used } levels, over the budget of { $budget } (by { $over }).
issue-over_power_levels = Supernatural powers total { $used } levels, over the budget of { $budget } (by { $over }).
issue-over_focus_points = Focus Powers spend { $used } points, over the pool of { $budget } (by { $over }).
# The named power is free text the player typed on the Supernatural tab, so the
# message quotes it back rather than trying to label it.
issue-power_dangling_target = { $item } names the power { $power }, which this character does not have; add it under Supernatural Powers or correct the name.
issue-might_realm_mismatch = The entered Might Realm ({ $base }) disagrees with the Realm its Virtues grant ({ $granted }).
# A character over 35 owes aging rolls before play begins (Core Rules.md:2232),
# and aging starts the Winter after 35 (`:16565`) — so this fires from 36 up. The
# rolls happen at the table, so the app can only say they are outstanding; a
# recorded aging log settles it, whatever the rolls produced.
issue-aging_rolls_pending = This character is { $age }, and a character over 35 must make aging rolls before play begins; none are recorded yet.
issue-unknown_living_condition = Living condition '{ $condition }' matches no row of the Living Conditions table, so it contributes nothing to the aging total.
# Only the asterisked rows of the table "are cumulative with each other"
# (`:16594`); the rest describe one situation each, so only one can apply.
issue-living_conditions_conflict = Living conditions '{ $condition }' and '{ $other }' are alternatives, so only one of them can apply.
# An aging roll advances the apparent age by at most one year per year (`:16577`),
# so it cannot outrun the actual age on its own — but `:5189` only says it "should
# be" less than or equal, and lets a character who is not basically human differ.
issue-apparent_age_above_age = The apparent age ({ $apparent_age }) is above the actual age ({ $age }); aging raises it by at most one year per year.
# Neither an entity state nor a rejected command: the age ↔ birth-year link derived
# an impossible pair, so the age clamped to 0 rather than underflowing (`age` is
# unsigned). Advisory — the pair is impossible, but nothing about it is illegal.
issue-saga_year_before_birth_year = The saga year ({ $saga_year }) is before the birth year ({ $birth_year }), so the character is not born yet; the age reads 0 until one of the two is changed.
# The six refusals an aging roll can meet. Unlike every finding above, these are
# command-input findings: the engine writes nothing when it refuses, so no saved
# character can hold one — each describes the roll just submitted.
issue-aging_rules_missing = These rules carry no aging table, so an aging roll cannot be resolved.
issue-aging_year_already_recorded = The aging roll for age { $age } is already recorded; take it back before rolling that year again.
issue-aging_distribution_mismatch = This roll leaves { $owed } aging point(s) for you to place, but { $distributed } were placed.
issue-aging_distribution_not_open = The table names this roll's Characteristics itself, so the { $count } you placed cannot be applied.
issue-aging_award_unpriceable = The next level of Decrepitude lies beyond the advancement table, so the points it costs cannot be determined.
issue-aging_year_not_recorded = No aging roll is recorded for age { $age }, so there is nothing to take back.
issue-unknown_equipment = Equipment '{ $item }' does not match any weapon, shield, or armor.
issue-equipment_min_strength = { $item } needs Strength { $required }, but this character has { $strength }.
issue-shield_with_two_handed_weapon = A shield cannot be used with a two-handed weapon, so its Attack and Defense modifiers are not applied (it still counts toward Load).

# Equipment (weapons / shields / armor). Combat totals, Soak, and Encumbrance
# are computed in a later slice; this surface only records the carried items.
weapon-kind-melee = Melee
weapon-kind-missile = Missile
weapon-kind-thrown = Thrown
equipment-group-weapons = Weapons
equipment-group-shields = Shields
equipment-group-armor = Armor
# Accessible name for the kind filter select, which carries no visible label.
equipment-group-filter-label = Filter by equipment type
# Accessible name for the tri-state loadout select (K5), which carries no
# visible label.
equipment-loadout-label = Loadout
# The three loadout states (K5): Stowed yields no Combat row and no Load;
# Carried yields a Combat row but no Load; Wielded yields both.
equipment-loadout-stowed = Stowed
equipment-loadout-carried = Carried
equipment-loadout-wielded = Wielded
equipment-specialization-label = Specialization applies (+1)
equipment-empty = No equipment.
# K3 (design-f0-book-template-engine.md § 2b): whole-character toggle, adds
# min(Ride, 3) to Attack/Defense on every combat line that is not a body
# attack (Fist/Kick/Dodge).
mounted-label = Mounted
# Appended to a combat line's name when it is the mounted twin (K3); the
# surrounding parentheses and leading space are composed where this is used.
derived-combat-mounted-suffix = mounted

# AppError kinds returned by Tauri commands.
error-io = A file could not be read or written.
error-ruleset = The ruleset could not be loaded.
error-not_loaded = No ruleset is loaded yet.
error-serialize = The character file could not be processed.
error-export = The character sheet could not be exported: the current language is missing text for { $missing }. Try switching to English and exporting again, or report this as a bug.
error-menu = The application menu could not be built.
# Label of the collapsed disclosure holding a failed ruleset's own integrity
# diagnostics (E4). The LABEL is localized; the diagnostics behind it are
# deliberately not — see `ErrorDetails.svelte`.
error-technical-details = Technical details

# Derived play-stat read-out (M5/5i). Read-only totals computed by the engine;
# the panel renders these numbers and computes no mechanics itself.
tab-totals = Totals
derived-section-summary = Summary
derived-section-lab = Lab Totals
derived-section-casting = Casting Totals
derived-section-penetration = Penetration
derived-section-magic-resistance = Magic Resistance
derived-section-longevity = Longevity Ritual
derived-section-masterpiece = Masterpiece
derived-section-familiar = Familiar Bond
derived-section-combat = Combat
derived-section-soak = Soak
derived-section-encumbrance = Encumbrance
derived-section-fatigue = Fatigue
derived-section-wounds = Wounds
derived-section-surfaced = Other modifiers
derived-section-decrepitude = Decrepitude
derived-section-warping = Warping
derived-size = Size
derived-aura-label = Assumed lab/covenant aura
derived-aura-out-of-range = Outside the rules range ({ $min } to { $max }); the value was adjusted to the nearest legal one.
derived-lab-enchanting = For enchanting
derived-lab-enchanting-hint = Halved from the Lab Total above for Weak Enchanter — use this figure when creating or investigating an enchanted item.
derived-within-focus = Within focus
# D79: the Potent-Magic-field figure beside the Magical-Focus one above,
# shown only when the character holds a Potent Magic Virtue.
derived-within-potent-field = Within Potent Magic field
derived-deficient = (deficient, halved)
# D81.8: a Lab/Casting Total grid cell (or spell) whose (Technique, Form) pair is
# barred by a held Incompatible Arts Flaw. The grid replaces the cell's number
# with this marker outright (0 is a legitimate total and would be indistinguishable
# from this); the Spells tab shows it next to the figure instead, since the number
# there still has informational value.
derived-unusable = Unusable
derived-unusable-tooltip = Incompatible Arts forbids using this Technique and Form together.
derived-weak-magic = (Weak Magic, halved)
derived-level = level
derived-range = Range
derived-load = Load
derived-burden = Burden
derived-lab-total = Lab Total
# On-demand Lab/Casting Total picker: choose a Technique and Form to see the
# single matching totals instead of the full combination tables.
derived-section-lab-casting = Lab & Casting Totals
derived-picker-technique = Technique
derived-picker-form = Form
derived-combat-empty = No weapons equipped.
derived-cast-formulaic = Formulaic
derived-cast-ritual = Ritual
derived-cast-spont-fatiguing = Spont. (fatiguing)
derived-cast-spont-non-fatiguing = Spont. (non-fatiguing)
derived-cast-non-standard = Non-standard
derived-cast-silent = No voice
derived-cast-still = No gestures
derived-cast-silent-still = No voice + gestures
derived-deft-form = (Deft Form)
derived-combat-init = Init
derived-combat-attack = Attack
derived-combat-defense = Defense
derived-combat-damage = Damage
# A derived cell the line has no value for — a weapon with no Attack, no Damage
# or no Range. It was a literal em dash in the markup until the round-1 audit
# (Sabine 7): a user-facing string with no key, and unreadable besides, since a
# screen reader announces a bare dash as nothing at all — so a blind user could
# not tell "not applicable" from "failed to render".
derived-not-applicable = n/a
# Joins a weapon to the shield carried with it on a combat line ("Long Sword &
# Round Shield"). It stands in for a word — the rulebook's own statblocks write
# both "&" and "and" — so it is translatable, not hardcoded. The surrounding
# spaces are added in code, since a Fluent value cannot begin or end with one.
derived-combat-shield-joiner = &
derived-longevity-self_made = Self-made
derived-longevity-external = External
# No bonus has been entered for the ritual yet, so there is no number to print.
derived-longevity-not-entered = not entered
# This panel shows the ritual as what it DOES: the modifier subtracted from aging
# rolls. So a stored bonus of 7 reads "-7 to aging rolls" here, while the editor
# shows the stored magnitude ("aging bonus +7") — each string names its own
# quantity, so the two surfaces can never be read as contradicting each other.
# { $modifier } comes from formatSigned, so a 0 renders "0", never "-0".
derived-longevity-aging-modifier = { $modifier } to aging rolls
derived-longevity-suggested = a ritual made now: { $modifier } to aging rolls
derived-masterpiece-cap = Max lesser-item level
derived-masterpiece-note = Guidance only: design the actual lesser enchanted item under Magic Items (vis costs ignored).
# Familiar bond: every figure is guidance. The within-focus Lab Total is
# conditional — whether this beast falls inside the focus is a troupe judgment.
derived-familiar-binding-level = Binding level
derived-familiar-cord-points = Cord points spent
derived-familiar-invested-levels = Invested power levels
derived-familiar-reaches = Lab Total reaches the binding level.
derived-familiar-falls-short = Lab Total falls short of the binding level.
derived-familiar-cords-fit = Cord points fit within the Lab Total.
derived-familiar-cords-exceed = Cord points exceed the Lab Total.
derived-familiar-note = Guidance only: which Arts suit the beast, and whether a Magical Focus applies, are troupe judgments (vis costs ignored).
# Joins the addend-breakdown tooltip's parts (S14, full-audit i18n) — same shape
# as restricted-xp-list-separator, routed through Fluent rather than a hardcoded
# ', ' literal so it can be retargeted for a language whose list convention differs.
derived-addend-list-separator = ,
derived-addend-intelligence = Intelligence
derived-addend-magic_theory = Magic Theory
derived-addend-technique = Technique
derived-addend-form = Form
derived-addend-aura = Aura
derived-addend-lab_mod = Lab modifier
derived-addend-stamina = Stamina
derived-addend-encumbrance = Encumbrance
derived-addend-artes_liberales = Artes Liberales
derived-addend-philosophiae = Philosophiae
# The flat CastingTotalMod is per scope, so it needs one label per cast type
# rather than the single `lab_mod`-style term the Lab Total gets.
derived-addend-casting_mod_formulaic = Casting modifier (formulaic)
derived-addend-casting_mod_ritual = Casting modifier (ritual)
derived-addend-casting_mod_spontaneous = Casting modifier (spontaneous)
derived-addend-parma = Parma Magica
derived-addend-might = Might
derived-addend-true_faith = True Faith
derived-addend-aura_bonus = Aura bonus
derived-addend-armor = Armor
derived-addend-soak_mod = Soak modifier
derived-addend-bronze_cord = Bronze cord
derived-addend-form_bonus = Form bonus
derived-fatigue-fresh = Fresh
derived-fatigue-winded = Winded
derived-fatigue-weary = Weary
derived-fatigue-tired = Tired
derived-fatigue-dazed = Dazed
derived-wound-light = Light
derived-wound-medium = Medium
derived-wound-heavy = Heavy
derived-wound-incapacitating = Incapacitating
derived-wound-dead = Dead
derived-surfaced-aging = Aging
derived-surfaced-advancement = Advancement
derived-surfaced-special_casting = Casting style
derived-surfaced-ability_roll = Ability roll
derived-surfaced-health_roll = Health roll
derived-surfaced-magic_resistance = Magic Resistance
derived-surfaced-physical_activity = Physical activity
# D45/F-423: names the Virtue/Flaw that produced a surfaced-modifier row, so two
# carriers of the same family+detail pair no longer read as one unattributed
# repeated line. { $source } is the item's own localized display name, never
# the raw id.
derived-surfaced-source = from { $source }
derived-detail-aging_roll = Aging roll
derived-detail-longevity_bonus = Longevity bonus
derived-detail-no_aging = Does not age
derived-detail-no_apparent_aging = Does not appear to age
derived-detail-decrepitude = Decrepitude
derived-detail-living_conditions = Living conditions
derived-detail-crisis_survival = Crisis survival roll
derived-detail-crisis_heavy_wound = Takes a Heavy Wound in a crisis
derived-detail-taught = Taught
derived-detail-book = Book
derived-detail-vis = Vis study
derived-detail-practice = Practice
derived-detail-adventure = Adventure
derived-detail-insight = Insight
derived-detail-teaching = Teaching
derived-detail-authoring = Authoring
derived-detail-spell_mastery = Spell mastery
derived-detail-all = All sources
derived-factor-half = Halved
derived-detail-quiet_words = Quiet magic
derived-detail-subtle_gestures = Subtle magic
derived-detail-deft_form = Deft Form
derived-detail-diedne = Diedne magic
derived-detail-faerie_raised = Faerie-Raised magic
derived-detail-life_linked_spontaneous = Life-Linked spontaneous
derived-detail-spell_improvisation = Spell improvisation
derived-detail-mercurian = Mercurian magic
derived-detail-life_boost = Life Boost
derived-detail-circumstantial = Circumstantial
# Susceptibility to Divine Power: the aura's own penalties to your magic — the
# Aura Modifier and the botch dice — count double in that realm's aura.
derived-detail-doubled_aura_penalty = Doubled aura penalties
derived-detail-fatigue_roll = Fatigue roll
derived-detail-casting_fatigue = Casting fatigue resistance
derived-detail-recovery = Recovery
derived-detail-susceptible_faerie = Susceptible to Faerie
derived-detail-susceptible_infernal = Susceptible to the Infernal
derived-detail-aura_bonus = Aura bonus
# Weak Magic Resistance: under the Flaw's stated condition an attacker keeps the
# spell level that Penetration normally subtracts. Nothing on the sheet changes,
# so the rule is listed rather than computed.
derived-detail-conditional_penetration_waiver = Spell level not subtracted under stated conditions

# Markdown export (Export sheet). Document chrome the exporter needs but no other
# surface names: table headers it composes itself, the two covenant point-item
# groups, and the yes/no markers a Markdown table has to spell out. Every other
# heading and label in the exported sheet reuses the keys above, so the document
# reads in the same words as the app. `arm_rules::export::LABEL_KEYS` is the
# authoritative list; a key missing here would print as its own name.
export-untitled = Untitled character
export-col-effective = Effective
export-col-magnitude = Magnitude
export-col-penalty = Penalty
export-col-points = Points
# Header of the exported spell list's one Arts-and-level column: the Technique and
# Form abbreviations followed by the level, the rulebook's own short form (CrIg20).
export-col-spell-code = TeFo/Level
# Rules-text column of the exported Virtue/Flaw tables: an uncomputed-rule entry's
# description (falling back to its summary), or a computed entry's summary.
export-col-summary = Summary
export-col-total = Total
# Type column of the exported Virtue/Flaw tables: the item's category (Hermetic,
# Story, …), the same value the in-app badge shows via `category-<id>`.
export-col-type = Type
export-items-boons = Boons
export-items-hooks = Hooks
# Sub-heading of the Virtue/Flaw tables that lists the free, off-budget items the
# character's House, mythic type or Warping grants (never counted in the balance).
export-granted = Granted
# D42/D70/D74: prefixes the resolved realm a Supernatural Virtue/Flaw is
# associated with, appended to that row's text cell (e.g. "Realm: Faerie").
export-vf-realm-label = Realm
export-xp-restricted = Restricted experience
