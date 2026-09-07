#!/usr/bin/env python3
"""Extract the Core Rules Hermetic spell catalogue into the engine's rules JSON.

DEV / BUILD TOOL ONLY. This script is never loaded at runtime by `arm-app` or
`arm-rules`; it regenerates hand-checkable data from the authoritative Markdown,
consistent with CLAUDE.md ("JSON is generated from the Markdown by extraction").

It parses the Spells chapter of
`rules/source/en/Ars Magica - Definitive Edition (Core Rules).md` and emits:

  * rules/core/spells.json       — language-neutral mechanics (id, technique,
                                    form, level, requisites, ritual, R/D/T,
                                    creates_lasting, parameter slots, source
                                    line-range)
  * rules/i18n/en/spells.json    — English name + description prose, keyed by id
  * rules/i18n/de/spells.json    — German names from the canonical translation
                                    table (zauber-nach-form.md); EN fallback
                                    (documented policy) when a spell is absent.
                                    Existing German `description` prose is
                                    *preserved*, never regenerated (see below).

Output is canonically sorted by id with stable, deterministic formatting so
re-runs produce zero-noise diffs. In-loop checks fail loudly; the real trust
gate is the engine's load-time referential-integrity + ritual-legality check.

Three properties of this generator are load-bearing and easy to break:

* **Tables are not description prose.** Where the rulebook interrupts a spell's
  body with a Markdown table (*Mists of Change*, *The Shadow of Life Renewed*,
  *Visions of the Infernal Terrors*), the rows are deliberately left out of the
  `description`. That field is prose rendered as prose by the UI, which cannot
  lay out a raw Markdown table; the authoritative tabular text stays in
  `rules/source/<lang>/`. The prose *around* the table is still collected, and
  the entry's `source` range still spans the table, so provenance is complete.
  `no_spell_description_carries_markdown_table` in
  `crates/arm-rules/tests/data_integrity.rs` is the witness.
* **Rewrites are non-destructive.** This script cannot author German prose, so
  it carries every existing German `description` (and any other hand-authored
  per-id field) across a rewrite. Before writing, it audits the regenerated
  content against the files on disk and aborts if any field or id would be
  lost, rather than silently deleting curated data.
* **Bounds are asserted, loudly.** `CHAPTER_START`/`CHAPTER_END` are checked
  against the actual heading text in `main()`; a shifted source aborts the run
  and names the line it found.
"""

from __future__ import annotations

import json
import re
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
CORE_FILE = "Ars Magica - Definitive Edition (Core Rules).md"
SOURCE = REPO / "rules" / "source" / "en" / CORE_FILE
DE_TABLE = REPO / "rules" / "source" / "de" / "translation-tables" / "zauber-nach-form.md"
OUT_CORE = REPO / "rules" / "core" / "spells.json"
OUT_I18N_EN = REPO / "rules" / "i18n" / "en" / "spells.json"
OUT_I18N_DE = REPO / "rules" / "i18n" / "de" / "spells.json"

# The Spells chapter runs from the first "<Te> <Fo> Guidelines/Spells" section
# to the start of Chapter 10 (Long Term Events). Bounds are asserted below.
CHAPTER_START = 12385  # "## Animal Spells"
CHAPTER_END = 15941  # last line before "# Chapter 10: Long Term Events" (15942)

TECHNIQUES = {"Creo", "Intellego", "Muto", "Perdo", "Rego"}
FORMS = {
    "Animal", "Aquam", "Auram", "Corpus", "Herbam", "Ignem",
    "Imaginem", "Mentem", "Terram", "Vim",
}

