//! Equipment table-row guard (data-integrity, PHASE 1 — test + triage only,
//! see `tmp/table-guard-handover.md`): every `rules/core/equipment.json`
//! weapon/shield/armor entry's shipped stats must match the actual Markdown
//! table row its own `source.lines[0]` cites — the Armor Table
//! (`ArMDE:16940-16949`), Melee Weapon Statistics (`ArMDE:16955-16990`),
//! Missile Weapon Statistics (`ArMDE:17001-17013`), and the Natural Weapons
//! Table's lone `weapon.grapple` row (`ArMDE:18557-18572`).
//!
//! Every row's table header is read fresh by scanning upward from the cited
//! data row for the nearest Markdown separator line (`| --- | --- |`) and
//! taking the row immediately above it — cells are mapped to fields **by
//! that header text**, never a hardcoded column position. The one exception
//! is the Armor Table's header, which repeats the word "Load" under both its
//! "Partial" and "Full" super-columns (`ArMDE:16942`); [`disambiguate_headers`]
//! resolves that by carrying the nearest preceding "Partial"/"Full" word
//! forward, which is itself header-driven rather than a literal index.
//!
//! No `rules/core/` or `src` change lands in this phase — [`RULED_EXCEPTIONS`]
//! is the only departure from a literal table-vs-data comparison, same shape
//! as `vf_descriptor_line.rs::RULED_EXCEPTIONS`.

use serde_json::Value;

const EQUIPMENT_JSON: &str = include_str!("../../../rules/core/equipment.json");
const EN_CORE_RULES: &str =
    include_str!("../../../rules/source/en/Ars Magica - Definitive Edition (Core Rules).md");

fn equipment_json() -> Value {
    serde_json::from_str(EQUIPMENT_JSON).expect("equipment.json is valid JSON")
}

fn source_lines() -> Vec<&'static str> {
    EN_CORE_RULES.lines().collect()
}

/// True for a Markdown table separator row (`| --- | --- |`): only pipes,
/// dashes, colons and whitespace, with at least one dash — what every
/// Markdown table prints directly under its header row.
fn is_separator_row(line: &str) -> bool {
    let t = line.trim();
    !t.is_empty() && t.contains('-') && t.chars().all(|c| matches!(c, '|' | '-' | ':' | ' '))
}

/// Splits one Markdown table row into trimmed cells, dropping the leading and
/// trailing empty cell a `| a | b |`-style line produces, and collapsing a
/// `<br>`-joined two-line cell (`"Partial<br>Prot"`) into one space-joined
/// string (`"Partial Prot"`).
fn split_row(line: &str) -> Vec<String> {
    let line = line.replace("<br>", " ");
    let mut cells: Vec<String> = line.split('|').map(|c| c.trim().to_string()).collect();
    if cells.first().is_some_and(String::is_empty) {
        cells.remove(0);
    }
    if cells.last().is_some_and(String::is_empty) {
        cells.pop();
    }
    cells
        .into_iter()
        .map(|c| c.split_whitespace().collect::<Vec<_>>().join(" "))
        .collect()
}

/// Disambiguates the Armor Table's two same-named "Load" header cells into
/// "Partial Load" / "Full Load" by carrying forward the nearest preceding
/// "Partial"/"Full" header word (`ArMDE:16942`) — still header-driven, not a
/// hardcoded index. Every other table's headers are already unique and pass
/// through unchanged.
fn disambiguate_headers(headers: Vec<String>) -> Vec<String> {
    let mut prefix: Option<&'static str> = None;
    headers
        .into_iter()
        .map(|h| {
            if h.starts_with("Partial") {
                prefix = Some("Partial");
            } else if h.starts_with("Full") {
                prefix = Some("Full");
            }
            if h == "Load"
                && let Some(p) = prefix
            {
                return format!("{p} Load");
            }
            h
        })
        .collect()
}

