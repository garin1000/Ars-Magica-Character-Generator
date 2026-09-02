import { describe, expect, it } from 'vitest';
import { render } from 'svelte/server';
import { createRawSnippet } from 'svelte';

import SelectionList from './SelectionList.svelte';

// E2 (full-audit backlog, Tier 3-4): SelectionList.svelte had no test file at all,
// despite being the shared skeleton every "chosen items" panel renders through
// (V/F, Abilities, Equipment, Spells — see the component's own doc comment). A
// defect here surfaces everywhere at once, so this exercises its real branching
// (single vs multi column, empty vs grouped, headed vs header-less groups) rather
// than a smoke test.

/** A row snippet rendering `<li data-testid="row-{id}">{id}</li>` per item. */
const idRow = createRawSnippet<[string]>((getItem) => ({
  render: () => `<li data-testid="row-${getItem()}">${getItem()}</li>`,
}));

/** `render()` cannot carry the component's own generic `T` through (matches the
 * same shape SourcePicker.test.ts and VirtueFlawTab.test.ts already accept), so
 * every call site here infers `unknown` and is asserted past it. */
function html(columns: unknown[]): string {
  return render(SelectionList, {
    props: { columns, row: idRow },
  } as never).body;
}

describe('SelectionList single-column layout', () => {
  it('renders the one panel directly, with no multi-column wrapper', () => {
    const body = html([{ key: 'a', groups: [{ key: 'g', rows: [{ key: 'x', item: 'x' }] }] }]);
    expect(body).not.toContain('region-columns');
    expect(body).toContain('data-testid="row-x"');
  });

  it('renders nothing for zero columns', () => {
    // Svelte 5's SSR output carries its own block-boundary comments even for an
    // empty `{#if}`/`{:else if}` fallthrough; strip those before asserting there
    // is no real content.
    expect(
      html([])
        .replace(/<!--.*?-->/g, '')
        .trim(),
    ).toBe('');
  });

  it('renders the panel title only when supplied', () => {
    const withTitle = html([{ key: 'a', title: 'Virtues', groups: [{ key: 'g', rows: [] }] }]);
    expect(withTitle).toContain('<h2>Virtues</h2>');

    const withoutTitle = html([{ key: 'a', groups: [{ key: 'g', rows: [] }] }]);
    expect(withoutTitle).not.toContain('<h2>');
  });

  it('sets the panel data-testid from panelTestid', () => {
    const body = html([
      { key: 'a', panelTestid: 'selection-list-virtues', groups: [{ key: 'g', rows: [] }] },
    ]);
    expect(body).toContain('data-testid="selection-list-virtues"');
  });

  it('renders the header snippet above the groups when supplied', () => {
    const header = createRawSnippet(() => ({
      render: () => '<div data-testid="budget-bar">120/240</div>',
    }));
    const body = html([{ key: 'a', header, groups: [{ key: 'g', rows: [] }] }]);
    const headerIndex = body.indexOf('data-testid="budget-bar"');
    const groupIndex = body.indexOf('<ul');
    expect(headerIndex).toBeGreaterThan(-1);
    // The header renders before the groups' list markup.
    expect(headerIndex).toBeLessThan(groupIndex === -1 ? Infinity : groupIndex);
  });
});

