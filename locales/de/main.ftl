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

# Filter-/Suchsteuerung für lange Auswahllisten.
filter-search-placeholder = Suchen…
filter-magnitude-all = Alle Stufen
filter-category-all = Alle Typen

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
characteristic-size = Größe: { $size }
# Reiter-Bezeichnungen für den Hauptbereich des Editors.
tab-characteristics = Eigenschaften
tab-virtues-flaws = Tugenden & Fehler
tab-abilities = Fertigkeiten
tab-arts = Künste
tab-house-specialisation = Haus
tab-mythic-type = Typ
tab-spells = Zauber
tab-possessions = Magische Gegenstände
tab-equipment = Ausrüstung
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
spell-mastery-xp = Meisterschafts-EP: { $xp }
spell-mastery-pool = Meisterschafts-EP: { $used } / { $pool }
spell-mastery-floor = Alle Zauber gemeistert auf { $score }
spell-mastery-label = Meisterschaft
spell-mastery-increment = Zauber-Meisterschaft erhöhen
spell-mastery-decrement = Zauber-Meisterschaft verringern
spell-remove = Entfernen
# Details-Reiter: Alter, Selbstvertrauen (abgeleitet, schreibgeschützt), Persönlichkeit, Ruf.
age-label = Alter
age-cap-note = Maximaler Fertigkeitswert: { $cap }
confidence-label = Selbstvertrauen
confidence-readout = Wert { $score }, { $points } Punkte
warping-label = Verzerrung
warping-readout = Wert { $score }, { $points } Punkte
true-faith-label = Wahrer Glaube
true-faith-readout = Wert { $score }
decrepitude-label = Gebrechlichkeit
decrepitude-readout = Wert { $score }
item-levels-label = Zauberartefakte
item-levels-readout = { $levels } Stufen
# Identitäts-/Hintergrundfelder (Freitext, keine mechanische Wirkung).
identity-label = Identität
identity-name = Name
identity-gender = Geschlecht
identity-birth-year = Geburtsjahr
identity-sigil = Signum
identity-covenant = Bund
identity-parens = Parens
# Direkt eingegebener gealterter / verzerrter Zustand. Die angezeigten Werte für
# Gebrechlichkeit und Verzerrung berechnet die Engine aus diesen Punkten.
aging-label = Alterung
aging-points-heading = Alterungspunkte pro Eigenschaft
aging-reductions-heading = Eigenschaftsminderungen
warping-points-label = Verzerrungspunkte
twilight-scars-label = Zwielichtnarben
twilight-scar-placeholder = Beschreibe die Narbe
twilight-scar-add = Zwielichtnarbe hinzufügen
twilight-scars-empty = Noch keine Zwielichtnarben.
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
reputation-type-academic = Akademisch
# Tab "Magische Gegenstände" (Magi): Aura, Zauberartefakte, Vertrautenbindung,
# Talismanabstimmungen und das Langlebigkeitsritual. Das genutzte/verbleibende
# Artefaktbudget stammt aus der Engine, wird hier nie neu berechnet.
aura-label = Aura
possessions-devices-label = Zauberartefakte
device-name-placeholder = Name des Artefakts
device-level-label = Stufe
device-add = Artefakt hinzufügen
devices-empty = Noch keine Zauberartefakte.
item-level-used = Artefaktstufen: { $used } / { $budget }
familiar-label = Vertrauter
familiar-name-placeholder = Name des Vertrauten
familiar-cord-gold = Goldene Kordel
familiar-cord-silver = Silberne Kordel
familiar-cord-bronze = Bronzene Kordel
familiar-add = Vertrauten hinzufügen
familiar-remove = Vertrauten entfernen
talisman-label = Talismanabstimmungen
talisman-desc-placeholder = Was verstärkt wird
talisman-bonus-label = Bonus
talisman-add = Abstimmung hinzufügen
talisman-empty = Noch keine Talismanabstimmungen.
longevity-label = Langlebigkeitsritual
longevity-source-label = Herkunft
longevity-source-self_made = Selbst erstellt
longevity-source-external = Extern
longevity-bonus-label = Alterungsbonus
longevity-add = Langlebigkeitsritual hinzufügen
longevity-remove = Langlebigkeitsritual entfernen
longevity-self-made-note = Der Bonus wird aus deinem Creo-+-Corpus-Labortotal berechnet.
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
param-label-form = Form
param-label-realm = Sphäre
param-label-land = Land
param-label-being = Wesen
param-label-terrain = Gelände
param-label-subject = Fach
param-label-sin = Sünde
param-label-faculty = Fakultät
param-label-commodity = Ware
param-label-role = Rolle

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
issue-multiple_magical_foci = Ein Magus darf nur eine Magische Fokussierung haben, es sind aber { $count } gewählt.
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
issue-spell_ritual_legality = Zauber { $spell } wird auf Stufe { $level } erlernt und verletzt die Ritualgrenzen (Rituale mindestens 20, Nicht-Rituale höchstens 50).
issue-ability_above_age_cap = { $ability } mit Wert { $score } überschreitet das Maximum von { $cap } für Alter { $age }.
issue-supernatural_ability_requires_virtue = { $ability } ist eine übernatürliche Fähigkeit und erfordert eine verleihende Tugend (oder die eine freie Fähigkeit der Gabe).
issue-personality_trait_out_of_range = Persönlichkeitsmerkmal { $name } ({ $value }) liegt außerhalb des zulässigen Bereichs (±{ $max }).
issue-reputation_not_granted = Ein Ruf ({ $kind }, { $content }) benötigt eine Tugend oder einen Fehler, der ihn verleiht.
issue-over_item_level = Zauberartefakte umfassen { $used } Stufen, über dem Budget von { $budget } (um { $over }).
issue-over_power_levels = Übernatürliche Kräfte umfassen { $used } Stufen, über dem Budget von { $budget } (um { $over }).
issue-might_realm_mismatch = Die eingegebene Macht-Sphäre ({ $base }) stimmt nicht mit der von den Tugenden verliehenen Sphäre ({ $granted }) überein.
issue-excessive_aging_reduction = Alterungsminderungen für { $characteristic } ({ $reduction }) würden den Wert unter das Minimum ({ $min }) senken.
issue-aging_points_force_drop = { $characteristic } hat { $points } Alterungspunkte, mehr als der Wert ({ $score }) — eine Wertminderung steht an.
issue-unknown_equipment = Ausrüstung '{ $item }' passt zu keiner Waffe, keinem Schild und keiner Rüstung.
issue-equipment_min_strength = { $item } benötigt Stärke { $required }, aber dieser Charakter hat { $strength }.

