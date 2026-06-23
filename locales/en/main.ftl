# UI chrome for the Ars Magica character generator.
# Rules text (virtue/flaw names, summaries) is NOT here — it comes from
# rules/i18n/<lang>/ via the load_ruleset command.

app-title = Ars Magica Character Generator

language-label = Language
mode-label = Validation

mode-enforced = Enforced
mode-advisory = Advisory
mode-silent = Silent

items-title = Available Virtues & Flaws
selections-title = Selections
validation-title = Validation

balance-virtues = Virtues: { $used } / { $budget }
balance-flaws = Flaws: { $used } / { $budget }

action-add = Add
action-remove = Remove
action-save = Save
action-load = Load

param-prompt = Choose { $param }
param-placeholder = enter an id

empty-selections = No virtues or flaws selected yet.
no-issues = No issues.
loading = Loading…

# One key per validation issue code emitted by the engine. Each message
# interpolates the engine's `args` (see crates/arm-rules/src/validation.rs):
# arg names are stable per code and passed through verbatim by the UI.
issue-over_budget_virtues = Virtue points ({ $points }) exceed budget ({ $budget }).
issue-over_budget_flaws = Flaw points ({ $points }) exceed budget ({ $budget }).
issue-too_many_major_virtues = Too many Major Virtues ({ $count } of max { $max }).
issue-too_many_major_flaws = Too many Major Flaws ({ $count } of max { $max }).
issue-prereq_not_met = Prerequisite not met for { $item }.
issue-prereq_unevaluated = Prerequisite for { $item } could not be checked yet.
issue-incompatible = { $item } is incompatible with { $other }.
issue-forbidden_category = { $item } belongs to a forbidden category ({ $category }).
issue-category_not_permitted = { $item } is not in a permitted category ({ $category }).
issue-wrong_entity_kind = { $item } cannot be taken by a { $entity_kind }.
issue-duplicate_selection = { $item } is selected { $count } times.
issue-missing_required_trait = A required trait is missing: { $item }.
issue-forbidden_trait = A forbidden trait is present: { $item }.
issue-missing_param = { $item } is missing the parameter { $key }.
issue-unexpected_param = { $item } has an unexpected parameter { $key }.
issue-unknown_param_value = { $item } parameter { $key } has unknown { $domain } value { $value }.
issue-gift_required = This type requires The Gift.
issue-gift_forbidden = This type cannot have The Gift.
issue-unknown_ref = Unknown item reference: { $item }.
issue-unknown_type = Unknown entity type: { $type_id }.

# AppError kinds returned by Tauri commands.
error-io = A file could not be read or written.
error-ruleset = The ruleset could not be loaded.
error-not_loaded = No ruleset is loaded yet.
error-serialize = The character file could not be processed.
