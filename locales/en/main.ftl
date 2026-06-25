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

available-title = Available
items-virtues-title = Virtues
items-flaws-title = Flaws
selections-title = Selected
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

# Ability category labels, keyed by the engine's category value.
ability-category-general = General
ability-category-academic = Academic
ability-category-arcane = Arcane
ability-category-martial = Martial
ability-category-supernatural = Supernatural

# Characteristic point-buy readout.
characteristic-points = Points: { $used } / { $budget }
# Tab labels for the editor's main area.
tab-characteristics = Characteristics
tab-virtues-flaws = Virtues & Flaws
tab-abilities = Abilities
# Ability XP summary and controls.
xp-pool = XP pool
xp-spent = Spent: { $spent }
xp-available = Available: { $available }
ability-score-label = Score
ability-specialty-label = Specialty
ability-add = Add ability
ability-increment = Raise
ability-decrement = Lower
characteristic-increment = Raise
characteristic-decrement = Lower
characteristic-description-label = Description

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
issue-characteristic_effective_out_of_range = Characteristic { $characteristic } effective score { $effective } exceeds the maximum of { $max }.
issue-characteristic_bonus_base_too_low = { $item } requires { $characteristic } to be at least { $min } (currently { $base }).
issue-unknown_ability = Unknown ability: { $ability }.
issue-duplicate_ability = { $ability } is listed { $count } times with the same specialty.
issue-not_enough_xp = Abilities cost { $spent } XP, more than the { $pool } in the pool.
issue-ability_parameter_required = { $ability } needs a value (e.g. the specific Area or Language).
issue-ability_score_out_of_range = Ability { $ability } score { $score } is outside the allowed range (0 to { $max }).
issue-ability_bonus_dangling_target = { $item } targets an Ability the character does not have ({ $ability } { $parameter }); add it first.

# AppError kinds returned by Tauri commands.
error-io = A file could not be read or written.
error-ruleset = The ruleset could not be loaded.
error-not_loaded = No ruleset is loaded yet.
error-serialize = The character file could not be processed.