/// Finds the table header governing the data row at 1-based `row_line`: scans
/// upward for the nearest Markdown separator row, then reads the row directly
/// above it.
fn header_for(lines: &[&str], row_line: usize) -> Result<Vec<String>, String> {
    let mut i = row_line.saturating_sub(1); // 0-based index of the data row
    while i > 0 {
        i -= 1;
        if is_separator_row(lines[i]) {
            if i == 0 {
                return Err(format!(
                    "line {row_line}: separator row has no header above it"
                ));
            }
            return Ok(disambiguate_headers(split_row(lines[i - 1])));
        }
    }
    Err(format!(
        "line {row_line}: no Markdown table separator row found above it"
    ))
}

/// The cited row's own cells, read straight from `source.lines[0]`.
fn row_at(lines: &[&str], row_line: usize) -> Result<Vec<String>, String> {
    let line = lines
        .get(row_line - 1)
        .ok_or_else(|| format!("line {row_line} is past the end of the source file"))?;
    Ok(split_row(line))
}

fn cell<'a>(headers: &'a [String], cells: &'a [String], name: &str) -> Option<&'a str> {
    headers
        .iter()
        .position(|h| h == name)
        .and_then(|i| cells.get(i))
        .map(String::as_str)
}

/// The first cell of a row (its name/label column, whatever the table calls
/// it) — used only to look for a trailing footnote asterisk.
fn name_cell(cells: &[String]) -> &str {
    cells.first().map(String::as_str).unwrap_or("")
}

fn has_footnote_marker(name: &str) -> bool {
    name.trim_end().ends_with('*')
}

fn parse_signed(raw: &str) -> Result<i8, String> {
    let t = raw.trim().replace('\u{2212}', "-"); // normalize a stray math minus, belt-and-braces
    let t = t.strip_prefix('+').unwrap_or(&t);
    t.parse::<i8>()
        .map_err(|e| format!("cannot parse signed number {raw:?}: {e}"))
}

fn parse_signed_opt(raw: &str) -> Result<Option<i8>, String> {
    if raw.trim().eq_ignore_ascii_case("n/a") {
        return Ok(None);
    }
    parse_signed(raw).map(Some)
}

fn parse_u8(raw: &str) -> Result<u8, String> {
    raw.trim()
        .parse::<u8>()
        .map_err(|e| format!("cannot parse unsigned number {raw:?}: {e}"))
}

fn parse_u16_opt(raw: &str) -> Result<Option<u16>, String> {
    let t = raw.trim();
    if t.eq_ignore_ascii_case("n/a") {
        return Ok(None);
    }
    t.parse::<u16>()
        .map(Some)
        .map_err(|e| format!("cannot parse range {raw:?}: {e}"))
}

/// Maps the table's "Ability" column word to the engine's `ability.*` id,
/// including the one OCR slip the source ships at `ArMDE:16965` ("Braw1" for
/// "Brawl", a digit standing in for the letter l — `rules/source/` is a COPY,
/// so the parser recognizes the typo rather than a local hand-edit).
fn ability_id(word: &str) -> Result<&'static str, String> {
    match word.trim() {
        "Brawl" | "Braw1" => Ok("ability.brawl"),
        "Single" => Ok("ability.single_weapon"),
        "Great" => Ok("ability.great_weapon"),
        "Thrown" => Ok("ability.thrown_weapon"),
        "Bow" => Ok("ability.bows"),
        other => Err(format!("unrecognized Ability word {other:?} — parser gap")),
    }
}