# Key a section purely on "### <Technique> <Form>": the trailing word is normally
# "Guidelines" or "Spells", but the source has at least one typo ("Svells"), and
# spells sometimes live directly under a "Guidelines" header (no separate "Spells"
# header). Because <Form> is drawn from the closed set of 10 Forms, "### Te Fo …"
# is unambiguous.
# Normally "### Te Fo …" but the source also has a stray "## Creo Mentem Spells"
# (two hashes), so accept 2-3 hashes. This is checked before the generic h2/h1
# clear branch so a "## Te Fo …" heading opens (not closes) a section.
SECTION_RE = re.compile(
    r"^#{2,3} (%s) (%s)\b"
    % ("|".join(sorted(TECHNIQUES)), "|".join(sorted(FORMS)))
)
LEVEL_RE = re.compile(r"^#### LEVEL (\d+)$")
GENERAL_RE = re.compile(r"^#### GENERAL$")
SPELL_RE = re.compile(r"^##### (.+?)\s*$")
# R/D/T line: separators may be comma or period; a trailing ", Ritual" flags it.
# The R/D/T line always carries the markers "R:", "D:", "T:"; field text between
# them is free-form (stray periods, missing commas, or compound durations like
# "Sun & Year"), so we split on the markers and token-scan each field.
RDT_MARKERS_RE = re.compile(r"^R:(.*?)\bD:(.*?)\bT:(.*)$")
REQ_RE = re.compile(r"^Req:\s*(.+?)\s*(?:<br>)?\s*$")

RANGE_MAP = {
    "per": "personal", "personal": "personal",
    "touch": "touch",
    "eye": "eye", "eve": "eye",  # "Eve" is a source typo for Eye (one occurrence)
    "voice": "voice",
    "sight": "sight",
    "arc": "arcane_connection", "arcane": "arcane_connection",
}
DURATION_MAP = {
    "mom": "momentary", "momentary": "momentary",
    "conc": "concentration", "concentration": "concentration",
    "diam": "diameter", "diameter": "diameter",
    "sun": "sun",
    "ring": "ring",
    "moon": "moon",
    "year": "year",
    "spec": None, "special": None,  # Special duration: no enum scalar -> None
}
TARGET_MAP = {
    "ind": "individual", "individual": "individual",
    "circle": "circle",
    "part": "part",
    "group": "group",
    "room": "room",
    "str": "structure", "struct": "structure", "structure": "structure",
    "bound": "boundary", "boundary": "boundary",
    "taste": "taste",
    "touch": "touch",
    "smell": "smell",
    "hearing": "hearing", "hear": "hearing",
    "vision": "vision",
    "special": None, "spec": None,  # Special target: no enum scalar -> None
}
ART_NAMES = TECHNIQUES | FORMS

# A parenthetical in a spell name that is a bare Art-*class* word — "(Form)",
# "(form)", "(Technique)" — is not literal text: it marks a parameter slot the
# character sheet fills in with one concrete Art. There are ten *Wizard's Boost*
# spells, one per Form, and the catalogue carries a single entry with a `form`
# slot. The class words are exactly the `art_type` values in
# rules/core/arts.json, spelled per language: the German translation table
# writes the same marker in German ("Das (Form)-Gefüge auflösen").
SLOT_WORDS = {
    "form": "form",  # EN + DE (identical spelling)
    "technique": "technique",  # EN
    "technik": "technique",  # DE
}
SLOT_RE = re.compile(r"\(([A-Za-z]+)\)")


class ExtractError(Exception):
    pass


def parameter_slots(name: str):
    """Return (parameters, display_name) for a spell name with Art-class slots.

    The parenthetical is replaced by a `{key}` placeholder *inside* the
    parentheses ("Wizard's Boost ({form})"), which is the shape the i18n files
    ship and the UI substitutes into. A name with no Art-class parenthetical is
    returned unchanged with no parameters, so this is a no-op for the other 356
    spells. Deliberately narrow: only the closed `SLOT_WORDS` set qualifies, so
    an ordinary parenthetical in a spell name is never rewritten.
    """
    params: list[dict[str, str]] = []

    def substitute(match: re.Match) -> str:
        key = SLOT_WORDS.get(match.group(1).lower())
        if key is None:
            return match.group(0)
        if all(p["key"] != key for p in params):
            params.append({"key": key, "type": "ref", "domain": key})
        return "({%s})" % key

    return params, SLOT_RE.sub(substitute, name)


def snake_id(name: str) -> str:
    s = name.lower().replace("’", "").replace("'", "")
    s = re.sub(r"[^a-z0-9]+", "_", s).strip("_")
    return "spell." + s


def art_id(name: str) -> str:
    return "art." + name.strip().lower()


