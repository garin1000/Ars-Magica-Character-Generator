# UI chrome for the Ars Magica character generator.
# Rules text (virtue/flaw names, summaries) is NOT here — it comes from
# rules/i18n/<lang>/ via the load_ruleset command.

app-title = Ars Magica Character Generator
app-logo-alt = Ars Magica Open License logo

language-label = Language
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
start-create-title = Create a new character
start-create-hint = The character type is chosen here, once — it cannot be changed later.
start-wizard-title = Guided creation
start-wizard-hint = Step by step through this type's creation phases, in order. The character type is chosen here too, once — it cannot be changed later.

# Creation-phase labels, keyed by the engine's `CreationPhase` slug. They name the
# guided wizard's steps and are the only rendering of a phase — the slug itself
# never reaches the screen. `review` is the wizard's own terminal step, appended
# after whatever phases the character type declares.
phase-concept = Concept
phase-type = Character type
phase-characteristics = Characteristics
phase-virtues_flaws = Virtues & Flaws
phase-abilities = Abilities
phase-arts = Arts
phase-spells = Spells
phase-house_specialisation = House
phase-mythic_type = Mythic companion type
phase-personality_reputations = Personality & Reputations
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
# Shown while the validation mode is Advisory or Silent: nothing is enforced, so
# no step gates and Finish is always available.
wizard-unchecked-hint = Validation is not enforced, so no step blocks progress.

# The Character type step: a read-only account of what the chosen type commits
# this character to. The type itself was fixed when the character was created.
phase-type-explainer = The character type is fixed for this character. It sets the Virtue and Flaw budget, which categories may be taken, and which creation steps follow.
phase-type-budget = Up to { $flaws } points of Flaws, funding up to { $virtues } points of Virtues.
phase-type-gift-required = This type has The Gift, granted automatically.
phase-type-gift-forbidden = This type cannot have The Gift.
phase-type-gift-optional = This type may take The Gift.

