# Ars Magica Definitive Edition – Deutsche Übersetzungstabellen

Thematisch gegliederte Übersetzungstabellen für die deutsche Ausgabe von Ars Magica Definitive Edition.

**Quelldateien:**
- `ArM_DE_Uebersetzungstabelle.md` – Haupttabelle (22 Sektionen)
- `ArM_DE_Zusatztermini.md` – 30 ergänzende Begriffe mit Korrekturnoten
- `Zauberübersetzungstabelle.md` – 362 Zaubernamen

**Legende (gillt für alle Tabellen):**
- (Lat.) = Lateinischer Begriff, wird unübersetzt beibehalten
- m. = Maskulinum · f. = Femininum · n. = Neutrum · Sg. = Singular · Pl. = Plural

---

## ⚠️ Diese Tabellen sind KI-erzeugt und können Fehler enthalten

Die Tabellen wurden mit Claude aus den Quelldateien erzeugt. Das V/F-Audit
(2026-09-19 bis -21, alle 655 Tugenden und Fehler) hat **zehn belegte Fehler**
gefunden, darunter drei Zeilen, die in zwei Magnitudentabellen gleichzeitig
standen, und fünf Namen, die der Überschrift im deutschen Regelbuch
widersprachen. Sie sind korrigiert — aber die Trefferquote sagt etwas über die
übrigen Tabellen aus, die noch niemand Zeile für Zeile geprüft hat.

**Diese Kopie wird aus `arm-de-translation` synchronisiert.** Das Quellprojekt
ist führend; diese Verzeichnis ist eine Kopie und **driftet**. Eine Zeile hier,
die dem Regelbuch widerspricht, kann deshalb zweierlei sein: ein echter Fehler
(dann in **beiden** Projekten korrigieren) oder eine veraltete Kopie (dann von
dort neu synchronisieren). **Erst im Quellprojekt nachsehen, dann urteilen.**

### Wofür die Tabellen maßgeblich sind — und wofür nicht

Sie sind maßgeblich für **Terminologie**: der deutsche Begriff für einen
englischen Term ist der Wert in der Spalte `Deutsch (DE)`.

Sie sind **nicht** maßgeblich für **Sachaussagen** über die Regeln — welcher
Tugend eine Reputation zusteht, welche Stufe sie hat, welcher Magnitude sie
angehört. Dafür gilt ausschließlich das Regelbuch. **Sieben der zehn gefundenen
Fehler waren genau das:** eine Terminologietabelle, die eine Behauptung über die
Regeln aufstellte, die sie gar nicht hätte treffen müssen.

### Rangfolge bei Widerspruch

Maßgeblich ist `docs/vf-audit/decisions.md`, **D6** und **D7**. Kurzfassung:

1. **Sachaussage über die Regeln** → das **Regelbuch** gewinnt, immer.
2. **Name eines Eintrags** → die **Überschrift des zitierten Regelbuchs**
   gewinnt. Das Glossar gewinnt nur dort, wo die Überschrift **defekt** ist —
   etwa wenn sie mit der eines anderen Eintrags kollidiert (`Gefesselte Magie`
   überschreibt im deutschen Grundregelwerk zwei verschiedene Einträge) oder wenn
   das Regelbuch den Begriff gar nicht wiedergibt.
3. **Sonstige Terminologie** → thematische Tabelle vor `tugenden-fehler.md`
   (die breiteste Tabelle, und damit die mit der höchsten Fehlerwahrscheinlichkeit).
4. **Eine Zeile mit Buchkürzel eines *anderen* Buches** (`HdH:WL`, `SdM:G`)
   steht nicht im Widerspruch — sie beschreibt die Terminologie *jenes* Buches.

### Gefundene und korrigierte Fehler (V/F-Audit 2026-09-19 bis -21)

