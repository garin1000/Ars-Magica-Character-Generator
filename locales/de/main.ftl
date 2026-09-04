# Oberflächentexte für den Ars-Magica-Charaktergenerator.
# Regeltexte (Namen und Zusammenfassungen von Tugenden/Fehlern) stehen NICHT
# hier — sie kommen über den Befehl load_ruleset aus rules/i18n/<lang>/.

app-title = Ars-Magica-Charaktergenerator
app-logo-alt = Logo „Ars Magica Open License“

language-label = Sprache
# Beschriftungen der Sprachauswahl selbst: das jeweilige Endonym der Sprache
# (nie eine Übersetzung in die gerade aktive Sprache — eine Sprachauswahl zeigt
# immer „Deutsch“, unabhängig davon, welche Sprache aktiv ist).
language-name-en = English
language-name-de = Deutsch
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
# Der zweite Weg über eine Datei: dasselbe Öffnen, das aber in der geführten
# Erstellung landet statt im Editor.
action-open-into-wizard = In geführter Erstellung öffnen
start-open-wizard-hint = Ein Charakter, der mitten in der geführten Erstellung gespeichert wurde, wird bei dem Schritt fortgesetzt, auf dem er verlassen wurde. Ein im Editor angelegter Charakter wird mit allen erreichbaren Schritten geöffnet.
start-create-title = Neuen Charakter anlegen
start-create-hint = Der Charaktertyp wird hier einmalig gewählt — er lässt sich später nicht mehr ändern.
# S28 (full-audit UX): benennt die Prüfmodus-Achse (Erzwungen/Beratend/Still),
# die direkt-geprüfte von direkt-ungeprüfter Erstellung unterscheidet — auf
# diesem Bildschirm sonst unsichtbar. Die Prüfung-Steuerung in der Werkzeugleiste
# erscheint erst, sobald ein Charakter existiert (App.svelte), daher kann dieser
# Hinweis nur darauf vorausweisen.
start-create-mode-hint = Wie streng die Regeln geprüft werden (Prüfung, in der Werkzeugleiste), lässt sich jederzeit ändern, sobald der Charakter geöffnet ist.
start-wizard-title = Geführte Erstellung
start-wizard-hint = Schritt für Schritt durch die Erstellungsphasen dieses Typs, in ihrer Reihenfolge. Der Charaktertyp wird auch hier einmalig gewählt — er kann später nicht geändert werden.

# Bezeichnungen der Erstellungsschritte, geschlüsselt nach dem `CreationPhase`-Slug
# der Engine. Sie benennen die Schritte der geführten Erstellung und sind die
# einzige Darstellung einer Phase — der Slug selbst erscheint nie auf dem
# Bildschirm. `review` ist der eigene Abschlussschritt des Assistenten, angefügt
# nach den Phasen, die der Charaktertyp deklariert.
phase-concept = Konzept
phase-characteristics = Eigenschaften
phase-virtues_flaws = Tugenden & Fehler
phase-experience = Erfahrung
phase-abilities = Fertigkeiten
phase-arts = Künste
phase-spells = Zauber
phase-house_specialisation = Haus
phase-mythic_type = Typ des mythischen Gefährten
phase-personality_reputations = Persönlichkeit & Reputationen
phase-aging = Alterung
phase-review = Überprüfung

# Rahmen der geführten Erstellung. Die Leiste listet die Schritte; der Inhalt eines
# Schritts ist dieselbe Eingabefläche, die auch die Reiter des Editors verwenden.
wizard-rail-label = Erstellungsschritte
wizard-step-progress = Schritt { $current } von { $total }
wizard-back = Zurück
wizard-next = Weiter
wizard-finish = Fertigstellen
# Warum „Weiter" deaktiviert ist: der aktuelle Schritt enthält einen Fehler. Nur
# Fehler blockieren — ein Hinweis nie — und der Prüfmodus „Beratend" hebt die
# Sperre auf.
wizard-blocked-hint = Behebe die Fehler dieses Schritts, um fortzufahren, oder wechsle den Prüfmodus auf „Beratend".
# Kennzeichnet einen Schritt in der Leiste, der noch einen Fehler enthält.
wizard-step-blocked-label = enthält Fehler
# Kennzeichnet einen Schritt in der Leiste, der noch eine WARNUNG enthält — offene
# Arbeit, die nichts blockiert; der Schritt bleibt also offen statt gesperrt.
# Bewusst schwächer als `wizard-step-blocked-label` und nur für bereits erreichte
# Schritte.
wizard-step-pending-label = enthält offene Warnungen
# Kennzeichnet einen Schritt in der Leiste, für den noch nichts eingetragen wurde.
# Regelkonform ist nicht dasselbe wie fertig: ein leerer Schritt wird gekennzeichnet,
# aber nie blockiert.
wizard-step-incomplete-label = nicht begonnen
# Wird angezeigt, solange der Prüfmodus „Beratend" oder „Stumm" ist: es wird nichts
# erzwungen, also blockiert kein Schritt und „Fertigstellen" ist immer möglich.
wizard-unchecked-hint = Die Prüfung wird nicht erzwungen, daher blockiert kein Schritt.

