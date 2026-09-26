import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { describe, expect, it } from 'vitest';

// C3 (D35, `docs/vf-audit/design-c0-parameter-model.md` § 10.1): the first
// slice where the TS mirror of an engine-owned taxonomy must learn a
// genuinely new *kind*, not just widen a value type (C0b's `Selection.params`
// change did not need this). `ParamType`/`ParameterDomain` are exactly the
// closed, engine-owned enums CLAUDE.md's architecture invariant names ("the
// engine surfaces them... so there is a single source of truth"), so this
// mirrors `prereq-parity.test.ts`'s technique: diff the Rust source against
// the TS source as text, so a future Rust variant is a missing TS member here
// rather than a silently-stale mirror.
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
 * Every variant name declared in a Rust enum block, in snake_case — the tag
 * an externally-tagged `#[serde(rename_all = "snake_case")]` enum serializes
 * as, for BOTH a unit variant (`Ref` -> `"ref"`) and a struct variant
 * (`Number { min, max }` -> `{ "number": { "min": ..., "max": ... } }`, whose
 * TAG is still `"number"`). No renamed variant exists on either enum today,
 * so no rename-override handling is needed (unlike `Prereq`'s `Nor` -> `"none"`).
 */
function rustEnumTags(enumHead: string): Set<string> {
  const body = blockAfter(rustTypes, enumHead);
  const tags = new Set<string>();
  for (const line of body.split('\n')) {
    const variant = /^ {4}([A-Z]\w*)\s*[({,]/.exec(line);
    if (variant) tags.add(snakeCase(variant[1]));
  }
  return tags;
}

/**
 * Every member of a TS union type declared as `export type <Name> = ...`,
 * reading the run of `| ...` lines that follows — tolerating the doc-comment
 * lines this file interleaves between members (e.g. `ParameterDomain`'s
 * `category`/`realm` each carry one). A plain string-literal member
 * (`| 'ability'`) contributes its literal; an object-shaped member
 * (`| { number: { ... } }`, `ParamType::Number`'s externally-tagged wire
 * form) contributes its one key — the same tag Rust's own serialization
 * would produce for that struct variant. The scan stops at the first line
 * that is neither a member nor a comment (the terminating `;` always sits on
 * a member's own line, never alone).
 */
function tsUnionTags(head: string): Set<string> {
  const lines = tsTypes.split('\n');
  const headIdx = lines.findIndex((line) => line.startsWith(head));
  if (headIdx === -1) throw new Error(`not found: ${head}`);
  const tags = new Set<string>();
  for (const line of lines.slice(headIdx + 1)) {
    const trimmed = line.trim();
    if (trimmed === '' || trimmed.startsWith('//')) continue;
    if (!trimmed.startsWith('|')) break;
    const literal = /^\|\s*'([^']+)'/.exec(trimmed);
    if (literal) {
      tags.add(literal[1]);
      continue;
    }
    const objectKey = /^\|\s*\{\s*([a-zA-Z_]+)\s*:/.exec(trimmed);
    if (objectKey) tags.add(objectKey[1]);
  }
  return tags;
}

describe('ParamType TS/Rust parity', () => {
  it('reads a non-empty variant list from both sources', () => {
    expect(rustEnumTags('pub enum ParamType {').size).toBeGreaterThan(0);
    expect(tsUnionTags('export type ParamType =').size).toBe(
      rustEnumTags('pub enum ParamType {').size,
    );
  });

  it('declares the same set of tags in types.ts as the engine serializes', () => {
    expect([...tsUnionTags('export type ParamType =')].sort()).toEqual(
      [...rustEnumTags('pub enum ParamType {')].sort(),
    );
  });
});

describe('ParameterDomain TS/Rust parity', () => {
  it('reads a non-empty variant list from both sources', () => {
    expect(rustEnumTags('pub enum ParameterDomain {').size).toBeGreaterThan(0);
    expect(tsUnionTags('export type ParameterDomain =').size).toBe(
      rustEnumTags('pub enum ParameterDomain {').size,
    );
  });

  it('declares the same set of tags in types.ts as the engine serializes', () => {
    expect([...tsUnionTags('export type ParameterDomain =')].sort()).toEqual(
      [...rustEnumTags('pub enum ParameterDomain {')].sort(),
    );
  });
});