/// Compares one `weapons[]` entry's shipped stats against its own cited
/// table row, collecting every mismatch rather than stopping at the first.
fn check_weapon(entry: &Value, lines: &[&str]) -> Result<(), Vec<String>> {
    let anchor = entry["source"]["anchor"]
        .as_str()
        .expect("weapon has source.anchor");
    let row_line = entry["source"]["lines"][0]
        .as_u64()
        .expect("weapon has source.lines[0]") as usize;
    let heading = anchor.split('/').next().unwrap_or("");

    let headers = match header_for(lines, row_line) {
        Ok(h) => h,
        Err(e) => return Err(vec![e]),
    };
    let cells = match row_at(lines, row_line) {
        Ok(c) => c,
        Err(e) => return Err(vec![e]),
    };
    if headers.len() != cells.len() {
        return Err(vec![format!(
            "header has {} column(s) but row at line {row_line} has {} — column count mismatch",
            headers.len(),
            cells.len()
        )]);
    }

    let mut problems = Vec::new();

    match cell(&headers, &cells, "Init") {
        Some(raw) => match parse_signed(raw) {
            Ok(v) => {
                let shipped = entry["init_mod"].as_i64().expect("init_mod") as i8;
                if v != shipped {
                    problems.push(format!("init_mod: table says {v} but shipped {shipped}"));
                }
            }
            Err(e) => problems.push(format!("Init cell: {e}")),
        },
        None => problems.push("no \"Init\" column found".to_string()),
    }

    match cell(&headers, &cells, "Atk") {
        Some(raw) => match parse_signed_opt(raw) {
            Ok(v) => {
                let shipped = entry
                    .get("attack_mod")
                    .and_then(Value::as_i64)
                    .map(|n| n as i8);
                if v != shipped {
                    problems.push(format!(
                        "attack_mod: table says {v:?} but shipped {shipped:?}"
                    ));
                }
            }
            Err(e) => problems.push(format!("Atk cell: {e}")),
        },
        None => problems.push("no \"Atk\" column found".to_string()),
    }

    match cell(&headers, &cells, "Dfn") {
        Some(raw) => match parse_signed(raw) {
            Ok(v) => {
                let shipped = entry["defense_mod"].as_i64().expect("defense_mod") as i8;
                if v != shipped {
                    problems.push(format!("defense_mod: table says {v} but shipped {shipped}"));
                }
            }
            Err(e) => problems.push(format!("Dfn cell: {e}")),
        },
        None => problems.push("no \"Dfn\" column found".to_string()),
    }

    match cell(&headers, &cells, "Dam") {
        Some(raw) => match parse_signed_opt(raw) {
            Ok(v) => {
                let shipped = entry
                    .get("damage_mod")
                    .and_then(Value::as_i64)
                    .map(|n| n as i8);
                if v != shipped {
                    problems.push(format!(
                        "damage_mod: table says {v:?} but shipped {shipped:?}"
                    ));
                }
            }
            Err(e) => problems.push(format!("Dam cell: {e}")),
        },
        None => problems.push("no \"Dam\" column found".to_string()),
    }

    // Natural Weapons Table has no Load/Str/Ability columns at all: "Strength,
    // Load and Cost are not applicable" / "All natural weapons use the Brawl
    // Ability" (ArMDE:18572) fix these to constants instead of reading a cell.
    if heading == "natural-weapons-table" {
        let shipped_load = entry["load"].as_u64().expect("load") as u8;
        if shipped_load != 0 {
            problems.push(format!(
                "load: Natural Weapons Table has no Load column (ArMDE:18572: \"Load... \
                 not applicable\"), so load must be 0, but shipped {shipped_load}"
            ));
        }
        if entry.get("min_strength").is_some() {
            problems.push(
                "min_strength: Natural Weapons Table has no Str column (ArMDE:18572: \
                 \"Strength... not applicable\"), so min_strength must be absent"
                    .to_string(),
            );
        }
        let shipped_ability = entry["ability"].as_str().expect("ability");
        if shipped_ability != "ability.brawl" {
            problems.push(format!(
                "ability: Natural Weapons Table has no Ability column (ArMDE:18572: \"All \
                 natural weapons use the Brawl Ability\"), so ability must be ability.brawl, \
                 but shipped {shipped_ability}"
            ));
        }
    } else {
        match cell(&headers, &cells, "Load") {
            Some(raw) => match parse_u8(raw) {
                Ok(v) => {
                    let shipped = entry["load"].as_u64().expect("load") as u8;
                    if v != shipped {
                        problems.push(format!("load: table says {v} but shipped {shipped}"));
                    }
                }
                Err(e) => problems.push(format!("Load cell: {e}")),
            },
            None => problems.push("no \"Load\" column found".to_string()),
        }

        match cell(&headers, &cells, "Str") {
            Some(raw) => match parse_signed_opt(raw) {
                Ok(v) => {
                    let shipped = entry
                        .get("min_strength")
                        .and_then(Value::as_i64)
                        .map(|n| n as i8);
                    if v != shipped {
                        problems.push(format!(
                            "min_strength: table says {v:?} but shipped {shipped:?}"
                        ));
                    }
                }
                Err(e) => problems.push(format!("Str cell: {e}")),
            },
            None => problems.push("no \"Str\" column found".to_string()),
        }

        match cell(&headers, &cells, "Ability") {
            Some(raw) => match ability_id(raw) {
                Ok(expected) => {
                    let shipped = entry["ability"].as_str().expect("ability");
                    if shipped != expected {
                        problems.push(format!(
                            "ability: table says \"{raw}\" ({expected}) but shipped {shipped}"
                        ));
                    }
                }
                Err(e) => problems.push(format!("Ability cell: {e}")),
            },
            None => problems.push("no \"Ability\" column found".to_string()),
        }
    }

    match cell(&headers, &cells, "Range") {
        Some(raw) => match parse_u16_opt(raw) {
            Ok(v) => {
                let shipped = entry.get("range").and_then(Value::as_u64).map(|n| n as u16);
                if v != shipped {
                    problems.push(format!("range: table says {v:?} but shipped {shipped:?}"));
                }
            }
            Err(e) => problems.push(format!("Range cell: {e}")),
        },
        None => {
            if entry.get("range").is_some() {
                problems
                    .push("range: shipped a range but this table has no Range column".to_string());
            }
        }
    }

    // two_handed is derived per table, never a column: Melee table rows whose
    // Ability is "Great" (9 rows, ArMDE:7494); Missile table rows whose name
    // cell carries the footnote asterisk (ArMDE:17008-17013).
    let ability_word = cell(&headers, &cells, "Ability").map(str::trim);
    let expected_two_handed = match heading {
        "melee-weapon-statistics" => ability_word == Some("Great"),
        "missile-weapon-statistics" => has_footnote_marker(name_cell(&cells)),
        _ => false,
    };
    let shipped_two_handed = entry
        .get("two_handed")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    if expected_two_handed != shipped_two_handed {
        problems.push(format!(
            "two_handed: expected {expected_two_handed} ({heading} rule) but shipped \
             {shipped_two_handed}"
        ));
    }

    if problems.is_empty() {
        Ok(())
    } else {
        Err(problems)
    }
}

