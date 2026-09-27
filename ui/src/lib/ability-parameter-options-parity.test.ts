import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { describe, expect, it } from 'vitest';

// CV6 (design-cv-catalogued-values.md § 6.5): "a new parity test confirms
// `AbilityParameterOptions`'s shape matches its Rust source." Mirrors
// `param-type-parity.test.ts`'s technique (diff Rust against TS as text) but
// for a plain struct's field set rather than an enum's tag set, so a future
// Rust field is a missing TS member here rather than a silently-stale mirror.
//
// `ssr` project (plain `*.test.ts`): pure text parsing over repo files, no
// component, no DOM, no `$effect`.

function repoFile(relative: string): string {
  return readFileSync(fileURLToPath(new URL(relative, import.meta.url)), 'utf-8');
}

const rustSource = repoFile('../../../crates/arm-rules/src/effective/parameter_options.rs');
const tsTypes = repoFile('./types.ts');

/** The body of a brace-delimited block whose opening `{` follows `header`. */
function blockAfter(source: string, header: string): string {
  const start = source.indexOf(header);
  if (start === -1) throw new Error(`not found: ${header}`);
  const open = source.indexOf('{', start);
  const close = source.indexOf('\n}', open);
  if (open === -1 || close === -1) throw new Error(`unterminated block: ${header}`);
  return source.slice(open + 1, close);
}

/** Every `pub <name>:` field declared in a Rust struct block. */
function rustStructFields(source: string, structHeader: string): Set<string> {
  const body = blockAfter(source, structHeader);
  const fields = new Set<string>();
  for (const line of body.split('\n')) {
    const field = /^\s*pub (\w+):/.exec(line);
    if (field) fields.add(field[1]);
  }
  return fields;
}

/** Every `<name>:`/`<name>?:` field declared in a TS interface block. */
function tsInterfaceFields(source: string, interfaceHeader: string): Set<string> {
  const body = blockAfter(source, interfaceHeader);
  const fields = new Set<string>();
  for (const line of body.split('\n')) {
    const field = /^\s*(\w+)\??:/.exec(line);
    if (field) fields.add(field[1]);
  }
  return fields;
}

describe('AbilityParameterOptions Rust/TS field parity', () => {
  it('LinkTarget carries the same fields on both sides', () => {
    const rust = rustStructFields(rustSource, 'pub struct LinkTarget');
    const ts = tsInterfaceFields(tsTypes, 'export interface LinkTarget');
    expect(rust.size).toBeGreaterThan(0);
    expect(ts).toEqual(rust);
  });

  it('AbilityParameterOptions carries the same fields on both sides', () => {
    const rust = rustStructFields(rustSource, 'pub struct AbilityParameterOptions');
    const ts = tsInterfaceFields(tsTypes, 'export interface AbilityParameterOptions');
    expect(rust.size).toBeGreaterThan(0);
    expect(ts).toEqual(rust);
  });
});
