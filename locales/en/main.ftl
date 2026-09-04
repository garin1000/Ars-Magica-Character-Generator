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
# on this screen otherwise. The toolbar's Validation control only mounts once a
# character exists (App.svelte), so this can only point ahead to it.
start-create-mode-hint = How strictly the rules are checked (Validation, in the toolbar) can be changed at any time once the character is open.
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
# The same reading, said on the step itself.
wizard-step-incomplete-hint = Nothing has been recorded on this step yet. It does not block: you can continue and come back to it.
# Shown while the validation mode is Advisory or Silent: nothing is enforced, so
# no step gates and Finish is always available.
wizard-unchecked-hint = Validation is not enforced, so no step blocks progress.

# Per-step guidance: what the player decides on this step, and what the rules say
# about it. Keyed by the engine's `CreationPhase` slug, one line per phase, so a
# new phase without copy fails the locale-parity test. Instructional chrome about
# the flow, not catalogue text keyed by item ID — hence Fluent, not
# `rules/i18n/`. Every factual claim is sourced from
# `Ars Magica - Definitive Edition (Core Rules).md`; the line ranges are recorded
# in `crates/arm-rules/RULES.md`. Numbers the ruleset already carries are
# interpolated ({ $points }, { $flaws }, { $virtues }) and never written out.
wizard-guidance-concept = Start from a concept: who this character is, and what they are in the saga. A magus might be a fire wizard, a companion a scholar far from home, a grog any sort of warrior or member of the covenant staff.
wizard-guidance-characteristics = Characteristics are your character's inborn attributes, and normal means never raise them later. You have { $points } points to spend, and a score below zero gives points back.
wizard-guidance-virtues_flaws = Flaws pay for Virtues: up to { $flaws } points of Flaws fund up to { $virtues } points of Virtues. You need not take the maximum, and every character takes a Social Status.
wizard-guidance-experience = Experience is acquired in blocks: the first five years of childhood, then later life a year at a time — for a magus, apprenticeship and the years after it as well. Enter one total yourself, or let those life stages earn it from the character's age.
wizard-guidance-abilities = Abilities are learned skills, bought with the experience the previous step supplies. Your character's age sets the highest score any Ability may reach at creation.
wizard-guidance-arts = Techniques and Forms are the magus's magic: every spell combines one of each. The same apprenticeship experience buys Arts and Abilities, so what is spent here is not spent there.
wizard-guidance-spells = Apprenticeship grants levels of spells rather than experience points, and they are spent here. No spell may be of a higher level than the magus's Technique, Form, Intelligence and Magic Theory allow.
wizard-guidance-house_specialisation = Every Hermetic magus belongs to exactly one House, which grants a benefit at creation. It comes before Virtues and Flaws because that benefit is a free Minor Virtue, which needs no Flaw to fund it.
wizard-guidance-mythic_type = A Mythic Companion's type is a free Virtue saying what he is. The types are incompatible with one another and with The Gift, and one normally brings a free Minor Virtue with it.
wizard-guidance-personality_reputations = Pick a few words for the character's personality and give each a score from +3 to -3. Grogs should have a score in Loyal, and warriors one in Brave. A Reputation belongs here only if a Virtue or Flaw granted one.
wizard-guidance-aging = A character over 35 makes an aging roll for every year before play begins, which may cost apparent age or Characteristic points. Aging Points build up in a Characteristic until they exceed it, and it then drops by one.
# No rules passage describes a review step — it is this application's own closing
# step, so this line makes no rules claim.
wizard-guidance-review = Nothing new is chosen here — this is the last look before the character leaves the guided flow. Everything stays editable after finishing.

