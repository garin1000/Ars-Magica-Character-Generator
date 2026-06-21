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

# One key per validation issue code emitted by the engine. { $context } is the
# offending item id when present.
issue-over_budget_virtues = Too many virtue points spent.
issue-over_budget_flaws = Too many flaw points taken.
issue-too_many_major_virtues = Too many Major Virtues.
issue-too_many_major_flaws = Too many Major Flaws.
issue-prereq_not_met = Prerequisite not met for { $context }.
issue-prereq_unevaluated = Prerequisite for { $context } could not be checked yet.
issue-incompatible = { $context } is incompatible with another selection.
issue-forbidden_category = { $context } belongs to a forbidden category.
issue-category_not_permitted = { $context } is not in a permitted category.
issue-wrong_entity_kind = { $context } cannot be taken by this entity kind.
issue-duplicate_selection = { $context } is selected more than once.
issue-missing_required_trait = A required trait is missing: { $context }.
issue-forbidden_trait = A forbidden trait is present: { $context }.
issue-gift_required = This type requires The Gift.
issue-gift_forbidden = This type cannot have The Gift.
issue-unknown_ref = Unknown item reference: { $context }.
issue-unknown_type = Unknown entity type.

# AppError kinds returned by Tauri commands.
error-io = A file could not be read or written.
error-ruleset = The ruleset could not be loaded.
error-not_loaded = No ruleset is loaded yet.
error-serialize = The character file could not be processed.
