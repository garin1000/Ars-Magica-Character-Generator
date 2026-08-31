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

function html(groups: { key: string; header: string; items: string[] }[]): string {
  return render(SourcePicker, {
    props: {
      groups,
      // `render()` cannot carry SourcePicker's own generic `T` through, so it
      // infers `unknown` for every callback prop here — matched rather than
      // fought, since `groups` is always `string[]` items in this file.
      getId: (item: unknown) => item as string,
      onAdd: () => {},
      row: emptySnippet,
      filters: emptySnippet,
    },
  }).body;
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
