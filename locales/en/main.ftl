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
category-social_status = Social Status
category-special = Special

# Magnitude (level) labels, keyed by the engine's magnitude value.
magnitude-free = Free
magnitude-minor = Minor
magnitude-major = Major

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
# Tab labels for the editor's main area.
tab-characteristics = Characteristics
tab-virtues-flaws = Virtues & Flaws
tab-abilities = Abilities
tab-arts = Arts
tab-house-specialisation = House
# Ability XP summary and controls.
xp-pool = XP pool
xp-spent = Spent: { $spent }
xp-available = Available: { $available }
# Restricted experience pools (Educated/Warrior/Privileged): extra XP spendable
# only on the listed Abilities/categories. `$eligibility` is a localized list.
restricted-xp-pool = { $eligibility }: { $used } / { $amount }
restricted-xp-list-separator = ,
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

balance-virtues = Virtues: { $used } / { $budget }
balance-flaws = Flaws: { $used } / { $budget }

action-save = Save
action-load = Load

param-prompt = Choose { $param }
param-placeholder = enter an id
# Hint shown for an unfilled parameter slot in an item name, e.g. "Puissant (Ability)".
param-hint = ({ $label })
# Localized parameter labels, keyed by the engine's parameter key.
param-label-ability = Ability
param-label-technique = Technique
param-label-area = Area
param-label-language = Language
param-label-characteristic = Characteristic
param-label-organization = Organization
param-label-mystery_cult = Mystery Cult
param-label-craft = Craft
param-label-profession = Profession

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
issue-restricted_xp_unspent = { $unspent } of { $amount } restricted experience points are unspent and will be wasted.
issue-ability_parameter_required = { $ability } needs a value (e.g. the specific Area or Language).
issue-ability_score_out_of_range = Ability { $ability } score { $score } is outside the allowed range (0 to { $max }).
issue-ability_bonus_dangling_target = { $item } targets an Ability the character does not have ({ $ability } { $parameter }); add it first.
issue-unknown_art = Unknown Art: { $art }.
issue-duplicate_art = { $art } is listed { $count } times.
issue-art_score_out_of_range = Art { $art } score { $score } is outside the allowed range (0 to { $max }).
issue-house_choice_unresolved = House { $house } has an unresolved specialisation choice ({ $choice_key }).
issue-house_grant_constraint = The House { $house } grant { $choice_key } picks { $item }, which does not meet its constraint.
issue-house_unset = A magus should belong to a Hermetic House.
issue-missing_hermetic_flaw = A magus should take at least one Hermetic Flaw.
issue-mythic_type_unset = A Mythic Companion should choose a type.
issue-mythic_choice_unresolved = The Mythic Companion type { $mythic_type } has an unresolved choice ({ $choice_key }).
issue-mythic_grant_constraint = The Mythic Companion type { $mythic_type } grant { $choice_key } picks { $item }, which does not meet its constraint.
issue-mythic_required_trait_missing = A required Virtue or Flaw (or a suitable substitute) is missing: { $item }.

# AppError kinds returned by Tauri commands.
error-io = A file could not be read or written.
error-ruleset = The ruleset could not be loaded.
error-not_loaded = No ruleset is loaded yet.
error-serialize = The character file could not be processed.
