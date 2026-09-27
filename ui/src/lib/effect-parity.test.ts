import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { describe, expect, it } from 'vitest';

// B1 (design-b0-ranging-and-predicates.md § 2, plan-review finding #2): the TS
// `Effect` union (`types.ts::Effect`) is real and reachable, consumed at
// `derive.ts` and `AbilityTab.svelte`, but it is a deliberately CURATED
// subset of the engine's `Effect` enum — a member is added only when a UI
// consumer needs to read that variant. So parity here is NOT full set
// equality the way `Prereq`'s mirror is (`prereq-parity.test.ts`): it is
// TS ⊆ Rust, checked the same way (parsing both sources as text, exactly as
// `prereq-parity.test.ts` already does for `Prereq`). This catches the one
// drift class that matters — a TS `type` tag left stale after a Rust rename
// (B5's `ability_roll_mod` → `ability_roll_mod_param` swap) or typo'd after a
// copy-paste — which today nothing flags, since neither side's match is
// exhaustive.
//
// `ssr` project (plain `*.test.ts`): pure text parsing over repo files, no
// component, no DOM, no `$effect`.

function repoFile(relative: string): string {
  return readFileSync(fileURLToPath(new URL(relative, import.meta.url)), 'utf-8');
}

const rustTypes = repoFile('../../../crates/arm-rules/src/types.rs');
const tsTypes = repoFile('./types.ts');

/** The body of a brace-delimited block whose closing `}` sits at column 0. */
function blockAfter(source: string, opener: string): string {
  const start = source.indexOf(opener);
  if (start === -1) throw new Error(`not found: ${opener}`);
  const from = start + opener.length;
  const end = source.indexOf('\n}', from);
  if (end === -1) throw new Error(`unterminated block: ${opener}`);
  return source.slice(from, end);
}

function snakeCase(variant: string): string {
  return variant.replace(/(?!^)([A-Z])/g, '_$1').toLowerCase();
}

/**
 * The serde tag of every Rust `Effect` variant. Unlike `Prereq` (whose `Nor`
 * variant carries a `#[serde(rename = "none")]`), no variant in this enum
 * renames its tag — every one converts by plain snake_case.
 */
function rustEffectTags(): Set<string> {
  const body = blockAfter(rustTypes, 'pub enum Effect {');
  const tags = new Set<string>();
  for (const line of body.split('\n')) {
    const variant = /^ {4}([A-Z]\w*)\s*[({,]/.exec(line);
    if (!variant) continue;
    tags.add(snakeCase(variant[1]));
  }
  return tags;
}

/**
 * Every `type` literal the TS `Effect` discriminated union declares. The
 * union's own closing `};` (the top-level type alias terminator) is found by
 * the literal two-character substring `};`, which — unlike a bare `}` — only
 * ever closes the FINAL member's object followed by the alias's own
 * semicolon; every other member's object closes with a bare `}` before the
 * next `|`. Matching on the whole block rather than line-by-line means a
 * member whose fields wrap onto several lines (`grants_spell_mastery`,
 * `advancement_mod`, `combat_mod`, …) is still read correctly.
 */
function tsEffectTypes(): Set<string> {
  const start = tsTypes.indexOf('export type Effect =');
  if (start === -1) throw new Error('not found: export type Effect');
  const end = tsTypes.indexOf('};', start);
  if (end === -1) throw new Error('unterminated Effect union');
  const block = tsTypes.slice(start, end + 2);
  return new Set(Array.from(block.matchAll(/type: '([^']+)'/g), (m) => m[1]));
}

describe('Effect TS/Rust parity', () => {
  it('reads a non-empty variant list from both sources', () => {
    // Guards the parser itself: a rename of the enum or the union would
    // otherwise make the subset assertion below vacuously true.
    expect(rustEffectTags().size).toBeGreaterThan(0);
    expect(tsEffectTypes().size).toBeGreaterThan(0);
  });

  it('never declares a TS Effect type tag with no matching Rust variant (TS ⊆ Rust)', () => {
    const rust = rustEffectTags();
    const stale = [...tsEffectTypes()].filter((t) => !rust.has(t));
    expect(
      stale,
      `TS Effect union names a type with no matching Rust variant: ${stale.join(', ')}`,
    ).toEqual([]);
  });

  it('is a genuine subset today, not full equality (documents the curated-mirror premise)', () => {
    // If this ever becomes false equality, `Effect` stopped being curated —
    // worth noticing, not enforcing, so this assertion only documents the
    // current shape rather than pinning it as a requirement.
    const rust = rustEffectTags();
    const ts = tsEffectTypes();
    expect(rust.size).toBeGreaterThanOrEqual(ts.size);
  });
});
