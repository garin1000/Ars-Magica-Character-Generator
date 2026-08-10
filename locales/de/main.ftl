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
# Wird anstelle von `type-<id>` angezeigt, wenn eine geladene Datei einen
# Charaktertyp nennt, für den das aktive Regelwerk kein Profil hat — die rohe
# Kennung darf nie auf dem Bildschirm erscheinen.
type-unknown = Unbekannter Charaktertyp

# Startbildschirm: Hier beginnt die Anwendung, noch ohne geladenen Charakter. Der
# Charaktertyp wird einmalig gewählt, indem ein Charakter dieses Typs angelegt wird.
start-title = Charakter anlegen oder öffnen
start-open-title = Vorhandenen Charakter öffnen
start-create-title = Neuen Charakter anlegen
start-create-hint = Der Charaktertyp wird hier einmalig gewählt — er lässt sich später nicht mehr ändern.
start-wizard-title = Geführte Erstellung
start-wizard = Geführte Erstellung starten
start-wizard-hint = Die schrittweise geführte Erstellung kommt in einer späteren Version.

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

# Grund, der bei einer im erzwingenden Modus ausgegrauten Tugend/einem Fehler
# angezeigt wird, weil eine bereits gewählte Auswahl sie ausschließt (Groß vs.
# Klein derselben Tugend, Sanfte vs. Offensichtliche Gabe, …). { $other } ist
# die gewählte Auswahl, die sie blockiert.
vf-blocked-incompatible = Unvereinbar mit { $other }

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
# Tooltip auf dem Eigenschaften-Reiter, der den effektiven Wert erklärt (den
# Basiswert nach Alterungsminderungen und freien Tugend-Deltas). Nur sichtbar,
# wenn der effektive Wert vom Basiswert abweicht. $bought/$effective sind
# vorformatierte vorzeichenbehaftete Zahlen; $drops ist eine positive
# Minderungszahl; $bonus ist ein vorzeichenbehaftetes Delta.
characteristic-effective-tooltip-summary = Basiswert { $bought }, effektiv { $effective }.
characteristic-effective-tooltip-breakdown-label = Enthält
characteristic-effective-tooltip-aging = Alterung -{ $drops }
characteristic-effective-tooltip-virtue = Tugend { $bonus }
# Reiter-Bezeichnungen für den Hauptbereich des Editors.
tab-characteristics = Eigenschaften
tab-virtues-flaws = Tugenden & Fehler
tab-abilities = Fertigkeiten
tab-arts = Künste
tab-house-specialisation = Haus
tab-mythic-type = Typ
tab-supernatural = Übernatürlich
tab-spells = Zauber
tab-possessions = Magische Gegenstände
tab-equipment = Ausrüstung
tab-details = Details
# Gemeinsame EP-Übersicht (Fertigkeiten + Künste). Eine Zeile: das Label, der
# schreibgeschützte verbrauchte Wert, der bearbeitbare Gesamtwert des allgemeinen
# Vorrats (in Klammern), dann Verfügbar und etwaige eingeschränkte Teilvorräte.
# `xp-pool` benennt die gesamte Gruppe des allgemeinen Vorrats.
xp-pool = EP-Vorrat
xp-available = Verfügbar: { $available }
# Eingeschränkte EP-Vorräte (Gebildet/Krieger/Privilegierte Erziehung): zusätzliche
# EP, nur für die aufgeführten Fertigkeiten/Kategorien. `$eligibility` ist eine Liste.
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
# sich den EP-Vorrat der Fertigkeiten (Schlüssel `xp-pool`).
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
spell-level-min-label = Stufe min
spell-level-max-label = Stufe max
spell-group-header = { $technique } { $form }
spell-level-general = Gen
spell-cap-reason = Über deiner Zaubergrenze ({ $cap })
spell-budget-reason = Nicht genug Zauberstufen übrig
spell-already-taken-reason = Bereits ausgewählt
spell-general-level-label = Stufe (General)
spell-add = Zauber hinzufügen
spell-none = — Zauber wählen —
spell-levels-pool = Zauberstufen
spell-levels-available = Verfügbar: { $available }
spell-levels-bonus-pool = Tugenden & Fehler: { $used } / { $amount }
spell-levels-bonus = Tugenden & Fehler: { $bonus }
spell-levels-base-label = Zauberstufen-Budget
spell-mastery-xp = Meisterschafts-EP: { $xp }
spell-mastery-pool = Meisterschafts-EP: { $used } / { $pool }
spell-mastery-floor = Alle Zauber gemeistert auf { $score }
spell-mastery-label = Meisterschaft
spell-mastery-increment = Zauber-Meisterschaft erhöhen
spell-mastery-decrement = Zauber-Meisterschaft verringern
spell-mastery-abilities-label = Besondere Fähigkeiten
spell-mastery-ability-add = Besondere Fähigkeit hinzufügen
spell-mastery-ability-remove = Besondere Fähigkeit entfernen
spell-remove = Entfernen
# Details-Reiter: Alter, Selbstvertrauen (abgeleitet, schreibgeschützt), Persönlichkeit, Reputation.
age-label = Alter
apparent-age-label = Scheinbares Alter
age-cap-note = Maximaler Fertigkeitswert: { $cap }
confidence-label = Selbstvertrauen
confidence-readout = Wert { $score }, { $points } Punkte
warping-label = Verzerrung
warping-readout = Wert { $score }, { $points } Punkte
warping-effect-label = Verzerrungseffekt
warping-owed-label = Verzerrungs-Tugenden & -Fehler
warping-owed-hint = Dein Verzerrungswert gewährt diese Tugenden und Fehler (außerhalb des Budgets). Wähle für jeden Platz einen aus.
warping-owed-minor-flaws = { $count ->
    [one] { $count } Kleiner Fehler
   *[other] { $count } Kleine Fehler
}
warping-owed-supernatural-virtues = { $count ->
    [one] { $count } Übernatürliche Kleine Tugend
   *[other] { $count } Übernatürliche Kleine Tugenden
}
warping-owed-major-flaws = { $count ->
    [one] { $count } Großer Fehler
   *[other] { $count } Große Fehler
}
warping-slot-minor-flaw = Kleiner Fehler
warping-slot-supernatural-virtue = Übernatürliche Kleine Tugend
warping-slot-major-flaw = Großer Fehler
warping-choose-prompt = Auswählen…
true-faith-label = Wahrer Glaube
true-faith-readout = Wert { $score }
decrepitude-label = Gebrechlichkeit
decrepitude-readout = Wert { $score }
decrepitude-effect-label = Gebrechlichkeitseffekt (gesamt)
item-levels-label = Artefakte
item-levels-readout = { $levels } Stufen
# Identitäts-/Hintergrundfelder (Freitext, keine mechanische Wirkung).
identity-label = Identität
identity-name = Name
identity-name-placeholder = Charaktername
identity-description = Kurzbeschreibung
identity-description-placeholder = z. B. Ritter des Deutschen Ordens, Kreuzfahrer im IV. Kreuzzug
identity-concept = Konzept
identity-concept-placeholder = Beschreibe das Charakterkonzept
identity-gender = Geschlecht
identity-birth-year = Geburtsjahr
identity-sigil = Zauberer-Sigil
identity-covenant = Konvent
identity-parens = Parens
# Direkt eingegebener gealterter / verzerrter Zustand. Die angezeigten Werte für
# Gebrechlichkeit und Verzerrung berechnet die Engine aus diesen Punkten.
aging-label = Alterung
aging-points-heading = Alterungspunkte pro Eigenschaft
aging-points-note = Wertminderungen werden automatisch angewendet, sobald die angesammelten Alterungspunkte den Eigenschaftswert übersteigen; die Minderung erscheint in den abgeleiteten Werten.
warping-points-label = Verzerrungspunkte
twilight-scars-label = Zwielichtnarben
twilight-scar-placeholder = Beschreibe die Narbe
twilight-scar-add = Zwielichtnarbe hinzufügen
twilight-scars-empty = Noch keine Zwielichtnarben.
aging-log-heading = Alterungsprotokoll (pro Jahr)
aging-log-year-label = Jahr
aging-log-effect-placeholder = Beschreibe die Wirkung des Alterungswurfs
aging-log-add = Alterungseintrag hinzufügen
aging-log-empty = Noch keine Alterungseinträge.
personality-label = Persönlichkeitseigenschaften
personality-name-placeholder = Eigenschaft
personality-add = Eigenschaft hinzufügen
personality-empty = Noch keine Persönlichkeitseigenschaften.
reputations-label = Reputationen
reputation-content-placeholder = Wofür
reputation-add = { $kind }-Reputation hinzufügen (Stufe { $score })
reputation-empty = Keine Reputation verfügbar (nimm eine Tugend oder einen Fehler, der eine verleiht).
reputation-type-local = Lokal
reputation-type-ecclesiastical = Kirchlich
reputation-type-hermetic = Hermetisch
reputation-type-academic = Akademisch
# Tab "Magische Gegenstände" (Magi): Aura, Artefakte, Vertrautenbindung, Talisman
# und das Langlebigkeitsritual. Das genutzte/verbleibende Artefaktbudget stammt aus
# der Engine, wird hier nie neu berechnet.
aura-label = Angenommene Labor-/Konventaura
possessions-devices-label = Artefakte
device-name-placeholder = Name des Artefakts
device-level-label = Stufe
device-add = Artefakt hinzufügen
devices-empty = Noch keine Artefakte.
item-level-used = Artefaktstufen: { $used } / { $budget }
# Übernatürliches Wesen (Machtwert + Kräfte): Budget und effektive Werte
# stammen aus der Engine und werden hier nie neu berechnet.
supernatural-might-label = Machtwert
might-realm-label = Sphäre
might-score-label = Grund-Machtwert
might-add = Machtwert hinzufügen
might-clear = Machtwert entfernen
might-empty = Noch kein Machtwert (eine Macht-Tugend verleiht einen).
might-effective = Effektive Macht: { $realm } { $score }
might-mr = Magieresistenz (aus Macht): { $total }
supernatural-powers-label = Übernatürliche Kräfte
power-levels-used = Kraftstufen: { $used } / { $budget }
power-name-placeholder = Name der Kraft
power-level-label = Stufe
power-add = Kraft hinzufügen
powers-empty = Noch keine übernatürlichen Kräfte.
realm-magic = Magie
realm-faerie = Fee
realm-divine = Das Göttliche
realm-infernal = Das Infernale
familiar-label = Vertrauter
familiar-name-placeholder = Name des Vertrauten
familiar-cord-gold = Goldene Kordel
familiar-cord-silver = Silberne Kordel
familiar-cord-bronze = Bronzene Kordel
familiar-add = Vertrauten hinzufügen
familiar-remove = Vertrauten entfernen
# Der Kreaturenblock des Vertrauten selbst. Seine Eigenschaften gehören dem Tier
# und werden nicht aus den Punkten des Magus bezahlt; sein Machtwert erhält keine
# Tugendboni obendrauf — daher eine eigene Beschriftung statt might-score-label
# ("Grund-Machtwert"). Die Liste investierter Kräfte hat KEINE Budgetanzeige.
familiar-animal-label = Tier
familiar-animal-placeholder = z. B. ein Rabe
familiar-size-label = Größe
familiar-might-label = Magische Macht
familiar-might-score-label = Machtwert
familiar-might-add = Magische Macht hinzufügen
familiar-might-clear = Magische Macht entfernen
familiar-might-empty = Keine Magische Macht eingetragen.
familiar-characteristics-note = Die eigenen Werte des Tieres — sie werden nicht aus den Eigenschaftspunkten des Magus bezahlt.
familiar-powers-label = Investierte Kräfte
familiar-powers-note = Die Anzahl der Kräfte, die in eine Vertrautenbindung investiert werden können, ist nicht begrenzt.
familiar-bond-note = Die Bindung verleiht beiden Partnern die Kleine Tugend Wahrer Freund und die Persönlichkeitseigenschaft Loyal (Partner) +3. Ein Vertrauter ohne menschliche Intelligenz erhält sie mit Intelligenz -3. Dies wird nicht automatisch angewendet.
# Talisman: das persönliche Zaubergerät des Magus. Seine Kapazität stammt als
# Richtwert aus der Engine (höchste Technik + höchste Form, in Bauern Vim-Vis) und
# wird hier nie neu berechnet; die eingebetteten Effekte belasten kein Budget.
talisman-label = Talisman
talisman-add = Talisman hinzufügen
talisman-remove-item = Talisman entfernen
talisman-empty-item = Noch kein Talisman.
talisman-description-label = Form und Material
talisman-description-placeholder = z. B. ein Eschenstab mit Silberbeschlag
talisman-capacity = { $pawns ->
    [one] Kapazität: { $pawns } Bauer Vim-Vis
   *[other] Kapazität: { $pawns } Bauern Vim-Vis
}
# Beide Künste werden neben ihren Werten benannt, damit die Herleitung mit dem
# Charakterbogen abgeglichen werden kann; die Namen kommen aus der Regel-i18n.
talisman-capacity-note = Höchste Technik { $technique } { $techniqueScore } + höchste Form { $form } { $formScore }
talisman-attunements-label = Talismanabstimmungen
talisman-desc-placeholder = Was verstärkt wird
talisman-bonus-label = Bonus
talisman-attunement-add = Abstimmung hinzufügen
talisman-empty = Noch keine Talismanabstimmungen.
talisman-effects-label = Eingebettete Effekte
talisman-effect-name-placeholder = Name des Effekts
talisman-effect-level-label = Stufe
talisman-effect-add = Effekt hinzufügen
talisman-effects-empty = Noch keine eingebetteten Effekte.
longevity-label = Langlebigkeitsritual
longevity-source-label = Herkunft
longevity-source-self_made = Selbst erschaffen
longevity-source-external = Extern
longevity-bonus-label = Alterungsbonus
longevity-add = Langlebigkeitsritual hinzufügen
longevity-remove = Langlebigkeitsritual entfernen
# Der gespeicherte Bonus ist leer: Das Ritual wurde in einem früheren Quartal
# erschaffen, der Wert wird also eingetragen und nie berechnet. Unterscheidet ein
# leeres Feld von einer bewusst eingetragenen 0.
longevity-not-entered = Nicht eingetragen
# Der Hinweis neben dem Eingabefeld: Was ein jetzt erschaffenes Ritual wert wäre.
# { $bonus } ist bereits vorzeichenbehaftet; { $total } ist die heutige Laborsumme.
# Die Größe wird benannt („Alterungsbonus“), denn es ist der gespeicherte Wert für
# das Eingabefeld — die Gesamtwerte-Ansicht zeigt dieselbe Zahl als Modifikator.
longevity-hint = Ein jetzt erschaffenes Ritual: Alterungsbonus { $bonus } (Creo Corpus-Laborsumme { $total })
longevity-hint-halved = halbiert
longevity-focus-label = Fokus
longevity-focus-placeholder = Worin das Ritual gipfelt
longevity-sterility-note = Der Anker des Rituals verhindert, dass der Magus seine Lebenskraft auf normale menschliche Weise verausgabt; der Magus wird dadurch dauerhaft unfruchtbar.
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
# Titel der rechten Spalte (Beschreibung des gewählten Hauses + Gewährungen).
house-grants-title = Hausdetails
# In der rechten Spalte gezeigt, solange kein Haus gewählt ist.
house-none-selected = Wähle ein Haus, um seine Details zu sehen.

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

