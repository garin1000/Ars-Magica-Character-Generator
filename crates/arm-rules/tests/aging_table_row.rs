//! Aging table-row guard (data-integrity, PHASE 1 — test + triage only, see
//! `tmp/table-guard-handover.md`): every `rules/core/aging.json` row must
//! match the English core rulebook's own `## Aging` section
//! (`ArMDE:16563-16642`) — the Living Conditions table (`ArMDE:16581-16594`),
//! the Aging Roll table (`ArMDE:16597-16611`), the Crisis Roll table
//! (`ArMDE:16624-16632`), and the two prose-derived facts that are not table
//! rows at all: `crisis.die` (`ArMDE:472-474`, the Simple Die) and
//! `crisis.attendant` (`ArMDE:16634`, the attending doctor).
//!
//! Table rows are read the same way as `equipment_table_row.rs`: the header
//! governing a cited data row is found by scanning upward for the nearest
//! Markdown separator line, and cells are mapped to fields by that header
//! text. Two aging-specific wrinkles:
//!
//! - The row-key column (`"Aging Roll"` / `"Crisis Roll"`) is itself a band —
//!   `"10–12"` (note: an **en dash**, U+2013, not a hyphen), a bare number
//!   (`"13"`), or an open end (`"22+"`, `"8 or less"`) — parsed into
//!   `(min, max)` by [`parse_band`].
//! - The `"Result"` column is English prose, not a stat; [`expected_aging_effect`]
//!   and [`expected_crisis_outcome`] parse it into the same tagged-union shape
//!   `rules/core/aging.json` ships (`AgingRowEffect`/`CrisisOutcome`,
//!   `crates/arm-rules/src/aging.rs`), compared structurally as
//!   `serde_json::Value` rather than against the Rust enum — this guard reads
//!   raw JSON throughout, matching `vf_descriptor_line.rs`'s own reasoning for
//!   doing so.
//!
//! No `rules/core/` or `src` change lands in this phase — [`RULED_EXCEPTIONS`]
//! is empty: the Phase 1 triage found zero `aging.json` mismatches (kept, not
//! deleted, so a future finding has a ready-made, already-wired place to
//! record a ruling — same reasoning as `ability_category_line.rs`'s empty
//! list).

use regex::Regex;
use serde_json::Value;

const AGING_JSON: &str = include_str!("../../../rules/core/aging.json");
const EN_CORE_RULES: &str =
    include_str!("../../../rules/source/en/Ars Magica - Definitive Edition (Core Rules).md");

fn aging_json() -> Value {
    serde_json::from_str(AGING_JSON).expect("aging.json is valid JSON")
}

fn source_lines() -> Vec<&'static str> {
    EN_CORE_RULES.lines().collect()
}

/// True for a Markdown table separator row (`| --- | --- |`): only pipes,
/// dashes, colons and whitespace, with at least one dash.
fn is_separator_row(line: &str) -> bool {
    let t = line.trim();
    !t.is_empty() && t.contains('-') && t.chars().all(|c| matches!(c, '|' | '-' | ':' | ' '))
}

/// Splits one Markdown table row into trimmed cells, dropping the leading and
/// trailing empty cell a `| a | b |`-style line produces.
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
            return Ok(split_row(lines[i - 1]));
        }
    }
    Err(format!(
        "line {row_line}: no Markdown table separator row found above it"
    ))
}

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

fn parse_signed(raw: &str) -> Result<i32, String> {
    let t = raw.trim().replace('\u{2212}', "-");
    let t = t.strip_prefix('+').unwrap_or(&t);
    t.parse::<i32>()
        .map_err(|e| format!("cannot parse signed number {raw:?}: {e}"))
}