# Der Abschlussschritt des Assistenten.
wizard-review-title = Überprüfung
wizard-review-clean = Keine Fehler oder Hinweise — dieser Charakter ist regelkonform.
# Ehrlich darüber, was die Sperren prüfen und was nicht: die Schritte blockieren nur
# bei Fehlern, ein regelkonformer Charakter kann also unfertig sein. Leitet die Liste
# der leer gebliebenen Schritte ein; nichts davon hält „Fertigstellen" auf.
wizard-review-incomplete = Ein regelkonformer Charakter ist nicht zwangsläufig ein fertiger: Schritte blockieren nur bei Fehlern, diese hier blieben also leer. Du kannst trotzdem fertigstellen.
# Wird anstelle dieser Liste angezeigt, sobald jeder Schritt eine Eintragung enthält.
wizard-review-complete = Für jeden Schritt dieser Erstellung sind Eintragungen vorhanden.
wizard-review-hint = Ausrüstung, magische Gegenstände, Macht und Verzerrung sind nicht Teil der geführten Erstellung — sie werden nach dem Fertigstellen bearbeitet.

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
# Barrierefreie Namen für Filterauswahlen ohne eigene sichtbare Beschriftung.
ability-category-filter-label = Nach Fertigkeitskategorie filtern
# { $side } ist der lokalisierte Seitentitel ("Tugenden"/"Fehler").
vf-category-filter-label = { $side } nach Kategorie filtern
vf-magnitude-filter-label = { $side } nach Stufe filtern

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
# Reiter-Bezeichnungen für den Hauptbereich des Editors. Jeder Reiter, der einer
# Phase des Assistenten entspricht, trägt denselben Text wie diese Phase.
tab-characteristics = Eigenschaften
tab-virtues-flaws = Tugenden & Fehler
tab-experience = Erfahrung
tab-abilities = Fertigkeiten
tab-personality-reputations = Persönlichkeit & Reputationen
tab-aging = Alterung
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
# Der Beitrag von Tugenden & Fehlern zum allgemeinen Vorrat: Erfahrener Parens
# gewährt 60 zusätzliche Erfahrungspunkte während der Lehrzeit
# (Basisregeln.md:4966), Schwacher Parens nimmt 60 weg. Ein POSITIVER Beitrag
# ist ein eigener Vorrat, der vor der Basis ausgegeben wird — genau wie ein
# eingeschränkter Vorrat; ein NEGATIVER hat nichts auszugeben und wird der Basis
# angelastet, erscheint also als vorzeichenbehafteter Modifikator. Gleiche
# Aufteilung und gleiche Formulierung wie bei den Zauberstufen.
xp-bonus-pool = Tugenden & Fehler: { $used } / { $amount }
xp-bonus = Tugenden & Fehler: { $bonus }
# Eingeschränkte EP-Vorräte (Gebildet/Krieger/Privilegierte Erziehung): zusätzliche
# EP, nur für die aufgeführten Fertigkeiten/Kategorien. `$eligibility` ist eine Liste.
restricted-xp-pool = { $eligibility }: { $used } / { $amount }
restricted-xp-list-separator = ,
# Erfahrungsblöcke der Lebensabschnitte, geschlüsselt nach dem `LifeStageBlock`-Slug
# der Engine. Jeder ist ein eigener eingeschränkter Vorrat: der erste kauft die
# Muttersprache und nichts anderes, der zweite die Kindheitsfertigkeiten, aber nie
# jene Sprache. Das spätere Leben erscheint bei einem Magus, dessen allgemeiner
# Vorrat stattdessen die Lehrlingszeit ist — seine Jahre erbringen nur
# Fertigkeiten, niemals eine Kunst, was das Label ausdrücklich sagt.
#
# Diese Schlüssel benennen einen Block INNERHALB EINES SATZES, nicht in der Leiste:
# `resolveIssueArgValue` löst damit das `origin`-Argument einer Warnung über nicht
# ausgegebene Erfahrung auf. Die Chips der EP-Leiste sind die `xp-pool-block-*`-Familie
# weiter unten, die die Herleitung eines Blocks mit seinem Vorrat zusammenführt — die
# Leiste darf nicht auf diese Schlüssel zurückgreifen, sonst trägt ein Block wieder
# zwei Labels (guided-creation-review-2026-08 #14).
xp-pool-childhood_native_language = Muttersprache
xp-pool-childhood_spread = Frühe Kindheit
xp-pool-later_life = Späteres Leben (nur Fertigkeiten)
# Der Umschalter für die Herkunft der Erfahrung der Fertigkeiten. Sie stammt
# entweder aus einem selbst eingetragenen Vorrat oder aus den Lebensabschnitten des
# Charakters. Der Umschalter ist kein gespeichertes Kennzeichen: ein Plan der
# Lebensabschnitte am Charakter IST die geführte Finanzierung, eine geladene Datei
# landet also in dem Modus, den ihre eigenen Daten vorgeben.
ability-funding-label = Herkunft der Erfahrung
ability-funding-pool = EP-Vorrat
ability-funding-life_stages = Lebensabschnitte
life-stage-no-budget = Noch keine Erfahrung aus Lebensabschnitten.
# DIE CHIPS DER LEBENSABSCHNITTE — ein Chip pro Block, in der Reihenfolge, in der der
# Charakter sie gelebt hat (guided-creation-review-2026-08 #14). Die Regeln nennen die
# Blöcke als geordnete Folge — „Frühe Kindheit … Späteres Leben … Lehrlingszeit …
# Jahre nach der Lehrlingszeit“ (Basisregeln.md:2213-2216) und nochmals als Abfolge
# von Perioden (`:2364`) — die Leiste liest sich von oben nach unten also als diese
# Abfolge.
#
# Jeder Schlüssel führt die HERLEITUNG des Blocks mit dem Ausgegeben/Gesamt seines
# eingeschränkten Vorrats zusammen, denn zwei Chips mit dem Namen eines Blocks (das
# frühere `life-stage-later-life` neben der Zeile `xp-pool-later_life`) ließen dasselbe
# Label zweimal erscheinen und wirkten wie zwei verschiedene Vorräte.
#
# Die 75 der Kindheit für die Muttersprache und die 45 für die Verteilung sind ein
# Block, nicht zwei (`:2378`), und teilen daher eine Überschrift. Die Variante nur mit
# der Verteilung ist der Zustand, bevor eine Muttersprache benannt ist: die Engine
# bildet jenen Vorrat erst dann (`effective/xp.rs`), und ein Chip darf keinen Vorrat
# behaupten, den es nicht gibt.
xp-pool-block-early-childhood = Frühe Kindheit — Muttersprache { $nativeUsed } / { $nativeAmount } · Übrige Fertigkeiten { $spreadUsed } / { $spreadAmount }
xp-pool-block-early-childhood-spread-only = Frühe Kindheit — Übrige Fertigkeiten { $spreadUsed } / { $spreadAmount }
# Späteres Leben: „15 Erfahrungspunkte pro Jahr (bis zur Lehrlingszeit für Magi)“
# (`:2214`). Die Altersspanne ist die dieses Charakters — von den Jahren der Kindheit
# bis zum Beginn der Lehrlingszeit — und sagt dem Spieler, aus welchen Jahren diese
# Punkte stammen: das Beispiel Darius gibt genau diesen Block über die Jahre 5 bis 10
# aus (`:2402`). Die Rate ist ebenfalls die dieses Charakters; die Tugend Wohlhabend
# und der Fehler Arm ändern sie (`:2394`). Zwei Varianten, weil das spätere Leben nur
# bei einem Magus ein eigener Vorrat ist, dessen allgemeiner Vorrat die Lehrlingszeit
# ist; bei allen anderen IST das spätere Leben der allgemeine Vorrat, der schon als
# Gesamtwert der Leiste steht — ihn zu wiederholen wäre ein zweites Ausgegeben/Gesamt
# für einen Vorrat.
xp-pool-block-later-life = Späteres Leben (Alter { $from }-{ $to }): { $years } × { $rate } = { $xp } EP
xp-pool-block-later-life-restricted = Späteres Leben (Alter { $from }-{ $to }): { $years } × { $rate } = { $xp } EP — { $used } / { $amount }
# Die Lehrlingszeit, allein für einen Magus: fünfzehn feste Jahre, deren Erfahrung
# auf Künste oder Fertigkeiten verwendet werden kann (Basisregeln.md:2435), was sie
# zum allgemeinen Vorrat macht — dieser Chip benennt also den Block, aus dem der
# Gesamtwert stammt, und braucht kein eigenes Ausgegeben/Gesamt. Fehlt bei jedem, der
# keine Lehrlingszeit dient.
xp-pool-block-apprenticeship = Lehrlingszeit: { $years } Jahre = { $xp } EP
# Die Jahre nach der Lehrlingszeit: „Für jedes Jahr erhält der Magus 30 Punkte“
# (Basisregeln.md:2471), abzüglich 10 für jedes angerechnete Quartal Laborarbeit
# (`:2482`). Jeder Punkt ist ein Erfahrungspunkt oder eine Zauberstufe, daher werden
# sowohl die Punkte als auch die nach den Zauberstufen verbleibende Erfahrung genannt.
#
# „Nach der Lehrlingsprüfung“, nicht „Als Magus“ (#14.4): Der Block hängt am Feld
# „Alter bei der Lehrlingsprüfung“, und den Chip nach der Prüfung zu benennen bindet
# Label an Feld. Zudem liest sich der Chip so nicht mehr als ZUSTAND des Charakters —
# was an seiner früheren Stelle das darunter stehende „Späteres Leben“ wie das Leben
# nach der Lehrlingsprüfung wirken ließ.
xp-pool-block-after-gauntlet = Nach der Lehrlingsprüfung: { $years } × { $rate } - { $lab } für Laborarbeit = { $points } Punkte, { $xp } EP
# Das Alter wird im Bereich wiederholt, weil das spätere Leben in Jahren gemessen
# wird — es wird hier ebenso bearbeitet wie im Details-Reiter.
life-stage-age-label = Alter
life-stage-gauntlet-age-label = Alter bei der Lehrlingsprüfung
life-stage-lab-seasons-label = Laborquartale
life-stage-spell-levels-label = Zauberstufen
# Warum beide Felder darüber schreibgeschützt sind: die Aussage über einen
# schreibgeschützten Zustand, den die Felder selbst nicht treffen können; sie ist
# das `aria-describedby`-Ziel beider Felder.
life-stage-post-gauntlet-no-years-note = Noch keine Jahre als Magus, daher können Laborquartale und Zauberstufen nichts aufnehmen.
life-stage-post-gauntlet-summary = { $years } Jahre als Magus: { $points } Punkte = { $xp } EP + { $levels } Zauberstufen
# Notausgang für eine von Hand bearbeitete Datei: ein Charakter, der über seine
# Lebensabschnitte finanziert wird, darf keinen eingetragenen Vorrat führen, und die
# geführte Erstellung bietet kein Feld, um ihn zu berichtigen — dies leert ihn.
# Die Muttersprache: der erste Block der Kindheit kauft diese Fertigkeit und nichts
# anderes, ohne sie gibt es kein Budget der Kindheit.
native-language-label = Muttersprache
native-language-placeholder = z. B. Deutsch
# Beispielhafte Kindheiten: fertige Fertigkeitspakete für die Erfahrung der frühen
# Kindheit. Die Namen der Pakete sind Regeltext (rules/i18n/<lang>/childhoods.json),
# niemals Schlüssel hier. Die Punkte selbst zu verteilen ist eine gleichwertige Wahl
# und kein Verzicht, daher ist es die erste Option der Auswahl und keine leere Wahl.
childhood-label = Beispielhafte Kindheit
childhood-taken = Genommenes Fertigkeitspaket: { $name }
childhood-choose-prompt = — Erfahrung der Kindheit selbst verteilen —
childhood-preview-label = Vorschau des Pakets
childhood-entry = { $name } { $score }
# Ein Platz, den das Paket offen lässt (das Gebiet einer Gebietskunde, die Sprache
# einer Lebenden Sprache). Sein Label ist der Fertigkeitsname des Eintrags —
# Platz-Kennungen sind Paketdaten und dürfen nie angezeigt werden — mit einer
# Ordnungszahl nur dort, wo ein Paket dieselbe Fertigkeit zweimal vergibt (die zwei
# Gebietskunden der Reisenden Kindheit).
childhood-slot-label = { $name }
childhood-slot-label-nth = { $name } ({ $index })
childhood-apply = Diese Kindheit nehmen
# Warum das Nehmen des Pakets blockiert ist. Zwei Plätze derselben Fertigkeit mit
# gleichem Wert würden zu einer Zeile verschmelzen und die Erfahrung des anderen
# verschwenden; eine Sprache der Kindheit muss sich von der Muttersprache
# unterscheiden, die ihren eigenen Block hat.
childhood-slot-empty-reason = Fülle jede Fertigkeit aus, die das Paket offen lässt.
childhood-slot-duplicate-reason = Zwei Plätze derselben Fertigkeit brauchen verschiedene Werte, sonst verschmelzen sie zu einer Zeile und Erfahrung geht verloren.
childhood-slot-native-reason = Eine Sprache der Kindheit muss sich von der Muttersprache unterscheiden.
# Die hermetischen Mindestfertigkeiten, die ein Magus schuldet. Die erste Gruppe ist
# die Aufnahme in den Orden — Charaktere mit niedrigeren Werten würden nicht
# aufgenommen (Basisregeln.md:2437) —, die zweite das empfohlene Paket des Regelwerks
# (`:2451-2461`), das ein Rat und daher eine Warnung ist. Jede Zeile nennt ihren
# Zustand als ganzen Satz, statt eine Vorlage um ein angeklebtes erfüllt/nicht erfüllt
# zu ergänzen, damit nichts von der Farbe getragen wird und das Deutsche natürlich ist.
magus-minimums-label = Mindestfertigkeiten
# Die Beschriftung der eingeklappten Liste (guided-creation-review-2026-08 #12) und
# daher mit eigenem Bezug: sie ist jetzt die Zusammenfassung des Aufklappelements und
# keine Zeile unter der Überschrift „Mindestfertigkeiten" mehr, und { $total } zählt
# jede Zeile darunter — die verlangten wie die empfohlenen.
magus-minimums-summary = Anforderungen an Fertigkeiten: { $unmet } von { $total } noch nicht erfüllt
magus-minimum-met = { $ability } { $min }{ $qualifier } ist erfüllt: dieser Charakter hat { $score }.
magus-minimum-unmet = { $ability } { $min }{ $qualifier } ist nicht erfüllt: dieser Charakter hat { $score }.
magus-recommended-label = Empfohlene Mindestfertigkeiten
ability-score-label = Wert
ability-specialty-label = Spezialisierung
# Überschrift für die im Auswahlbereich gezeigten Beispiel-Spezialisierungen.
ability-specialties-label = Spezialisierungen
ability-add = Fertigkeit hinzufügen
# Je Zeile benannt ($name ist der Anzeigename der Fertigkeit), damit eine
# Sprachausgabe in einer langen Fertigkeiten-Liste nicht auf jeder Zeile
# dasselbe bloße "Erhöhen"/"Verringern" vorliest.
ability-increment = { $name } erhöhen
ability-decrement = { $name } verringern
# Hermetische Künste: die beiden Klassen und die Bedienelemente. Künste teilen
# sich den EP-Vorrat der Fertigkeiten (Schlüssel `xp-pool`).
art-type-technique = Techniken
art-type-form = Formen
art-add = Kunst hinzufügen
# Je Zeile benannt ($name ist der Anzeigename der Kunst) — dieselbe Begründung
# wie bei ability-increment/-decrement oben.
art-increment = { $name } erhöhen
art-decrement = { $name } verringern
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
# Der Grundwert als eigener Eintrag, denn er ist NICHT MEHR der Nenner neben dem
# ausgegebenen Wert (guided-creation-review-2026-08 #18): Ein Magus nach seiner
# Lehrlingsprüfung wird gegen Grundwert + Stufen nach der Prüfung belastet, der
# ausgegebene Wert neben dem Grundwert allein zeigte also „150 / 120  Verfügbar: 0“ —
# eine Überschreitung, die die Engine nie meldete. Das Paar schließt jetzt als
# „150 / 150“, und der Grundwert steht daneben: im Editor bearbeitbar, in der
# geführten Erstellung nur zur Anzeige (#19).
spell-levels-base-entry = Grundwert
spell-levels-bonus-pool = Tugenden & Fehler: { $used } / { $amount }
spell-levels-bonus = Tugenden & Fehler: { $bonus }
# Die Zauberstufen, die die Jahre des Magus seit seiner Lehrlingsprüfung
# eingebracht haben: der gewählte Anteil der 30 Punkte pro Jahr, von denen jeder
# entweder ein Erfahrungspunkt in einer Kunst oder Fertigkeit oder eine
# Zauberstufe sein kann (Basisregeln.md:2471). Benannt wie derselbe Block in der
# EP-Leiste (`xp-pool-block-after-gauntlet`), damit ein Block in beiden Leisten
# gleich liest — deshalb wurde dieses Label mit umbenannt
# (guided-creation-review-2026-08 #14.4). Hier NUR ZUR ANZEIGE: Die Aufteilung wird einmal im Schritt
# Fertigkeiten gewählt, denn die Phasenfolge des Magus lautet Fertigkeiten,
# Künste, Zauber — eine Änderung hier würde einen zwei Schritte zuvor bereits
# ausgegebenen Vorrat nachträglich verkleinern. Anders als der Tugenden-/Fehler-
# Modifikator sind diese Stufen bereits verdient und erhöhen daher das Verfügbar,
# statt einen eigenen Vorrat zu bilden. Nur sichtbar, wenn es welche gibt — ein
# Magus an seiner Lehrlingsprüfung bleibt so unverändert.
spell-levels-post-gauntlet = Nach der Lehrlingsprüfung: { $levels }
# Überschreibt allein den Grundwert des Typprofils — der Tugenden-/Fehler-
# Modifikator und die Zauberstufen aus den Jahren als Magus kommen obendrauf.
spell-levels-base-label = Zauberstufen-Budget
spell-mastery-xp = Meisterschafts-EP: { $xp }
spell-mastery-pool = Meisterschafts-EP: { $used } / { $pool }
spell-mastery-floor = Alle Zauber gemeistert auf { $score }
spell-mastery-label = Meisterschaft
# Je Zeile benannt ($name ist der Anzeigename des Zaubers) — dieselbe
# Begründung wie bei ability-increment/-decrement oben.
spell-mastery-increment = Zauber-Meisterschaft für { $name } erhöhen
spell-mastery-decrement = Zauber-Meisterschaft für { $name } verringern
spell-mastery-abilities-label = Besondere Fähigkeiten
spell-mastery-ability-add = Besondere Fähigkeit hinzufügen
# Generischer Entfernen-Regler für die Listenzeilen (Fertigkeiten, Tugenden &
# Fehler, Alterungsprotokoll, Persönlichkeitsmerkmale, Dämmerungsnarben,
# Vertrauten-Eigenschaften/-Kräfte, Magische Gegenstände, Talisman-Bindungen/
# -Effekte, Reputationen, Zauber und deren Meisterschafts-Sonderfähigkeiten):
# nennt den Inhalt der jeweiligen Zeile, damit eine Sprachausgabe nicht in
# jeder Zeile derselben langen Liste dasselbe bloße "Entfernen" vorliest.
remove-item = { $name } entfernen
# Details-Reiter: Alter, Selbstvertrauen (abgeleitet, schreibgeschützt), Persönlichkeit, Reputation.
age-label = Alter
apparent-age-label = Scheinbares Alter
confidence-label = Selbstvertrauen
confidence-readout = Wert { $score }, { $points } Punkte
warping-label = Verzerrung
warping-readout = Wert { $score }, { $points } Punkte
warping-effect-label = Verzerrungseffekt
warping-owed-label = Verzerrungs-Tugenden & -Fehler
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
# Das Jahr der Saga (guided-creation-review-2026-08 #25): eine appweite Einstellung,
# nicht Teil eines Charakters. "Saga" bleibt unübersetzt
# (rules/source/de/translation-tables/grundbegriffe.md:104).
saga-year-label = Jahr der Saga
identity-sigil = Zauberer-Sigil
identity-covenant = Konvent
identity-parens = Parens
# Direkt eingegebener gealterter / verzerrter Zustand. Die angezeigten Werte für
# Gebrechlichkeit und Verzerrung berechnet die Engine aus diesen Punkten.
aging-label = Alterung
aging-points-heading = Alterungspunkte pro Eigenschaft
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
# Was ein aufgelöstes Jahr bewirkt hat, gelesen aus den Feldern, die die Engine
# festgehalten hat: der geworfene Würfel, der Gesamtwert, die vergebenen
# Alterungspunkte und das Jahr scheinbaren Alters. Wird angezeigt, nie gespeichert —
# ein Spielstand hält Entscheidungen fest, und gespeicherter Text würde eine Sprache
# in die Datei einfrieren. Punkte und Satz zum scheinbaren Alter stammen aus dem
# Rechner (`aging-outcome-*`), damit Wurf und Eintrag gleich lauten.
aging-log-roll = Alterungswurf gesamt { $total } bei einem Stresswürfel von { $die }.
aging-log-points-none = Keine Alterungspunkte.
aging-log-note-placeholder = Notiz (optional)
# Eine eingetragene Krise, gelesen aus dem Eintrag, der sie festgehalten hat. Drei
# Zustände sind unterscheidbar: keine Krise, eine von der Tabelle geforderte, die
# niemand gewürfelt hat, und eine auf der Krisentabelle aufgelöste
# (Core Rules.md:16619-16632).
aging-log-crisis-unrolled = Krise: offen, der Einfachwürfel wurde nicht geworfen.
aging-log-crisis = Krise: { $row } — Krisen-Gesamtwert { $total } bei einem Einfachwürfel von { $die }.
aging-log-crisis-severity = Krise: { $row } ({ $severity }) — Krisen-Gesamtwert { $total } bei einem Einfachwürfel von { $die }.
# Die Übersicht der Alterungswürfe: für welche Jahre ein Wurf fällig ist, wie
# viele bereits eingetragen sind, und die feststehende (würfelunabhängige) Hälfte
# des Alterungswurfs. Alle Zahlen stammen aus der Engine — die Altersschwelle
# wird aus den Regeln gelesen, nie hier als feste Zahl geschrieben.
aging-schedule-label = Alterungswürfe
aging-rolls-none = Bisher sind keine Alterungswürfe fällig.
aging-first-roll-age = Die Alterung beginnt nach dem Alter { $begins }; der erste Wurf fällt im Alter von { $first } an.
aging-rolls-owed = { $count ->
    [one] { $count } Alterungswurf fällig, im Alter von { $from }.
   *[other] { $count } Alterungswürfe fällig, für die Alter { $from } bis { $to }.
}
aging-rolls-years = { $count ->
    [one] Kalenderjahr { $from }.
   *[other] Kalenderjahre { $from } bis { $to }.
}
aging-rolls-recorded = { $recorded } von { $owed } eingetragen
# Alterungswurf = Stresswürfel (kein Patzer) + Alter/10 (aufgerundet)
# - Lebensumständemodifikator - Langlebigkeitsritual (Core Rules.md:16567-16569);
# ein hoher Modifikator bedeutet also ein längeres Leben. Jeder Term kommt
# bereits mit Vorzeichen an.
# Der vierte Term ist der Alterungswurf-Modifikator aus Tugenden und Fehlern
# (Feenblut -1, Core Rules.md:3801). Die drei Zeilen des Buches nennen ihn nicht,
# die Gesamtsumme rechts vom `=` enthält ihn aber — ohne ihn ging die Rechnung
# nicht auf. Wortgleich mit `aging-total-parts` weiter unten, damit beide
# Anzeigen übereinstimmen.
aging-total-formula = Stresswürfel { $age } (Alter) { $conditions } (Lebensumstände) { $longevity } (Langlebigkeitsritual) { $traits } (Tugenden und Fehler) = Stresswürfel { $fixed }
# Die Auswahl der Lebensumstände (Core Rules.md:16581-16594). Der angezeigte Wert
# ist der von der Engine berechnete Lebensumständemodifikator — er enthält auch
# die Beiträge von Tugenden und Fehlern, die hier keine eigene Zeile haben.
living-conditions-label = Lebensumstände
living-conditions-total = Lebensumständemodifikator: { $modifier }
living-conditions-cumulative-label = kumulativ
# Der Alterungswurf-Rechner (Core Rules.md:16567-16615). Der Spieler wirft den
# Stresswürfel am Tisch und trägt ihn hier ein — die App würfelt nie, und der
# Würfelwurf wird nie am Charakter gespeichert. Alle Zahlen stammen aus der
# Engine, und nichts wird eingetragen, bevor der Wurf angewendet wird.
aging-roll-label = Alterungswurf
aging-year-label = Jahr für den Wurf
aging-year-option = { $recorded ->
    [yes] Alter { $age } — bereits eingetragen
   *[no] Alter { $age }
}
aging-year-option-dated = { $recorded ->
    [yes] Alter { $age } ({ $year }) — bereits eingetragen
   *[no] Alter { $age } ({ $year })
}
aging-die-label = Stresswürfel
aging-total-readout = Alterungswurf gesamt: { $total }
aging-total-parts = { $die } (Stresswürfel) { $age } (Alter) { $conditions } (Lebensumstände) { $longevity } (Langlebigkeitsritual) { $traits } (Tugenden und Fehler)
aging-die-capped = Das Langlebigkeitsritual begrenzt diesen Wurf: { $uncapped } zählt als { $total }.
aging-outcome-apparent_age = Das scheinbare Alter steigt um ein Jahr.
aging-outcome-no_apparent_aging = Das scheinbare Alter steigt nicht.
aging-outcome-points_any = { $points ->
    [one] 1 Alterungspunkt in einer Eigenschaft deiner Wahl.
   *[other] { $points } Alterungspunkte in beliebigen Eigenschaften deiner Wahl.
}
aging-outcome-points_fixed = { $points ->
    [one] 1 Alterungspunkt in { $characteristic }.
   *[other] { $points } Alterungspunkte in { $characteristic }.
}
aging-outcome-decrepitude_and_crisis = { $points } Alterungspunkte — genug für die nächste Stufe Gebrechlichkeit — und eine Krise.
aging-outcome-decrepitude_unpriceable = Genug Alterungspunkte für die nächste Stufe Gebrechlichkeit, und eine Krise. Die Steigerungstabelle reicht nicht bis zu dieser Stufe; legt die Zahl am Tisch fest.
# Die Krise (Core Rules.md:16619-16638). Zwei Würfe, beide vom Spieler: der
# Stresswürfel oben hat das Jahr hierher geschickt, und auf der Krisentabelle wird
# ein Einfachwürfel geworfen. Die App würfelt keinen von beiden, macht nie den
# Überlebenswurf und entscheidet nie, ob der Charakter überlebt.
crisis-label = Krise
crisis-die-label = Einfachwürfel
crisis-die-unrolled = Solange der Einfachwürfel fehlt, trägt dieses Jahr die Krise als offen und ungewürfelt ein.
crisis-total-readout = Krisen-Gesamtwert: { $total }
# Dieselbe Größe als reines Spaltenlabel für das exportierte Charakterblatt.
crisis-total-label = Krisen-Gesamtwert
crisis-total-parts = { $die } (Einfachwürfel) { $age } (Alter) { $decrepitude } (Gebrechlichkeitswert)
crisis-row-readout = Krisentabelle: { $row }
crisis-row-readout-severity = Krisentabelle: { $row } ({ $severity })
# Die fünf Schweregrade aus Core Rules.md:16628-16632, als eigenständige Label.
crisis-severity-minor = Leicht
crisis-severity-serious = Ernst
crisis-severity-major = Schwer
crisis-severity-critical = Kritisch
crisis-severity-terminal = Tödlich
crisis-bedridden = Bettlägerig ist Zeit und kein Wurf: es gibt keinen Überlebenswurf und keine Zauberstufe zu erreichen.
crisis-survival-label = Die Krise überstehen
crisis-survival-ease-factor = Ausdauer-Stresswurf gegen Schwierigkeitsgrad { $ease }.
crisis-survival-no-roll = Für diese Krise gibt es keinen Überlebenswurf.
crisis-survival-ritual = Ein Momentan-Ritual aus Creo Corpus der Stufe { $level } behebt sie stattdessen.
crisis-modifier-row = { $source } { $amount }
crisis-modifier-bronze_cord = Bronzene Kordel
crisis-modifier-total = Modifikatoren auf den Überlebenswurf: { $total }
crisis-allowance-attendant = Ein behandelnder Arzt darf { $characteristic } + { $ability } gegen Schwierigkeitsgrad { $ease } würfeln; bei Erfolg wird sein { $ability }-Wert zum Überlebenswurf addiert, bei einem Patzer gilt { $botch }. Nur ein Arzt kann sinnvoll beistehen.
# Was ein angewendetes Jahr dem Spieler sagen muss, im Unterschied zu dem, was es
# eingetragen hat.
aging-note-longevity_ritual_spent = Die Krise verbraucht das Langlebigkeitsritual: es stellt sicher, dass der Charakter überlebt, aber seine Kraft ist erschöpft und das Ritual muss erneut durchgeführt werden. Der Eintrag bleibt stehen — trage das neue Ritual selbst ein.
aging-note-heavy_wound = Dieser Charakter erleidet durch die Krise zusätzlich zu jedem anderen Ergebnis eine Schwere Wunde. Die App trägt keine Wunde ein — vermerke sie selbst auf der Gesundheitsleiste.
aging-distribute = { $points ->
    [one] Verteile 1 Alterungspunkt auf eine Eigenschaft deiner Wahl.
   *[other] Verteile { $points } Alterungspunkte auf beliebige Eigenschaften deiner Wahl.
}
aging-distribute-remaining = { $placed } von { $owed } verteilt
aging-apply = Dieses Jahr anwenden
aging-revert = Alter { $age } zurücknehmen
aging-calculator-clear = Diesen Wurf verwerfen
personality-label = Persönlichkeitseigenschaften
personality-name-placeholder = Eigenschaft
# Barrierefreier Name für das gebundene Werteingabefeld (S29, full-audit UX):
# die Zeile trägt kein eigenes sichtbares Label, daher muss das Feld nennen,
# welche Eigenschaft es bewertet — analog zu characteristic-description-for.
personality-value-label = Wert von { $name }
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
aura-out-of-range = Außerhalb des Regelbereichs ({ $min } bis { $max }); wird beim Speichern auf den nächstgültigen Wert begrenzt.
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
# Bestätigung vor dem Entfernen des Vertrauten (S18): Der gesamte Werteblock —
# Name, Magische Macht, Eigenschaften, Persönlichkeitseigenschaften, Kordeln,
# investierte Kräfte — wird in einem Schritt gelöscht, ohne Rückgängig-Funktion.
familiar-remove-confirm-title = Vertrauten entfernen?
familiar-remove-confirm-message = Dadurch wird der gesamte Werteblock des Vertrauten gelöscht — Name, Magische Macht, Eigenschaften, Persönlichkeitseigenschaften, Kordeln und investierte Kräfte. Dies kann nicht rückgängig gemacht werden.
familiar-remove-confirm-confirm = Vertrauten entfernen
familiar-remove-confirm-cancel = Abbrechen
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
familiar-bond-note = Die Bindung verleiht beiden Partnern die Kleine Tugend Wahrer Freund und die Persönlichkeitseigenschaft Loyal (Partner) +3. Ein Vertrauter ohne menschliche Intelligenz erhält sie mit Intelligenz -3. Dies wird nicht automatisch angewendet.
# Talisman: das persönliche Zaubergerät des Magus. Seine Kapazität stammt als
# Richtwert aus der Engine (höchste Technik + höchste Form, in Bauern Vim-Vis) und
# wird hier nie neu berechnet; die eingebetteten Effekte belasten kein Budget.
talisman-label = Talisman
talisman-add = Talisman hinzufügen
talisman-remove-item = Talisman entfernen
# Bestätigung vor dem Entfernen des Talismans (S30): Identität, Abstimmungen
# und eingebettete Effekte werden in einem Schritt gelöscht, ohne
# Rückgängig-Funktion.
talisman-remove-confirm-title = Talisman entfernen?
talisman-remove-confirm-message = Dadurch werden Form und Material, die Abstimmungen und die eingebetteten Effekte des Talismans gelöscht. Dies kann nicht rückgängig gemacht werden.
talisman-remove-confirm-confirm = Talisman entfernen
talisman-remove-confirm-cancel = Abbrechen
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
# Grund, warum eine übernatürliche Fähigkeit im Auswähler ausgegraut ist.
ability-requires-virtue = Erfordert eine verleihende Tugend (oder die eine freie Fähigkeit der Gabe)
# Nur für Screenreader: Text auf einer gewählten Fähigkeitszeile, auf die ein
# Prüfhinweis mit Schweregrad „Fehler" zeigt (S7, full-audit a11y) — ergänzt ein
# sichtbares Symbol, damit die Ungültigkeit nie allein über die Farbe vermittelt
# wird (WCAG 1.4.1).
ability-invalid-selection = Ungültige Auswahl
# Kennzeichnet eine reine Anzeigezeile einer Fertigkeit (#17): eine Tugend gewährt
# dieser Fertigkeit einen Bonus oder einen freien Startwert, es wurde aber kein Wert
# gekauft — die Zeile zeigt daher nur den Bonus und bietet keine Bedienelemente. Der
# Grund steht in Worten da; der ausgegraute Regler allein ist keine Begründung
# (WCAG 1.4.1). Zum Kaufen die Fertigkeit aus der Liste „Verfügbar" hinzufügen.
ability-unbought-marker = Nicht gekauft
# Je Zeile benannt ($name ist der Anzeigename der Eigenschaft bzw. des
# Persönlichkeitsmerkmals — dieser Schlüssel wird von CharacteristicPicker, dem
# Persönlichkeitsmerkmal-Regler in FamiliarPanel und von PersonalityTraits
# gemeinsam genutzt) — dieselbe Begründung wie bei ability-increment/-decrement
# oben.
characteristic-increment = { $name } erhöhen
characteristic-decrement = { $name } verringern
characteristic-description-label = Beschreibung
# Barrierefreier Name für ein Beschreibungsfeld: die acht Felder je Eigenschaft
# teilen sich den Platzhalter oben, daher braucht jedes einen eigenen Namen, der
# nennt, welche Eigenschaft es beschreibt.
characteristic-description-for = Beschreibung von { $name }

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
# Übernimmt den bereits angezeigten Charakter in die geführte Erstellung und setzt
# beim weitesten Schritt fort, den seine Datei festgehalten hat. Wird nur dort
# angeboten, wo diese Erstellung durchlaufen werden kann.
action-continue-in-wizard = In geführter Erstellung fortsetzen

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
# Eine Forderung, die die Engine weiter prüft, als die Regeln sie formulieren. Das
# Beispiel der Regeln steht vorn, damit der Wert unmittelbar folgt („Latein 1“, wie
# das Regelwerk es schreibt), und DIESER Zusatz folgt dem Wert und nennt, was die
# Engine wirklich prüft: „unter Latein 1 (Tote Sprache genügt)“. Er bringt ein
# eigenes Leerzeichen mit, weil die tragenden Meldungen ihn ohne Trenner einsetzen —
# bei einer Forderung ohne Beispiel ist er leer. Ohne Artikel und ohne Adjektiv
# formuliert, damit das eingesetzte Fertigkeitswort in jedem Genus passt.
requirement-exemplar = { " " }({ $ability } genügt)
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
# Für die Domäne `item`: noch kein ausgelieferter Katalogeintrag deklariert eine, aber
# die Domäne gehört zum geschlossenen Enum der Engine, und ihr Auswahlfeld benennt sich
# hier, statt auf den rohen Schlüssel zurückzufallen. Begriff aus
# rules/source/de/translation-tables/sphären-mächte.md:199 („Awakened Item“ →
# „Erwachter Gegenstand“).
param-label-item = Gegenstand
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
# Wird von SourcePicker angezeigt, wenn Suche/Filter alle Zeilen des Katalogs
# ausschließen (S25, full-audit UX) — sonst bleibt die Liste völlig leer, nicht
# von einem langsam ladenden oder defekten Feld zu unterscheiden.
filter-no-results = Keine Treffer für den aktuellen Filter.

