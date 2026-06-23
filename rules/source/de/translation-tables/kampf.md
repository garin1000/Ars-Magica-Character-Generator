# Ars Magica Definitive Edition – Kampf / Combat

**Teil der deutschen Übersetzungstabellen** | Quellen: Ars Magica – Definitive Edition (Core Rules) · ArsMagica_DE_Gesamt_work.md

**Legende:**
- (Lat.) = Lateinischer Begriff, wird unübersetzt beibehalten
- → = Querverweis auf andere Tabellendatei

---

## Kernbegriffe / Core Terms

| Englisch (EN) | Deutsch (DE) | Anmerkung |
|---|---|---|
| Combat | Kampf | |
| Combat round | Kampfrunde | ca. 6 Sekunden |
| Combat sequence | Kampfablauf | |
| Combat option | Kampfoption | |
| Initiative | Initiative | Einmalig zu Beginn bestimmt; gilt jede Runde |
| Initiative Total | Initiativewert | |
| Attack | Angriff | |
| Attack Total | Angriffssumme | |
| Defense | Verteidigung | |
| Defense Total | Verteidigungssumme | |
| Damage | Schaden | |
| Damage Total | Schadenssumme | |
| Soak | Absorption | Rüstungsschutz + Ausdauer |
| Soak Total | Absorptionssumme | |
| Attack Advantage | Angriffsvorteil | Angriffssumme − Verteidigungssumme (wenn > 0) |
| Load | Last | Belastungs­beitrag von Waffe oder Rüstung |
| Encumbrance | Belastung | Gesamte Beladung − Stärke; Abzug auf athletische Handlungen |
| Burden | Beladung | Zwischenwert aus Gesamtlast; bestimmt Belastung |
| Range (missile) | Reichweite | Entfernung für Fernkampf; −3 pro Schritt über erste Reichweite |
| Pace | Schritt | Bewegungseinheit; 1 Schritt = 1,5 m (DE-Abweichung vom EN-Original) |

---

## Kampfformeln / Combat Formulas

| Größe | Formel |
|---|---|
| Initiative | Initiative + Schnelligkeit + Stresswürfel |
| Angriffssumme | Kampffertigkeit + Angriffsmodifikator + Stresswürfel |
| Verteidigungssumme | Kampffertigkeit + Verteidigungsmodifikator + Schnelligkeit + Stresswürfel |
| Schadenssumme | Stärke + Schadensmodifikator + Angriffsvorteil |
| Absorption | Ausdauer + Rüstungsschutz |
| Erlittener Schaden | Schadenssumme − Absorptionssumme (bestimmt Wundstufe) |
| Bewegung (Gehen) | 10 + Schnelligkeit − Behinderung Schritte/Runde |
| Bewegung (Eilen) | 2 × (10 + Schnelligkeit − Behinderung) Schritte/Runde |
| Bewegung (Rennen) | 4 × (10 + Schnelligkeit − Behinderung) Schritte/Runde |

---

## Wundstufen / Wound Levels

*→ Vollständige Wundtabelle nach Größen­kategorie: [grundbegriffe.md](grundbegriffe.md)*

| Englisch (EN) | Deutsch (DE) | Abzug |
|---|---|---|
| Light Wound | Leichte Wunde | −1 |
| Medium Wound | Mittelschwere Wunde | −3 |
| Heavy Wound | Schwere Wunde | −5 |
| Incapacitating Wound | Lähmende Wunde | Keine Handlungen möglich |
| Dead | Tot | – |

In manchen Stat-Blöcken auch:

| Incapacitated | Kampfunfähig | Keine Handlungen möglich |

---

## Erschöpfungsstufen im Kampf / Fatigue Levels in Combat

*→ Vollständige Erschöpfungsregeln: [grundbegriffe.md](grundbegriffe.md)*

| Englisch (EN) | Deutsch (DE) | Abzug |
|---|---|---|
| Fresh | Ausgeruht | – |
| Winded | Außer Atem | – |
| Weary | Erschöpft | −1 |
| Tired | Müde | −3 |
| Dazed | Betäubt | −5 |
| Unconscious | Bewusstlos | – |

---

## Kampfoptionen / Combat Options