describe('SelectionList empty vs grouped rows', () => {
  it('shows the empty message and renders no groups when empty is set', () => {
    const body = html([
      {
        key: 'a',
        empty: true,
        emptyText: 'Nothing selected yet',
        emptyClass: 'muted',
        groups: [{ key: 'g', rows: [{ key: 'x', item: 'x' }] }],
      },
    ]);
    expect(body).toContain('<p class="muted">Nothing selected yet</p>');
    expect(body).not.toContain('data-testid="row-x"');
  });

  it('defaults the empty paragraph class to "empty" when emptyClass is omitted', () => {
    const body = html([
      { key: 'a', empty: true, emptyText: 'None yet', groups: [{ key: 'g', rows: [] }] },
    ]);
    expect(body).toContain('<p class="empty">None yet</p>');
  });

  it('renders every row of every group via the row snippet, in order', () => {
    const body = html([
      {
        key: 'a',
        groups: [
          {
            key: 'g1',
            rows: [
              { key: 1, item: 'one' },
              { key: 2, item: 'two' },
            ],
          },
          { key: 'g2', rows: [{ key: 3, item: 'three' }] },
        ],
      },
    ]);
    const oneIndex = body.indexOf('data-testid="row-one"');
    const twoIndex = body.indexOf('data-testid="row-two"');
    const threeIndex = body.indexOf('data-testid="row-three"');
    expect(oneIndex).toBeGreaterThan(-1);
    expect(twoIndex).toBeGreaterThan(oneIndex);
    expect(threeIndex).toBeGreaterThan(twoIndex);
  });
});

describe('SelectionList group headers', () => {
  it('renders an h3.category for a group that carries a header', () => {
    const body = html([
      {
        key: 'a',
        groups: [{ key: 'g', header: 'General', rows: [{ key: 'x', item: 'x' }] }],
      },
    ]);
    expect(body).toContain('<h3 class="category">General</h3>');
  });

  it('renders no h3 for a header-less group', () => {
    const body = html([{ key: 'a', groups: [{ key: 'g', rows: [{ key: 'x', item: 'x' }] }] }]);
    expect(body).not.toContain('<h3');
  });

  it('mixes headed and header-less groups within the same panel', () => {
    const body = html([
      {
        key: 'a',
        groups: [
          { key: 'g1', header: 'Weapons', rows: [{ key: 'w', item: 'w' }] },
          { key: 'g2', rows: [{ key: 'u', item: 'u' }] },
        ],
      },
    ]);
    expect(body).toContain('<h3 class="category">Weapons</h3>');
    expect((body.match(/<h3/g) ?? []).length).toBe(1);
  });

  it('defaults the <ul> class to selection-list and sets ulTestid when supplied', () => {
    const body = html([
      {
        key: 'a',
        groups: [{ key: 'g', ulTestid: 'equipment-list', rows: [{ key: 'x', item: 'x' }] }],
      },
    ]);
    expect(body).toMatch(/<ul class="selection-list" data-testid="equipment-list">/);
  });

  it('uses a custom listClass when supplied, instead of the default', () => {
    const body = html([
      {
        key: 'a',
        groups: [{ key: 'g', listClass: 'equipment-list', rows: [{ key: 'x', item: 'x' }] }],
      },
    ]);
    expect(body).toMatch(/<ul class="equipment-list"/);
    expect(body).not.toMatch(/<ul class="selection-list"/);
  });
});

describe('SelectionList multi-column layout', () => {
  it('wraps two or more columns in a region-columns div, rendering both panels', () => {
    const body = html([
      { key: 'virtues', title: 'Virtues', groups: [{ key: 'g', rows: [{ key: 'v', item: 'v' }] }] },
      { key: 'flaws', title: 'Flaws', groups: [{ key: 'g', rows: [{ key: 'f', item: 'f' }] }] },
    ]);
    expect(body).toContain('region-columns');
    expect(body).toContain('<h2>Virtues</h2>');
    expect(body).toContain('<h2>Flaws</h2>');
    expect(body).toContain('data-testid="row-v"');
    expect(body).toContain('data-testid="row-f"');
  });

  it('renders the Virtues column before the Flaws column, in the given order', () => {
    const body = html([
      { key: 'virtues', title: 'Virtues', groups: [{ key: 'g', rows: [] }] },
      { key: 'flaws', title: 'Flaws', groups: [{ key: 'g', rows: [] }] },
    ]);
    expect(body.indexOf('Virtues')).toBeLessThan(body.indexOf('Flaws'));
  });
});