/// Parses a row-key band cell into `(min, max)`, handling every shape the
/// Aging Roll / Crisis Roll tables use: `"22+"` (open above), `"8 or less"`
/// (open below), `"10–12"` (an en-dash-separated range — **not** a hyphen),
/// and a bare single number (`"13"`).
fn parse_band(raw: &str) -> Result<(Option<i32>, Option<i32>), String> {
    let t = raw.trim();
    if let Some(n) = t.strip_suffix('+') {
        let v = n
            .trim()
            .parse::<i32>()
            .map_err(|e| format!("cannot parse band {raw:?}: {e}"))?;
        return Ok((Some(v), None));
    }
    if let Some(n) = t.strip_suffix("or less") {
        let v = n
            .trim()
            .parse::<i32>()
            .map_err(|e| format!("cannot parse band {raw:?}: {e}"))?;
        return Ok((None, Some(v)));
    }
    if let Some((lo, hi)) = t.split_once('\u{2013}') {
        let lo = lo
            .trim()
            .parse::<i32>()
            .map_err(|e| format!("cannot parse band {raw:?}: {e}"))?;
        let hi = hi
            .trim()
            .parse::<i32>()
            .map_err(|e| format!("cannot parse band {raw:?}: {e}"))?;
        return Ok((Some(lo), Some(hi)));
    }
    let v = t
        .parse::<i32>()
        .map_err(|e| format!("cannot parse band {raw:?}: {e}"))?;
    Ok((Some(v), Some(v)))
}

fn characteristic_slug(word: &str) -> Result<&'static str, String> {
    match word {
        "Int" => Ok("int"),
        "Per" => Ok("per"),
        "Str" => Ok("str"),
        "Sta" => Ok("sta"),
        "Prs" => Ok("pre"), // the table abbreviates Presence as "Prs", not "Pre"
        "Com" => Ok("com"),
        "Dex" => Ok("dex"),
        "Qik" => Ok("qik"),
        other => Err(format!(
            "unrecognized Characteristic abbreviation {other:?} — parser gap"
        )),
    }
}

/// Parses the Aging Roll table's "Result" prose into the same tagged-union
/// shape `AgingRowEffect` serializes to (`crates/arm-rules/src/aging.rs`).
fn expected_aging_effect(result_text: &str) -> Result<Value, String> {
    let result = result_text.trim();
    if result == "1 Aging Point in any Characteristic" {
        return Ok(serde_json::json!({ "type": "any_characteristic", "points": 1 }));
    }
    if result.starts_with("Gain sufficient Aging Points") {
        return Ok(serde_json::json!({ "type": "next_decrepitude_level_and_crisis" }));
    }
    let re = Regex::new(r"^1 Aging Point in (\w+)(?: and (\w+))?$").expect("valid regex");
    if let Some(caps) = re.captures(result) {
        let mut characteristics = vec![characteristic_slug(&caps[1])?];
        if let Some(second) = caps.get(2) {
            characteristics.push(characteristic_slug(second.as_str())?);
        }
        return Ok(serde_json::json!({
            "type": "named_characteristics",
            "points": 1,
            "characteristics": characteristics,
        }));
    }
    Err(format!(
        "unrecognized Aging Roll \"Result\" text {result:?} — parser gap"
    ))
}

/// Parses the Crisis Roll table's "Result" prose into the same tagged-union
/// shape `CrisisOutcome` serializes to.
fn expected_crisis_outcome(result_text: &str) -> Result<Value, String> {
    let result = result_text.trim();
    if result.starts_with("Bedridden for a week") || result.starts_with("Bedridden for a month") {
        return Ok(serde_json::json!({ "type": "bedridden" }));
    }

    let ease_re = Regex::new(r"Ease Factor of (\d+)").expect("valid regex");
    let ritual_re = Regex::new(r"CrCo(\d+)").expect("valid regex");
    let ritual_level = ritual_re
        .captures(result)
        .and_then(|c| c[1].parse::<i64>().ok());

    for (label, slug) in [
        ("Minor illness", "minor"),
        ("Serious illness", "serious"),
        ("Major illness", "major"),
        ("Critical illness", "critical"),
    ] {
        if result.contains(label) {
            let ease_factor = ease_re
                .captures(result)
                .and_then(|c| c[1].parse::<i64>().ok())
                .ok_or_else(|| format!("no \"Ease Factor of N\" found in {result:?}"))?;
            let ritual_level = ritual_level
                .ok_or_else(|| format!("no \"CrCoN\" ritual level found in {result:?}"))?;
            return Ok(serde_json::json!({
                "type": "illness",
                "severity": slug,
                "ease_factor": ease_factor,
                "ritual_level": ritual_level,
            }));
        }
    }

    if result.contains("Terminal illness") {
        let ritual_level =
            ritual_level.ok_or_else(|| format!("no \"CrCoN\" ritual level found in {result:?}"))?;
        return Ok(serde_json::json!({
            "type": "illness",
            "severity": "terminal",
            "ritual_level": ritual_level,
        }));
    }

    Err(format!(
        "unrecognized Crisis Roll \"Result\" text {result:?} — parser gap"
    ))
}

