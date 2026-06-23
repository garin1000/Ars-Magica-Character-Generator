# Oberflächentexte für den Ars-Magica-Charaktergenerator.
# Regeltexte (Namen und Zusammenfassungen von Tugenden/Fehlern) stehen NICHT
# hier — sie kommen über den Befehl load_ruleset aus rules/i18n/<lang>/.

app-title = Ars-Magica-Charaktergenerator

language-label = Sprache
mode-label = Prüfung

mode-enforced = Erzwungen
mode-advisory = Hinweise
mode-silent = Aus

items-virtues-title = Tugenden
items-flaws-title = Fehler
selections-title = Auswahl
validation-title = Prüfung

# Kategorie- (Typ-)Bezeichnungen für Tugenden/Fehler, je Kategorie-ID der Engine.
category-general = Allgemein
category-hermetic = Hermetisch
category-social_status = Sozialer Status
category-special = Spezial

# Stufenbezeichnungen (Magnitude), je Magnitude-Wert der Engine.
magnitude-free = Frei
magnitude-minor = Klein
magnitude-major = Groß

balance-virtues = Tugenden: { $used } / { $budget }
balance-flaws = Fehler: { $used } / { $budget }

action-add = Hinzufügen
action-remove = Entfernen
action-save = Speichern
action-load = Laden

param-prompt = { $param } wählen
param-placeholder = ID eingeben

empty-selections = Noch keine Tugenden oder Fehler ausgewählt.
no-issues = Keine Probleme.
loading = Wird geladen…

# Ein Schlüssel je Prüfcode der Engine. Jede Nachricht interpoliert die `args`
# der Engine (siehe crates/arm-rules/src/validation.rs); die Argumentnamen sind
# je Code stabil und werden von der UI unverändert weitergereicht.
issue-over_budget_virtues = Tugendpunkte ({ $points }) überschreiten das Budget ({ $budget }).
issue-over_budget_flaws = Fehlerpunkte ({ $points }) überschreiten das Budget ({ $budget }).
issue-unbalanced_virtues = Tugendpunkte ({ $virtue_points }) übersteigen die Fehlerpunkte ({ $flaw_points }); Tugenden müssen durch Fehler finanziert werden.
issue-too_many_major_virtues = Zu viele große Tugenden ({ $count } von max. { $max }).
issue-too_many_major_flaws = Zu viele große Fehler ({ $count } von max. { $max }).
issue-too_many_minor_flaws = Zu viele kleine Fehler ({ $count } von max. { $max }).
issue-too_many_major_personality_flaws = Zu viele große Persönlichkeitsfehler ({ $count } von max. { $max }).
issue-too_many_personality_flaws = Mehr Persönlichkeitsfehler als empfohlen ({ $count } von { $max }).
issue-too_many_story_flaws = Mehr Geschichte-Fehler als empfohlen ({ $count } von { $max }).
issue-prereq_not_met = Voraussetzung für { $item } nicht erfüllt.
issue-prereq_unevaluated = Voraussetzung für { $item } konnte noch nicht geprüft werden.
issue-incompatible = { $item } ist mit { $other } unvereinbar.
issue-forbidden_category = { $item } gehört zu einer verbotenen Kategorie ({ $category }).
issue-category_not_permitted = { $item } ist in keiner erlaubten Kategorie ({ $category }).
issue-wrong_entity_kind = { $item } ist für die Wesensart { $entity_kind } nicht zulässig.
issue-duplicate_selection = { $item } ist { $count }-mal ausgewählt.
issue-missing_required_trait = Eine erforderliche Eigenschaft fehlt: { $item }.
issue-forbidden_trait = Eine verbotene Eigenschaft ist vorhanden: { $item }.
issue-missing_param = { $item } fehlt der Parameter { $key }.
issue-unexpected_param = { $item } hat einen unerwarteten Parameter { $key }.
issue-unknown_param_value = { $item }: Parameter { $key } hat unbekannten { $domain }-Wert { $value }.
issue-gift_required = Dieser Typ erfordert die Gabe.
issue-gift_forbidden = Dieser Typ darf die Gabe nicht haben.
issue-unknown_ref = Unbekannte Element-Referenz: { $item }.
issue-unknown_type = Unbekannte Wesensart: { $type_id }.

# AppError-Arten der Tauri-Befehle.
error-io = Eine Datei konnte nicht gelesen oder geschrieben werden.
error-ruleset = Das Regelwerk konnte nicht geladen werden.
error-not_loaded = Es ist noch kein Regelwerk geladen.
error-serialize = Die Charakterdatei konnte nicht verarbeitet werden.