/// Compares one `shields[]` entry's shipped stats against its own cited table
/// row (shields share the Melee Weapon Statistics table, ArMDE:16975-16977).
/// A shield has no Ability/Dam/Range/two_handed of its own, so those columns
/// are not checked.
fn check_shield(entry: &Value, lines: &[&str]) -> Result<(), Vec<String>> {
    let row_line = entry["source"]["lines"][0]
        .as_u64()
        .expect("shield has source.lines[0]") as usize;

    let headers = match header_for(lines, row_line) {
        Ok(h) => h,
        Err(e) => return Err(vec![e]),
    };
    let cells = match row_at(lines, row_line) {
        Ok(c) => c,
        Err(e) => return Err(vec![e]),
    };
    if headers.len() != cells.len() {
        return Err(vec![format!(
            "header has {} column(s) but row at line {row_line} has {} — column count mismatch",
            headers.len(),
            cells.len()
        )]);
    }

    let mut problems = Vec::new();

    match cell(&headers, &cells, "Init") {
        Some(raw) => match parse_signed(raw) {
            Ok(v) => {
                let shipped = entry["init_mod"].as_i64().expect("init_mod") as i8;
                if v != shipped {
                    problems.push(format!("init_mod: table says {v} but shipped {shipped}"));
                }
            }
            Err(e) => problems.push(format!("Init cell: {e}")),
        },
        None => problems.push("no \"Init\" column found".to_string()),
    }

    match cell(&headers, &cells, "Atk") {
        Some(raw) => match parse_signed(raw) {
            Ok(v) => {
                let shipped = entry["attack_mod"].as_i64().expect("attack_mod") as i8;
                if v != shipped {
                    problems.push(format!("attack_mod: table says {v} but shipped {shipped}"));
                }
            }
            Err(e) => problems.push(format!("Atk cell: {e}")),
        },
        None => problems.push("no \"Atk\" column found".to_string()),
    }

    match cell(&headers, &cells, "Dfn") {
        Some(raw) => match parse_signed(raw) {
            Ok(v) => {
                let shipped = entry["defense_mod"].as_i64().expect("defense_mod") as i8;
                if v != shipped {
                    problems.push(format!("defense_mod: table says {v} but shipped {shipped}"));
                }
            }
            Err(e) => problems.push(format!("Dfn cell: {e}")),
        },
        None => problems.push("no \"Dfn\" column found".to_string()),
    }

    match cell(&headers, &cells, "Load") {
        Some(raw) => match parse_u8(raw) {
            Ok(v) => {
                let shipped = entry["load"].as_u64().expect("load") as u8;
                if v != shipped {
                    problems.push(format!("load: table says {v} but shipped {shipped}"));
                }
            }
            Err(e) => problems.push(format!("Load cell: {e}")),
        },
        None => problems.push("no \"Load\" column found".to_string()),
    }

    match cell(&headers, &cells, "Str") {
        Some(raw) => match parse_signed(raw) {
            Ok(v) => {
                let shipped = entry["min_strength"].as_i64().expect("min_strength") as i8;
                if v != shipped {
                    problems.push(format!(
                        "min_strength: table says {v} but shipped {shipped}"
                    ));
                }
            }
            Err(e) => problems.push(format!("Str cell: {e}")),
        },
        None => problems.push("no \"Str\" column found".to_string()),
    }

    if problems.is_empty() {
        Ok(())
    } else {
        Err(problems)
    }
}