fn check_living_condition(entry: &Value, lines: &[&str]) -> Result<(), Vec<String>> {
    let row_line = entry["source"]["lines"][0]
        .as_u64()
        .expect("living_condition has source.lines[0]") as usize;
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

    match cell(&headers, &cells, "Modifier") {
        Some(raw) => match parse_signed(raw) {
            Ok(v) => {
                let shipped = entry["modifier"].as_i64().expect("modifier") as i32;
                if v != shipped {
                    problems.push(format!("modifier: table says {v} but shipped {shipped}"));
                }
            }
            Err(e) => problems.push(format!("Modifier cell: {e}")),
        },
        None => problems.push("no \"Modifier\" column found".to_string()),
    }

    let name = cell(&headers, &cells, "Living Conditions").unwrap_or("");
    let expected_cumulative = name.trim_end().ends_with('*');
    let shipped_cumulative = entry
        .get("cumulative")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    if expected_cumulative != shipped_cumulative {
        problems.push(format!(
            "cumulative: name cell {name:?} {} a trailing asterisk, so cumulative should be \
             {expected_cumulative}, but shipped {shipped_cumulative}",
            if expected_cumulative {
                "carries"
            } else {
                "lacks"
            }
        ));
    }

    if problems.is_empty() {
        Ok(())
    } else {
        Err(problems)
    }
}

fn check_aging_roll(entry: &Value, lines: &[&str]) -> Result<(), Vec<String>> {
    let row_line = entry["source"]["lines"][0]
        .as_u64()
        .expect("aging roll row has source.lines[0]") as usize;
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

    match cell(&headers, &cells, "Aging Roll") {
        Some(raw) => match parse_band(raw) {
            Ok((min, max)) => {
                let shipped_min = entry.get("min").and_then(Value::as_i64).map(|v| v as i32);
                let shipped_max = entry.get("max").and_then(Value::as_i64).map(|v| v as i32);
                if min != shipped_min {
                    problems.push(format!(
                        "min: table band {raw:?} implies {min:?} but shipped {shipped_min:?}"
                    ));
                }
                if max != shipped_max {
                    problems.push(format!(
                        "max: table band {raw:?} implies {max:?} but shipped {shipped_max:?}"
                    ));
                }
            }
            Err(e) => problems.push(format!("\"Aging Roll\" cell: {e}")),
        },
        None => problems.push("no \"Aging Roll\" column found".to_string()),
    }

    match cell(&headers, &cells, "Result") {
        Some(raw) => match expected_aging_effect(raw) {
            Ok(expected) => {
                let shipped = entry["effect"].clone();
                if shipped != expected {
                    problems.push(format!(
                        "effect: table Result {raw:?} implies {expected} but shipped {shipped}"
                    ));
                }
            }
            Err(e) => problems.push(e),
        },
        None => problems.push("no \"Result\" column found".to_string()),
    }

    if problems.is_empty() {
        Ok(())
    } else {
        Err(problems)
    }
}

fn check_crisis_row(entry: &Value, lines: &[&str]) -> Result<(), Vec<String>> {
    let row_line = entry["source"]["lines"][0]
        .as_u64()
        .expect("crisis row has source.lines[0]") as usize;
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

    match cell(&headers, &cells, "Crisis Roll") {
        Some(raw) => match parse_band(raw) {
            Ok((min, max)) => {
                let shipped_min = entry.get("min").and_then(Value::as_i64).map(|v| v as i32);
                let shipped_max = entry.get("max").and_then(Value::as_i64).map(|v| v as i32);
                if min != shipped_min {
                    problems.push(format!(
                        "min: table band {raw:?} implies {min:?} but shipped {shipped_min:?}"
                    ));
                }
                if max != shipped_max {
                    problems.push(format!(
                        "max: table band {raw:?} implies {max:?} but shipped {shipped_max:?}"
                    ));
                }
            }
            Err(e) => problems.push(format!("\"Crisis Roll\" cell: {e}")),
        },
        None => problems.push("no \"Crisis Roll\" column found".to_string()),
    }

    match cell(&headers, &cells, "Result") {
        Some(raw) => match expected_crisis_outcome(raw) {
            Ok(expected) => {
                let shipped = entry["outcome"].clone();
                if shipped != expected {
                    problems.push(format!(
                        "outcome: table Result {raw:?} implies {expected} but shipped {shipped}"
                    ));
                }
            }
            Err(e) => problems.push(e),
        },
        None => problems.push("no \"Result\" column found".to_string()),
    }

    if problems.is_empty() {
        Ok(())
    } else {
        Err(problems)
    }
}