# Aktionen der Dokument-Werkzeugleiste.
action-new = Neu
action-open = Öffnen
action-save = Speichern
action-save-as = Speichern unter
action-export = Exportieren

# Bestätigung beim Schließen oder Beenden der Anwendung mit ungespeicherten Änderungen.
close-unsaved-title = Ungespeicherte Änderungen
close-unsaved-message = Dieser Charakter hat ungespeicherte Änderungen. Wenn du jetzt schließt, gehen sie verloren.
close-unsaved-discard = Verwerfen und schließen
close-unsaved-cancel = Abbrechen

# Bestätigung beim Anlegen eines neuen Dokuments oder Öffnen einer anderen Datei,
# während das aktuelle Dokument ungespeicherte Änderungen hat.
discard-changes-title = Ungespeicherte Änderungen
discard-changes-message = Dieser Charakter hat ungespeicherte Änderungen. Wenn du fortfährst, gehen sie verloren.
discard-changes-confirm = Änderungen verwerfen
discard-changes-cancel = Abbrechen

# Fenstertitel. { $name } ist der Dateiname, { $app } der Anwendungsname; die
# Variante mit ungespeicherten Änderungen stellt eine ASCII-Markierung voran.
app-title-document = { $name } — { $app }
app-title-document-dirty = *{ $name } — { $app }