def strip_br(line: str) -> str:
    return re.sub(r"\s*<br>\s*$", "", line).rstrip()


def _words(field: str):
    return [w for w in re.split(r"[^A-Za-z]+", field) if w]


def _first_mapped(words, mapping, field_name, line):
    """First word that is a key in `mapping` (value may be None for Special)."""
    for w in words:
        if w.lower() in mapping:
            return mapping[w.lower()]
    raise ExtractError(f"no known {field_name} token in {words!r} ({line!r})")


def parse_rdt(line: str):
    """Return (range, duration, target, ritual) scalars for an R:/D:/T: line."""
    m = RDT_MARKERS_RE.match(strip_br(line))
    if not m:
        raise ExtractError(f"unparseable R/D/T line: {line!r}")
    rfield, dfield, tfield = m.group(1), m.group(2), m.group(3)
    ritual = "ritual" in line.lower()
    rng = _first_mapped(_words(rfield), RANGE_MAP, "Range", line)
    dwords = _words(dfield)
    # Compound duration (e.g. "Sun & Year"): the ritual-forcing Year dominates.
    if any(w.lower() == "year" for w in dwords):
        dur = "year"
    else:
        dur = _first_mapped(dwords, DURATION_MAP, "Duration", line)
    tgt = _first_mapped([w for w in _words(tfield) if w.lower() != "ritual"],
                        TARGET_MAP, "Target", line)
    return rng, dur, tgt, ritual


def parse_requisites(text: str):
    reqs = []
    for part in text.split(","):
        name = part.strip()
        # keep only leading art word; ignore any parenthetical notes
        word = name.split()[0] if name.split() else ""
        if word in ART_NAMES:
            reqs.append(art_id(word))
    return reqs


def extract(lines):
    spells = []
    i18n_en = {}
    te = fo = None
    level = None
    level_set = False
    n = len(lines)
    idx = 0
    while idx < n:
        lineno = idx + 1  # 1-based
        raw = lines[idx]
        line = raw.rstrip("\n").rstrip("\r")  # source has mixed CRLF/LF regions

        if lineno < CHAPTER_START or lineno > CHAPTER_END:
            idx += 1
            continue

        sec = SECTION_RE.match(line)
        if sec:
            te, fo = art_id(sec.group(1)), art_id(sec.group(2))
            level_set = False
            idx += 1
            continue
        if line.startswith("## ") or line.startswith("# "):
            te = fo = None
            level_set = False
            idx += 1
            continue
        if line.startswith("### "):
            # a non-spell "###" section ends the current Te/Fo context
            te = fo = None
            level_set = False
            idx += 1
            continue

        m = LEVEL_RE.match(line)
        if m:
            level = int(m.group(1))
            level_set = True
            idx += 1
            continue
        if GENERAL_RE.match(line):
            level = None
            level_set = True
            idx += 1
            continue

        sm = SPELL_RE.match(line)
        if sm and te and fo:
            name = sm.group(1).strip()
            start = lineno
            # find the next non-blank content line: must be an R:/D:/T: line
            j = idx + 1
            while j < n and lines[j].strip() == "":
                j += 1
            if j >= n or not lines[j].lstrip().startswith("R:"):
                idx += 1
                continue  # a heading that is not a spell entry
            if not level_set:
                raise ExtractError(f"spell {name!r} at line {lineno} has no LEVEL/GENERAL context")
            rng, dur, tgt, ritual = parse_rdt(lines[j].rstrip("\n").strip())
            k = j + 1
            requisites = []
            # optional Req: line(s)
            while k < n and lines[k].strip().startswith("Req:"):
                rq = REQ_RE.match(lines[k].strip())
                if rq:
                    requisites.extend(parse_requisites(rq.group(1)))
                k += 1
            # description prose: paragraphs until the "(Base ...)" design line or
            # the next heading. A Markdown table inside the body is skipped for
            # the description but still counted in the source range — see the
            # module docstring: the description is prose the UI renders as
            # prose, and a raw table cannot be rendered there, so the
            # authoritative tabular text stays in the Markdown source.
            desc_paras = []
            end = j  # track last content line for source range
            while k < n:
                cl = lines[k].rstrip("\n")
                stripped = cl.strip()
                if stripped == "":
                    k += 1
                    continue
                if stripped.startswith("#"):
                    break
                if stripped.startswith("("):
                    end = k + 1
                    break
                if stripped.startswith("|"):
                    # A table row: excluded from the prose, included in the
                    # source range. Collection continues rather than stopping,
                    # because prose can resume after the table (Mists of Change
                    # closes with a full paragraph below its table).
                    end = k + 1
                    k += 1
                    continue
                desc_paras.append(strip_br(stripped))
                end = k + 1
                k += 1
            spell_id = snake_id(name)
            parameters, display_name = parameter_slots(name)
            requisites = sorted(set(requisites))
            creates_lasting = ritual and te == "art.creo" and dur == "momentary"
            rec = {
                "id": spell_id,
                "technique": te,
                "form": fo,
                "level": level,
                "requisites": requisites,
                "ritual": ritual,
                "range": rng,
                "duration": dur,
                "target": tgt,
                "creates_lasting": creates_lasting,
                "parameters": parameters,
                "source_lines": [start, end],
            }
            spells.append(rec)
            desc = "\n\n".join(p for p in desc_paras if p).strip()
            entry = {"name": display_name}
            if desc:
                entry["description"] = desc
            i18n_en[spell_id] = entry
            idx = k
            continue

        idx += 1
    return spells, i18n_en


