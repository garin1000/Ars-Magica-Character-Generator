import { readdirSync, readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { describe, expect, it } from 'vitest';

// Sabine 5 (full-audit round 4). CLAUDE.md: "No user-facing string hardcoded in
// Rust or Svelte." A typographic glyph standing in for a WORD is such a string —
// the repo has already decided this twice, keying a literal `&` as
// `derived-combat-and` ("It stands in for a word … so it is translatable, not
// hardcoded") and a literal em dash as `derived-not-applicable`. An arrow between
// a base and an effective score is the same shape: it states a relationship, and
// a locale may want to state it differently, or with a word.
//
// Scoped to the ARROWS deliberately. The other glyph in the codebase is `×`, and
// its eleven occurrences are all `.icon-btn` CONTENT behind a
// `store.t('remove-item', …)` `aria-label` — a keyed accessible name with a
// decorative mark inside it, which is an established and consistent pattern.
// Arrows have no such exemption, so the rule can be absolute here and stay
// honest, rather than growing a heuristic that would eventually excuse the very
// case it was written to catch.
//
// `ssr` project (plain `*.test.ts`): text parsing over repo files, no component
// mounted. Same technique as `prereq-parity.test.ts`.

const srcDir = fileURLToPath(new URL('..', import.meta.url));

/** Every `.svelte` file under `ui/src`, in stable path order. */
function svelteFiles(): string[] {
  return readdirSync(srcDir, { recursive: true, encoding: 'utf-8' })
    .filter((path) => path.endsWith('.svelte'))
    .sort();
}

/**
 * Markup only. A glyph in the `<script>` block is data or a comment, and a glyph
 * in an HTML comment is prose about one — neither reaches the screen.
 */
function markupOf(source: string): string {
  return source.replace(/<script[\s\S]*?<\/script>/g, '').replace(/<!--[\s\S]*?-->/g, '');
}

/** Text content — what is left once every tag and every `{expression}` is gone. */
function textContentOf(markup: string): string {
  return markup.replace(/<[^>]*>/g, ' ').replace(/\{[^{}]*\}/g, ' ');
}

describe('no arrow glyph is rendered as unkeyed markup (Sabine 5)', () => {
  it('reads real text content out of the components', () => {
    // Guards the stripper: if it returned nothing, the sweep below would pass
    // vacuously. Some component must still show plain text after the strip.
    const texts = svelteFiles().map((file) =>
      textContentOf(markupOf(readFileSync(`${srcDir}${file}`, 'utf-8'))),
    );
    expect(texts.some((text) => /\S/.test(text))).toBe(true);
  });

  it('writes no arrow into any component, keying it through Fluent instead', () => {
    const offenders: string[] = [];
    for (const file of svelteFiles()) {
      const text = textContentOf(markupOf(readFileSync(`${srcDir}${file}`, 'utf-8')));
      for (const [glyph] of text.matchAll(/[→←↑↓↔⇒⇐]/g)) offenders.push(`${file}: ${glyph}`);
    }
    expect(offenders).toEqual([]);
  });
});