| Datei | Fehler | Korrektur | Beleg |
|---|---|---|---|
| `reputationen.md` | Reputation Stufe 3 „Unter Angehörigen der Blutlinie" war **Mythic Blood** zugeordnet | gehört zu **Magical Blood (Magic Human)** | ArMDE:4588 „does not grant any Reputation"; ArMDE:4367 |
| `tugenden-fehler.md` | `Sense Passions`, `Summon Animals` standen in **beiden** Magnitudentabellen | beide sind **Groß**; Klein-Zeilen entfernt | ArMDE:4931, :5086 |
| `tugenden-fehler.md` | `Monstrous Blood` stand in **beiden** Magnitudentabellen | ist **Klein**; Groß-Zeile entfernt | ArMDE:6455 |
| `tugenden-fehler.md` | `Spirit Votary` als *Sozialer Status, Klein* | **Frei, Mythischer Gefährte** | ArMDE:5007 |
| `tugenden-fehler.md` | `Deteriorating Power` → *Schwindende Kraft* | **Schwindende Macht** | ArMDE:5944 |
| `tugenden-fehler.md` | `Disorientating Magic` → *Desorientierungsmagie* | **Desorientierende Magie** | ArMDE:5984 |
| `tugenden-fehler.md` | `Enfeebled` → *Entkräftet* | **Geschwächt** | ArMDE:6008 |
| `tugenden-fehler.md` | `Environmental Magic Condition` → *Umgebungs-*, doppelt geführt | **Umweltbedingung**, Doppelzeile entfernt | ArMDE:6020, Index :5292 |
| `tugenden-fehler.md` | `Environmental Sensitivity` → *Umgebungsempfindlichkeit* | **Umweltempfindlichkeit** | ArMDE:6024 |
| `grundbegriffe.md` | `Vulnerable Magic` → *Verwundbare Magie*; `Vulnerable to Folk Tradition` → *Volkszauber* | **Anfällige Magie**; **Anfällig für Volksüberlieferungen** | ArMDE:7005, :7011 |

**Muster, keine Einzelfälle.** Zwei Fehlerarten traten mehrfach auf und lohnen
eine gezielte Durchsicht der noch ungeprüften Tabellen: derselbe englische
Schlüssel in **zwei Magnitudentabellen** (dreimal), und ein systematisch
falsches Präfix (*Umgebungs-* statt *Umwelt-*) über mehrere Nachbarzeilen.

**Kein Tabellenfehler, aber der Befund, aus dem alles andere folgt:**
`Covenfolk` und `Magical Covenfolk` standen hier als *Konventsmitglied* /
*Magisches Konventsmitglied*, während `arm-de-translation` längst
**Konventsbewohner** führte — mitsamt Begründung („Konventsmitglied" wäre
missverständlich, weil es die Magi meint). Das Quellprojekt war **richtig**,
diese Kopie war **veraltet**. Genau daran wurde sichtbar, dass die Tabellen
kopiert und nicht referenziert werden und die Kopie driftet — der Grund für den
Hinweis am Anfang dieses Abschnitts.

---

## Übersicht der Tabellendateien