# ---------------------------------------------------------------------------
# In-loop checks
# ---------------------------------------------------------------------------

def load_art_types():
    arts = json.loads((REPO / "rules" / "core" / "arts.json").read_text())["arts"]
    return {a["id"]: a["art_type"] for a in arts}


def check(spells, art_types):
    errors = []
    seen = {}
    for s in spells:
        sid = s["id"]
        if sid in seen:
            errors.append(f"duplicate spell id {sid} (also at lines {seen[sid]})")
        seen[sid] = s["source_lines"]
        if art_types.get(s["technique"]) != "technique":
            errors.append(f"{sid}: technique {s['technique']} is not a Technique-class Art")
        if art_types.get(s["form"]) != "form":
            errors.append(f"{sid}: form {s['form']} is not a Form-class Art")
        for r in s["requisites"]:
            if r not in art_types:
                errors.append(f"{sid}: requisite {r} is not a known art")
        lvl = s["level"]
        if lvl is not None:
            if lvl < 1:
                errors.append(f"{sid}: non-positive level {lvl}")
            # levels below 5 exist individually (2,3,4); otherwise multiples of 5
            if lvl > 4 and lvl % 5 != 0:
                errors.append(f"{sid}: level {lvl} is neither <5 nor a multiple of 5")
            if s["ritual"] and lvl < 20:
                errors.append(f"{sid}: ritual below level 20 ({lvl})")
            if not s["ritual"] and lvl > 50:
                errors.append(f"{sid}: non-ritual above level 50 ({lvl})")
        if not s["ritual"]:
            if s["duration"] == "year":
                errors.append(f"{sid}: Year duration requires ritual")
            if s["target"] == "boundary":
                errors.append(f"{sid}: Boundary target requires ritual")
            if s["technique"] == "art.creo" and s["duration"] == "momentary" and s["creates_lasting"]:
                errors.append(f"{sid}: Momentary Creo creating lasting requires ritual")
        a, b = s["source_lines"]
        if not (CHAPTER_START <= a <= b <= CHAPTER_END + 1):
            errors.append(f"{sid}: source range {s['source_lines']} out of chapter bounds")
    return errors


# ---------------------------------------------------------------------------
# DE translation table
# ---------------------------------------------------------------------------

# The German table's English column occasionally differs from the book's spelling
# (a typo or a British/American variant). These map the book's normalized key to
# the table's normalized key so we still honour the canonical German name rather
# than falling back to English.
DE_KEY_ALIASES = {
    # book "Thread the Earth" vs table "Tread the Earth" (source typo)
    "sense the feet that thread the earth": "sense the feet that tread the earth",
    # book "Unravelling" (British) vs table "Unraveling" (American)
    "unravelling the fabric of form": "unraveling the fabric of form",
}


