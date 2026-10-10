import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { render } from 'svelte/server';

// `ssr`: the consent prompt's rendered markup (M6.6) — its dialog semantics, its
// two answers and its words in both locales. Focus and keys are in
// `UpdatePrompt.client.test.ts`.
vi.mock('../ipc', () => ({
  validateEntity: vi.fn().mockResolvedValue({ issues: [] }),
}));

import { store } from '../state.svelte';
import UpdatePrompt from './UpdatePrompt.svelte';

function html(): string {
  return render(UpdatePrompt).body;
}

function openTag(body: string, testid: string): string | null {
  return new RegExp(`<[^>]*data-testid="${testid}"[^>]*>`, 'i').exec(body)?.[0] ?? null;
}

function textOf(body: string, testid: string): string {
  const match = new RegExp(`<(\\w+)[^>]*data-testid="${testid}"[^>]*>([\\s\\S]*?)</\\1>`, 'i').exec(
    body,
  );
  if (!match) throw new Error(`no element with data-testid="${testid}"`);
  return match[2]
    .replace(/<[^>]*>/g, ' ')
    .replace(/\s+/g, ' ')
    .trim();
}

beforeEach(() => {
  store.lang = 'en';
  store.updateCheck.promptOpen = true;
});

afterEach(() => {
  store.lang = 'en';
  store.updateCheck.promptOpen = false;
});

describe('the update prompt', () => {
  it('renders nothing while closed', () => {
    store.updateCheck.promptOpen = false;

    expect(openTag(html(), 'update-prompt')).toBeNull();
  });

  it('is a labelled modal dialog', () => {
    const body = html();

    expect(openTag(body, 'update-prompt')).not.toBeNull();
    const dialog = /<[^>]*role="dialog"[^>]*>/.exec(body)?.[0] ?? '';
    expect(dialog).toMatch(/aria-modal="true"/);
    expect(dialog).toMatch(/aria-labelledby="update-prompt-title"/);
    expect(body).toMatch(/id="update-prompt-title"[^>]*>Check for updates\?</);
  });

  it('says what the check does and offers two answers', () => {
    const body = html();

    expect(body).toContain(
      'At each start, ask GitHub whether a newer version exists. Nothing is downloaded.',
    );
    expect(textOf(body, 'update-prompt-enable')).toBe('Check at start');
    expect(textOf(body, 'update-prompt-decline')).toBe("Don't check");
  });

  it('marks neither answer as destructive', () => {
    const body = html();

    expect(openTag(body, 'update-prompt-enable')).not.toBeNull();
    expect(openTag(body, 'update-prompt-decline')).not.toBeNull();
    expect(openTag(body, 'update-prompt-enable')).not.toMatch(/danger/);
    expect(openTag(body, 'update-prompt-decline')).not.toMatch(/danger/);
  });

  it('speaks German', () => {
    store.lang = 'de';
    const body = html();

    expect(body).toMatch(/id="update-prompt-title"[^>]*>Nach Updates suchen\?</);
    expect(textOf(body, 'update-prompt-enable')).toBe('Beim Start suchen');
    expect(textOf(body, 'update-prompt-decline')).toBe('Nicht suchen');
  });
});
