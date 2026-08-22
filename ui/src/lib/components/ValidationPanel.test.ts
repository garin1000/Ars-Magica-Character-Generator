import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { render } from 'svelte/server';

import type { CreationPhase, Entity, LocalizedRuleset, ValidationIssue } from '../types';

// The panel reads the shared store singleton (result, ruleset, Fluent bundle) and
// nothing else; the store's actions go over the Tauri bridge, so mock it away.
// Harness mirrors StartScreen.test.ts.
vi.mock('../ipc', () => ({
  loadRuleset: vi.fn(),
  validateEntity: vi.fn().mockResolvedValue({ issues: [] }),
  effectiveScores: vi.fn().mockResolvedValue({}),
  derivedTotals: vi.fn().mockResolvedValue({}),
  saveEntity: vi.fn(),
  loadEntity: vi.fn(),
  updateCloseGuard: vi.fn(),
  exportMarkdown: vi.fn(),
  exportLabelKeys: vi.fn(),
  applyChildhoodPackage: vi.fn(),
}));

import { SCHEMA_VERSION, store } from '../state.svelte';
import ValidationPanel from './ValidationPanel.svelte';

function issue(
  code: string,
  phase: CreationPhase,
  severity: 'error' | 'warning',
  args: Record<string, string> = {},
): ValidationIssue {
  return { severity, code, phase, args };
}

/** Every `data-code` present in the rendered issue list, in document order. */
function renderedCodes(body: string): string[] {
  return [...body.matchAll(/data-code="([^"]+)"/g)].map((m) => m[1]);
}

/** The rendered `<li>` markup for one issue code, so a test can inspect its content. */
function issueMarkup(body: string, code: string): string {
  const match = new RegExp(`<li[^>]*data-code="${code}"[^>]*>([\\s\\S]*?)</li>`).exec(body);
  if (!match) throw new Error(`no <li> for code ${code}`);
  return match[1];
}

beforeEach(() => {
  vi.useFakeTimers();
  store.lang = 'en';
  store.ruleset = {
    ruleset: {
      id: 'test',
      version: '1',
      point_items: {},
      type_profiles: {},
      magnitude_points: { free: 0, minor: 1, major: 3 },
      ability_category_order: ['general'],
      art_type_order: ['technique', 'form'],
    },
    i18n: {},
  } as unknown as LocalizedRuleset;
  store.entity = {
    schema_version: SCHEMA_VERSION,
    ruleset: { id: 'test', version: '1' },
    entity_kind: 'character',
    type_id: 'companion',
    selections: [],
    characteristics: {} as Entity['characteristics'],
    characteristic_descriptions: {},
    ability_scores: [],
    xp_pool: 0,
    art_scores: [],
    personality_traits: [],
    reputations: [],
  };
  // Each carries the args its `issue-<code>` message interpolates (the contract
  // table in `validation/mod.rs`); Fluent throws on a missing variable.
  store.result = {
    issues: [
      issue('unbalanced_virtues', 'virtues_flaws', 'error', {
        virtue_points: '3',
        flaw_points: '0',
      }),
      issue('characteristic_points_unspent', 'characteristics', 'warning', {
        cost: '4',
        points: '7',
      }),
      issue('unknown_equipment', 'review', 'error', { item: 'weapon.nonesuch' }),
    ],
  };
});

afterEach(() => {
  vi.clearAllTimers();
  vi.useRealTimers();
  store.result = null;
});

describe('ValidationPanel', () => {
  it('shows every finding for the whole character when given no phase', () => {
    expect(renderedCodes(render(ValidationPanel).body)).toEqual([
      'unbalanced_virtues',
      'characteristic_points_unspent',
      'unknown_equipment',
    ]);
  });

  // What makes a wizard step's footer about that step: a finding the user cannot
  // act on here would read as a problem with the step they are on.
  it("shows only the given phase's findings", () => {
    const body = render(ValidationPanel, { props: { phase: 'virtues_flaws' } }).body;
    expect(renderedCodes(body)).toEqual(['unbalanced_virtues']);
  });

  it('keeps warnings, which inform without blocking', () => {
    const body = render(ValidationPanel, { props: { phase: 'characteristics' } }).body;
    expect(renderedCodes(body)).toEqual(['characteristic_points_unspent']);
  });

  it('reports a clean step when the phase filter empties the list', () => {
    const body = render(ValidationPanel, { props: { phase: 'arts' } }).body;
    expect(body).toContain('data-testid="no-issues"');
    expect(renderedCodes(body)).toEqual([]);
  });

  it('renders each message through its issue-<code> key, never the raw code', () => {
    const body = render(ValidationPanel, { props: { phase: 'virtues_flaws' } }).body;
    expect(body).not.toContain('issue-unbalanced_virtues<');
    expect(body).toMatch(/Virtue points.*Flaw points|funded/i);
  });

  it('localizes the messages to German', () => {
    store.lang = 'de';
    const body = render(ValidationPanel, { props: { phase: 'virtues_flaws' } }).body;
    expect(body).not.toContain('issue-unbalanced_virtues<');
    expect(body).toMatch(/Tugend|Fehler/);
  });

  // S2 (tmp/review/review-round-2-sabine.md): round 1's fix made the severity
  // prefix `sr-only`, which is invisible to SIGHTED users — so a colourblind
  // sighted user still had only the border/background hue swap to go on,
  // still color-only in practice. The real fix (verified against
  // ValidationPanel.svelte + app.css) renders the severity word VISIBLY via
  // the `issue-severity` class (bold, uppercase, non-`sr-only`) — a different
  // WORD, not just a different hue — and app.css additionally gives the Error
  // row a non-colour `::after { content: ' !' }` glyph mirroring the wizard
  // rail's own blocked-step marker. CSS `::after` content is invisible to
  // `render()`'s string output, so only the visible text itself is asserted
  // here; the glyph rule is read directly from app.css instead.
  it('renders each issue with a visible (not screen-reader-only) severity label', () => {
    const body = render(ValidationPanel).body;
    expect(issueMarkup(body, 'unbalanced_virtues')).toMatch(
      /<span class="issue-severity">Error:<\/span> /,
    );
    expect(issueMarkup(body, 'characteristic_points_unspent')).toMatch(
      /<span class="issue-severity">Warning:<\/span> /,
    );
    // Not screen-reader-only: `sr-only` visually hides content, which would
    // reintroduce the exact colour-only failure this fix closes.
    expect(issueMarkup(body, 'unbalanced_virtues')).not.toContain('sr-only');
    expect(issueMarkup(body, 'characteristic_points_unspent')).not.toContain('sr-only');
  });

  it('gives the blocking Error severity a non-colour marker beyond the visible word', () => {
    // Verified directly against app.css rather than rendered markup: `::after`
    // pseudo-element content never appears in server-rendered HTML, so this is
    // the only way to pin the rule without a client-mounted computed-style
    // check, which would be disproportionate for a static CSS selector.
    const appCssPath = fileURLToPath(new URL('../../app.css', import.meta.url));
    const css = readFileSync(appCssPath, 'utf-8');
    expect(css).toMatch(/\.issue\.error\s+\.issue-severity::after\s*{\s*content:\s*' !';?\s*}/);
  });

  it('localizes the visible severity label to German', () => {
    store.lang = 'de';
    const body = render(ValidationPanel).body;
    expect(issueMarkup(body, 'unbalanced_virtues')).toMatch(
      /<span class="issue-severity">Fehler:<\/span> /,
    );
    expect(issueMarkup(body, 'characteristic_points_unspent')).toMatch(
      /<span class="issue-severity">Warnung:<\/span> /,
    );
  });
});