# Dokumentstatus in der Kopfzeile (nicht nur im Fenstertitel): der Dateiname der
# aktiven Speicherung, mit vorangestellter ASCII-Markierung für ungespeicherte
# Änderungen, oder eine Bezeichnung für ein noch nie gespeichertes Dokument.
app-document-name = { $name }
app-document-name-dirty = *{ $name }
app-document-unsaved = Ungespeichertes Dokument
app-document-unsaved-dirty = *Ungespeichertes Dokument

# Hinweis für einen nicht ausgefüllten Parameter-Platzhalter im Namen, z. B. „Begabung in (Fertigkeit)“.
param-hint = ({ $label })
# Lokalisierte Parameter-Bezeichnungen, je Parameter-Schlüssel der Engine. Dienen auch
# als typbezogener Platzhalter/Hinweis für ein leeres Parameter-Eingabefeld.
param-label-ability = Fertigkeit
param-label-technique = Technik
param-label-art = Kunst
param-label-focus = Fokus
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
issue-multiple_magical_foci = Ein Magus darf nur einen Magischen Fokus haben, es sind aber { $count } gewählt.
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
issue-not_enough_xp = Fertigkeiten kosten { $spent } EP, mehr als die { $pool } im Vorrat.
issue-restricted_xp_unspent = { $unspent } von { $amount } eingeschränkten Erfahrungspunkten sind nicht ausgegeben und verfallen.
issue-ability_parameter_required = { $ability } braucht einen Wert (z. B. das konkrete Gebiet oder die Sprache).
issue-ability_score_out_of_range = Fertigkeit { $ability } mit Wert { $score } liegt außerhalb des erlaubten Bereichs (0 bis { $max }).
issue-ability_bonus_dangling_target = { $item } zielt auf eine Fertigkeit, die der Charakter nicht hat ({ $ability } { $parameter }); füge sie zuerst hinzu.
issue-unknown_art = Unbekannte Kunst: { $art }.
issue-duplicate_art = { $art } ist { $count }-mal aufgeführt.
issue-art_score_out_of_range = Kunst { $art } mit Wert { $score } liegt außerhalb des erlaubten Bereichs (0 bis { $max }).
issue-house_choice_unresolved = Haus { $house } hat eine offene Spezialisierungswahl ({ $choice_key }).
issue-house_grant_constraint = Haus { $house }: Die Wahl { $choice_key } fällt auf { $item }, was die Vorgabe nicht erfüllt.
issue-warping_owed_minor_flaws = Dir fehlen noch { $count } Kleine Fehler aus der Verzerrung.
issue-warping_owed_supernatural_virtues = Dir fehlen noch { $count } Übernatürliche Kleine Tugenden aus der Verzerrung.
issue-warping_owed_major_flaws = Dir fehlen noch { $count } Große Fehler aus der Verzerrung.
issue-warping_fill_constraint = Die Verzerrungswahl { $choice_key } fällt auf { $item }, was nicht zum geschuldeten Platz passt.
issue-warping_fill_ineligible = Die Verzerrungswahl { $choice_key } fällt auf { $item }, das selbst Verzerrung gewährt und keinen Verzerrungsplatz füllen kann.
issue-warping_fill_excess = Die Verzerrungswahl { $choice_key } wird nicht geschuldet und sollte entfernt werden.
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
issue-unknown_mastery_ability = Unbekannte Meisterschaftsfähigkeit { $ability } für { $spell } gewählt.
issue-too_many_mastery_abilities = { $spell } hat { $chosen } Meisterschaftsfähigkeiten, mehr als der Meisterschaftswert von { $mastery } (eine je Stufe).
issue-duplicate_mastery_ability = Meisterschaftsfähigkeit { $ability } wurde { $count }-mal für { $spell } gewählt, darf aber nur einmal genommen werden.
issue-ability_above_age_cap = { $ability } mit Wert { $score } überschreitet das Maximum von { $cap } für Alter { $age }.
issue-supernatural_ability_requires_virtue = { $ability } ist eine übernatürliche Fähigkeit und erfordert eine verleihende Tugend (oder die eine freie Fähigkeit der Gabe).
issue-personality_trait_out_of_range = Persönlichkeitseigenschaft { $name } ({ $value }) liegt außerhalb des zulässigen Bereichs (±{ $max }).
issue-reputation_not_granted = Eine Reputation ({ $kind }, { $content }) benötigt eine Tugend oder einen Fehler, der sie verleiht.
issue-over_item_level = Artefakte umfassen { $used } Stufen, über dem Budget von { $budget } (um { $over }).
issue-over_power_levels = Übernatürliche Kräfte umfassen { $used } Stufen, über dem Budget von { $budget } (um { $over }).
issue-might_realm_mismatch = Die eingegebene Macht-Sphäre ({ $base }) stimmt nicht mit der von den Tugenden verliehenen Sphäre ({ $granted }) überein.
issue-excessive_aging_reduction = Die Alterungsminderungen für { $characteristic } ({ $reduction }) würden unter den Mindestwert ({ $min }) fallen; er wird dort begrenzt.
issue-unknown_equipment = Ausrüstung '{ $item }' passt zu keiner Waffe, keinem Schild und keiner Rüstung.
issue-equipment_min_strength = { $item } benötigt Stärke { $required }, aber dieser Charakter hat { $strength }.
issue-shield_with_two_handed_weapon = Ein Schild kann nicht mit einer zweihändigen Waffe geführt werden, daher gelten seine Angriffs- und Verteidigungsmodifikatoren nicht (er zählt weiterhin zur Last).