# Nur für Screenreader: Schweregrad-Präfix vor jedem Prüfhinweis
# (ValidationPanel.svelte) — Farbe/Rahmen allein dürfen nicht das einzige
# Signal sein (WCAG 1.4.1).
issue-severity-error = Fehler
issue-severity-warning = Warnung

# Angehängt an einen Hinweis, den ein Schritt des Assistenten anzeigt, weil das
# genannte Element HIER gewählt wurde, während die Engine ihn dem Schritt zuordnet,
# dem der beanstandete Wert gehört (eine Tugend wie „Große Eigenschaft", deren Wert
# eine Eigenschaft ist). Ein solcher Hinweis liest sich als Fehler, deaktiviert
# „Weiter" aber nicht — er muss also sagen, wo die Behebung liegt. `$step` ist
# immer eine `phase-<slug>`-Bezeichnung, nie der Slug selbst.
issue-other-step = Im Schritt „{ $step }" zu beheben.

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
issue-xp_solve_bound_exceeded = Dieser Charakter hat zu viele Fertigkeits- und Künste-Werte sowie gemeisterte Zaubersprüche ({ $spends } gekaufte Werte über { $pools } Erfahrungspunkte-Vorräte, { $nodes } insgesamt), um die Erfahrungspunkte zuzuteilen — die Grenze liegt bei { $limit }. Das deutet meist auf eine beschädigte Spielstand-Datei hin.
issue-restricted_xp_unspent = { $origin }: { $unspent } von { $amount } eingeschränkten Erfahrungspunkten sind nicht ausgegeben und verfallen.
# guided-creation-review-2026-08 #30. Bewusst nur eine Zählung: das eingeschränkte
# Gegenstück oben darf vom Verfallen sprechen, weil die Blöcke der Kindheit entweder
# ausgegeben oder verloren sind — für den allgemeinen Vorrat und die 120 Stufen Zauber
# sagen die Basisregeln nichts Vergleichbares. Diese beiden nennen also den Rest und
# behaupten nichts darüber hinaus.
issue-general_xp_unspent = { $unspent } von { $pool } Erfahrungspunkten sind noch nicht ausgegeben.
issue-spell_levels_unspent = { $unspent } von { $budget } Stufen Zauber sind noch nicht ausgegeben.
issue-ability_category_requires_virtue = { $ability } ist { $category } und benötigt eine Tugend, die den Zugang bei der Charaktererschaffung gewährt.
issue-academic_ability_without_scholarly_language = Eine akademische Fertigkeit erfordert normalerweise { $ability }{ $qualifier } auf { $min } oder höher.
issue-life_stage_age_unset = Trage das Alter des Charakters ein: das spätere Leben erbringt Erfahrungspunkte pro Jahr, ohne Alter zählen daher nur die Blöcke der Kindheit.
issue-life_stage_age_before_childhood = Alter { $age } liegt innerhalb der Kindheit, die { $min } Jahre dauert — es gibt keine späteren Lebensjahre, in denen Erfahrung erworben wird.
issue-life_stage_age_before_gauntlet = Kein Magus legt die Lehrlingsprüfung mit { $age } Jahren ab: Sie kommt frühestens mit { $min } — Kindheit plus fünfzehn Jahre Lehrlingszeit.
issue-life_stage_gauntlet_age_after_age = Die Lehrlingsprüfung mit { $gauntlet_age } liegt für diesen Magus mit { $age } Jahren noch in der Zukunft; die Jahre als Magus werden ab der Lehrlingsprüfung gezählt.
issue-life_stage_lab_seasons_out_of_range = { $seasons } Quartale Laborarbeit sind mehr als die { $max }, die { $years } Jahre als Magus tragen können.
issue-life_stage_lab_seasons_without_years = { $seasons } Quartale Laborarbeit sind eingetragen, aber dieser Charakter hat keine Jahre als Magus, in denen sie stattfinden könnten.
issue-life_stage_spell_level_split_exceeds_points = { $levels } Zauberstufen aus den Jahren als Magus zu nehmen übersteigt, was diese Jahre gewähren: Sie sind { $points } Punkte wert, die zwischen Erfahrung und Zauberstufen aufzuteilen sind.
issue-life_stage_native_language_unset = Wähle eine Muttersprache: der größte Erfahrungsblock der Kindheit kann für nichts anderes ausgegeben werden.
issue-life_stage_native_language_missing_score = Kein Wert in { $language } gekauft, daher bleibt die Erfahrung der Kindheit für die Muttersprache unausgegeben.
issue-magus_minimum_ability = Kein Magus wird unter { $ability } { $min }{ $qualifier } in den Orden aufgenommen; dieser Charakter hat { $score }.
issue-magus_recommended_ability = { $ability } { $min }{ $qualifier } wird für einen Magus empfohlen, der gerade aus der Lehrlingszeit kommt; dieser Charakter hat { $score }.
issue-childhood_package_unknown = Unbekanntes Fertigkeitspaket der Kindheit: { $package }.
issue-childhood_slot_unfilled = Trage { $key } für { $ability } ein, bevor das Fertigkeitspaket der Kindheit angewendet wird.
issue-childhood_slot_is_native_language = { $key } für { $ability } muss sich von der Muttersprache { $language } unterscheiden.
issue-childhood_slot_duplicate_value = { $key } { $value } für { $ability } ist bereits von einem anderen Eintrag des Fertigkeitspakets belegt; wähle einen anderen Wert.
issue-ability_parameter_required = { $ability } braucht einen Wert (z. B. das konkrete Gebiet oder die Sprache).
issue-ability_score_out_of_range = Fertigkeit { $ability } mit Wert { $score } liegt außerhalb des erlaubten Bereichs (0 bis { $max }).
issue-ability_bonus_dangling_target = Füge { $ability } { $parameter } zu den Fertigkeiten des Charakters hinzu; { $item } zielt darauf.
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
# Ein Charakter über 35 muss vor Spielbeginn Alterungswürfe ablegen
# (Basisregeln.md:2232); die Alterung beginnt im Winter nach dem 35. Geburtstag
# (`:16565`), daher greift dies ab 36. Die Würfe fallen am Spieltisch, die App
# kann sie nur anmahnen; ein eingetragenes Alterungsprotokoll erledigt den
# Hinweis, gleich was die Würfe ergeben haben.
issue-aging_rolls_pending = Dieser Charakter ist { $age } Jahre alt, und ein Charakter über 35 muss vor Spielbeginn Alterungswürfe ablegen; bisher ist keiner eingetragen.
issue-unknown_living_condition = Der Lebensumstand '{ $condition }' entspricht keiner Zeile der Lebensumstände-Tabelle und geht daher nicht in den Alterungswurf ein.
# Nur die mit Sternchen markierten Zeilen der Tabelle sind untereinander kumulativ
# (`:16594`); die übrigen beschreiben je eine Situation, es gilt also nur eine.
issue-living_conditions_conflict = Die Lebensumstände '{ $condition }' und '{ $other }' schließen einander aus, es kann also nur einer davon gelten.
# Ein Alterungswurf erhöht das scheinbare Alter um höchstens ein Jahr pro Jahr
# (`:16577`), kann es also von sich aus nicht über das tatsächliche Alter treiben —
# `:5189` sagt allerdings nur, es "sollte" höchstens so hoch sein, und lässt
# Ausnahmen für Charaktere zu, die nicht im Grunde menschlich sind.
issue-apparent_age_above_age = Das scheinbare Alter ({ $apparent_age }) liegt über dem tatsächlichen Alter ({ $age }); die Alterung erhöht es um höchstens ein Jahr pro Jahr.
# Weder ein Zustand des Charakters noch eine abgelehnte Eingabe: die Verknüpfung von
# Alter und Geburtsjahr hat ein unmögliches Paar ergeben, das Alter wurde daher auf 0
# begrenzt statt unter null zu laufen. Nur ein Hinweis — das Paar ist unmöglich, aber
# nichts daran ist regelwidrig.
issue-saga_year_before_birth_year = Das Jahr der Saga ({ $saga_year }) liegt vor dem Geburtsjahr ({ $birth_year }), der Charakter ist also noch nicht geboren; das Alter steht auf 0, bis einer der beiden Werte geändert wird.
# Die sechs Gründe, aus denen ein Alterungswurf abgelehnt wird. Anders als alle
# Befunde darüber gehören diese zur Eingabe eines Befehls: die Engine schreibt bei
# einer Ablehnung nichts, kein gespeicherter Charakter kann sie also tragen — jeder
# beschreibt den gerade abgeschickten Wurf.
issue-aging_rules_missing = Dieses Regelwerk enthält keine Alterungstabelle, daher lässt sich kein Alterungswurf auswerten.
issue-aging_year_already_recorded = Der Alterungswurf für das Alter { $age } ist bereits eingetragen; er muss zurückgenommen werden, bevor dieses Jahr erneut gewürfelt wird.
issue-aging_distribution_mismatch = Dieser Wurf lässt { $owed } Alterungspunkt(e) zu verteilen, verteilt wurden aber { $distributed }.
issue-aging_distribution_not_open = Die Eigenschaften dieses Wurfs gibt die Tabelle selbst vor, daher lassen sich die { $count } verteilten Punkte nicht anwenden.
issue-aging_award_unpriceable = Die nächste Stufe der Gebrechlichkeit liegt jenseits der Steigerungstabelle, daher lassen sich ihre Kosten nicht bestimmen.
issue-aging_year_not_recorded = Für das Alter { $age } ist kein Alterungswurf eingetragen, es gibt also nichts zurückzunehmen.
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
# Barrierefreier Name für die Typ-Filterauswahl ohne eigene sichtbare Beschriftung.
equipment-group-filter-label = Nach Ausrüstungstyp filtern
equipment-equipped-label = Ausgerüstet
equipment-specialization-label = Spezialisierung greift (+1)
equipment-empty = Keine Ausrüstung.

