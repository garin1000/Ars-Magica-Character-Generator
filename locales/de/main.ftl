# Oberflächentexte für den Ars-Magica-Charaktergenerator.
# Regeltexte (Namen und Zusammenfassungen von Tugenden/Fehlern) stehen NICHT
# hier — sie kommen über den Befehl load_ruleset aus rules/i18n/<lang>/.

app-title = Ars-Magica-Charaktergenerator
app-logo-alt = Logo „Ars Magica Open License“

language-label = Sprache
mode-label = Prüfung

mode-enforced = Erzwungen
mode-advisory = Hinweise
mode-silent = Aus

type-label = Charaktertyp
type-grog = Grog
type-companion = Gefährte
type-mythic_companion = Mythischer Gefährte
type-magus = Magus

available-title = Verfügbar
items-virtues-title = Tugenden
items-flaws-title = Fehler
selections-title = Gewählt
# Kennzeichnung einer gewählten Zeile, die der Charaktertyp erfordert (z. B. Die
# Gabe und Hermetischer Magus eines Magus) — automatisch gewählt, nicht entfernbar.
selection-required-label = Erforderlich
validation-title = Prüfung
characteristics-title = Eigenschaften
abilities-title = Fertigkeiten

# Kategorie- (Typ-)Bezeichnungen für Tugenden/Fehler, je Kategorie-ID der Engine.
category-general = Allgemein
category-hermetic = Hermetisch
category-personality = Persönlichkeit
category-social_status = Sozialer Status
category-special = Spezial
category-story = Geschichte
category-supernatural = Übernatürlich

# Stufenbezeichnungen (Magnitude), je Magnitude-Wert der Engine.
magnitude-free = Frei
magnitude-minor = Klein
magnitude-major = Groß

# Typ-Label für Tugenden/Fehler: eine befleckte (infernal-assoziierte) Tugend/Fehler.
vf-tag-tainted = Befleckt

# Die acht Eigenschaften, je Eigenschaftswert der Engine.
characteristic-int = Intelligenz
characteristic-per = Wahrnehmung
characteristic-str = Stärke
characteristic-sta = Ausdauer
characteristic-pre = Präsenz
characteristic-com = Kommunikation
characteristic-dex = Geschicklichkeit
characteristic-qik = Schnelligkeit
# Kurze Tooltip-Beschreibungen, verdichtet aus dem Kapitel Eigenschaften der Basisregeln.
characteristic-desc-int = Die Fähigkeit, Konzepte zu analysieren und zu synthetisieren, sowie einfaches Gedächtnis; ausschlaggebend für die Hermetischen Künste.
characteristic-desc-per = Die Fähigkeit, Dinge zu bemerken, sowie die Kraft der Intuition; wichtig für Aufmerksamkeit, Jagen und Menschenkenntnis.
characteristic-desc-str = Körperliche Kraft – Heben, Stoßen und Bewegen – und die Wucht hinter einer Nahkampfwaffe.
characteristic-desc-sta = Geistiges und körperliches Durchhaltevermögen; Zauberwirken, Lastentragen und Wundenwiderstand stützen sich darauf.
characteristic-desc-pre = Erscheinung, Auftreten und Charisma – Eindruck machen, führen und einschüchtern.
characteristic-desc-com = Die Begabung zum Selbstausdruck – andere beeinflussen und mit ihnen kommunizieren.
characteristic-desc-dex = Gewandtheit und präzises, kunstfertiges Handhaben von Objekten; Hand-Augen-Koordination und körperliche Anmut.
characteristic-desc-qik = Reaktionsgeschwindigkeit und Reflexe – wer in der Eile zuerst handelt; durch Belastung modifiziert.

# Fertigkeitskategorien, je Kategoriewert der Engine.
ability-category-general = Allgemein
ability-category-academic = Akademisch
ability-category-arcane = Arkan
ability-category-martial = Kampf
ability-category-supernatural = Übernatürlich
# Nachgestellte Markierung hinter dem Namen einer „mit Stern“ versehenen
# Fertigkeit – einer Fertigkeit, die ohne mindestens 1 Erfahrungspunkt nicht
# einsetzbar ist (kein ungelernter Wurf), gemäß Regelwerk. Betrifft allgemeine,
# akademische, arkane und übernatürliche Fertigkeiten.
ability-requires-training-marker = *