# Ausrüstung (Waffen / Schilde / Rüstungen). Kampfwerte, Absorption und Belastung
# werden in einem späteren Schritt berechnet; diese Oberfläche erfasst nur die
# getragenen Gegenstände.
weapon-kind-melee = Nahkampf
weapon-kind-missile = Fernkampf
weapon-kind-thrown = Wurf
equipment-group-weapons = Waffen
equipment-group-shields = Schilde
equipment-group-armor = Rüstungen
equipment-equipped-label = Ausgerüstet
equipment-specialization-label = Spezialisierung greift (+1)
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
derived-section-familiar = Vertrautenbindung
derived-section-combat = Kampf
derived-section-soak = Absorption
derived-section-encumbrance = Belastung
derived-section-fatigue = Erschöpfung
derived-section-wounds = Wunden
derived-section-surfaced = Weitere Modifikatoren
derived-section-decrepitude = Gebrechlichkeit
derived-section-warping = Verzerrung
derived-size = Größe
derived-aura-label = Angenommene Labor-/Konventaura
derived-within-focus = Im Fokus
derived-deficient = (defizitär, halbiert)
derived-weak-magic = (Schwache Magie, halbiert)
derived-level = Stufe
derived-range = Reichweite
derived-load = Last
derived-burden = Beladung
derived-lab-total = Laborsumme
# Auf-Abruf-Auswahl für Labor-/Zaubersumme: Technik und Form wählen, um nur die
# passenden Summen statt der vollständigen Kombinationstabellen zu sehen.
derived-section-lab-casting = Labor- & Zaubersummen
derived-picker-technique = Technik
derived-picker-form = Form
derived-combat-empty = Keine Waffen ausgerüstet.
derived-cast-formulaic = Formulaisch
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
# Verbindet eine Waffe mit dem dazu getragenen Schild in einer Kampfzeile
# ("Langschwert & Tartsche"); die deutschen Regeln schreiben ebenfalls "&".
# Die umgebenden Leerzeichen setzt der Code, da ein Fluent-Wert nicht mit einem
# Leerzeichen beginnen oder enden darf.
derived-combat-shield-joiner = &
derived-longevity-self_made = Selbst erschaffen
derived-longevity-external = Extern
# Für das Ritual wurde noch kein Bonus eingetragen, es gibt also keine Zahl.
derived-longevity-not-entered = nicht eingetragen
# Diese Ansicht zeigt das Ritual als Wirkung: den Modifikator, der vom
# Alterungswurf abgezogen wird. Ein gespeicherter Bonus von 7 liest sich hier also
# als „-7 auf Alterungswürfe“, während das Eingabefeld die gespeicherte Größe
# zeigt („Alterungsbonus +7“) — jede Zeichenkette benennt ihre eigene Größe.
# { $modifier } kommt aus formatSigned, eine 0 wird also „0“ und nie „-0“.
derived-longevity-aging-modifier = { $modifier } auf Alterungswürfe
derived-longevity-suggested = ein jetzt erschaffenes Ritual: { $modifier } auf Alterungswürfe
derived-masterpiece-cap = Höchste Stufe schlichtes Artefakt
derived-masterpiece-note = Nur ein Hinweis: Das eigentliche schlichte Artefakt unter Magische Gegenstände entwerfen (Vis-Kosten werden ignoriert).
# Vertrautenbindung: Alle Werte sind reine Hinweise. Die Laborsumme im Fokus ist
# bedingt — ob das Tier in den Fokus fällt, entscheidet die Spielgruppe.
derived-familiar-binding-level = Bindungsstufe
derived-familiar-cord-points = Aufgewendete Kordelpunkte
derived-familiar-invested-levels = Investierte Kraftstufen
derived-familiar-reaches = Laborsumme erreicht die Bindungsstufe.
derived-familiar-falls-short = Laborsumme erreicht die Bindungsstufe nicht.
derived-familiar-cords-fit = Kordelpunkte liegen innerhalb der Laborsumme.
derived-familiar-cords-exceed = Kordelpunkte übersteigen die Laborsumme.
derived-familiar-note = Nur ein Hinweis: Welche Künste zum Tier passen und ob ein Magischer Fokus greift, entscheidet die Spielgruppe (Vis-Kosten werden ignoriert).
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
derived-addend-might = Macht
derived-addend-armor = Rüstung
derived-addend-soak_mod = Absorptionsmodifikator
derived-addend-bronze_cord = Bronzene Kordel
derived-addend-form_bonus = Formbonus
derived-fatigue-fresh = Frisch
derived-fatigue-winded = Außer Atem
derived-fatigue-weary = Erschöpft
derived-fatigue-tired = Müde
derived-fatigue-dazed = Betäubt
derived-wound-light = Leicht
derived-wound-medium = Mittelschwer
derived-wound-heavy = Schwer
derived-wound-incapacitating = Lähmend
derived-wound-dead = Tot
derived-surfaced-aging = Alterung
derived-surfaced-advancement = Fortschritt
derived-surfaced-special_casting = Zauberstil
derived-surfaced-ability_roll = Fertigkeitswurf
derived-surfaced-health_roll = Gesundheitswurf
derived-surfaced-magic_resistance = Magiewiderstand
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
derived-detail-teaching = Unterrichten
derived-detail-spell_mastery = Zaubermeisterschaft
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
derived-detail-susceptible_divine = Anfällig für das Göttliche
derived-detail-susceptible_faerie = Anfällig für das Feenreich
derived-detail-susceptible_infernal = Anfällig für das Infernale
derived-detail-aura_bonus = Aurabonus