def de_match_key(name: str) -> str:
    s = name.lower().replace("’", "").replace("'", "")
    for art in (", the", ", a", ", an"):
        if s.endswith(art):
            s = s[: -len(art)]
            break
    for art in ("the ", "a ", "an "):
        if s.startswith(art):
            s = s[len(art):]
            break
    return re.sub(r"[^a-z0-9]+", " ", s).strip()


def load_de_table():
    table = {}
    for line in DE_TABLE.read_text().splitlines():
        line = line.strip()
        if not line.startswith("|"):
            continue
        cells = [c.strip() for c in line.strip("|").split("|")]
        if len(cells) < 2:
            continue
        en, de = cells[0], cells[1]
        if en in ("Englisch (EN)", "") or set(en) <= {"-"}:
            continue
        if not de:
            continue
        key = de_match_key(en)
        table.setdefault(key, de)
    return table


# ---------------------------------------------------------------------------
# Canonical writers
# ---------------------------------------------------------------------------

def write_core(spells):
    spells = sorted(spells, key=lambda s: s["id"])
    out = ["{", '  "spells": [']
    for n, s in enumerate(spells):
        parts = [
            f'"id": {json.dumps(s["id"])}',
            f'"technique": {json.dumps(s["technique"])}',
            f'"form": {json.dumps(s["form"])}',
        ]
        if s["level"] is not None:
            parts.append(f'"level": {s["level"]}')
        if s["requisites"]:
            parts.append(f'"requisites": {json.dumps(s["requisites"])}')
        if s["ritual"]:
            parts.append('"ritual": true')
        if s["range"] is not None:
            parts.append(f'"range": {json.dumps(s["range"])}')
        if s["duration"] is not None:
            parts.append(f'"duration": {json.dumps(s["duration"])}')
        if s["target"] is not None:
            parts.append(f'"target": {json.dumps(s["target"])}')
        if s["creates_lasting"]:
            parts.append('"creates_lasting": true')
        if s["parameters"]:
            slots = ", ".join(
                "{ "
                + ", ".join(f"{json.dumps(pk)}: {json.dumps(pv)}" for pk, pv in p.items())
                + " }"
                for p in s["parameters"]
            )
            parts.append(f'"parameters": [{slots}]')
        a, b = s["source_lines"]
        parts.append(
            f'"source": {{ "file": {json.dumps(CORE_FILE)}, "lines": [{a}, {b}] }}'
        )
        comma = "," if n < len(spells) - 1 else ""
        out.append("    { " + ", ".join(parts) + " }" + comma)
    out.append("  ]")
    out.append("}")
    return "\n".join(out) + "\n"


def write_i18n(entries):
    keys = sorted(entries)
    out = ["{"]
    for n, k in enumerate(keys):
        comma = "," if n < len(keys) - 1 else ""
        val = json.dumps(entries[k], ensure_ascii=False)
        out.append(f"  {json.dumps(k)}: {val}{comma}")
    out.append("}")
    return "\n".join(out) + "\n"


def load_existing(path: Path):
    """Parse an output file already on disk, so a rewrite can preserve it."""
    if not path.exists():
        return {}
    return json.loads(path.read_text())


def by_id(doc):
    """Normalize either output shape to an id -> entry map for the drop audit."""
    if isinstance(doc, dict) and "spells" in doc:
        return {s["id"]: s for s in doc["spells"]}
    return doc or {}


def dropped_data(existing, new_text: str):
    """Fields/ids present on disk that the regenerated content would delete.

    Compares against the *serialized* output (re-parsed), so it audits exactly
    the bytes about to be written — including fields the writer omits when
    falsy — rather than the in-memory records.
    """
    old = by_id(existing)
    new = by_id(json.loads(new_text))
    lost = []
    for sid in sorted(old):
        entry = old[sid]
        if not isinstance(entry, dict):
            continue
        if sid not in new:
            lost.append(f"{sid}: whole entry")
            continue
        for field in entry:
            if field not in new[sid]:
                lost.append(f"{sid}.{field}")
    return lost


