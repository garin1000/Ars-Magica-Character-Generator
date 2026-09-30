import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { describe, expect, it } from 'vitest';

// Review 2026-09-30b #1: the TS `ItemPredicate` union is a hand-maintained
// mirror of the engine's `ItemPredicate` enum (`types.rs`), the same relationship
// `prereq-parity.test.ts` already guards for `Prereq`. The engine enum is closed
// and serde-checked (its own doc comment says so); the TS union drifted behind it
// once before (`requires_hermetic_arts`/`affects_size` shipped in Rust with no TS
// counterpart), which is exactly the failure mode this test exists to catch
// mechanically instead of by review alone.
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

/** Every `ItemPredicate` variant, in the engine's own snake_case serde tag. */
function rustItemPredicateTags(): Set<string> {
  const body = blockAfter(rustTypes, 'pub enum ItemPredicate {');
  const tags = new Set<string>();
  for (const line of body.split('\n')) {
    const variant = /^ {4}([A-Z]\w*),/.exec(line);
    if (variant) tags.add(snakeCase(variant[1]));
  }
  return tags;
}

/**
 * Every string literal in the TS `ItemPredicate` union — spans one or more
 * `|`-prefixed lines following the alias head, terminated by the first line
 * that ends the statement with `;`.
 */
function tsItemPredicateKinds(): Set<string> {
  const lines = tsTypes.split('\n');
  const head = lines.findIndex((l) => l.startsWith('export type ItemPredicate ='));
  if (head === -1) throw new Error('not found: export type ItemPredicate');
  const kinds = new Set<string>();
  for (const line of lines.slice(head)) {
    for (const m of line.matchAll(/'([^']+)'/g)) kinds.add(m[1]);
    if (line.trim().endsWith(';')) break;
  }
  return kinds;
}

describe('ItemPredicate TS/Rust parity', () => {
  it('reads a non-empty variant list from both sources', () => {
    // Guards the parser itself: a rename of the enum or the union would
    // otherwise make every assertion below vacuously true.
    expect(rustItemPredicateTags().size).toBeGreaterThan(0);
    expect(tsItemPredicateKinds().size).toBe(rustItemPredicateTags().size);
  });

  it('declares the same set of predicates in types.ts as the engine serializes', () => {
    expect([...tsItemPredicateKinds()].sort()).toEqual([...rustItemPredicateTags()].sort());
  });
});