# The wizard's closing step.
wizard-review-title = Review
wizard-review-clean = No errors or warnings — this character is legal.
# Honest about what the gating does and does not check: the steps block on errors
# only, so a legal character can still be an unfinished one.
wizard-review-incomplete = A legal character is not necessarily a finished one: steps only block on errors, so anything merely left empty passed through.
wizard-review-hint = Equipment, magic items, Might, Warping and aging are not part of the guided flow — they are edited after finishing.

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
# Tab labels for the editor's main area.
tab-characteristics = Characteristics
tab-virtues-flaws = Virtues & Flaws
tab-abilities = Abilities
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
# Restricted experience pools (Educated/Warrior/Privileged): extra XP spendable
# only on the listed Abilities/categories. `$eligibility` is a localized list.
restricted-xp-pool = { $eligibility }: { $used } / { $amount }
restricted-xp-list-separator = ,
# Life-stage experience blocks, keyed by the engine's `LifeStageBlock` slug. Each
# is a restricted pool of its own: the first buys the native language and nothing
# else, the second the childhood Abilities but never that language. Later life
# appears for a magus, whose general pool is apprenticeship instead — its years
# earn Abilities only, never an Art, so the label says so.
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
# Life-stage read-outs in the XP bar. Later life funds anything, so it replaces the
# editable pool total: years after childhood × experience per year. The rate is this
# character's own — the Wealthy Virtue and the Poor Flaw change it.
life-stage-later-life = Later life: { $years } × { $rate } = { $xp } XP
life-stage-no-budget = No life-stage experience yet.
# Apprenticeship, for a magus alone: fifteen fixed years whose experience "can be
# spent on Arts or Abilities" (Core Rules.md:2435), which makes it the general pool —
# so this line names the block the pool total comes from. Absent for anyone who
# serves no apprenticeship, which is what keeps the non-magus bar unchanged.
life-stage-apprenticeship = Apprenticeship: { $years } years = { $xp } XP
# Age is repeated inside the panel because later life is measured in years, so it
# is edited here as well as on the Details tab.
life-stage-age-label = Age
# What the age means for a magus: the life stages build one AT its Gauntlet, so the
# age entered is the Gauntlet age — apprenticeship is the fifteen years ending there
# (Core Rules.md:2435) and every year before them earns later-life experience
# (`:2214`). The second sentence is the actionable half: life as a magus after the
# Gauntlet (`:2216`, `:2471`) is not counted yet, so an older magus belongs on the
# flat pool rather than silently losing those years.
life-stage-gauntlet-note = Life stages build a magus at its Gauntlet: apprenticeship is the fifteen years ending at the age entered, and every year before them earns later-life experience. The years lived as a magus after the Gauntlet are not counted yet, so an older magus should use the experience pool instead.
# Escape hatch for a hand-edited save: a character funded by its life stages must
# not also carry an entered pool, and guided mode offers no field to correct one,
# so this empties it.
xp-pool-clear = Clear pool
xp-pool-clear-hint = Experience comes from the life stages here, so a pool entered by hand has to go back to 0.
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
magus-minimums-summary = { $unmet } of { $total } still unmet
magus-minimum-met = { $ability } { $min } is met: this character has { $score }.
magus-minimum-unmet = { $ability } { $min } is not met: this character has { $score }.
magus-recommended-label = Recommended minimum Abilities
magus-recommended-hint = The recommended Abilities cost { $xp } experience points in total; below them the magus is weak relative to other magi.
ability-score-label = Score
ability-specialty-label = Specialty
# Heading for the rulebook's list of example specialties shown in the picker.
ability-specialties-label = Specialties
ability-add = Add ability
ability-increment = Raise
ability-decrement = Lower
# Hermetic Arts: the two classes and the spinner controls. Arts share the
# Abilities XP pool (the `xp-pool` key), so no Art-specific pool label.
art-type-technique = Techniques
art-type-form = Forms
art-add = Add Art
art-increment = Raise
art-decrement = Lower
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
# figure, the editable base (bracketed), then Available and any V/F bonus.
spell-levels-pool = Spell levels
spell-levels-available = Available: { $available }
# A POSITIVE Virtue/Flaw contribution: an extra pool of levels spent before the
# base, so it reads used/amount exactly like a restricted XP pool.
spell-levels-bonus-pool = Virtues/Flaws: { $used } / { $amount }
# A NEGATIVE Virtue/Flaw modifier (Weak Parens): no pool to spend, so it is charged
# to the base and reported as the signed modifier. `$bonus` arrives already signed.
spell-levels-bonus = Virtues/Flaws: { $bonus }
# Accessible name for the editable BASE spell-levels field; empty = use the type
# profile's default (shown as the field's placeholder).
spell-levels-base-label = Spell-levels budget
spell-mastery-xp = Mastery XP: { $xp }
# The per-spell Spell-Mastery XP pool bar: how much of the pool is spent.
spell-mastery-pool = Mastery XP: { $used } / { $pool }
spell-mastery-floor = All spells mastered at { $score }
# The per-spell Spell-Mastery stepper.
spell-mastery-label = Mastery
spell-mastery-increment = Increase spell mastery
spell-mastery-decrement = Decrease spell mastery
spell-mastery-abilities-label = Special abilities
spell-mastery-ability-add = Add special ability
spell-mastery-ability-remove = Remove special ability
spell-remove = Remove
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
personality-label = Personality Traits
personality-name-placeholder = Trait
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
characteristic-increment = Raise
characteristic-decrement = Lower
characteristic-description-label = Description

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
issue-restricted_xp_unspent = { $origin }: { $unspent } of { $amount } restricted experience points are unspent and will be wasted.
issue-ability_category_requires_virtue = { $ability } is { $category }, which needs a Virtue granting access at character creation.
issue-academic_ability_without_scholarly_language = An Academic Ability normally requires { $ability } at { $min } or better.
issue-life_stage_xp_pool_conflict =This character earns its experience through its life stages, so the directly entered pool of { $xp_pool } must be 0.
issue-life_stage_age_unset = Enter the character's age: later life earns experience per year, so with no age only childhood's blocks can be counted.
issue-life_stage_age_before_childhood = Age { $age } falls inside childhood, which lasts { $min } years — there are no later-life years to earn experience in.
issue-life_stage_age_before_gauntlet = No magus is gauntleted at { $age }: the Gauntlet comes no earlier than { $min } — childhood plus fifteen years of apprenticeship.
issue-life_stage_native_language_unset = Choose a native language: childhood's largest block of experience can be spent on nothing else.
issue-life_stage_native_language_missing_score = No { $language } score is bought, so childhood's native-language experience is unspent.
issue-magus_minimum_ability = No magus is admitted to the Order below { $ability } { $min }; this character has { $score }.
issue-magus_recommended_ability = { $ability } { $min } is recommended for a magus just out of apprenticeship; this character has { $score }.
issue-childhood_package_unknown = Unknown childhood package: { $package }.
issue-childhood_slot_unfilled = Fill in the { $key } for { $ability } before applying the childhood package.
issue-childhood_slot_is_native_language = The { $key } for { $ability } must differ from the native language { $language }.
issue-childhood_slot_duplicate_value = The { $key } { $value } for { $ability } is already used by another entry of the childhood package; choose a different one.
issue-ability_parameter_required = { $ability } needs a value (e.g. the specific Area or Language).
issue-ability_score_out_of_range = Ability { $ability } score { $score } is outside the allowed range (0 to { $max }).
issue-ability_bonus_dangling_target = { $item } targets an Ability the character does not have ({ $ability } { $parameter }); add it first.
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
equipment-equipped-label = Equipped
equipment-specialization-label = Specialization applies (+1)
equipment-remove = Remove equipment
equipment-empty = No equipment.

# AppError kinds returned by Tauri commands.
error-io = A file could not be read or written.
error-ruleset = The ruleset could not be loaded.
error-not_loaded = No ruleset is loaded yet.
error-serialize = The character file could not be processed.

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
derived-detail-decrepitude = Decrepitude
derived-detail-living_conditions = Living conditions
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