/// `crisis.die` is prose, not a table row: "Roll a ten-sided die. Each number
/// counts for its value, except that a zero counts as ten." (`ArMDE:474`).
fn check_crisis_die(entry: &Value, lines: &[&str]) -> Result<(), Vec<String>> {
    let start = entry["source"]["lines"][0]
        .as_u64()
        .expect("crisis.die has source.lines[0]") as usize;
    let end = entry["source"]["lines"][1]
        .as_u64()
        .expect("crisis.die has source.lines[1]") as usize;
    let Some(bracketed) = lines.get(start.saturating_sub(1)..end) else {
        return Err(vec![format!(
            "source lines {start}-{end} are out of range of the source file"
        )]);
    };
    let text = bracketed.join(" ");

    let mut problems = Vec::new();
    if !text.contains("ten-sided die") {
        problems.push(format!(
            "expected \"ten-sided die\" in cited prose (lines {start}-{end}) — citation/parser \
             gap"
        ));
    }
    if !text.contains("a zero counts as ten") {
        problems.push(format!(
            "expected \"a zero counts as ten\" in cited prose (lines {start}-{end}) — \
             citation/parser gap"
        ));
    }
    let shipped_min = entry["min"].as_i64().expect("min");
    let shipped_max = entry["max"].as_i64().expect("max");
    if shipped_min != 1 {
        problems.push(format!(
            "min: a ten-sided die numbered so its 0 face counts as 10 runs 1-10, but shipped \
             min={shipped_min}"
        ));
    }
    if shipped_max != 10 {
        problems.push(format!(
            "max: a ten-sided die numbered so its 0 face counts as 10 runs 1-10, but shipped \
             max={shipped_max}"
        ));
    }

    if problems.is_empty() {
        Ok(())
    } else {
        Err(problems)
    }
}

/// `crisis.attendant` is prose, not a table row (`ArMDE:16634`): "An Int +
/// Medicine roll against an Ease Factor of 6 allows the character to add the
/// attendant's Medicine score to the roll to survive the crisis. ... if the
/// doctor botches the character must subtract 3 from the survival roll."
fn check_crisis_attendant(entry: &Value, lines: &[&str]) -> Result<(), Vec<String>> {
    let line_no = entry["source"]["lines"][0]
        .as_u64()
        .expect("crisis.attendant has source.lines[0]") as usize;
    let Some(text) = lines.get(line_no - 1) else {
        return Err(vec![format!(
            "line {line_no} is past the end of the source file"
        )]);
    };

    let mut problems = Vec::new();

    if !text.contains("Int + Medicine") {
        problems.push(format!(
            "expected \"Int + Medicine\" wording at line {line_no} — citation/parser gap"
        ));
    }

    let ease_re = Regex::new(r"Ease Factor of (\d+)").expect("valid regex");
    let ease_factor = ease_re
        .captures(text)
        .and_then(|c| c[1].parse::<i64>().ok());
    let shipped_ease = entry["ease_factor"].as_i64();
    if ease_factor != shipped_ease {
        problems.push(format!(
            "ease_factor: prose says {ease_factor:?} but shipped {shipped_ease:?}"
        ));
    }

    let botch_re = Regex::new(r"subtract (\d+) from the survival roll").expect("valid regex");
    let botch_penalty = botch_re
        .captures(text)
        .and_then(|c| c[1].parse::<i64>().ok())
        .map(|v| -v);
    let shipped_botch = entry["botch_penalty"].as_i64();
    if botch_penalty != shipped_botch {
        problems.push(format!(
            "botch_penalty: prose says {botch_penalty:?} but shipped {shipped_botch:?}"
        ));
    }

    let shipped_ability = entry["ability"].as_str().expect("ability");
    if shipped_ability != "ability.medicine" {
        problems.push(format!(
            "ability: prose names Medicine but shipped {shipped_ability}"
        ));
    }

    let shipped_characteristic = entry["characteristic"].as_str().expect("characteristic");
    if shipped_characteristic != "int" {
        problems.push(format!(
            "characteristic: prose names Int but shipped {shipped_characteristic}"
        ));
    }

    if problems.is_empty() {
        Ok(())
    } else {
        Err(problems)
    }
}

