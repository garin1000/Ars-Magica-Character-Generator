import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { render } from 'svelte/server';

// `ssr`: the newer-release notice's markup (M6.6, decision 8). Its appearance
// after a real check, and its buttons, are in `UpdateNotice.client.test.ts`.
vi.mock('../ipc', () => ({
  validateEntity: vi.fn().mockResolvedValue({ issues: [] }),
}));

import { store } from '../state.svelte';
import UpdateNotice from './UpdateNotice.svelte';

function html(): string {
  return render(UpdateNotice).body;
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
  store.updateCheck.available = '99.0.0';
  store.updateCheck.dismissed = false;
});

afterEach(() => {
  store.lang = 'en';
  store.updateCheck.available = null;
  store.updateCheck.dismissed = false;
});

describe('the update notice', () => {
  it('renders nothing when no newer release is known', () => {
    store.updateCheck.available = null;

    expect(openTag(html(), 'update-notice')).toBeNull();
  });

  it('renders nothing once dismissed', () => {
    store.updateCheck.dismissed = true;

    expect(openTag(html(), 'update-notice')).toBeNull();
  });

  it('is a polite status naming the version, with its two buttons', () => {
    const body = html();

    expect(openTag(body, 'update-notice')).not.toBeNull();
    expect(openTag(body, 'update-notice')).toMatch(/role="status"/);
    expect(textOf(body, 'update-notice')).toContain('Version 99.0.0 is available.');
    expect(textOf(body, 'update-notice-open')).toBe('Open release page');
    expect(textOf(body, 'update-notice-dismiss')).toBe('Dismiss');
  });

  // The version is passed as a STRING, so Fluent formats no number into it.
  it('speaks German, the version untouched', () => {
    store.lang = 'de';
    store.updateCheck.available = '1000.2.3';
    const body = html();

    expect(openTag(body, 'update-notice')).not.toBeNull();
    expect(textOf(body, 'update-notice')).toContain('Version 1000.2.3 ist verfügbar.');
    expect(textOf(body, 'update-notice-open')).toBe('Release-Seite öffnen');
    expect(textOf(body, 'update-notice-dismiss')).toBe('Ausblenden');
  });
});
