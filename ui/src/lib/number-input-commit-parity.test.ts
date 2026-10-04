import { readdirSync, readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { describe, expect, it } from 'vitest';

// N2 (try-out 2026-10-04): a number field bound one-way to a clamping store setter
// showed the typed text, not the stored value, whenever the clamp left the store
// unchanged ("5000" in an age field holding 500). The sweep found 36 such fields in
// 22 components and one that wrote back by hand, so fixing the reported field and
// not the class would have left 35. `actions.ts::commitStored` writes the stored
// value back on commit; this sweep makes every number field either carry it or say,
// in a comment directly before the tag, why it does not:
//
//     <!-- commitStored-exempt: <reason> -->
//
// (local filter state on `bind:value`, which no store setter clamps, is the case
// the opt-out exists for). A number field added tomorrow fails here on its own.
//
// `ssr` project (plain `*.test.ts`): text parsing over repo files, as in
// `tooltip-host-parity.test.ts`. The write-back itself is proved once for the action
// in `commit-stored.client.test.ts` and per component family in
// `components/commit-stored-fields.client.test.ts`.

const srcDir = fileURLToPath(new URL('..', import.meta.url));

/** Every `.svelte` file under `ui/src`, in stable path order. */
function svelteFiles(): string[] {
  return readdirSync(srcDir, { recursive: true, encoding: 'utf-8' })
    .filter((path) => path.endsWith('.svelte'))
    .sort();
}

/** The `<script>` blocks blanked out, so offsets and line numbers still index the file. */
function markupOf(source: string): string {
  return source.replace(/<script[\s\S]*?<\/script>/g, (block) => block.replace(/[^\n]/g, ' '));
}

/** `[start, end)` of every HTML comment, which can mention a field but not be one. */
function commentRanges(markup: string): [number, number][] {
  return [...markup.matchAll(/<!--[\s\S]*?-->/g)].map((m) => [m.index, m.index + m[0].length]);
}

/**
 * The opening `<` and full start tag enclosing the character at `at`. Walks forward
 * tracking quotes and `{}` depth, because a Svelte start tag is full of `>`
 * characters that do not end it (arrow functions in handlers).
 */
function startTagAt(markup: string, at: number): { open: number; tag: string } {
  const open = markup.lastIndexOf('<', at);
  if (open === -1) throw new Error('no start tag opens before a `type="number"`');
  let quote: string | null = null;
  let depth = 0;
  for (let i = open; i < markup.length; i += 1) {
    const char = markup[i];
    if (quote) {
      if (char === quote) quote = null;
      continue;
    }
    if (char === '"' || char === "'" || char === '`') quote = char;
    else if (char === '{') depth += 1;
    else if (char === '}') depth -= 1;
    else if (char === '>' && depth === 0) return { open, tag: markup.slice(open, i + 1) };
  }
  throw new Error('unterminated start tag around a `type="number"`');
}

type NumberField = { file: string; line: number; tag: string; exemptReason: string | null };

/** The reason an opt-out comment directly before the tag gives, or `null` if none. */
function exemptReasonBefore(markup: string, open: number): string | null {
  const before = markup.slice(0, open).trimEnd();
  if (!before.endsWith('-->')) return null;
  const comment = before.slice(before.lastIndexOf('<!--'));
  const reason = /^<!--\s*commitStored-exempt:\s*([\s\S]*?)\s*-->/.exec(comment)?.[1] ?? '';
  return reason === '' ? null : reason;
}

/** Every `<input type="number">` in the frontend's markup. */
function numberFields(): NumberField[] {
  const fields: NumberField[] = [];
  for (const file of svelteFiles()) {
    const markup = markupOf(readFileSync(`${srcDir}${file}`, 'utf-8'));
    const comments = commentRanges(markup);
    for (const match of markup.matchAll(/type="number"/g)) {
      if (comments.some(([start, end]) => match.index >= start && match.index < end)) continue;
      const { open, tag } = startTagAt(markup, match.index);
      fields.push({
        file,
        line: markup.slice(0, open).split('\n').length,
        tag,
        exemptReason: exemptReasonBefore(markup, open),
      });
    }
  }
  return fields;
}

describe('every number field writes the stored value back on commit (N2, class-level)', () => {
  it('finds the number fields at all, and reads the right tag for each', () => {
    // Guards the parser: a mis-sliced tag would make the assertion below vacuous.
    const fields = numberFields();
    expect(fields.length).toBeGreaterThan(0);
    for (const field of fields) {
      expect(field.tag, `${field.file}:${field.line} is not an input`).toMatch(/^<input\b/);
      expect(field.tag, `${field.file}:${field.line} lost its type`).toContain('type="number"');
    }
  });

  it('skips a `type="number"` that only a comment mentions', () => {
    // ParameterPicker's D35 comment names "the first `<input type=\"number\">`", so
    // the raw text holds more mentions than there are fields.
    const mentions = svelteFiles().reduce(
      (count, file) =>
        count +
        (markupOf(readFileSync(`${srcDir}${file}`, 'utf-8')).match(/type="number"/g)?.length ?? 0),
      0,
    );
    expect(numberFields().length).toBeLessThan(mentions);
  });

  it('carries `use:commitStored` or a stated `commitStored-exempt` reason on every one', () => {
    const unguarded = numberFields()
      .filter((field) => !field.tag.includes('use:commitStored') && field.exemptReason == null)
      .map((field) => `${field.file}:${field.line}`);
    expect(unguarded).toEqual([]);
  });

  it('never exempts a field that carries the action anyway', () => {
    const both = numberFields()
      .filter((field) => field.tag.includes('use:commitStored') && field.exemptReason != null)
      .map((field) => `${field.file}:${field.line}`);
    expect(both).toEqual([]);
  });
});
