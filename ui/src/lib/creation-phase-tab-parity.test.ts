import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { describe, expect, it } from 'vitest';

// A2/D56 § 7 risk 3 follow-up: `App.svelte`'s tab list reads
// `EffectiveScores.phases_in_force` — a `Set<CreationPhase>` — through calls
// like `phasesInForce.has('arts')`. Nothing forces those string literals to
// name a real `CreationPhase`: a typo (`'artz'`) or a stale slug (a phase
// later renamed in the engine) would compile fine on both sides and simply
// resolve to `false` forever, silently hiding the tab — the same class of
// drift `prereq-parity.test.ts` exists to catch for `Prereq`'s kinds, but
// nothing before this file covered `CreationPhase`. Mirrors that file's
// technique: diff the Rust source against the Svelte source as text, no
// hand-copied slug list on either side.
//
// `ssr` project (plain `*.test.ts`): pure text parsing, no component, no DOM.

function repoFile(relative: string): string {
  return readFileSync(fileURLToPath(new URL(relative, import.meta.url)), 'utf-8');
}

const rustTypes = repoFile('../../../crates/arm-rules/src/types.rs');
const appSource = repoFile('../App.svelte');

/**
 * Every slug `CreationPhase`'s `Display` impl serializes, read from the
 * engine's own source rather than hand-copied — the same slug the `#[serde]`
 * derive emits (asserted identical by the Rust test
 * `creation_phase_slugs_are_the_profile_phase_strings`), so this is also the
 * vocabulary `phases_in_force`'s JSON carries.
 */
function rustCreationPhaseSlugs(): Set<string> {
  const start = rustTypes.indexOf('impl fmt::Display for CreationPhase {');
  if (start === -1) throw new Error('not found: impl fmt::Display for CreationPhase');
  const end = rustTypes.indexOf('\n}', start);
  if (end === -1) throw new Error('unterminated impl: Display for CreationPhase');
  const body = rustTypes.slice(start, end);
  return new Set(Array.from(body.matchAll(/=> "([a-z_]+)"/g), (m) => m[1]));
}

/**
 * Every phase name `App.svelte` names in a `phasesInForce.has('<slug>')`
 * call — the tab list's own resolved-phase reads.
 */
function svelteHasCalls(): string[] {
  return Array.from(appSource.matchAll(/phasesInForce\.has\('([^']+)'\)/g), (m) => m[1]);
}

describe('CreationPhase / App.svelte tab-gate parity (A2/D56 § 7 risk 3)', () => {
  it('reads a non-empty slug list and a non-empty call list', () => {
    // Guards the parsers themselves: a rename of the impl or the tab list would
    // otherwise make the assertion below vacuously true.
    expect(rustCreationPhaseSlugs().size).toBeGreaterThan(0);
    expect(svelteHasCalls().length).toBeGreaterThan(0);
  });

  it('names only real CreationPhase slugs in every phasesInForce.has(...) gate', () => {
    for (const phase of svelteHasCalls()) {
      expect(rustCreationPhaseSlugs(), `'${phase}' is not a real CreationPhase slug`).toContain(
        phase,
      );
    }
  });
});