# AppError-Arten der Tauri-Befehle.
error-io = Eine Datei konnte nicht gelesen oder geschrieben werden.
error-ruleset = Das Regelwerk konnte nicht geladen werden.
error-not_loaded = Es ist noch kein Regelwerk geladen.
error-serialize = Die Charakterdatei konnte nicht verarbeitet werden.
error-export = Das Charakterblatt konnte nicht exportiert werden: In der aktuellen Sprache fehlt Text für { $missing }. Versuchen Sie es mit Englisch als Sprache erneut, oder melden Sie dies als Fehler.

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
derived-aura-out-of-range = Außerhalb des Regelbereichs ({ $min } bis { $max }); wird beim Speichern auf den nächstgültigen Wert begrenzt.
derived-lab-enchanting = Für Verzauberung
derived-lab-enchanting-hint = Von der obigen Laborsumme wegen Schwacher Verzauberer halbiert — für das Erschaffen oder Untersuchen verzauberter Gegenstände verwenden.
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
# Verbindet die Teile der Summanden-Aufschlüsselung im Tooltip (S14, full-audit
# i18n) — gleiche Form wie restricted-xp-list-separator, über Fluent statt eines
# fest codierten „, "-Literals, damit es für andere Sprachkonventionen anpassbar ist.
derived-addend-list-separator = ,
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
derived-detail-no_apparent_aging = Scheinbares Alter steigt nicht
derived-detail-decrepitude = Gebrechlichkeit
derived-detail-living_conditions = Lebensumstände
derived-detail-crisis_survival = Krisen-Überlebenswurf
derived-detail-crisis_heavy_wound = Schwere Wunde bei jeder Krise
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