/// Entries whose own data is known, by a recorded ruling, to disagree with
/// its cited passage — same shape as `vf_descriptor_line.rs::RULED_EXCEPTIONS`.
/// Empty: the Phase 1 triage of this guard found zero `aging.json` mismatches.
const RULED_EXCEPTIONS: &[(&str, &str)] = &[];

fn exception_reason(id: &str) -> Option<&'static str> {
    RULED_EXCEPTIONS
        .iter()
        .find(|(i, _)| *i == id)
        .map(|(_, r)| *r)
}

/// One `aging.json` entry carrying a `source`, found anywhere in the file —
/// `(category, id, value)` — so the main test and the self-tests below share
/// a single enumeration and dispatch path.
fn all_entries(doc: &Value) -> Vec<(&'static str, String, Value)> {
    let mut out = Vec::new();
    for lc in doc["living_conditions"]
        .as_array()
        .expect("living_conditions array")
    {
        out.push((
            "living_condition",
            lc["id"].as_str().expect("living_condition id").to_string(),
            lc.clone(),
        ));
    }
    for row in doc["outcomes"].as_array().expect("outcomes array") {
        out.push((
            "aging_roll",
            row["id"].as_str().expect("aging roll id").to_string(),
            row.clone(),
        ));
    }
    for row in doc["crisis"]["rows"].as_array().expect("crisis.rows array") {
        out.push((
            "crisis_row",
            row["id"].as_str().expect("crisis row id").to_string(),
            row.clone(),
        ));
    }
    if let Some(die) = doc["crisis"].get("die") {
        out.push((
            "crisis_die",
            die["id"].as_str().unwrap_or("crisis.die").to_string(),
            die.clone(),
        ));
    }
    if let Some(attendant) = doc["crisis"].get("attendant") {
        out.push((
            "crisis_attendant",
            attendant["id"]
                .as_str()
                .unwrap_or("crisis.attendant")
                .to_string(),
            attendant.clone(),
        ));
    }
    out
}

fn check_by_category(category: &str, entry: &Value, lines: &[&str]) -> Result<(), Vec<String>> {
    match category {
        "living_condition" => check_living_condition(entry, lines),
        "aging_roll" => check_aging_roll(entry, lines),
        "crisis_row" => check_crisis_row(entry, lines),
        "crisis_die" => check_crisis_die(entry, lines),
        "crisis_attendant" => check_crisis_attendant(entry, lines),
        other => panic!("unknown aging category {other:?}"),
    }
}

#[test]
fn every_aging_entry_matches_its_own_source_passage() {
    let lines = source_lines();
    let doc = aging_json();
    let all = all_entries(&doc);

    let shipped_count = doc["living_conditions"]
        .as_array()
        .expect("living_conditions array")
        .len()
        + doc["outcomes"].as_array().expect("outcomes array").len()
        + doc["crisis"]["rows"]
            .as_array()
            .expect("crisis.rows array")
            .len()
        + usize::from(doc["crisis"].get("die").is_some())
        + usize::from(doc["crisis"].get("attendant").is_some());
    assert!(shipped_count > 0, "aging.json is empty — nothing to guard");
    assert_eq!(
        all.len(),
        shipped_count,
        "must check every living_condition/outcome/crisis row/die/attendant entry, never a subset"
    );

    let mut failures = Vec::new();
    for (category, id, entry) in &all {
        if exception_reason(id).is_some() {
            continue;
        }
        if let Err(problems) = check_by_category(category, entry, &lines) {
            failures.push(format!("{id}: {}", problems.join("; ")));
        }
    }
    assert!(
        failures.is_empty(),
        "{} aging.json entries disagree with their own source passage:\n{}",
        failures.len(),
        failures.join("\n")
    );
}

