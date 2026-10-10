import { beforeEach, describe, expect, it, vi } from 'vitest';
import { render } from 'svelte/server';

import type { LabYears } from '../types';

// The four lab-year counts (row 56) as their own component, so phase 2's life
// sections can mount one per section under their own testid prefix. Rendered with
// `svelte/server`: everything asserted here is markup. The labels come from the
// store's Fluent bundle, and importing the store means mocking the IPC bridge.
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
import LabYearsFields from './LabYearsFields.svelte';

const PREFIX = 'section-2-lab-years';
const testids = [1, 2, 3, 4].map((n) => `${PREFIX}-${n}-input`);

function html(
  props: Partial<{ value: LabYears; readonly: boolean; describedBy: string; legend: string }> = {},
): string {
  return render(LabYearsFields, {
    props: { value: undefined, testidPrefix: PREFIX, onchange: () => {}, ...props },
  }).body;
}

/** The opening tag of the element carrying `testid`. */
function openTag(body: string, testid: string): string {
  const match = new RegExp(`<[^>]*data-testid="${testid}"[^>]*>`, 'i').exec(body);
  if (!match) throw new Error(`no element with data-testid="${testid}"`);
  return match[0];
}

beforeEach(() => {
  store.lang = 'en';
});

describe('LabYearsFields (row 56)', () => {
  it('renders one number input per count, named by the testid prefix', () => {
    const body = html();
    for (const testid of testids) expect(openTag(body, testid)).toMatch(/type="number"/);
  });

  it('wraps them in a fieldset whose legend names the years of lab work', () => {
    const body = html();
    const fieldset = /<fieldset[\s\S]*?<\/fieldset>/i.exec(body);
    expect(fieldset).not.toBeNull();
    const legend = /<legend[^>]*>([\s\S]*?)<\/legend>/i.exec(fieldset![0]);
    expect(legend![1].replace(/<[^>]*>/g, '').trim()).toBe('Years of lab work');
    for (const testid of testids) expect(fieldset![0]).toContain(`data-testid="${testid}"`);
  });

  // G3: one group per life section, so each needs a legend telling it apart from the
  // others — a screen reader landing in one hears whose lab years these are.
  it('takes the legend it is given, for one section’s own counts', () => {
    const body = html({ legend: 'Squire: years of lab work' });
    const legend = /<legend[^>]*>([\s\S]*?)<\/legend>/i.exec(body);
    expect(legend![1].replace(/<[^>]*>/g, '').trim()).toBe('Squire: years of lab work');
  });

  it('shows each stored count in its own field, and an absent one as blank', () => {
    const body = html({ value: { one_season: 2, four_seasons: 5 } });
    expect(openTag(body, testids[0])).toMatch(/value="2"/);
    expect(openTag(body, testids[1])).toMatch(/value=""/);
    expect(openTag(body, testids[2])).toMatch(/value=""/);
    expect(openTag(body, testids[3])).toMatch(/value="5"/);
  });

  it('goes read-only and points every field at the explanation when told to', () => {
    const body = html({ readonly: true, describedBy: 'why-closed' });
    for (const testid of testids) {
      const open = openTag(body, testid);
      // `readonly`, not `disabled`: the field keeps its place in the tab order and
      // its label, so the explanation reaches the user who lands on it.
      expect(open).toMatch(/readonly/);
      expect(open).not.toMatch(/disabled/);
      expect(open).toMatch(/aria-describedby="why-closed"/);
    }
  });

  it('carries neither readonly nor a description by default', () => {
    const body = html();
    for (const testid of testids) {
      const open = openTag(body, testid);
      expect(open).not.toMatch(/readonly/);
      expect(open).not.toMatch(/aria-describedby/);
    }
  });
});