/// Compares one `armor[]` entry's shipped `protection`/`load` against its own
/// cited Armor Table row, picking the "Partial" or "Full" sub-column pair by
/// the entry id's own `_partial`/`_full` suffix.
fn check_armor(entry: &Value, lines: &[&str]) -> Result<(), Vec<String>> {
    let id = entry["id"].as_str().expect("armor has id");
    let row_line = entry["source"]["lines"][0]
        .as_u64()
        .expect("armor has source.lines[0]") as usize;

    let variant = if id.ends_with("_partial") {
        "Partial"
    } else if id.ends_with("_full") {
        "Full"
    } else {
        return Err(vec![format!(
            "armor id {id:?} ends in neither \"_partial\" nor \"_full\" — parser gap"
        )]);
    };

    let headers = match header_for(lines, row_line) {
        Ok(h) => h,
        Err(e) => return Err(vec![e]),
    };
    let cells = match row_at(lines, row_line) {
        Ok(c) => c,
        Err(e) => return Err(vec![e]),
    };
    if headers.len() != cells.len() {
        return Err(vec![format!(
            "header has {} column(s) but row at line {row_line} has {} — column count mismatch",
            headers.len(),
            cells.len()
        )]);
    }

    let mut problems = Vec::new();
    let prot_col = format!("{variant} Prot");
    let load_col = format!("{variant} Load");

    match cell(&headers, &cells, &prot_col) {
        Some(raw) => match parse_u8(raw) {
            Ok(v) => {
                let shipped = entry["protection"].as_u64().expect("protection") as u8;
                if v != shipped {
                    problems.push(format!("protection: table says {v} but shipped {shipped}"));
                }
            }
            Err(e) => problems.push(format!("{prot_col} cell: {e}")),
        },
        None => problems.push(format!("no \"{prot_col}\" column found")),
    }

    match cell(&headers, &cells, &load_col) {
        Some(raw) => match parse_u8(raw) {
            Ok(v) => {
                let shipped = entry["load"].as_u64().expect("load") as u8;
                if v != shipped {
                    problems.push(format!("load: table says {v} but shipped {shipped}"));
                }
            }
            Err(e) => problems.push(format!("{load_col} cell: {e}")),
        },
        None => problems.push(format!("no \"{load_col}\" column found")),
    }

    if problems.is_empty() {
        Ok(())
    } else {
        Err(problems)
    }
}