# Ausrüstung (Waffen / Schilde / Rüstungen). Kampfwerte, Absorption und Belastung
# werden in einem späteren Schritt berechnet; diese Oberfläche erfasst nur die
# getragenen Gegenstände.
weapon-kind-melee = Nahkampf
weapon-kind-missile = Fernkampf
weapon-kind-thrown = Wurf
equipment-add-label = Ausrüstung hinzufügen
equipment-none = Gegenstand wählen…
equipment-group-weapons = Waffen
equipment-group-shields = Schilde
equipment-group-armor = Rüstungen
equipment-add = Hinzufügen
equipment-carried-label = Getragene Ausrüstung
equipment-equipped-label = Ausgerüstet
equipment-remove = Ausrüstung entfernen
equipment-empty = Keine Ausrüstung.

# AppError-Arten der Tauri-Befehle.
error-io = Eine Datei konnte nicht gelesen oder geschrieben werden.
error-ruleset = Das Regelwerk konnte nicht geladen werden.
error-not_loaded = Es ist noch kein Regelwerk geladen.
error-serialize = Die Charakterdatei konnte nicht verarbeitet werden.

# Abgeleitete Spielwerte (M5/5i). Schreibgeschützte, von der Engine berechnete
# Werte; das Panel zeigt diese Zahlen nur an und berechnet keine Mechanik selbst.
tab-totals = Werte
derived-section-summary = Übersicht
derived-section-lab = Laborsummen
derived-section-casting = Zaubersummen
derived-section-penetration = Penetration
derived-section-magic-resistance = Magieresistenz
derived-section-longevity = Langlebigkeitsritual
derived-section-masterpiece = Meisterstück
derived-section-combat = Kampf
derived-section-soak = Absorption
derived-section-encumbrance = Belastung
derived-section-fatigue = Erschöpfung
derived-section-wounds = Wunden
derived-section-surfaced = Weitere Modifikatoren
derived-section-decrepitude = Gebrechlichkeit
derived-section-warping = Verzerrung
derived-size = Größe
derived-aura-label = Aura
derived-within-focus = Im Fokus
derived-deficient = (defizitär, halbiert)
derived-weak-magic = (Schwache Magie, halbiert)
derived-level = Stufe
derived-range = Reichweite
derived-load = Last
derived-burden = Bürde
derived-lab-total = Laborsumme
derived-combat-empty = Keine Waffen ausgerüstet.
derived-cast-formulaic = Formelhaft
derived-cast-ritual = Ritual
derived-cast-spont-fatiguing = Spontan (ermüdend)
derived-cast-spont-non-fatiguing = Spontan (nicht ermüdend)
derived-cast-non-standard = Nicht-Standard
derived-cast-silent = Ohne Stimme
derived-cast-still = Ohne Gesten
derived-cast-silent-still = Ohne Stimme + Gesten
derived-deft-form = (Gewandte Form)
derived-combat-init = Initiative
derived-combat-attack = Angriff
derived-combat-defense = Verteidigung
derived-combat-damage = Schaden
derived-longevity-self_made = Selbst erschaffen
derived-longevity-external = Extern
derived-masterpiece-cap = Höchste Stufe schlichtes Artefakt
derived-masterpiece-note = Nur ein Hinweis: Das eigentliche schlichte Artefakt unter Magische Gegenstände entwerfen (Vis-Kosten werden ignoriert).
derived-addend-intelligence = Intelligenz
derived-addend-magic_theory = Magietheorie
derived-addend-technique = Technik
derived-addend-form = Form
derived-addend-aura = Aura
derived-addend-lab_mod = Labormodifikator
derived-addend-stamina = Ausdauer
derived-addend-encumbrance = Belastung
derived-addend-artes_liberales = Artes Liberales
derived-addend-philosophiae = Philosophiae
derived-addend-parma = Parma Magica
derived-addend-armor = Rüstung
derived-addend-soak_mod = Absorptionsmodifikator
derived-addend-bronze_cord = Bronzeband
derived-addend-form_bonus = Formbonus
derived-fatigue-fresh = Frisch
derived-fatigue-winded = Außer Atem
derived-fatigue-weary = Erschöpft
derived-fatigue-tired = Müde
derived-fatigue-dazed = Benommen
derived-wound-light = Leicht
derived-wound-medium = Mittelschwer
derived-wound-heavy = Schwer
derived-wound-incapacitating = Lähmend
derived-wound-dead = Tod
derived-surfaced-aging = Alterung
derived-surfaced-advancement = Steigerung
derived-surfaced-special_casting = Zauberstil
derived-surfaced-ability_roll = Fertigkeitswurf
derived-surfaced-health_roll = Gesundheitswurf
derived-detail-aging_roll = Alterungswurf
derived-detail-longevity_bonus = Langlebigkeitsbonus
derived-detail-no_aging = Altert nicht
derived-detail-decrepitude = Gebrechlichkeit
derived-detail-living_conditions = Lebensumstände
derived-detail-taught = Unterrichtet
derived-detail-book = Buch
derived-detail-vis = Vis-Studium
derived-detail-practice = Übung
derived-detail-adventure = Abenteuer
derived-detail-insight = Einsicht
derived-detail-teaching = Lehren
derived-detail-spell_mastery = Zauberbeherrschung
derived-detail-all = Alle Quellen
derived-detail-quiet_words = Stille Magie
derived-detail-subtle_gestures = Subtile Magie
derived-detail-deft_form = Gewandte Form
derived-detail-diedne = Diedne-Magie
derived-detail-faerie_raised = Feenerweckte Magie
derived-detail-life_linked_spontaneous = Lebensgebundene Spontanmagie
derived-detail-spell_improvisation = Zauberimprovisation
derived-detail-mercurian = Merkurische Magie
derived-detail-life_boost = Lebensschub
derived-detail-circumstantial = Umständeabhängig
derived-detail-fatigue_roll = Erschöpfungswurf
derived-detail-casting_fatigue = Zauber-Erschöpfung
derived-detail-recovery = Genesung
