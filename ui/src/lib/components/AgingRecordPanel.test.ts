import { beforeEach, describe, expect, it, vi } from 'vitest';
import { render } from 'svelte/server';

import {
  CHARACTERISTICS,
  type EffectiveScores,
  type Entity,
  type LocalizedRuleset,
} from '../types';

// The panel reads the shared store singleton (the entity's apparent age, aging
// points and aging log, the engine's Decrepitude score) and the Fluent bundle.
// The store schedules a debounced revalidate over the Tauri IPC bridge; mock the
// bridge so nothing reaches a backend. Harness mirrors LifeStagePanel.test.ts.
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
  agingPreview: vi.fn(),
  agingApply: vi.fn(),
  agingRevert: vi.fn(),
}));

import { SCHEMA_VERSION, store } from '../state.svelte';
import AgingRecordPanel from './AgingRecordPanel.svelte';

function resetEntity(): void {
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
    ability_funding: 'pool',
    art_scores: [],
    personality_traits: [],
    reputations: [],
    spells: [],
  };
  store.effective = null;
}

/** The engine's derived Decrepitude score, which the read-out reports. */
function setDecrepitude(score: number): void {
  store.effective = {
    ...(store.effective ?? {}),
    decrepitude_score: score,
  } as unknown as EffectiveScores;
}

/** Render the panel to an HTML string (node env, no DOM). */
function html(): string {
  return render(AgingRecordPanel, { props: {} }).body;
}

/** Whether any element carries the exact data-testid. */
function has(body: string, testid: string): boolean {
  return new RegExp(`data-testid="${testid}"`).test(body);
}

/** The visible text of the element carrying a data-testid, tags stripped. */
function text(body: string, testid: string): string {
  const opening = new RegExp(`<([a-z]+)[^>]*data-testid="${testid}"[^>]*>`, 'i').exec(body);
  if (!opening) throw new Error(`no element with data-testid="${testid}"`);
  const inner = body.slice(opening.index + opening[0].length);
  const closing = new RegExp(`</${opening[1]}>`, 'i').exec(inner);
  return (closing ? inner.slice(0, closing.index) : inner)
    .replace(/<[^>]*>/g, ' ')
    .replace(/[⁨⁩]/g, '')
    .replace(/\s+/g, ' ')
    .trim();
}

/**
 * A minimal localized ruleset, so a logged Crisis row resolves to its name in
 * `rules/i18n/<lang>/aging.json` rather than printing its id.
 */
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
    i18n: { 'crisis.minor_illness': { name: 'Minor illness' } },
  } as unknown as LocalizedRuleset;
}

beforeEach(() => {
  vi.useFakeTimers();
  store.lang = 'en';
  resetEntity();
});

