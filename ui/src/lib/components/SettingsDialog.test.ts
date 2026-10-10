import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { render } from 'svelte/server';

// `ssr`: the Settings dialog's update-check checkbox (M6.6, decision 3) as
// rendered markup. Toggling it is in `UpdateCheckToggle.client.test.ts`.
vi.mock('../ipc', () => ({
  validateEntity: vi.fn().mockResolvedValue({ issues: [] }),
}));

import { store } from '../state.svelte';
import SettingsDialog from './SettingsDialog.svelte';

function html(): string {
  return render(SettingsDialog).body;
}

/** The opening tag of the element carrying `testid`, asserted to exist. */
function openTag(body: string, testid: string): string {
  const tag = new RegExp(`<[^>]*data-testid="${testid}"[^>]*>`, 'i').exec(body)?.[0] ?? null;
  expect(tag, `an element with data-testid="${testid}"`).not.toBeNull();
  return tag!;
}

beforeEach(() => {
  store.lang = 'en';
  store.settingsOpen = true;
  store.updateCheck.enabled = false;
});

afterEach(() => {
  store.lang = 'en';
  store.settingsOpen = false;
  store.updateCheck.enabled = false;
});

describe('the update-check setting', () => {
  it('is a labelled checkbox in the dialog', () => {
    const body = html();

    const toggle = openTag(body, 'update-check-toggle');
    expect(toggle).toMatch(/type="checkbox"/);
    expect(body).toMatch(
      /<label[^>]*>[\s\S]*?update-check-toggle[\s\S]*?Check for updates at start/,
    );
  });

  it('reflects the stored consent', () => {
    expect(openTag(html(), 'update-check-toggle')).not.toMatch(/\bchecked\b/);

    store.updateCheck.enabled = true;

    expect(openTag(html(), 'update-check-toggle')).toMatch(/\bchecked\b/);
  });

  it('speaks German', () => {
    store.lang = 'de';

    expect(html()).toContain('Beim Start nach Updates suchen');
  });
});
