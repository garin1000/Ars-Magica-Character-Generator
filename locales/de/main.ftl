# Oberflächentexte für den Ars-Magica-Charaktergenerator.
# Regeltexte (Namen und Zusammenfassungen von Vorzügen/Schwächen) stehen NICHT
# hier — sie kommen über den Befehl load_ruleset aus rules/i18n/<lang>/.

app-title = Ars-Magica-Charaktergenerator

language-label = Sprache
mode-label = Prüfung

mode-enforced = Erzwungen
mode-advisory = Hinweise
mode-silent = Aus

items-title = Verfügbare Vorzüge & Schwächen
selections-title = Auswahl
validation-title = Prüfung

balance-virtues = Vorzüge: { $used } / { $budget }
balance-flaws = Schwächen: { $used } / { $budget }

action-add = Hinzufügen
action-remove = Entfernen
action-save = Speichern
action-load = Laden

param-prompt = { $param } wählen
param-placeholder = ID eingeben

empty-selections = Noch keine Vorzüge oder Schwächen ausgewählt.
no-issues = Keine Probleme.
loading = Wird geladen…

# Ein Schlüssel je Prüfcode der Engine. { $context } ist die betroffene ID.
issue-over_budget_virtues = Zu viele Vorzugspunkte ausgegeben.
issue-over_budget_flaws = Zu viele Schwächenpunkte genommen.
issue-too_many_major_virtues = Zu viele große Vorzüge.
issue-too_many_major_flaws = Zu viele große Schwächen.
issue-prereq_not_met = Voraussetzung für { $context } nicht erfüllt.
issue-prereq_unevaluated = Voraussetzung für { $context } konnte noch nicht geprüft werden.
issue-incompatible = { $context } ist mit einer anderen Auswahl unvereinbar.
issue-forbidden_category = { $context } gehört zu einer verbotenen Kategorie.
issue-category_not_permitted = { $context } ist in keiner erlaubten Kategorie.
issue-wrong_entity_kind = { $context } ist für diese Wesensart nicht zulässig.
issue-duplicate_selection = { $context } ist mehrfach ausgewählt.
issue-missing_required_trait = Eine erforderliche Eigenschaft fehlt: { $context }.
issue-forbidden_trait = Eine verbotene Eigenschaft ist vorhanden: { $context }.
issue-gift_required = Dieser Typ erfordert die Gabe.
issue-gift_forbidden = Dieser Typ darf die Gabe nicht haben.
issue-unknown_ref = Unbekannte Element-Referenz: { $context }.
issue-unknown_type = Unbekannte Wesensart.

# AppError-Arten der Tauri-Befehle.
error-io = Eine Datei konnte nicht gelesen oder geschrieben werden.
error-ruleset = Das Regelwerk konnte nicht geladen werden.
error-not_loaded = Es ist noch kein Regelwerk geladen.
error-serialize = Die Charakterdatei konnte nicht verarbeitet werden.