# Markdown-Export (Bogen exportieren). Dokumenttext, den nur der Export benötigt:
# selbst zusammengesetzte Tabellenköpfe, die beiden Konvents-Punktegruppen und die
# Ja/Nein-Marker, die eine Markdown-Tabelle ausschreiben muss. Alle übrigen
# Überschriften und Beschriftungen des Bogens nutzen die Schlüssel oben, damit das
# Dokument dieselben Wörter verwendet wie die App. Maßgebliche Liste ist
# `arm_rules::export::LABEL_KEYS`; ein fehlender Schlüssel würde als sein eigener
# Name ausgegeben.
export-untitled = Unbenannter Charakter
export-col-effective = Effektiv
export-col-magnitude = Magnitude
export-col-penalty = Abzug
export-col-points = Punkte
# Kopf der einen Kunst-und-Stufe-Spalte der exportierten Zauberliste: die Kürzel von
# Technik und Form, gefolgt von der Stufe — die Kurzform des Regelwerks (CrIg20).
export-col-spell-code = TeFo/Stufe
export-col-total = Summe
# Typ-Spalte der exportierten Tugend-/Fehler-Tabellen: die Kategorie des Eintrags
# (Hermetisch, Geschichte, …) — derselbe Wert, den das Abzeichen in der App über
# `category-<id>` anzeigt.
export-col-type = Typ
export-items-boons = Vorzüge
export-items-hooks = Haken
# Unterüberschrift der Tugend-/Fehler-Tabellen für die freien Einträge außerhalb des
# Budgets, die Haus, mythischer Typ oder Verzerrung gewähren (nie in der Bilanz).
export-granted = Gewährt
export-xp-restricted = Eingeschränkte Erfahrungspunkte
export-yes = Ja
export-no = Nein