/// Entries whose own data is known, by a recorded ruling, to disagree with
/// its cited table row — same shape as `vf_descriptor_line.rs::RULED_EXCEPTIONS`.
/// Each row must still fail the relevant `check_*` ([`ruled_exceptions_still_mismatch`]).
const RULED_EXCEPTIONS: &[(&str, &str)] = &[(
    "weapon.sling",
    "RULES.md \"Two-handed weapon\" row: the Missile Weapon Statistics \
     table's asterisk footnote (\"Requires two free hands to load and \
     fire\", ArMDE:17008-17013) covers the Sling row too, but the existing \
     ruling keeps it unflagged — \"the Sling shares that asterisk but \
     stays unflagged: it is a thrown weapon and keeps the status-quo \
     shield handling\" — so two_handed stays false despite the footnote \
     this guard reads literally. Pre-existing, documented ruling, not a \
     new finding.",
)];

fn exception_reason(id: &str) -> Option<&'static str> {
    RULED_EXCEPTIONS
        .iter()
        .find(|(i, _)| *i == id)
        .map(|(_, r)| *r)
}

fn find_equipment_entry(doc: &Value, id: &str) -> (&'static str, Value) {
    if let Some(w) = doc["weapons"]
        .as_array()
        .expect("weapons array")
        .iter()
        .find(|w| w["id"].as_str() == Some(id))
    {
        return ("weapon", w.clone());
    }
    if let Some(s) = doc["shields"]
        .as_array()
        .expect("shields array")
        .iter()
        .find(|s| s["id"].as_str() == Some(id))
    {
        return ("shield", s.clone());
    }
    if let Some(a) = doc["armor"]
        .as_array()
        .expect("armor array")
        .iter()
        .find(|a| a["id"].as_str() == Some(id))
    {
        return ("armor", a.clone());
    }
    panic!("{id} not found in equipment.json");
}

fn check_by_category(category: &str, entry: &Value, lines: &[&str]) -> Result<(), Vec<String>> {
    match category {
        "weapon" => check_weapon(entry, lines),
        "shield" => check_shield(entry, lines),
        "armor" => check_armor(entry, lines),
        other => panic!("unknown equipment category {other:?}"),
    }
}

#[test]
fn every_weapon_shield_and_armor_entry_matches_its_own_table_row() {
    let lines = source_lines();
    let doc = equipment_json();
    let weapons = doc["weapons"].as_array().expect("weapons array");
    let shields = doc["shields"].as_array().expect("shields array");
    let armor = doc["armor"].as_array().expect("armor array");
    let shipped_count = weapons.len() + shields.len() + armor.len();
    assert!(
        shipped_count > 0,
        "equipment.json is empty — nothing to guard"
    );

    let mut checked = 0usize;
    let mut failures = Vec::new();

    for w in weapons {
        checked += 1;
        let id = w["id"].as_str().expect("weapon id").to_string();
        if exception_reason(&id).is_some() {
            continue;
        }
        if let Err(problems) = check_weapon(w, &lines) {
            failures.push(format!("{id}: {}", problems.join("; ")));
        }
    }
    for s in shields {
        checked += 1;
        let id = s["id"].as_str().expect("shield id").to_string();
        if exception_reason(&id).is_some() {
            continue;
        }
        if let Err(problems) = check_shield(s, &lines) {
            failures.push(format!("{id}: {}", problems.join("; ")));
        }
    }
    for a in armor {
        checked += 1;
        let id = a["id"].as_str().expect("armor id").to_string();
        if exception_reason(&id).is_some() {
            continue;
        }
        if let Err(problems) = check_armor(a, &lines) {
            failures.push(format!("{id}: {}", problems.join("; ")));
        }
    }

    assert_eq!(
        checked, shipped_count,
        "must check every weapon/shield/armor entry, never a subset"
    );
    assert!(
        failures.is_empty(),
        "{} equipment entries disagree with their own table row:\n{}",
        failures.len(),
        failures.join("\n")
    );
}

