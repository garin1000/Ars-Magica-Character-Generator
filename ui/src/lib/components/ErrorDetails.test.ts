import { beforeEach, describe, expect, it } from 'vitest';
import { render } from 'svelte/server';

import type { AppError } from '../types';
import { store } from '../state.svelte';
import ErrorDetails from './ErrorDetails.svelte';

function html(): string {
  return render(ErrorDetails, { props: {} }).body;
}

beforeEach(() => {
  store.lang = 'en';
  store.error = null;
});

// E4 (open-todos row 25): `AppError::Ruleset` has always carried the engine's
// own integrity diagnostics — each one naming the offending ids and the rulebook
// to open — across IPC, and nothing on the frontend read them. They are the only
// thing that makes a broken `rules/` edit actionable, so they get a disclosure of
// their own beside the localized sentence.
describe('ErrorDetails', () => {
  it('renders nothing when nothing has failed', () => {
    expect(html()).not.toContain('<details');
  });

  it('renders nothing for a failure that carries no ruleset diagnostics', () => {
    store.error = { kind: 'io', message: 'no such file' } as AppError;
    expect(html()).not.toContain('<details');
  });

  // `#reloadRuleset` stores whatever the bridge rejected with, and a rejection
  // shaped `{ kind: 'ruleset' }` with no list at all already occurs in the
  // suite. An empty disclosure promising details and holding none is worse than
  // no disclosure.
  it('renders nothing when a ruleset failure carries an empty message list', () => {
    store.error = { kind: 'ruleset', ruleset_kind: 'integrity', errors: [] } as AppError;
    expect(html()).not.toContain('<details');
  });

  it('collapses the payload behind a native disclosure', () => {
    store.error = {
      kind: 'ruleset',
      ruleset_kind: 'integrity',
      errors: ["unknown prerequisite 'virtue.x' referenced by 'virtue.a'"],
    } as AppError;

    const body = html();
    expect(body).toMatch(/<details[^>]*data-testid="error-details"/);
    expect(body).toContain('<summary');
    // Collapsed by default: the sentence is what the user reads, the payload is
    // what they open when they are the one editing the rules.
    expect(body).not.toMatch(/<details[^>]*\sopen[\s>]/);
  });

  it('labels the disclosure through its Fluent key, never the raw key', () => {
    store.error = {
      kind: 'ruleset',
      ruleset_kind: 'parse',
      errors: ['virtues_flaws.json: expected value at line 1'],
    } as AppError;

    const body = html();
    expect(body).toContain(store.t('error-technical-details'));
    expect(body).not.toContain('error-technical-details');
  });

  it('translates the label with the rest of the UI', () => {
    store.error = {
      kind: 'ruleset',
      ruleset_kind: 'parse',
      errors: ['virtues_flaws.json: expected value at line 1'],
    } as AppError;
    const english = html();

    store.lang = 'de';
    const german = html();

    expect(german).toContain(store.t('error-technical-details'));
    expect(german).not.toBe(english);
  });

  // The payload itself is DELIBERATELY untranslated — see the component's own
  // comment. An integrity failure produces one message per violation, so every
  // one of them has to survive, verbatim and in order.
  it('lists every diagnostic verbatim, one row each, in both languages', () => {
    const errors = [
      "unknown prerequisite 'virtue.x' referenced by 'virtue.a'",
      "unknown prerequisite 'virtue.y' referenced by 'virtue.b'",
    ];
    store.error = { kind: 'ruleset', ruleset_kind: 'integrity', errors } as AppError;

    for (const lang of ['en', 'de'] as const) {
      store.lang = lang;
      const body = html();
      const rows = [...body.matchAll(/<li[^>]*>([\s\S]*?)<\/li>/g)].map((m) => m[1].trim());
      expect(rows).toEqual(errors);
    }
  });
});
