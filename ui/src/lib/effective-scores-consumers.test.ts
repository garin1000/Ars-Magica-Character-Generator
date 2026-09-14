import { readdirSync, readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { describe, expect, it } from 'vitest';

// V1 (full-audit round 4). `EffectiveScores` is the engine→UI DTO: every field is
// recomputed on each `refresh()` and crosses IPC on every validation pass. A
// field nobody reads is not merely dead weight — `age_ability_cap` documented
// itself as "surfaced so the UI shows one source of truth" while surfacing a
// number that is NOT the cap validation enforces (that one is per-ability and
// halves under Foreign Upbringing, `effective/reputation_and_caps.rs::ability_age_cap`).
// A DTO field that advertises a consumer it does not have is an invitation to
// bind a control to the wrong number.
//
// So the invariant is cross-site rather than per-field: whatever the interface
// declares, something must read. The round-3 lesson is that a defect fixed at the
// one site a reviewer named comes back at the site nobody swept; this fails on
// the NEXT orphaned field without anyone sweeping again.
//
// Test files do not count as consumers, deliberately. Three of the four files
// mentioning `age_ability_cap` asserted it must not be rendered, which is the
// opposite of a consumer — counting them would have made the guard agree that a
// field proven unrendered was in use.
//
// `ssr` project (plain `*.test.ts`): text parsing over repo files, no component.

const srcDir = fileURLToPath(new URL('..', import.meta.url));

/** Every file under `ui/src` that ships in the app, in stable path order. */
function shippedSources(): string[] {
  return (
    readdirSync(srcDir, { recursive: true, encoding: 'utf-8' })
      .filter((path) => path.endsWith('.svelte') || path.endsWith('.ts'))
      .filter((path) => !/\.test\.ts$/.test(path))
      // `types.ts` is the declaration itself, never a reading of it.
      .filter((path) => path !== 'lib/types.ts')
      .sort()
  );
}

/**
 * The top-level field names of the `EffectiveScores` interface. Two-space indent
 * only: a deeper one is a member of a nested object type, not a DTO field.
 */
function effectiveScoreFields(): string[] {
  const source = readFileSync(`${srcDir}lib/types.ts`, 'utf-8');
  const start = source.indexOf('export interface EffectiveScores {');
  if (start === -1) throw new Error('not found: export interface EffectiveScores');
  const end = source.indexOf('\n}', start);
  if (end === -1) throw new Error('unterminated interface: EffectiveScores');
  return [...source.slice(start, end).matchAll(/^ {2}(\w+)\??:/gm)].map((match) => match[1]);
}

describe('every EffectiveScores field the DTO declares has a consumer (V1)', () => {
  it('reads a plausible field list and a plausible file list', () => {
    // Guards both parsers: an interface rename or a glob that matched nothing
    // would make the sweep below vacuously green rather than loud.
    expect(effectiveScoreFields().length).toBeGreaterThan(40);
    expect(effectiveScoreFields()).toContain('confidence_score');
    expect(shippedSources().some((path) => path.endsWith('.svelte'))).toBe(true);
  });

  it('is read by at least one shipped component or module', () => {
    const sources = shippedSources().map((path) => readFileSync(`${srcDir}${path}`, 'utf-8'));
    const orphans = effectiveScoreFields().filter(
      (field) => !sources.some((source) => source.includes(field)),
    );
    expect(orphans).toEqual([]);
  });
});