| Englisch (EN) | Deutsch (DE) | Mechanik |
|---|---|---|
| Exertion | Anstrengung | Erschöpfungsstufe ausgeben → Bonus = Kampffertigkeit für eine Runde (auf Angriff oder alle Verteidigungen) |
| Withdrawal / Disengaging | Rückzug | Verteidigungssumme statt Angriff; +3 kumulativ pro Folgeversuch; automatisch wenn nicht angegriffen |
| Mounted combat | Berittener Kampf | + Reitenwert (max. +3) auf Angriff und Verteidigung |
| Missile combat | Fernkampf | −3 Angriffssumme pro Reichweiteninkrement über das erste; Schildverteidigungsbonus gilt |

---

## Nahkampfmanöver / Melee Maneuvers

### Raufen / Scuffling (nicht-tödlich)

*−3 auf Angriffssumme mit echter Waffe; Waffen-Schadensbonus entfällt.*

| Wundresultat | Rauf-Effekt |
|---|---|
| Leichte Wunde | 1 Erschöpfungsstufe |
| Mittelschwere Wunde | 2 Erschöpfungsstufen |
| Schwere Wunde | 2 Erschöpfungsstufen + Leichte Wunde |
| Lähmende Wunde | 3 Erschöpfungsstufen + Leichte Wunde |
| Tod | 5 Erschöpfungsstufen + Mittelschwere Wunde |

### Ringen / Grappling

| Schritt | Regelung |
|---|---|
| Greifen | Angriffsvorteil > 0 → Ringkampfstärke = Angriffsvorteil |
| Befreien | Gegner würfelt Angriff; Erfolg → Angriffsvorteil von Ringkampfstärke subtrahieren |
| Flucht | Ringkampfstärke ≤ 0 → Entkommen; Ringer erleidet Leichte Wunde |
| Einschränkung | Geringter kann nicht angreifen oder sich bewegen |
| Fertigkeit | Raufen (zum Ringen); jede Nahkampffertigkeit (zum Befreien) |

### Sondermanöver / Special Maneuvers

| Englisch (EN) | Deutsch (DE) | Erforderlicher Angriffsvorteil |
|---|---|---|
| Disarm | Entwaffnen | 9 |
| Trip | Zu Fall bringen | 3 |
| Grab worn item | Getragenen Gegenstand greifen | 6 |
| Take opponent's weapon | Gegners Waffe nehmen | 12 |

---

## Gruppenregeln / Group Combat

| Englisch (EN) | Deutsch (DE) | Mechanik |
|---|---|---|
| Group | Gruppe | 1–6 Charaktere mit vergleichbaren Werten (Kampfwerte innerhalb 5 Punkte) |
| Vanguard | Vorkämpfer | Trägt Hauptlast; bestimmt Kampfwerte der Gruppe |
| Leader | Anführer | Nur bei ausgebildeten Gruppen |
| Untrained group | Unausgebildete Gruppe | Werte = Vorkämpfer-Werte |
| Trained group | Ausgebildete Gruppe | Bonus = Summe der Kampffertigkeiten aller Mitglieder, max. 3 × Führungswert des Anführers |
| Group Damage | Gruppenschaden | Berechneter Schaden × Anzahl der Kämpfer |
| Defenders | Beschützer | Beschützter Charakter wird nur verletzt wenn Beschützer patzen oder kampfunfähig werden |
| Splitting a group | Gruppe aufspalten | Angriffsvorteil ≥ 0 → spaltet statt Schaden; Gruppe bestimmt neue Anführer/Vorkämpfer |

---

## Rüstungen / Armor

| Englisch (EN) | Deutsch (DE) | Teilschutz | Vollschutz | Last (Teil/Voll) |
|---|---|---|---|---|
| Quilted / Fur | Gesteppt / Fell | 1 | 2 | 1 / 2 |
| Heavy Leather | Schweres Leder | 2 | 3 | 1 / 2 |
| Metal Reinforced Leather | Metallverstärktes Leder | 3 | 4 | 2 / 3 |
| Leather Scale | Lederschuppen | 3 | 4 | 2 / 3 |
| Metal Scale | Metallschuppen | 6 | 7 | 3 / 4 |
| Chain Mail | Kettenhemd | 9 | 12 | 4 / 6 |

*Vollrüstung umfasst Handschuhe, Unterarmschienen (Vambraces), Beinschienen (Chausses) und geschlossenen Helm.*

---

## Waffenfertigkeiten / Weapon Abilities

| Englisch (EN) | Deutsch (DE) | |
|---|---|---|
| Brawl | Raufen | Unbewaffneter Kampf und Raufwaffen |
| Single Weapon | Einzelwaffe | Einhändige Waffen und Schilde |
| Great Weapon | Große Waffe | Zweihändige Waffen |
| Thrown Weapon | Wurfwaffe | |
| Bow | Bogen | |

