import { beforeEach, describe, expect, it, vi } from 'vitest';
import { render } from 'svelte/server';

import type { EffectiveScores, Entity, LifeSection, LocalizedRuleset } from '../types';

// G3: the life timeline — the engine's `EffectiveScores.life_timeline`, one entry per
// span, oldest first, each stating its ages and filled/earned figures IN WORDS (the
// fill bar beside them is decoration, never the only channel). Nothing here is
// computed: every number is the engine's. Rendered with `svelte/server`.
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
import LifeTimeline from './LifeTimeline.svelte';

// The wire shapes, written out as the engine serializes them
// (`life_timeline.rs`): tagged `kind`s, never a raw label.
type Wire = Record<string, unknown>;

function entry(
  kind: Wire,
  [start_age, end_age]: [number, number],
  capacity: Wire,
  filled_xp: number,
  filled_spell_levels = 0,
): Wire {
  return {
    kind,
    start_age,
    end_age,
    years: end_age - start_age,
    capacity,
    filled_xp,
    filled_spell_levels,
  };
}
const fixed = (section: string): Wire => ({ kind: 'fixed', section });
const named = (index: number): Wire => ({ kind: 'named', index });
const remaining: Wire = { kind: 'remaining' };
const xpOnly = (xp: number): Wire => ({ kind: 'separate', xp, spell_levels: 0 });
const separate = (xp: number, spell_levels: number): Wire => ({
  kind: 'separate',
  xp,
  spell_levels,
});
const shared = (points: number): Wire => ({ kind: 'shared', points });

function install(timeline: Wire | null, sections: LifeSection[] = []): void {
  store.ruleset = {
    ruleset: {
      id: 'test',
      version: '1',
      point_items: {},
      type_profiles: {},
      abilities: {},
      magnitude_points: { free: 0, minor: 1, major: 3 },
      ability_category_order: ['general'],
      art_type_order: ['technique', 'form'],
    },
    i18n: { 'virtue.educated': { name: 'Educated' } },
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
    ability_funding: 'life_stages',
    life_stages: { life_sections_enabled: true, life_sections: sections },
    saga_year: 1220,
    art_scores: [],
    personality_traits: [],
    reputations: [],
    spells: [],
    age: 35,
  };
  store.effective = { life_timeline: timeline } as unknown as EffectiveScores;
}

/** A companion of 35: childhood, two named sections (the second unnamed), Remaining. */
function companionTimeline(overrun_years = 0, outside: Wire[] = []): Wire {
  return {
    entries: [
      entry(fixed('childhood'), [0, 5], xpOnly(120), 30),
      entry(named(0), [5, 10], xpOnly(75), 75),
      entry(named(1), [10, 20], xpOnly(150), 125),
      entry(remaining, [20, 35], xpOnly(225), 0),
    ],
    overrun_years,
    outside,
  };
}

/** Darius (ArMDE:2554, :2556): the magus shape, with a shared post-Gauntlet span. */
function magusTimeline(): Wire {
  return {
    entries: [
      entry(fixed('childhood'), [0, 5], xpOnly(120), 0),
      entry(fixed('later_life'), [5, 10], xpOnly(75), 40),
      entry(fixed('apprenticeship'), [10, 25], separate(240, 120), 240, 120),
      entry(named(0), [25, 33], shared(240), 100, 40),
      entry(remaining, [33, 34], shared(0), 0, 0),
    ],
    overrun_years: 0,
    outside: [],
  };
}

const squire: LifeSection[] = [
  { name: 'Squire', years: 5 },
  { name: '  ', years: 10 },
];

function html(): string {
  return render(LifeTimeline, { props: {} }).body;
}

function openTag(body: string, testid: string): string {
  const match = new RegExp(`<[^>]*data-testid="${testid}"[^>]*>`, 'i').exec(body);
  expect(match, `no element with data-testid="${testid}"`).not.toBeNull();
  return match![0];
}

/** The text content of the element carrying `testid`, tags stripped. */
function text(body: string, testid: string): string {
  const start = body.search(new RegExp(`<[^>]*data-testid="${testid}"`));
  expect(start, `no element with data-testid="${testid}"`).not.toBe(-1);
  const tag = /^<(\w+)/.exec(body.slice(start))![1];
  const end = body.indexOf(`</${tag}>`, start);
  return body
    .slice(start, end)
    .replace(/<[^>]*>/g, '')
    .replace(/\s+/g, ' ')
    .trim();
}

function has(body: string, testid: string): boolean {
  return new RegExp(`data-testid="${testid}"`).test(body);
}

beforeEach(() => {
  store.lang = 'en';
});