describe('AgingRecordPanel (slice 6b6b)', () => {
  it('keeps every recorded-aging control the details tab had', () => {
    // The whole recorded-aging surface, as CharacterDetails carried it before the
    // extraction: an entry in the log, aging points, a Decrepitude score.
    store.entity.apparent_age = 38;
    store.entity.aging_points = { sta: 2 };
    store.entity.aging_log = [{ year: 1220, effect: 'A hard winter' }];
    setDecrepitude(1);
    const body = html();

    expect(has(body, 'aging-record')).toBe(true);
    expect(has(body, 'apparent-age-input')).toBe(true);
    expect(has(body, 'decrepitude-readout')).toBe(true);
    expect(has(body, 'decrepitude-effect-input')).toBe(true);
    expect(has(body, 'aging-points-list')).toBe(true);
    for (const characteristic of CHARACTERISTICS) {
      expect(has(body, `aging-points-${characteristic}`)).toBe(true);
    }
    // manual-testing-findings #21: the "drops are applied automatically" note is not
    // a control and is gone; every control above it stays.
    expect(has(body, 'aging-points-note')).toBe(false);
    expect(has(body, 'aging-log-list')).toBe(true);
    expect(has(body, 'aging-log-year-0')).toBe(true);
    expect(has(body, 'aging-log-effect-0')).toBe(true);
    expect(has(body, 'aging-log-remove-0')).toBe(true);
    expect(has(body, 'aging-log-add')).toBe(true);
  });

  // manual-testing-findings-2026-09-03 #22/#24: the four accumulated-record blocks
  // (apparent age, the Decrepitude read-out, its narrative, the Aging Points) were
  // four separate items of the `.character-details` grid, auto-placed into whatever
  // cells were left around the tall roll calculator — the "band of columns". They are
  // one stage now, laid out across its own width (app.css `.aging-state`). And the
  // per-year log leads it: the log is the thing a player has just written to and the
  // one that must be reachable without scrolling, while these are its running totals.
  it('leads with the per-year log, then the accumulated totals as one block', () => {
    store.entity.aging_log = [{ year: 1220, effect: 'A hard winter' }];
    const body = html();
    expect(has(body, 'aging-state')).toBe(true);
    const at = (testid: string): number => {
      const index = body.indexOf(`data-testid="${testid}"`);
      expect(index, `no element with data-testid="${testid}"`).toBeGreaterThanOrEqual(0);
      return index;
    };
    expect(at('aging-log-block')).toBeLessThan(at('aging-state'));
    // Every accumulated read-out sits inside that one block, not scattered beside it.
    for (const testid of ['apparent-age-input', 'decrepitude-effect-input', 'aging-points-list']) {
      expect(at('aging-state')).toBeLessThan(at(testid));
    }
  });

  // guided-creation-review-2026-08 #26: `decrepitude_effect` is the cumulative
  // overall aging/decrepitude narrative (RULES.md, Core Rules.md:16563-16577), so it
  // grows over a character's life — its analogue `warping_effect` has always been a
  // `<textarea rows="3">` (CharacterDetails.svelte). A single-line input was simply
  // the wrong control for the data.
  it('renders the decrepitude effect as a textarea, like the warping effect', () => {
    store.entity.decrepitude_effect = 'Stooped, and slow to rise on cold mornings.';
    const body = html();
    const opening = /<([a-z]+)[^>]*data-testid="decrepitude-effect-input"[^>]*>/i.exec(body);
    expect(opening).not.toBeNull();
    expect(opening![1]).toBe('textarea');
    // Three rows, matching the warping field it is the analogue of.
    expect(opening![0]).toMatch(/rows="3"/);
    // The stored narrative is what the control shows.
    expect(text(body, 'decrepitude-effect-input')).toContain('cold mornings');
  });

  it('says the log is empty when no year is recorded', () => {
    const body = html();
    expect(has(body, 'aging-log-empty')).toBe(true);
    expect(has(body, 'aging-log-year-0')).toBe(false);
    // Decrepitude is hidden at 0 — a score of zero is not a state to report.
    expect(has(body, 'decrepitude-readout')).toBe(false);
  });

  it('reads a resolved Crisis back off the log entry that recorded it', () => {
    // The four crisis fields are the whole record of what the Crisis Table was
    // asked and what it answered (`:16621`, `:16624-16632`). Without them on
    // screen a resolved Crisis is invisible the moment the calculator is closed.
    installRuleset();
    store.entity.aging_log = [
      {
        year: 1220,
        age: 40,
        effect: '',
        die: 9,
        total: 13,
        crisis: true,
        crisis_die: 10,
        crisis_total: 15,
        crisis_row: 'crisis.minor_illness',
        crisis_severity: 'minor',
      },
    ];
    const body = html();
    const crisis = text(body, 'aging-log-crisis-0');
    // The row's text is rules data keyed by its id; the severity goes through
    // Fluent. Neither is ever rendered as its slug.
    expect(crisis).toContain('Minor illness');
    expect(crisis).toContain('15');
    expect(crisis).toContain('10');
    expect(crisis).not.toContain('crisis.minor_illness');
    expect(body).not.toContain('crisis_severity');
  });

  it('tells a Crisis owed and unrolled apart from a resolved one', () => {
    // Three states, not two: no Crisis, one the table demanded that nobody has
    // rolled, and one resolved. A `crisis` with no row is the middle state.
    installRuleset();
    store.entity.aging_log = [
      { year: 1220, age: 40, effect: '', die: 9, total: 13, crisis: true },
      { year: 1221, age: 41, effect: '', die: 4, total: 8 },
    ];
    const body = html();
    expect(text(body, 'aging-log-crisis-0').length).toBeGreaterThan(0);
    expect(has(body, 'aging-log-crisis-1')).toBe(false);
  });

  // S13 (full-audit a11y): the eight Aging Points inputs were named only by a
  // plain `<span class="char-name">`, with no programmatic association at all —
  // a screen-reader user tabbing through them heard eight anonymous spinbuttons.
  // Each span now gets an `id` and its neighbouring input an `aria-labelledby`
  // pointing at it, so the accessible name matches the visible Characteristic
  // name shown beside it.
  it('associates each Aging Points input with its visible Characteristic label (S13)', () => {
    const body = html();
    for (const characteristic of CHARACTERISTICS) {
      const input = new RegExp(
        `<input[^>]*data-testid="aging-points-${characteristic}"[^>]*>`,
      ).exec(body);
      expect(input, `no input for ${characteristic}`).not.toBeNull();
      const labelledby = /aria-labelledby="([^"]+)"/.exec(input![0]);
      expect(labelledby, `${characteristic} input has no aria-labelledby`).not.toBeNull();
      const labelTag = new RegExp(`<span[^>]*id="${labelledby![1]}"[^>]*>([^<]*)<`).exec(body);
      expect(labelTag, `no span with id ${labelledby![1]}`).not.toBeNull();
      expect(labelTag![1]).toBe(store.t(`characteristic-${characteristic}`));
    }
  });

  // manual-testing-findings-2026-09-03 #23: the × on an engine-recorded row no longer
  // merely deletes the row — it hands the year to `aging::revert_year`, which takes
  // the points and the apparent age back off with it. The accessible name has to
  // say so, or the control announces itself as something it is not. It reuses
  // `aging-revert`, the wording the calculator's own take-back button carries, so
  // the two controls read the same and no new string is invented.
  it('names the remove button after what it does to the row it sits on (#23)', () => {
    store.entity.aging_log = [
      { year: 1220, age: 40, effect: '', die: 9, total: 13, points: { sta: 1 } },
      { year: 1219, effect: 'A hard winter' },
    ];
    const body = html();
    const label = (testid: string): string => {
      const opening = new RegExp(`<button[^>]*data-testid="${testid}"[^>]*>`).exec(body);
      expect(opening, `no button ${testid}`).not.toBeNull();
      return /aria-label="([^"]*)"/.exec(opening![0])?.[1] ?? '';
    };
    // The recorded year: taken back, exactly as the calculator's button says it.
    expect(label('aging-log-remove-0').replace(/[⁨⁩]/g, '')).toBe(
      store.t('aging-revert', { age: '40' }).replace(/[⁨⁩]/g, ''),
    );
    // The hand-written row has nothing mechanical to undo, so it is still removed.
    expect(label('aging-log-remove-1').replace(/[⁨⁩]/g, '')).toBe(
      store.t('remove-item', { name: 'A hard winter' }).replace(/[⁨⁩]/g, ''),
    );
  });

  // guided-creation-review-2026-08 #27: the engine records the whole roll — the die,
  // the total, the points it awarded and whether the apparent age advanced — and the
  // row showed none of it, leaving the player to write by hand what the roll had just
  // done. The summary is RENDERED from the structured fields, never stored: prose in
  // the save would freeze one language into the file.
  it('reads the roll back off an engine-recorded row (#27)', () => {
    store.entity.aging_log = [
      {
        year: 1220,
        age: 40,
        effect: '',
        die: 9,
        total: 13,
        points: { qik: 1, sta: 2 },
        apparent_age_increased: true,
      },
    ];
    const body = html();
    const summary = text(body, 'aging-log-summary-0');

    // The roll that was made: both figures, worded as the Crisis line words its own.
    expect(summary).toContain('13');
    expect(summary).toContain('9');
    // Every awarded Characteristic, in words and with its count — never the slug.
    expect(summary).toContain('Quickness');
    expect(summary).toContain('Stamina');
    expect(summary).toContain('1 Aging Point in');
    expect(summary).toContain('2 Aging Points in');
    expect(summary).not.toContain('qik');
    expect(summary).not.toContain('sta');
    // And the year of apparent age the roll cost.
    expect(summary).toContain('Apparent age increases by one year.');

    // NOTHING IS WRITTEN BACK. The summary is display only; the stored free text is
    // still the player's, still empty.
    expect(store.entity.aging_log[0].effect).toBe('');
  });

  it('says so when a roll awarded nothing at all (#27)', () => {
    // A good roll is a real outcome, not a blank row: it awarded no points and did
    // not advance the apparent age, and both halves are stated rather than left to
    // silence — silence is indistinguishable from "not recorded".
    store.entity.aging_log = [
      { year: 1220, age: 40, effect: '', die: 1, total: 2, apparent_age_increased: false },
    ];
    const summary = text(html(), 'aging-log-summary-0');
    expect(summary).toContain('2');
    expect(summary).toContain('No Aging Points.');
    expect(summary).toContain('Apparent age does not advance.');
  });

  it('leaves a hand-written row without a summary (#27)', () => {
    // Nothing was rolled, so there is nothing to read back — `effect` stays the whole
    // record of such a row.
    store.entity.aging_log = [{ year: 1219, effect: 'A hard winter' }];
    const body = html();
    expect(has(body, 'aging-log-summary-0')).toBe(false);
    expect(has(body, 'aging-log-effect-0')).toBe(true);
  });

  it('keeps the Crisis on its own line after the roll summary (#27)', () => {
    // Two rolls against two tables, resolved in that order (`:16619`: the points
    // first, then the Crisis Table), so two lines rather than one run-on sentence.
    installRuleset();
    store.entity.aging_log = [
      {
        year: 1220,
        age: 40,
        effect: '',
        die: 9,
        total: 13,
        points: { sta: 1 },
        apparent_age_increased: true,
        crisis: true,
        crisis_die: 10,
        crisis_total: 15,
        crisis_row: 'crisis.minor_illness',
        crisis_severity: 'minor',
      },
    ];
    const body = html();
    expect(text(body, 'aging-log-summary-0')).toContain('Stamina');
    expect(text(body, 'aging-log-crisis-0')).toContain('Minor illness');
    expect(body.indexOf('aging-log-summary-0')).toBeLessThan(body.indexOf('aging-log-crisis-0'));
  });

  it('offers the free text as an optional note once a summary states the roll (#27)', () => {
    // The box stays — the player's colour ("a hard winter") is worth keeping — but on
    // an engine-recorded row it must stop ASKING for what the summary already says.
    store.entity.aging_log = [
      { year: 1220, age: 40, effect: '', die: 9, total: 13 },
      { year: 1219, effect: 'A hard winter' },
    ];
    const body = html();
    const placeholder = (testid: string): string => {
      const opening = new RegExp(`<input[^>]*data-testid="${testid}"[^>]*>`).exec(body);
      expect(opening, `no input ${testid}`).not.toBeNull();
      return /placeholder="([^"]*)"/.exec(opening![0])?.[1] ?? '';
    };
    expect(placeholder('aging-log-effect-0')).toBe(store.t('aging-log-note-placeholder'));
    expect(placeholder('aging-log-effect-1')).toBe(store.t('aging-log-effect-placeholder'));
  });

  it('labels every control through Fluent, never as a raw slug', () => {
    const body = html();
    expect(body).toContain('Apparent age');
    expect(body).toContain('Aging');
    // Characteristic rows are named in words, never by their slugs.
    expect(body).toContain('Stamina');
    expect(body).not.toMatch(/>\s*sta\s*</);
    expect(body).not.toMatch(/>\s*apparent_age\s*</);
  });
});