# Eigenschaften-Punkteanzeige.
characteristic-points = Punkte: { $used } / { $budget }
# Reiter-Bezeichnungen für den Hauptbereich des Editors.
tab-characteristics = Eigenschaften
tab-virtues-flaws = Tugenden & Fehler
tab-abilities = Fertigkeiten
tab-arts = Künste
tab-house-specialisation = Haus
tab-mythic-type = Typ
tab-spells = Zauber
tab-details = Details
# XP-Übersicht und Bedienelemente der Fertigkeiten.
xp-pool = XP-Vorrat
xp-spent = Ausgegeben: { $spent }
xp-available = Verfügbar: { $available }
# Eingeschränkte XP-Vorräte (Gebildet/Krieger/Privilegierte Erziehung): zusätzliche
# XP, nur für die aufgeführten Fertigkeiten/Kategorien. `$eligibility` ist eine Liste.
restricted-xp-pool = { $eligibility }: { $used } / { $amount }
restricted-xp-list-separator = ,
ability-score-label = Wert
ability-specialty-label = Spezialisierung
# Überschrift für die im Auswahlbereich gezeigten Beispiel-Spezialisierungen.
ability-specialties-label = Spezialisierungen
ability-add = Fertigkeit hinzufügen
ability-increment = Erhöhen
ability-decrement = Verringern
# Hermetische Künste: die beiden Klassen und die Bedienelemente. Künste teilen
# sich den XP-Vorrat der Fertigkeiten (Schlüssel `xp-pool`).
art-type-technique = Techniken
art-type-form = Formen
art-add = Kunst hinzufügen
art-increment = Erhöhen
art-decrement = Verringern
# Zauberauswahl (nur Magi). Die Zaubernamen stammen aus den Regel-i18n (nach
# Zauber-ID), nicht aus diesen Oberflächen-Schlüsseln. Filtere einen Zauber nach
# Technik + Form; ein Gen-Zauber fragt nach der erlernten Stufe.
spell-technique-label = Technik
spell-form-label = Form
spell-level-label = Stufe
spell-add = Zauber hinzufügen
spell-none = — Zauber wählen —
spell-levels-used = Zauberstufen: { $used } / { $budget }
spell-remove = Entfernen
# Details-Reiter: Alter, Selbstvertrauen (abgeleitet, schreibgeschützt), Persönlichkeit, Ruf.
age-label = Alter
age-cap-note = Maximaler Fertigkeitswert: { $cap }
confidence-label = Selbstvertrauen
confidence-readout = Wert { $score }, { $points } Punkte
personality-label = Persönlichkeitsmerkmale
personality-name-placeholder = Merkmal
personality-add = Merkmal hinzufügen
personality-empty = Noch keine Persönlichkeitsmerkmale.
reputations-label = Ruf
reputation-content-placeholder = Wofür
reputation-add = { $kind }-Ruf hinzufügen (Stufe { $score })
reputation-empty = Kein Ruf verfügbar (nimm eine Tugend oder einen Fehler, der einen verleiht).
reputation-type-local = Lokal
reputation-type-ecclesiastical = Kirchlich
reputation-type-hermetic = Hermetisch
# Grund, warum eine übernatürliche Fähigkeit im Auswähler ausgegraut ist.
ability-requires-virtue = Erfordert eine verleihende Tugend (oder die eine freie Fähigkeit der Gabe)
characteristic-increment = Erhöhen
characteristic-decrement = Verringern
characteristic-description-label = Beschreibung

# Auswahl des hermetischen Hauses + Spezialisierungs-Auswahl je Gewährung. Die
# Anzeigenamen der Häuser stammen aus der Regel-i18n (per Haus-ID), nicht aus
# diesen Oberflächen-Schlüsseln.
house-label = Haus
house-none = — Keines —
# Eine feste Gewährung, schreibgeschützt angezeigt (z. B. Bjornaers Herztier).
house-granted-label = Gewährt
# Text der leeren Option einer Spezialisierungs-Auswahl.
house-choose-prompt = Wählen…

# Typ-Wähler für mythische Gefährten. Typnamen stammen aus der Regel-i18n
# (per mythic_type-ID), nicht aus diesen Oberflächenschlüsseln.
mythic-type-label = Typ des mythischen Gefährten
mythic-type-none = — Keiner —
# Eine gewährte freie Status-/Neben-Tugend, schreibgeschützt angezeigt.
mythic-granted-label = Gewährt
# Text der leeren Option der Neben-Tugend-Auswahl.
mythic-choose-prompt = Wählen…
# Bezeichnung neben einem erforderlichen Fehler, dessen Vorgabe ersetzt werden darf.
mythic-required-flaw-label = Erforderlicher Fehler

balance-virtues = Tugenden: { $used } / { $budget }
balance-flaws = Fehler: { $used } / { $budget }