describe('LifeTimeline (G3)', () => {
  it('renders nothing when the engine draws no timeline', () => {
    install(null);
    expect(
      html()
        .replace(/<!--[\s\S]*?-->/g, '')
        .trim(),
    ).toBe('');
    store.effective = null;
    expect(
      html()
        .replace(/<!--[\s\S]*?-->/g, '')
        .trim(),
    ).toBe('');
  });

  it('is an ordered list named through Fluent, one item per span', () => {
    install(companionTimeline(), squire);
    const body = html();
    const list = openTag(body, 'life-timeline');
    expect(list).toMatch(/^<ol\b/);
    expect(list).toMatch(/aria-label="Life timeline"/);
    for (const i of [0, 1, 2, 3]) expect(has(body, `life-timeline-entry-${i}`)).toBe(true);
    expect(has(body, 'life-timeline-entry-4')).toBe(false);
  });

  it('states each span’s label, ages and filled / earned experience in words', () => {
    install(companionTimeline(), squire);
    const body = html();
    expect(text(body, 'life-timeline-entry-0')).toBe('Early childhood (ages 0-5): 30 / 120 XP');
    expect(text(body, 'life-timeline-entry-1')).toBe('Squire (ages 5-10): 75 / 75 XP');
    // A blank name falls back to its position, never to an empty label.
    expect(text(body, 'life-timeline-entry-2')).toBe('Section 2 (ages 10-20): 125 / 150 XP');
    expect(text(body, 'life-timeline-entry-3')).toBe('Remaining (ages 20-35): 0 / 225 XP');
  });

  it('states a magus’s apprenticeship as two numbers and its later years as shared points', () => {
    install(magusTimeline(), [{ name: 'Study', years: 8 }]);
    const body = html();
    expect(text(body, 'life-timeline-entry-1')).toBe('Later life (ages 5-10): 40 / 75 XP');
    expect(text(body, 'life-timeline-entry-2')).toBe(
      'Apprenticeship (ages 10-25): 240 / 240 XP, 120 / 120 levels of spells',
    );
    expect(text(body, 'life-timeline-entry-3')).toBe(
      'Study (ages 25-33): 100 XP + 40 level(s) of spells / 240 points',
    );
  });

  it('names an Abandoned Apprentice’s cut-short training', () => {
    install({
      entries: [entry(fixed('truncated_training'), [10, 15], separate(80, 40), 80, 35)],
      overrun_years: 0,
      outside: [],
    });
    expect(text(html(), 'life-timeline-entry-0')).toBe(
      'Abandoned apprenticeship (ages 10-15): 80 / 80 XP, 35 / 40 levels of spells',
    );
  });

  it('never renders a raw kind or section slug', () => {
    install(magusTimeline(), [{ name: 'Study', years: 8 }]);
    const visible = html()
      .replace(/<[^>]*>/g, ' ')
      .replace(/\s+/g, ' ');
    for (const slug of ['later_life', 'apprenticeship', 'remaining', 'named', 'shared']) {
      expect(visible).not.toContain(slug);
    }
  });

  it('draws each fill as decoration only — hidden from assistive tech, the words carry it', () => {
    install(companionTimeline(), squire);
    const body = html();
    const fills = [...body.matchAll(/<[^>]*class="[^"]*life-timeline-fill[^"]*"[^>]*>/g)].map(
      (m) => m[0],
    );
    expect(fills).toHaveLength(4);
    for (const fill of fills) expect(fill).toMatch(/aria-hidden="true"/);
    // Squire is full, Remaining empty.
    expect(fills[1]).toMatch(/width:\s*100%/);
    expect(fills[3]).toMatch(/width:\s*0%/);
  });
});

describe('LifeTimeline notes (G3)', () => {
  it('announces sections that run past the character’s age', () => {
    install(companionTimeline(3), squire);
    const body = html();
    expect(openTag(body, 'life-timeline-overrun')).toMatch(/role="status"/);
    expect(text(body, 'life-timeline-overrun')).toBe(
      "The sections run 3 year(s) past the character's age.",
    );
  });

  it('says nothing about an overrun when there is none', () => {
    install(companionTimeline(0), squire);
    expect(has(html(), 'life-timeline-overrun')).toBe(false);
  });

  it('shows a grant on top of the years as a named chip outside the timeline', () => {
    install(companionTimeline(0, [{ item: 'virtue.educated', amount: 50, used: 20 }]), squire);
    const body = html();
    expect(openTag(body, 'life-timeline-outside')).toMatch(
      /aria-label="Experience beyond the years"/,
    );
    expect(text(body, 'life-timeline-outside-0')).toBe('Educated: 20 / 50 XP');
  });

  it('shows no chip list when every experience point comes from the years', () => {
    install(companionTimeline(), squire);
    expect(has(html(), 'life-timeline-outside')).toBe(false);
  });
});

describe('LifeTimeline in German (G3)', () => {
  it('labels the spans in German', () => {
    store.lang = 'de';
    install(companionTimeline(2), squire);
    const body = html();
    expect(text(body, 'life-timeline-entry-0')).toBe('Frühe Kindheit (Alter 0-5): 30 / 120 EP');
    expect(text(body, 'life-timeline-entry-2')).toBe('Abschnitt 2 (Alter 10-20): 125 / 150 EP');
    expect(text(body, 'life-timeline-entry-3')).toBe('Übrige Jahre (Alter 20-35): 0 / 225 EP');
    expect(text(body, 'life-timeline-overrun')).toBe(
      'Die Abschnitte reichen 2 Jahr(e) über das Alter des Charakters hinaus.',
    );
  });
});