def assert_chapter_bounds(lines):
    """Fail loudly if the Spells chapter has moved in the source Markdown."""
    for lineno, expected in ((CHAPTER_START, "## Animal Spells"),
                             (CHAPTER_END + 1, "# Chapter 10")):
        found = lines[lineno - 1].rstrip()
        if not found.startswith(expected):
            raise ExtractError(
                f"chapter bound moved: {CORE_FILE} line {lineno} should start with "
                f"{expected!r} but is {found!r} — re-locate CHAPTER_START/CHAPTER_END "
                "against the source before regenerating"
            )


def main():
    lines = SOURCE.read_text().splitlines(keepends=True)
    assert_chapter_bounds(lines)
    spells, i18n_en = extract(lines)
    art_types = load_art_types()
    errors = check(spells, art_types)
    if errors:
        print("EXTRACTION CHECKS FAILED:", file=sys.stderr)
        for e in errors:
            print("  - " + e, file=sys.stderr)
        sys.exit(1)

    # DE i18n. Names come from the canonical translation table; German
    # `description` prose is hand-authored and CANNOT be regenerated, so it is
    # carried across from the file on disk (as is any other hand-added field).
    de_table = load_de_table()
    existing_de = load_existing(OUT_I18N_DE)
    i18n_de = {}
    de_fallbacks = []
    de_new_ids = []
    for sid, entry in i18n_en.items():
        # de_match_key strips every non-alphanumeric run, so the "({form})"
        # placeholder in the display name normalizes to the same key as the
        # table's "(Form)" marker.
        key = de_match_key(entry["name"])
        key = DE_KEY_ALIASES.get(key, key)
        de = de_table.get(key)
        if de:
            _, de_name = parameter_slots(de)
            i18n_de[sid] = {"name": de_name}
        else:
            i18n_de[sid] = {"name": entry["name"]}  # documented EN fallback
            de_fallbacks.append((sid, entry["name"]))
        preserved = {k: v for k, v in existing_de.get(sid, {}).items() if k != "name"}
        i18n_de[sid].update(preserved)
        if sid not in existing_de:
            de_new_ids.append((sid, entry["name"]))

    core_text = write_core(spells)
    en_text = write_i18n(i18n_en)
    de_text = write_i18n(i18n_de)

    # Non-destructive gate: never overwrite curated data with less data.
    losses = []
    for path, new_text in ((OUT_CORE, core_text),
                           (OUT_I18N_EN, en_text),
                           (OUT_I18N_DE, de_text)):
        for item in dropped_data(load_existing(path), new_text):
            losses.append(f"{path.relative_to(REPO)}: {item}")
    if losses:
        print("REWRITE WOULD DELETE EXISTING DATA — nothing written:", file=sys.stderr)
        for item in losses:
            print("  - " + item, file=sys.stderr)
        print("Teach the generator to produce or preserve these, or remove the "
              "stale entries deliberately.", file=sys.stderr)
        sys.exit(1)

    OUT_CORE.write_text(core_text)
    # EN i18n: names + description
    OUT_I18N_EN.write_text(en_text)
    OUT_I18N_DE.write_text(de_text)

    # -------- report --------
    from collections import Counter
    by_te = Counter(s["technique"].split(".")[1] for s in spells)
    by_fo = Counter(s["form"].split(".")[1] for s in spells)
    rituals = sum(1 for s in spells if s["ritual"])
    print(f"Extracted {len(spells)} spells ({rituals} rituals).")
    print("Per Technique: " + ", ".join(f"{k}={by_te[k]}" for k in sorted(by_te)))
    print("Per Form:      " + ", ".join(f"{k}={by_fo[k]}" for k in sorted(by_fo)))
    print(f"DE coverage: {len(spells) - len(de_fallbacks)}/{len(spells)} matched; "
          f"{len(de_fallbacks)} EN fallbacks.")
    if de_fallbacks:
        print("EN fallbacks (no DE table entry):")
        for sid, name in de_fallbacks:
            print(f"  - {sid}  ({name})")
    if de_new_ids:
        print(f"New spell ids ({len(de_new_ids)}): written name-only in "
              "rules/i18n/de/spells.json — German description prose is "
              "hand-authored and must be added by a translator:")
        for sid, name in de_new_ids:
            print(f"  - {sid}  ({name})")


if __name__ == "__main__":
    main()