| Datei | Inhalt | Quellsektionen |
|---|---|---|
| [grundbegriffe.md](grundbegriffe.md) | Allgemeine Spielbegriffe, Eigenschaften, Wunden, Erschöpfung | Sek. 1, 4–6 + Zusatztermini |
| [masseinheiten.md](masseinheiten.md) | **Maßeinheiten:** Umrechnungsfaktoren (Längen, Flächen, Volumina, Massen), Sonderfälle, reale geografische Werte | buchübergreifende Konvention |
| [magie-regeln.md](magie-regeln.md) | Techniken, Formen, Kampf, Reichweite/Dauer/Ziel, Magnitudes | Sek. 2–3, 7–9 |
| [labor-fortschritt.md](labor-fortschritt.md) | Laborterminologie, Fortschritt & Erfahrung, Langzeitereignisse | Sek. 10–12 |
| [tugenden-fehler.md](tugenden-fehler.md) | Alle Tugenden und Fehler (hermetisch, übernatürlich, allgemein, Sozialer Status) | Sek. 13–14 + Zusatztermini |
| [fertigkeiten.md](fertigkeiten.md) | Alle Fertigkeiten mit Typ (Allgemein, Akademisch, Arkan, Kampf, Übernatürlich) | Sek. 15 |
| [orden-tribunale.md](orden-tribunale.md) | Häuser, Tribunale, lateinische Personenbezeichnungen, Fachbegriffe (Lat./Gr.) | Sek. 16–17, 20–21 + Zusatztermini |
| [konvent.md](konvent.md) | Vis-Einheiten, Konventsjahreszeiten, Konventsbegriffe | Sek. 18–19, 22 + Zusatztermini |
| [zauber-nach-form.md](zauber-nach-form.md) | 362 Zaubernamen alphabetisch nach deutschem Namen, gegliedert nach Form | Zauberübersetzungstabelle.md + Zusatztermini |
| [uebersetzungsregeln.md](uebersetzungsregeln.md) | Sprachliche und formale Übersetzungsregeln (kein Glossar) | Sek. 23–24 |
| [persoenlichkeitseigenschaften.md](persoenlichkeitseigenschaften.md) | Persönlichkeitseigenschaften: Mechanik, Schwierigkeitstabelle für Würfe, vollständige A-Z-Liste EN↔DE | Core Rules + ArsMagica_DE_Gesamt_work.md |
| [reputationen.md](reputationen.md) | Reputationen: Mechanik, Typen, Schwierigkeitstabelle, Tugenden/Fehler mit Reputationsstufen, Beispiel-Inhalte | Core Rules + ArsMagica_DE_Gesamt_work.md |
| [alterung-twilight.md](alterung-twilight.md) | Altern, Gebrechlichkeit, Verwicklung und Zauberers Dämmerung: Formeln, Würfeltabellen, Twilight-Effekte | Core Rules + ArsMagica_DE_Gesamt_work.md |
| [sphären-mächte.md](sphären-mächte.md) | Die vier Sphären, Auren, Sphärenwechselwirkung, Machtwert/Machtvorrat, Kreaturenkräfte, Penetration | Core Rules + ArsMagica_DE_Gesamt_work.md |
| [kampf.md](kampf.md) | Kampfbegriffe, Formeln, Wundstufen, Erschöpfung, Manöver, Gruppenregeln, Rüstungen, Waffentabellen | Core Rules + ArsMagica_DE_Gesamt_work.md |
| [konvent-boons-hooks.md](konvent-boons-hooks.md) | Konventsvorzüge und -haken: Mechanik, vollständige Liste aller Kleinen/Großen Vorzüge und Haken nach Kategorie | Core Rules + ArsMagica_DE_Gesamt_work.md |
| [tiere-kreaturen.md](tiere-kreaturen.md) | Tiere und Kreaturen: Statblock-Begriffe, Größentabelle, vollständige Eigenschaftsliste (Tiertugenden), natürliche Waffen, Grundfertigkeiten | Core Rules + ArsMagica_DE_Gesamt_work.md |
| [magische-qualitaeten.md](magische-qualitaeten.md) | Magische Qualitäten und Mängel: alle Großen/Kleinen Qualitäten und Mängel | RoP:M Kap. 4 |
| [kreaturenkraefte.md](kreaturenkraefte.md) | Häufige Kreaturenkräfte (sphärenunabhängig) | RoP:M |
| [goettliche-kraefte.md](goettliche-kraefte.md) | Göttliche Kräfte: Engelskräfte nach Chor, allgemeine Engel-Mechaniken, Reliquienkräfte | RoP:D |
| [infernale-kraefte.md](infernale-kraefte.md) | Infernale Kräfte und Wesen: Dämonenkräfte nach Ordnung/Familie, Dämonentypen | RoP:I |
| [islamische-begriffe.md](islamische-begriffe.md) | Islamische Fachbegriffe: religiöse Grundbegriffe, Fünf Säulen, Recht, Mystik, Konfessionen | RoP:D Kap. 5 |
| [juedische-begriffe.md](juedische-begriffe.md) | Jüdische Fachbegriffe: Feiertage, religiöse/gesellschaftliche Begriffe, Personen, Orte/Texte | RoP:D Kap. 6 |

---

## Hinweise zu wichtigen Übersetzungsentscheidungen

### Typ-Label „Tainted"
EN „Tainted" als Typ-Label → DE **„Befleckt"** (~~nicht „Verdorben" oder „Korrumpiert"~~).
Der Flaw-Name „Depraved" → DE „Verdorben" bleibt unverändert.

### Maßeinheiten
Vollständige Konvention: [masseinheiten.md](masseinheiten.md).
- **pace** → **Schritt** (RPG-Einheit, ≈ 1 m; wird NICHT in Meter umgerechnet, Original-Zahl bleibt stehen)
- **league** (Entfernungsmaß) → **Wegstunde** (ca. 5 km) wenn km-Angabe vorhanden; sonst Kontext-Entscheidung
- **league** im Eigennamen „Seven-League Stride" → **Sieben-Meilen-Schritt** (bewusste Abweichung, Referenz auf Volksmärchen)
- **Geografische Angaben** (Berge, Flüsse, Inseln, Entfernungen realer Orte) nehmen den **realen Wert**, nicht die Umrechnung

### Gelöste Übersetzungskonflikte (Zaubernamen)
| Englisch | Kanonisch (DE) | Verworfene Variante |
|---|---|---|
| Demon's Eternal Oblivion | Ewige Auslöschung des Dämons | ~~Ewiges Vergessen des Dämons~~ |
| Unravelling the Fabric of (Form) | Das (Form)-Gefüge auflösen | ~~Das Gewebe der (Form) zerreißen~~ |
| Wind of Mundane Silence | Wind der weltlichen Stille | ~~Wind der gewöhnlichen Stille~~ |

---

*Grundlage: Ars Magica Definitive Edition Core Rules (Atlas Games, 2024; GitHub: OriginalMadman/Ars-Magica-Open-License) · Ars Magica Open License: https://atlas-games.com/arsmagica/openars*