action-save = Speichern
action-load = Laden

param-prompt = { $param } wählen
param-placeholder = ID eingeben
# Hinweis für einen nicht ausgefüllten Parameter-Platzhalter im Namen, z. B. „Begabung in (Fertigkeit)“.
param-hint = ({ $label })
# Lokalisierte Parameter-Bezeichnungen, je Parameter-Schlüssel der Engine.
param-label-ability = Fertigkeit
param-label-technique = Technik
param-label-area = Gebiet
param-label-language = Sprache
param-label-characteristic = Eigenschaft
param-label-organization = Organisation
param-label-mystery_cult = Mysterienkult
param-label-craft = Handwerk
param-label-profession = Beruf

# Effektiver Wert, neben dem Basiswert angezeigt, wenn ein Tugend-Bonus greift.
effective-score = { $score }

empty-selections-side = Noch keine.
no-issues = Keine Probleme.
loading = Wird geladen…

# Ein Schlüssel je Prüfcode der Engine. Jede Nachricht interpoliert die `args`
# der Engine (siehe crates/arm-rules/src/validation.rs); die Argumentnamen sind
# je Code stabil und werden von der UI unverändert weitergereicht.
issue-over_budget_virtues = Tugendpunkte ({ $points }) überschreiten das Budget ({ $budget }).
issue-over_budget_flaws = Fehlerpunkte ({ $points }) überschreiten das Budget ({ $budget }).
issue-unbalanced_virtues = Tugendpunkte ({ $virtue_points }) übersteigen die Fehlerpunkte ({ $flaw_points }); Tugenden müssen durch Fehler finanziert werden.
issue-too_many_major_virtues = Zu viele große Tugenden ({ $count } von max. { $max }).
issue-too_many_major_hermetic_virtues = Zu viele große hermetische Tugenden ({ $count } von max. { $max }).
issue-too_many_major_flaws = Zu viele große Fehler ({ $count } von max. { $max }).
issue-too_many_minor_flaws = Zu viele kleine Fehler ({ $count } von max. { $max }).
issue-too_many_major_personality_flaws = Zu viele große Persönlichkeitsfehler ({ $count } von max. { $max }).
issue-too_many_personality_flaws = Mehr Persönlichkeitsfehler als empfohlen ({ $count } von { $max }).
issue-too_many_story_flaws = Mehr Geschichte-Fehler als empfohlen ({ $count } von { $max }).
issue-too_many_tainted_virtues = Mehr als die Hälfte deiner Tugendpunkte sind befleckt ({ $tainted } von { $total }).
issue-too_many_tainted_flaws = Mehr als die Hälfte deiner Fehlerpunkte sind befleckt ({ $tainted } von { $total }).
issue-prereq_not_met = Voraussetzung für { $item } nicht erfüllt.
issue-prereq_unevaluated = Voraussetzung für { $item } konnte noch nicht geprüft werden.
issue-incompatible = { $item } ist mit { $other } unvereinbar.
issue-forbidden_category = { $item } gehört zu einer verbotenen Kategorie ({ $category }).
issue-category_not_permitted = { $item } ist in keiner erlaubten Kategorie ({ $category }).
issue-wrong_entity_kind = { $item } ist für die Wesensart { $entity_kind } nicht zulässig.
issue-duplicate_selection = { $item } ist { $count }-mal ausgewählt, darf aber höchstens { $max }-mal für dasselbe Ziel gewählt werden.
issue-missing_required_trait = Eine erforderliche Eigenschaft fehlt: { $item }.
issue-forbidden_trait = Eine verbotene Eigenschaft ist vorhanden: { $item }.
issue-missing_param = { $item } fehlt der Parameter { $key }.
issue-unexpected_param = { $item } hat einen unerwarteten Parameter { $key }.
issue-unknown_param_value = { $item }: Parameter { $key } hat unbekannten { $domain }-Wert { $value }.
issue-gift_required = Dieser Typ erfordert die Gabe.
issue-gift_forbidden = Dieser Typ darf die Gabe nicht haben.
issue-unknown_ref = Unbekannte Element-Referenz: { $item }.
issue-unknown_type = Unbekannte Wesensart: { $type_id }.
issue-characteristic_out_of_range = Eigenschaft { $characteristic } mit Wert { $score } liegt außerhalb des erlaubten Bereichs ({ $min } bis { $max }).
issue-characteristic_overspent = Eigenschaften kosten { $cost } Punkte, mehr als die verfügbaren { $points }.
issue-characteristic_points_unspent = Nur { $cost } von { $points } Eigenschaftspunkten ausgegeben.
issue-characteristic_above_cap = Eigenschaft { $characteristic } mit Wert { $score } überschreitet ihr Maximum von { $cap }; erhöhe die Grenze mit Hervorragende Eigenschaft.
issue-characteristic_below_floor = Eigenschaft { $characteristic } mit Wert { $score } liegt unter ihrem Minimum von { $floor }; senke die Grenze mit Schlechte Eigenschaft.
issue-characteristic_max_base_too_low = { $item } erfordert { $characteristic } mindestens auf { $min } (derzeit { $base }).
issue-characteristic_min_base_too_high = { $item } erfordert { $characteristic } höchstens auf { $max } (derzeit { $base }).
issue-unknown_ability = Unbekannte Fertigkeit: { $ability }.
issue-duplicate_ability = { $ability } ist { $count }-mal mit derselben Spezialisierung aufgeführt.
issue-not_enough_xp = Fertigkeiten kosten { $spent } XP, mehr als die { $pool } im Vorrat.
issue-restricted_xp_unspent = { $unspent } von { $amount } eingeschränkten Erfahrungspunkten sind nicht ausgegeben und verfallen.
issue-ability_parameter_required = { $ability } braucht einen Wert (z. B. das konkrete Gebiet oder die Sprache).
issue-ability_score_out_of_range = Fertigkeit { $ability } mit Wert { $score } liegt außerhalb des erlaubten Bereichs (0 bis { $max }).
issue-ability_bonus_dangling_target = { $item } zielt auf eine Fertigkeit, die der Charakter nicht hat ({ $ability } { $parameter }); füge sie zuerst hinzu.
issue-unknown_art = Unbekannte Kunst: { $art }.
issue-duplicate_art = { $art } ist { $count }-mal aufgeführt.
issue-art_score_out_of_range = Kunst { $art } mit Wert { $score } liegt außerhalb des erlaubten Bereichs (0 bis { $max }).
issue-house_choice_unresolved = Haus { $house } hat eine offene Spezialisierungswahl ({ $choice_key }).
issue-house_grant_constraint = Haus { $house }: Die Wahl { $choice_key } fällt auf { $item }, was die Vorgabe nicht erfüllt.
issue-house_unset = Ein Magus sollte einem hermetischen Haus angehören.
issue-missing_hermetic_flaw = Ein Magus sollte mindestens einen hermetischen Fehler wählen.
issue-mythic_type_unset = Ein mythischer Gefährte sollte einen Typ wählen.
issue-mythic_choice_unresolved = Der mythische Gefährtentyp { $mythic_type } hat eine offene Wahl ({ $choice_key }).
issue-mythic_grant_constraint = Mythischer Gefährtentyp { $mythic_type }: Die Wahl { $choice_key } fällt auf { $item }, was die Vorgabe nicht erfüllt.
issue-mythic_required_trait_missing = Eine erforderliche Tugend oder ein erforderlicher Fehler (oder ein geeigneter Ersatz) fehlt: { $item }.
issue-unknown_spell = Unbekannter Zauber: { $spell }.
issue-duplicate_spell = { $spell } ist { $count }-mal aufgeführt.
issue-spell_level_unresolved = Der Gen-Zauber { $spell } hat noch keine gewählte Stufe.
issue-over_spell_levels = Die Zauber ergeben { $used } Stufen und überschreiten das Budget von { $budget } (um { $over }).
issue-spell_level_exceeds_cap = Zauber { $spell } hat Stufe { $level }, über der höchsten erlernbaren Stufe ({ $cap }).
issue-ability_above_age_cap = { $ability } mit Wert { $score } überschreitet das Maximum von { $cap } für Alter { $age }.
issue-supernatural_ability_requires_virtue = { $ability } ist eine übernatürliche Fähigkeit und erfordert eine verleihende Tugend (oder die eine freie Fähigkeit der Gabe).
issue-personality_trait_out_of_range = Persönlichkeitsmerkmal { $name } ({ $value }) liegt außerhalb des zulässigen Bereichs (±{ $max }).
issue-reputation_not_granted = Ein Ruf ({ $kind }, { $content }) benötigt eine Tugend oder einen Fehler, der ihn verleiht.

# AppError-Arten der Tauri-Befehle.
error-io = Eine Datei konnte nicht gelesen oder geschrieben werden.
error-ruleset = Das Regelwerk konnte nicht geladen werden.
error-not_loaded = Es ist noch kein Regelwerk geladen.
error-serialize = Die Charakterdatei konnte nicht verarbeitet werden.