# The per-type Virtue/Flaw advice appended to `wizard-guidance-virtues_flaws`
# (guided-creation-review-2026-08 #7). One whole sentence per clause rather than one
# nested select inside the main message, so a translator reads sentences and not
# fragments; `derive.ts`'s `flawCapNotes` picks the key and supplies the args.
#
# `$cap` is the profile's own `flaw_category_caps` maximum and arrives as a NUMBER, so
# the variants below select on it: a cap of 0 forbids the category outright ("You
# should not take Story Flaws", Core Rules :2826), while a cap of 1 or more limits it
# ("not more than one Story Flaw", :2818, :2837). The `[1]` variant exists for the
# plural agreement alone — the figure itself is always interpolated, never written out.
#
# `$rule` is the cap's `hard` flag: the rules' own distinction between "may not"
# (enforced, and reported as an error by the validator) and "should not" (a guideline
# the troupe may set aside, :2818).
#
# There is deliberately NO "at least one Story Flaw" wording anywhere: Story Flaws
# have a recommended ceiling and no minimum. The only "at least one" the rules state
# is the magus's Hermetic Flaw, below.
wizard-guidance-story-flaw-cap =
    { $rule ->
        [hard]
            { $cap ->
                [0] You may not take Story Flaws.
                [1] You may not take more than { $cap } Story Flaw.
               *[other] You may not take more than { $cap } Story Flaws.
            }
       *[soft]
            { $cap ->
                [0] You should not take Story Flaws.
                [1] You should not take more than { $cap } Story Flaw.
               *[other] You should not take more than { $cap } Story Flaws.
            }
    }
wizard-guidance-personality-flaw-cap =
    { $rule ->
        [hard]
            { $cap ->
                [0] You may not take Personality Flaws.
                [1] You may not take more than { $cap } Personality Flaw.
               *[other] You may not take more than { $cap } Personality Flaws.
            }
       *[soft]
            { $cap ->
                [0] You should not take Personality Flaws.
                [1] You should not take more than { $cap } Personality Flaw.
               *[other] You should not take more than { $cap } Personality Flaws.
            }
    }
# Core Rules :2860. Worded exactly as `issue-missing_hermetic_flaw`, the engine
# warning for the same guideline, so the advice and the finding read alike.
wizard-guidance-hermetic-flaw = A magus should take at least one Hermetic Flaw.

