import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { describe, expect, it } from 'vitest';

import { PREREQ_MAX_DEPTH } from './derive';

// V1 (full-audit round 2): the TS `Prereq` union and `houseOnlyValue` are a
// hand-maintained mirror of the engine's `Prereq` enum. CLAUDE.md declares that
// enum load-bearing *because* adding a variant must be a compile error until
// every site handles it — Rust gets that from its exhaustive `match`, and the
// TS side now gets it from an explicit arm list plus a `never` guard. That guard
// only fires once the new kind reaches the TS union, so this file is the link
// that forces it there: it diffs the Rust source against the TS source as text,
// the same technique `crates/arm-app/tests/menu.rs`'s
// `the_frontend_and_rust_agree_on_the_menu_contract` already uses for the menu
// contract.
//
// `ssr` project (plain `*.test.ts`): pure text parsing over repo files, no
// component, no DOM, no `$effect`.

function repoFile(relative: string): string {
  return readFileSync(fileURLToPath(new URL(relative, import.meta.url)), 'utf-8');
}

const rustTypes = repoFile('../../../crates/arm-rules/src/types.rs');
const tsTypes = repoFile('./types.ts');
const deriveSource = repoFile('./derive.ts');

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
 * The serde tag of every `Prereq` variant, read from the engine's own source:
 * each variant's name in snake_case, unless `#[serde(rename = "...")]` overrides
 * it (as it does for `Nor` → `"none"`, a locked wire contract).
 */
function rustPrereqTags(): Set<string> {
  const body = blockAfter(rustTypes, 'pub enum Prereq {');
  const tags = new Set<string>();
  let rename: string | null = null;
  for (const line of body.split('\n')) {
    const renamed = /^\s*#\[serde\(rename = "([^"]+)"\)\]/.exec(line);
    if (renamed) {
      rename = renamed[1];
      continue;
    }
    const variant = /^ {4}([A-Z]\w*)\s*[({,]/.exec(line);
    if (!variant) continue;
    tags.add(rename ?? snakeCase(variant[1]));
    rename = null;
  }
  return tags;
}

/**
 * Every `kind` literal in the TS `Prereq` discriminated union — the run of
 * `| { … }` member lines following the alias head. (The union cannot be sliced
 * at its terminating `;`: each member carries one of its own, between `kind` and
 * `value`.)
 */
function tsPrereqKinds(): Set<string> {
  const lines = tsTypes.split('\n');
  const head = lines.findIndex((line) => line.startsWith('export type Prereq ='));
  if (head === -1) throw new Error('not found: export type Prereq');
  const kinds = new Set<string>();
  for (const line of lines.slice(head + 1)) {
    if (!/^\s*\|/.test(line)) break;
    const kind = /kind: '([^']+)'/.exec(line);
    if (kind) kinds.add(kind[1]);
  }
  return kinds;
}

/** Every `case '<kind>':` label inside `houseOnlyValue`'s switch. */
function houseOnlyValueCases(): Set<string> {
  const body = blockAfter(deriveSource, 'function houseOnlyValue(');
  return new Set(Array.from(body.matchAll(/case '([^']+)':/g), (m) => m[1]));
}

describe('Prereq TS/Rust parity', () => {
  it('reads a non-empty variant list from both sources', () => {
    // Guards the parser itself: a rename of the enum or the union would
    // otherwise make every assertion below vacuously true.
    expect(rustPrereqTags().size).toBeGreaterThan(0);
    expect(tsPrereqKinds().size).toBe(rustPrereqTags().size);
  });

  it('declares the same set of kinds in types.ts as the engine serializes', () => {
    expect([...tsPrereqKinds()].sort()).toEqual([...rustPrereqTags()].sort());
  });

  it('names every Prereq kind explicitly in houseOnlyValue, with no catch-all', () => {
    // The `default:` arm this replaces silently absorbed any kind the function
    // was not written for, which for an open-grant menu — the only House guard
    // there is — means silently ceasing to guard.
    expect([...houseOnlyValueCases()].sort()).toEqual([...tsPrereqKinds()].sort());
    // A `default:` LABEL, not the word in a comment: the arms above explain
    // themselves by naming the one they replaced.
    expect(blockAfter(deriveSource, 'function houseOnlyValue(')).not.toMatch(/^\s*default:/m);
  });

  // I1: the requirement text a prerequisite finding and the V/F picker show is a
  // second exhaustive walk over the same union. A kind it does not name would
  // fall through to no text at all — a requirement silently missing from the
  // very sentence that exists to state it.
  it('names every Prereq kind explicitly in describePrereqAt, with no catch-all', () => {
    const body = blockAfter(deriveSource, 'function describePrereqAt(');
    const cases = new Set(Array.from(body.matchAll(/case '([^']+)':/g), (m) => m[1]));
    expect([...cases].sort()).toEqual([...tsPrereqKinds()].sort());
    expect(body).not.toMatch(/^\s*default:/m);
  });

  it('caps recursion at the same depth the engine does', () => {
    // The second literal this mirror duplicates. The engine rejects a deeper
    // tree at load; the UI copy is defence in depth, and is only defence at all
    // while the two numbers agree.
    const rustLimit = /pub const PREREQ_MAX_DEPTH: usize = (\d+);/.exec(rustTypes);
    expect(rustLimit).not.toBeNull();
    expect(PREREQ_MAX_DEPTH).toBe(Number(rustLimit?.[1]));
  });
});