/// Every [`RULED_EXCEPTIONS`] row exists to silence the main guard for one
/// entry whose cited passage genuinely disagrees with the shipped data by a
/// recorded ruling. Empty today, so this loop is a no-op — it stays wired so
/// a future addition is covered immediately.
#[test]
fn ruled_exceptions_still_mismatch() {
    let lines = source_lines();
    let doc = aging_json();
    let all = all_entries(&doc);
    let mut stale = Vec::new();
    for (id, _) in RULED_EXCEPTIONS {
        let (category, _, entry) = all.iter().find(|(_, i, _)| i == id).unwrap_or_else(|| {
            panic!("RULED_EXCEPTIONS names \"{id}\", which is no longer in aging.json")
        });
        if check_by_category(category, entry, &lines).is_ok() {
            stale.push(*id);
        }
    }
    assert!(
        stale.is_empty(),
        "RULED_EXCEPTIONS row(s) {stale:?} now MATCH their own source passage — the defect they \
         recorded is gone, so remove the row(s)"
    );
}

/// Proves the `check_*` functions can actually fail, by corrupting one real
/// field at a time on a genuine catalogue entry and asserting the guard
/// rejects it.
#[test]
fn check_rejects_synthetic_wrong_entries() {
    let lines = source_lines();
    let doc = aging_json();
    let all = all_entries(&doc);

    let find = |id: &str| -> (&'static str, Value) {
        all.iter()
            .find(|(_, i, _)| i == id)
            .map(|(c, _, v)| (*c, v.clone()))
            .unwrap_or_else(|| panic!("{id} not found in aging.json"))
    };

    let (category, mut wrong_modifier) = find("living_condition.leper");
    wrong_modifier["modifier"] = serde_json::json!(2);
    assert!(
        check_by_category(category, &wrong_modifier, &lines).is_err(),
        "a deliberately wrong living_condition modifier must be rejected"
    );

    let (category, mut wrong_cumulative) = find("living_condition.average_peasant");
    wrong_cumulative["cumulative"] = serde_json::json!(true);
    assert!(
        check_by_category(category, &wrong_cumulative, &lines).is_err(),
        "a deliberately wrong cumulative flag must be rejected"
    );

    let (category, mut wrong_band) = find("aging.roll.14");
    wrong_band["min"] = serde_json::json!(99);
    assert!(
        check_by_category(category, &wrong_band, &lines).is_err(),
        "a deliberately wrong Aging Roll band must be rejected"
    );

    let (category, mut wrong_effect) = find("aging.roll.14");
    wrong_effect["effect"] = serde_json::json!({ "type": "any_characteristic", "points": 1 });
    assert!(
        check_by_category(category, &wrong_effect, &lines).is_err(),
        "a deliberately wrong Aging Roll effect must be rejected"
    );

    let (category, mut wrong_outcome) = find("crisis.minor_illness");
    wrong_outcome["outcome"]["ritual_level"] = serde_json::json!(99);
    assert!(
        check_by_category(category, &wrong_outcome, &lines).is_err(),
        "a deliberately wrong Crisis Roll outcome must be rejected"
    );

    let (category, mut wrong_die) = find("crisis.die");
    wrong_die["max"] = serde_json::json!(6);
    assert!(
        check_by_category(category, &wrong_die, &lines).is_err(),
        "a deliberately wrong crisis.die max must be rejected"
    );

    let (category, mut wrong_attendant) = find("crisis.attendant");
    wrong_attendant["botch_penalty"] = serde_json::json!(-99);
    assert!(
        check_by_category(category, &wrong_attendant, &lines).is_err(),
        "a deliberately wrong crisis.attendant botch_penalty must be rejected"
    );
}

/// Guards the guard: [`parse_band`] must handle every band shape the two
/// tables actually use, not just the common case.
#[test]
fn parse_band_handles_every_shape() {
    assert_eq!(parse_band("13"), Ok((Some(13), Some(13))));
    assert_eq!(parse_band("10\u{2013}12"), Ok((Some(10), Some(12))));
    assert_eq!(parse_band("22+"), Ok((Some(22), None)));
    assert_eq!(parse_band("8 or less"), Ok((None, Some(8))));
    assert!(parse_band("not a band").is_err());
}