# What the chosen character type commits this character to, stated on the banner
# above both the editor and the guided wizard. Relocated from the deleted
# `type` creation step (guided-creation review #1), which asked for nothing: the
# type is fixed when the character is created and can never change. Re-keyed to
# `character-type-*` so no key names a creation phase that no longer exists.
character-type-explainer = The character type is fixed for this character. It sets the Virtue and Flaw budget, which categories may be taken, and which creation steps follow.
character-type-budget = Up to { $flaws } points of Flaws, funding up to { $virtues } points of Virtues.
character-type-gift-required = This type has The Gift, granted automatically.
character-type-gift-forbidden = This type cannot have The Gift.
character-type-gift-optional = This type may take The Gift.

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
# The Abilities funding switch. Experience either comes from one pool the player
# enters, or the character's life stages earn it. The switch is not a stored flag:
# a life-stage plan on the character IS guided funding, so a loaded save lands in
# the mode its own data implies.
ability-funding-label = Source of experience
ability-funding-pool = Experience pool
ability-funding-life_stages = Life stages
ability-funding-pool-hint = Enter one total yourself and spend it on any Ability.
ability-funding-life_stages-hint = Early childhood and later life earn the experience: age sets the later-life total, while a native language and a sample Childhood fill the childhood blocks.
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
# What the two ages mean for a magus, which neither field can say for itself. The
# age is how old the magus is NOW; the Gauntlet age is when its apprenticeship
# ended — the fifteen years before it (Core Rules.md:2435), with every year before
# those earning later-life experience (`:2214`). The years between the two are life
# as a magus, worth "30 points" each (`:2216`, `:2471`). Nothing is stored for a
# blank field: the engine reads it as the ruleset's own baseline magus, "25 years
# old and just out of apprenticeship" (`:1601`), clamped to a younger character's
# age — which is the number the empty field shows as its placeholder.
life-stage-gauntlet-note = For a magus the age is how old it is now, and the Gauntlet age is when its apprenticeship ended: apprenticeship is the fifteen years before the Gauntlet, every year before those earns later-life experience, and every year from the Gauntlet to today is worth 30 points. Leave the Gauntlet age blank and the magus takes the usual age for one just out of apprenticeship, which the empty field shows.
# The years a magus has lived since its Gauntlet. Only the Gauntlet age is stored;
# the years, the points and the experience all follow from it and the age. Each
# point "can be an experience point in an Art or Ability or one level of spell"
# (Core Rules.md:2471), and a season of lab work costs 10 of that year's 30 — but
# only three seasons a year are charged, because the third has already taken the
# whole 30 (`:2482`).
life-stage-gauntlet-age-label = Gauntlet age
life-stage-gauntlet-age-hint = The age the apprenticeship ended at. Left blank it takes the age the empty field shows: the usual age for a magus just out of apprenticeship, or this character's own age if it is younger than that.
life-stage-lab-seasons-label = Lab seasons
life-stage-lab-seasons-hint = Seasons of lab work, totalled over all the years as a magus: each costs 10 of that year's 30 points, and only three a year are charged — the third already takes the whole 30, so a fourth is free.
life-stage-spell-levels-label = Levels of spells
life-stage-spell-levels-hint = How many of the points to take as levels of spells rather than experience.
life-stage-post-gauntlet-no-years-note = No years as a magus yet, so lab seasons and levels of spells can take nothing. Those years run from the Gauntlet age to the character's age.
life-stage-post-gauntlet-summary = { $years } years as a magus: { $points } points = { $xp } XP + { $levels } levels of spells
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
childhood-apply-hint = The package fills in these Ability scores; they stay editable afterwards.
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
magus-recommended-hint = The recommended Abilities cost { $xp } experience points in total; below them the magus is weak relative to other magi.
ability-score-label = Score
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
spell-budget-reason = Not enough spell levels remaining
spell-already-taken-reason = Already selected
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
age-cap-note = Max Ability score: { $cap }
confidence-label = Confidence
confidence-readout = Score { $score }, { $points } points
warping-label = Warping
warping-readout = Score { $score }, { $points } points
warping-effect-label = Warping effect
warping-owed-label = Warping Virtues & Flaws
warping-owed-hint = Your Warping Score grants these Virtues and Flaws (off-budget). Choose one for each.
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
# The saga year (guided-creation-review-2026-08 #25): an app-wide setting, not part
# of any character, against which the age and the birth year are two views of one
# fact. Changing it rewrites nothing — advancing a saga means aging rolls, not
# subtraction — so the hint says what it does and what it does not.
saga-year-label = Saga year
saga-year-hint = The year your saga is set in. It links the age and the birth year while you type; changing it leaves both as they are.
identity-sigil = Wizard's sigil
identity-covenant = Covenant
identity-parens = Parens
# Directly-entered aged / warped state. The Decrepitude and Warping scores shown
# are computed by the engine from these points, never recomputed here.
aging-label = Aging
aging-points-heading = Aging points per Characteristic
aging-points-note = Characteristic drops are applied automatically once accrued points exceed the Characteristic's score; the drop shows in the derived totals.
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
aging-longevity-clamp = A Longevity Ritual is in force: while it lasts, no total reaches the aging table's first result, so these rolls cannot age the character.
# The Living Conditions checklist (Core Rules.md:16581-16594). The total shown is
# the engine's own resolved modifier — it also carries the Virtue/Flaw
# contributions, which are not rows in this list.
living-conditions-label = Living Conditions
living-conditions-hint = The aging total subtracts this modifier, so a higher modifier means a longer life. With nothing chosen the character counts as an average peasant (0).
living-conditions-total = Living Conditions modifier: { $modifier }
living-conditions-cumulative-note = Conditions marked cumulative stack with each other; the rest are alternatives, so take at most one of those.
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
aging-die-hint = Roll a stress die (no botch) at the table and type it here — the app never rolls for you. A stress die explodes, so there is no upper value.
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
aging-outcome-decrepitude_and_crisis = { $points } Aging Points — enough to reach the next level of Decrepitude — and a Crisis.
aging-outcome-decrepitude_unpriceable = Enough Aging Points to reach the next level of Decrepitude, and a Crisis. The advancement table does not reach that level, so agree the number at the table.
aging-outcome-crisis-note = This roll is a Crisis. Place the Aging Points first, then roll the Crisis die below — the Decrepitude those points raise counts towards the crisis total.
# The Crisis (Core Rules.md:16619-16638). Two dice, both the player's: the stress
# die above sent the year here, and a simple die is thrown at the Crisis Table.
# The app never rolls either, never throws the survival roll, and never decides
# whether the character lives.
crisis-label = Crisis
crisis-die-label = Simple die
crisis-die-hint = Roll a simple die at the table and type it here — the app never rolls for you. A zero counts as ten.
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
crisis-note = The app never makes the survival roll and never decides whether the character lives; resolve that at the table.
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
aging-calculator-note = Nothing shown here is recorded on the character until you press Apply; the die itself is never saved.
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
reputation-add = Add { $kind } Reputation (level { $score })
reputation-empty = No Reputation is granted (take a Virtue or Flaw that grants one).
reputation-type-local = Local
reputation-type-ecclesiastical = Ecclesiastical
reputation-type-hermetic = Hermetic
reputation-type-academic = Academic
# Magic Items tab (magi): aura, enchanted devices, familiar bond cords, the
# talisman and the Longevity Ritual. The item-level budget used/remaining comes
# from the engine, never recomputed here.
aura-label = Assumed lab/covenant aura
aura-out-of-range = Outside the rules range ({ $min } to { $max }); will be clamped to the nearest legal value when saved.
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
power-add = Add power
powers-empty = No supernatural powers yet.
realm-magic = Magic
realm-faerie = Faerie
realm-divine = Divine
realm-infernal = Infernal
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
familiar-powers-note = There is no limit to the number of powers invested in a familiar bond.
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
longevity-sterility-note = The ritual's anchor stops the magus expending his life force in normal human fashion, so he becomes permanently sterile.
# Shown as the reason a Supernatural Ability is greyed in the picker.
ability-requires-virtue = Requires a granting Virtue (or the Gift's one free Ability)
# Screen-reader-only text on a selected Ability row an error-severity issue
# points at (S7, full-audit a11y) — pairs with a visible glyph so the row's
# invalidity is never colour-only (WCAG 1.4.1).
ability-invalid-selection = Invalid selection
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

# Document toolbar actions.
action-new = New
action-open = Open
action-save = Save
action-save-as = Save As
action-export = Export
# Takes the character already on screen into the guided flow, resuming from the
# furthest step its file recorded. Offered only where that flow can be walked.
action-continue-in-wizard = Continue in guided creation

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

# Window title. { $name } is the file name, { $app } the application name; the
# dirty variant prepends an ASCII marker for a document with unsaved edits.
app-title-document = { $name } — { $app }
app-title-document-dirty = *{ $name } — { $app }

# On-screen document status shown in the header (not only in the OS window
# title): the active save's file name, with an ASCII marker prepended for a
# document with unsaved edits, or a label for a document that has never been saved.
app-document-name = { $name }
app-document-name-dirty = *{ $name }
app-document-unsaved = Unsaved document
app-document-unsaved-dirty = *Unsaved document

# Hint shown for an unfilled parameter slot in an item name, e.g. "Puissant (Ability)".
param-hint = ({ $label })
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
param-label-art = Art
param-label-focus = Focus
param-label-area = Area
param-label-language = Language
param-label-characteristic = Characteristic
param-label-organization = Organization
param-label-mystery_cult = Mystery Cult
param-label-craft = Craft
param-label-profession = Profession
param-label-form = Form
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
param-label-commodity = Commodity
param-label-role = Role

# Effective score shown beside a base score when a virtue bonus applies.
effective-score = { $score }

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
issue-too_many_major_flaws = Too many Major Flaws ({ $count } of max { $max }).
issue-too_many_minor_flaws = Too many Minor Flaws ({ $count } of max { $max }).
issue-too_many_major_personality_flaws = Too many Major Personality Flaws ({ $count } of max { $max }).
issue-too_many_personality_flaws = More Personality Flaws than recommended ({ $count } of { $max }).
issue-too_many_story_flaws = More Story Flaws than recommended ({ $count } of { $max }).
issue-too_many_tainted_virtues = More than half your Virtue points are Tainted ({ $tainted } of { $total }).
issue-too_many_tainted_flaws = More than half your Flaw points are Tainted ({ $tainted } of { $total }).
issue-prereq_not_met = Prerequisite not met for { $item }.
issue-prereq_unevaluated = Prerequisite for { $item } could not be checked yet.
issue-incompatible = { $item } is incompatible with { $other }.
issue-forbidden_category = { $item } belongs to a forbidden category ({ $category }).
issue-category_not_permitted = { $item } is not in a permitted category ({ $category }).
issue-wrong_entity_kind = { $item } cannot be taken by a { $entity_kind }.
issue-duplicate_selection = { $item } is selected { $count } times, but may be taken at most { $max } time(s) for the same target.
issue-missing_required_trait = A required trait is missing: { $item }.
issue-forbidden_trait = A forbidden trait is present: { $item }.
issue-missing_param = { $item } is missing the parameter { $key }.
issue-unexpected_param = { $item } has an unexpected parameter { $key }.
issue-unknown_param_value = { $item } parameter { $key } has unknown { $domain } value { $value }.
issue-multiple_magical_foci = A magus may have only one Magical Focus, but { $count } are selected.
issue-gift_required = This type requires The Gift.
issue-gift_forbidden = This type cannot have The Gift.
issue-unknown_ref = Unknown item reference: { $item }.
issue-unknown_type = Unknown entity type: { $type_id }.
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
issue-ability_category_requires_virtue = { $ability } is { $category }, which needs a Virtue granting access at character creation.
issue-academic_ability_without_scholarly_language = An Academic Ability normally requires { $ability }{ $qualifier } at { $min } or better.
issue-life_stage_age_unset = Enter the character's age: later life earns experience per year, so with no age only childhood's blocks can be counted.
issue-life_stage_age_before_childhood = Age { $age } falls inside childhood, which lasts { $min } years — there are no later-life years to earn experience in.
issue-life_stage_age_before_gauntlet = No magus is gauntleted at { $age }: the Gauntlet comes no earlier than { $min } — childhood plus fifteen years of apprenticeship.
issue-life_stage_gauntlet_age_after_age = The Gauntlet at { $gauntlet_age } is still ahead of this magus, who is { $age }; the years as a magus are counted forward from the Gauntlet.
issue-life_stage_lab_seasons_out_of_range = { $seasons } lab seasons is more than the { $max } that { $years } year(s) as a magus can be charged for.
issue-life_stage_lab_seasons_without_years = { $seasons } lab seasons are recorded, but this character has no years as a magus to work them in.
issue-life_stage_spell_level_split_exceeds_points = Taking { $levels } levels of spells out of the years as a magus is more than those years grant: they are worth { $points } points, to be divided between experience and levels of spells.
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
issue-ability_bonus_dangling_target = Add { $ability } { $parameter } to the character's Abilities; { $item } targets it.
issue-unknown_art = Unknown Art: { $art }.
issue-duplicate_art = { $art } is listed { $count } times.
issue-art_score_out_of_range = Art { $art } score { $score } is outside the allowed range (0 to { $max }).
issue-house_choice_unresolved = House { $house } has an unresolved specialisation choice ({ $choice_key }).
issue-house_grant_constraint = The House { $house } grant { $choice_key } picks { $item }, which does not meet its constraint.
issue-warping_owed_minor_flaws = You still owe { $count } Minor Flaw(s) from Warping.
issue-warping_owed_supernatural_virtues = You still owe { $count } supernatural Minor Virtue(s) from Warping.
issue-warping_owed_major_flaws = You still owe { $count } Major Flaw(s) from Warping.
issue-warping_fill_constraint = The Warping fill { $choice_key } picks { $item }, which does not match the owed slot.
issue-warping_fill_ineligible = The Warping fill { $choice_key } picks { $item }, which itself grants Warping and cannot fill a Warping slot.
issue-warping_fill_excess = The Warping fill { $choice_key } is not owed and should be removed.
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
issue-unknown_mastery_ability = Unknown Spell Mastery ability { $ability } chosen for { $spell }.
issue-too_many_mastery_abilities = { $spell } has { $chosen } Mastery special abilities, above its Mastery score of { $mastery } (one per level).
issue-duplicate_mastery_ability = Mastery ability { $ability } is chosen { $count } times for { $spell }, but may be taken only once.
issue-ability_above_age_cap = { $ability } score { $score } exceeds the age-{ $age } maximum of { $cap }.
issue-supernatural_ability_requires_virtue = { $ability } is a Supernatural Ability and requires a granting Virtue (or the Gift's one free Ability).
issue-personality_trait_out_of_range = Personality Trait { $name } ({ $value }) is outside the allowed range (±{ $max }).
issue-reputation_not_granted = A { $kind } Reputation ({ $content }) needs a Virtue or Flaw that grants one.
issue-over_item_level = Enchanted devices total { $used } levels, over the budget of { $budget } (by { $over }).
issue-over_power_levels = Supernatural powers total { $used } levels, over the budget of { $budget } (by { $over }).
issue-might_realm_mismatch = The entered Might Realm ({ $base }) disagrees with the Realm its Virtues grant ({ $granted }).
issue-excessive_aging_reduction = The aging drops to { $characteristic } ({ $reduction }) would fall below the minimum score ({ $min }); it is clamped there.
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
equipment-equipped-label = Equipped
equipment-specialization-label = Specialization applies (+1)
equipment-empty = No equipment.

# AppError kinds returned by Tauri commands.
error-io = A file could not be read or written.
error-ruleset = The ruleset could not be loaded.
error-not_loaded = No ruleset is loaded yet.
error-serialize = The character file could not be processed.
error-export = The character sheet could not be exported: the current language is missing text for { $missing }. Try switching to English and exporting again, or report this as a bug.

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
derived-aura-out-of-range = Outside the rules range ({ $min } to { $max }); will be clamped to the nearest legal value when saved.
derived-lab-enchanting = For enchanting
derived-lab-enchanting-hint = Halved from the Lab Total above for Weak Enchanter — use this figure when creating or investigating an enchanted item.
derived-within-focus = Within focus
derived-deficient = (deficient, halved)
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
derived-addend-parma = Parma Magica
derived-addend-might = Might
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
derived-detail-spell_mastery = Spell mastery
derived-detail-all = All sources
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
derived-detail-fatigue_roll = Fatigue roll
derived-detail-casting_fatigue = Casting fatigue
derived-detail-recovery = Recovery
derived-detail-susceptible_divine = Susceptible to the Divine
derived-detail-susceptible_faerie = Susceptible to Faerie
derived-detail-susceptible_infernal = Susceptible to the Infernal
derived-detail-aura_bonus = Aura bonus

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
export-col-total = Total
# Type column of the exported Virtue/Flaw tables: the item's category (Hermetic,
# Story, …), the same value the in-app badge shows via `category-<id>`.
export-col-type = Type
export-items-boons = Boons
export-items-hooks = Hooks
# Sub-heading of the Virtue/Flaw tables that lists the free, off-budget items the
# character's House, mythic type or Warping grants (never counted in the balance).
export-granted = Granted
export-xp-restricted = Restricted experience
export-yes = Yes
export-no = No
