import { beforeEach, describe, expect, it, vi } from 'vitest';
import { render } from 'svelte/server';
import { createRawSnippet } from 'svelte';

import type { LocalizedRuleset } from '../types';

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

import { store } from '../state.svelte';
import SourcePicker from './SourcePicker.svelte';

function installRuleset(): void {
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
}

/** A no-op snippet, standing in for a caller's `filters`/`row` content. */
const emptySnippet = createRawSnippet(() => ({
  render: () => '<span></span>',
}));

function html(
  groups: { key: string; header: string; items: string[] }[],
  disabled?: (item: string) => boolean,
): string {
  return render(SourcePicker, {
    props: {
      groups,
      // `render()` cannot carry SourcePicker's own generic `T` through, so it
      // infers `unknown` for every callback prop here — matched rather than
      // fought, since `groups` is always `string[]` items in this file.
      getId: (item: unknown) => item as string,
      onAdd: () => {},
      disabled: disabled && ((item: unknown) => disabled(item as string)),
      row: emptySnippet,
      filters: emptySnippet,
    },
  }).body;
}

/** The opening `<button>` tag of the row whose add-testid is `add-{id}`. */
function rowTag(body: string, id: string): string {
  const match = new RegExp(`<button[^>]*data-testid="add-${id}"[^>]*>`).exec(body);
  expect(match, `no row rendered for ${id}`).not.toBeNull();
  return match![0];
}

beforeEach(() => {
  store.lang = 'en';
  installRuleset();
});

// S25 (full-audit UX): a filter that matches nothing left the list area
// completely blank — no feedback that the search/filter combination excluded
// every row, indistinguishable from a slow-loading or broken panel.
describe('SourcePicker empty state (S25)', () => {
  it('renders nothing extra when at least one group has items', () => {
    const body = html([{ key: 'general', header: 'General', items: ['ability.awareness'] }]);
    expect(body).not.toContain('data-testid="source-no-results"');
  });

  it('shows a no-results message when every group is filtered out', () => {
    const body = html([]);
    expect(body).toContain('data-testid="source-no-results"');
    expect(body).toContain('No matches for the current filter.');
  });

  it('localizes the no-results message to German', () => {
    store.lang = 'de';
    const body = html([]);
    expect(body).toContain('data-testid="source-no-results"');
    expect(body).not.toContain('filter-no-results');
  });
});

// Finding #18: every blocked treatment — the dim, the muted glyph, the suppressed
// hover, and the screen-reader "unavailable" announcement — hangs off ONE marker
// emitted here, for all three lists that mount this component (Abilities, Spells,
// Virtues/Flaws). The marker is `aria-disabled`, never the native `disabled`
// attribute: a natively disabled button is not focusable, and the row has to stay
// focusable or the tooltip explaining WHY it is blocked becomes unreachable by
// keyboard (`use:tooltip` opens on `focusin` and sets `aria-describedby`).
describe('SourcePicker blocked rows (#18)', () => {
  const twoRows = [{ key: 'general', header: 'General', items: ['takeable', 'blocked'] }];

  it('marks a non-takeable row aria-disabled', () => {
    const body = html(twoRows, (item) => item === 'blocked');
    expect(rowTag(body, 'blocked')).toContain('aria-disabled="true"');
  });

  it('leaves a takeable row not aria-disabled', () => {
    const body = html(twoRows, (item) => item === 'blocked');
    expect(rowTag(body, 'takeable')).toContain('aria-disabled="false"');
  });

  it('never natively disables a row, which would take it out of the tab order', () => {
    const body = html(twoRows, () => true);
    expect(body).not.toMatch(/<button[^>]*\sdisabled/);
  });

  it('marks nothing when the caller supplies no disabled predicate', () => {
    const body = html(twoRows);
    expect(body).not.toContain('aria-disabled="true"');
  });
});