---

## Waffentabellen / Weapon Tables

### Raufwaffen / Brawling Weapons

| Englisch (EN) | Deutsch (DE) | Init | Atk | Dfn | Dam | Last |
|---|---|---|---|---|---|---|
| Dodge | Ausweichen | +0 | – | +3 | – | 0 |
| Fist | Faust | +0 | +0 | +0 | +0 | 0 |
| Kick | Tritt | −1 | +0 | −1 | +3 | 0 |
| Gauntlet | Panzerhandschuh | +0 | +0 | +1 | +2 | 1 |
| Bludgeon | Knüppel | +0 | +0 | +0 | +2 | 0 |
| Dagger | Dolch | +0 | +2 | +0 | +3 | 0 |
| Knife | Messer | −1 | +1 | +0 | +2 | 0 |

### Einzelwaffen / Single Weapons

| Englisch (EN) | Deutsch (DE) | Init | Atk | Dfn | Dam | Stä | Last |
|---|---|---|---|---|---|---|---|
| Axe | Axt | +1 | +3 | +0 | +6 | 0 | 1 |
| Club | Keule | +0 | +2 | +1 | +4 | 0 | 1 |
| Hatchet | Beil | +0 | +1 | +0 | +4 | 0 | 1 |
| Lance | Lanze | +1 | +3 | +0 | +5 | 0 | 2 |
| Mace | Streitkolben | +0 | +3 | +1 | +8 | +1 | 2 |
| Mace and Chain | Flegel | −1 | +4 | +0 | +8 | 0 | 2 |
| Spear, Short | Speer, kurz | +2 | +2 | +0 | +5 | 0 | 1 |
| Sword, Short | Schwert, kurz | +1 | +3 | +1 | +5 | 0 | 1 |
| Sword, Long | Schwert, lang | +1 | +4 | +1 | +6 | 0 | 1 |
| Shield, Buckler | Faustschild | +0 | +0 | +3 | +0 | −2 | 1 |
| Shield, Round | Rundschild | +0 | +0 | +4 | +0 | −1 | 2 |
| Shield, Heater | Dreieckschild | +0 | +0 | +5 | +0 | 0 | 2 |

### Große Waffen / Great Weapons

| Englisch (EN) | Deutsch (DE) | Init | Atk | Dfn | Dam | Stä | Last |
|---|---|---|---|---|---|---|---|
| Cudgel | Prügel | +0 | +2 | +1 | +5 | 0 | 1 |
| Farm Implement | Ackergerät | +0 | +1 | +0 | +3 | 0 | 2 |
| Flail | Dreschflegel | −1 | +4 | +0 | +9 | +1 | 3 |
| Pole Arm | Stangenwaffe | +2 | +4 | +2 | +8 | +2 | 3 |
| Pole Axe | Streitaxt | +1 | +5 | +1 | +11 | +3 | 4 |
| Spear, Long | Speer, lang | +3 | +3 | +1 | +7 | +1 | 2 |
| Sword, Great | Großschwert | +1 | +5 | +2 | +9 | +1 | 2 |
| Staff | Stab | +2 | +2 | +3 | +4 | 0 | 1 |
| Warhammer | Kriegshammer | +0 | +4 | +0 | +11 | +3 | 4 |

### Fernkampfwaffen / Missile Weapons

| Englisch (EN) | Deutsch (DE) | Init | Atk | Dam | Reichweite | Last |
|---|---|---|---|---|---|---|
| Axe, Throwing | Wurfaxt | −2 | +2 | +6 | 5 | 1 |
| Javelin | Wurfspeer | +2 | +3 | +5 | 10 | 1 |
| Knife | Messer (Wurf) | +0 | +2 | +3 | 5 | 0 |
| Sling | Schleuder | −2 | +2 | +4 | 20 | 0 |
| Stone | Stein | +0 | +1 | +2 | 5 | 0 |
| Bow, Long | Langbogen | −3 | +4 | +8 | 30 | 2 |
| Bow, Short | Kurzbogen | −1 | +3 | +6 | 15 | 1 |

---

*Quellen: Ars Magica Definitive Edition Core Rules (Atlas Games, 2024; GitHub: OriginalMadman/Ars-Magica-Open-License) · Deutsche Übersetzung (ArsMagica_DE_Gesamt_work.md)*