/// Every [`RULED_EXCEPTIONS`] row exists to silence the main guard for one
/// entry whose cited table row genuinely disagrees with the shipped data by
/// a recorded ruling. If a future data change makes the comparison agree
/// again, the row must be deleted.
#[test]
fn ruled_exceptions_still_mismatch() {
    let lines = source_lines();
    let doc = equipment_json();
    let mut stale = Vec::new();
    for (id, _) in RULED_EXCEPTIONS {
        let (category, entry) = find_equipment_entry(&doc, id);
        if check_by_category(category, &entry, &lines).is_ok() {
            stale.push(*id);
        }
    }
    assert!(
        stale.is_empty(),
        "RULED_EXCEPTIONS row(s) {stale:?} now MATCH their own table row — the defect they \
         recorded is gone, so remove the row(s)"
    );
}

/// Proves the `check_*` functions can actually fail, by corrupting one real
/// field at a time on a genuine catalogue entry (so the rulebook text being
/// compared against is real) and asserting the guard rejects it.
#[test]
fn check_rejects_synthetic_wrong_entries() {
    let lines = source_lines();
    let doc = equipment_json();

    let (_, mut wrong_attack) = find_equipment_entry(&doc, "weapon.sword_long");
    wrong_attack["attack_mod"] = serde_json::json!(99);
    assert!(
        check_weapon(&wrong_attack, &lines).is_err(),
        "a deliberately wrong attack_mod must be rejected"
    );

    let (_, mut wrong_two_handed) = find_equipment_entry(&doc, "weapon.cudgel");
    wrong_two_handed["two_handed"] = serde_json::json!(false);
    assert!(
        check_weapon(&wrong_two_handed, &lines).is_err(),
        "a deliberately wrong two_handed on a Great-Ability weapon must be rejected"
    );

    let (_, mut wrong_ability) = find_equipment_entry(&doc, "weapon.axe");
    wrong_ability["ability"] = serde_json::json!("ability.great_weapon");
    assert!(
        check_weapon(&wrong_ability, &lines).is_err(),
        "a deliberately wrong ability must be rejected"
    );

    let (_, mut wrong_shield) = find_equipment_entry(&doc, "shield.heater");
    wrong_shield["defense_mod"] = serde_json::json!(0);
    assert!(
        check_shield(&wrong_shield, &lines).is_err(),
        "a deliberately wrong shield defense_mod must be rejected"
    );

    let (_, mut wrong_armor) = find_equipment_entry(&doc, "armor.chain_mail_full");
    wrong_armor["protection"] = serde_json::json!(1);
    assert!(
        check_armor(&wrong_armor, &lines).is_err(),
        "a deliberately wrong armor protection must be rejected"
    );
}

/// Guards the guard: [`ability_id`] must recognize the real catalogue's
/// own source typo (ArMDE:16965, "Braw1") without silently accepting an
/// arbitrary unrecognized word.
#[test]
fn ability_id_rejects_unrecognized_words() {
    assert_eq!(ability_id("Brawl"), Ok("ability.brawl"));
    assert_eq!(ability_id("Braw1"), Ok("ability.brawl"));
    assert!(ability_id("Nonsense").is_err());
}
