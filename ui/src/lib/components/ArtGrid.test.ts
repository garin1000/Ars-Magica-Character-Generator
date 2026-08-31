import { beforeEach, describe, expect, it } from 'vitest';
import { render } from 'svelte/server';

import type { EffectiveScores, Entity, LocalizedRuleset } from '../types';

import { SCHEMA_VERSION, store } from '../state.svelte';
import ArtGrid from './ArtGrid.svelte';

const CREO = 'art.creo';
const ANIMAL = 'art.animal';
const CORPUS = 'art.corpus';
const IGNEM = 'art.ignem';

/** One Technique and three Forms — enough Forms to exercise the two-column
 * split (`half = Math.ceil(forms.length / 2)`): 3 Forms split 2/1. */
function installRuleset(): void {
  store.ruleset = {
    ruleset: {
      id: 'test',
      version: '1',
      point_items: {},
      type_profiles: {},
      abilities: {},
      art_advancement: [
        { score: 1, total_xp: 5 },
        { score: 5, total_xp: 75 },
      ],
      arts: {
        [CREO]: { id: CREO, art_type: 'technique' },
        [ANIMAL]: { id: ANIMAL, art_type: 'form' },
        [CORPUS]: { id: CORPUS, art_type: 'form' },
        [IGNEM]: { id: IGNEM, art_type: 'form' },
      },
      magnitude_points: { free: 0, minor: 1, major: 3 },
      ability_category_order: ['general'],
      art_type_order: ['technique', 'form'],
    },
    i18n: {
      [CREO]: { name: 'Creo', abbreviation: 'Cr' },
      [ANIMAL]: { name: 'Animal', abbreviation: 'An' },
      [CORPUS]: { name: 'Corpus', abbreviation: 'Co' },
      [IGNEM]: { name: 'Ignem', abbreviation: 'Ig' },
    },
  } as unknown as LocalizedRuleset;
}

function resetEntity(): void {
  store.entity = {
    schema_version: SCHEMA_VERSION,
    ruleset: { id: 'test', version: '1' },
    entity_kind: 'character',
    type_id: 'magus',
    selections: [],
    characteristics: {} as Entity['characteristics'],
    characteristic_descriptions: {},
    ability_scores: [],
    xp_pool: 0,
    ability_funding: 'pool',
    art_scores: [],
    personality_traits: [],
    reputations: [],
  };
  store.effective = null;
}

function html(): string {
  return render(ArtGrid, { props: {} }).body;
}

/** The start tag of the element whose attributes contain `marker`. */
function tagContaining(body: string, marker: string): string {
  const tag = new RegExp(`<[a-z]+[^>]*${marker}[^>]*>`).exec(body);
  if (!tag) throw new Error(`no element matching ${marker}`);
  return tag[0];
}

/** The text content of the (non-void) element whose start tag contains `marker`. */
function textOf(body: string, marker: string): string {
  const match = new RegExp(`<[a-z]+[^>]*${marker}[^>]*>([\\s\\S]*?)</`).exec(body);
  if (!match) throw new Error(`no element matching ${marker}`);
  return match[1];
}

/** Fluent isolates interpolated values with bidi marks; strip them for text matching. */
function clean(text: string): string {
  return text.replace(/[⁦-⁩]/g, '');
}

beforeEach(() => {
  store.lang = 'en';
  installRuleset();
  resetEntity();
});

// G12 (full-audit test-adequacy): ArtGrid had no test file at all — its
// three-column layout (the Technique/Form split this component alone computes
// from `art_type_order`), the score spinners, and the effective-score badge
// were exercised only via the slow e2e layer (`arts.e2e.js`).
describe('ArtGrid columns and layout', () => {
  it('splits the Forms into two columns, book order within each', () => {
    const body = html();
    // Techniques first, then Forms split 2/1 (Math.ceil(3/2) = 2 in column one).
    const creoIdx = body.indexOf('art-score-art.creo');
    const animalIdx = body.indexOf('art-score-art.animal');
    const corpusIdx = body.indexOf('art-score-art.corpus');
    const ignemIdx = body.indexOf('art-score-art.ignem');
    expect(creoIdx).toBeGreaterThanOrEqual(0);
    expect(creoIdx).toBeLessThan(animalIdx);
    expect(animalIdx).toBeLessThan(corpusIdx);
    expect(corpusIdx).toBeLessThan(ignemIdx);
  });

  it('shows the localized Art name and its abbreviation', () => {
    const body = html();
    expect(body).toContain('Creo');
    expect(body).toContain('(Cr)');
  });

  it('renders a score of 0 for every Art with no entity data yet', () => {
    const body = html();
    for (const id of [CREO, ANIMAL, CORPUS, IGNEM]) {
      expect(textOf(body, `data-testid="art-score-${id}"`)).toBe('0');
    }
  });

  it('reads a bought score back from the entity', () => {
    store.entity.art_scores = [{ art: CREO, score: 4 }];
    const body = html();
    expect(textOf(body, 'data-testid="art-score-art.creo"')).toBe('4');
  });
});

describe('ArtGrid score spinners', () => {
  it('disables decrement at 0', () => {
    const body = html();
    expect(tagContaining(body, 'data-testid="art-dec-art.creo"')).toContain('disabled');
  });

  it('enables decrement once the score is above 0', () => {
    store.entity.art_scores = [{ art: CREO, score: 1 }];
    const body = html();
    expect(tagContaining(body, 'data-testid="art-dec-art.creo"')).not.toContain('disabled');
  });

  it('disables increment at the advancement table max', () => {
    store.entity.art_scores = [{ art: CREO, score: 5 }];
    const body = html();
    expect(tagContaining(body, 'data-testid="art-inc-art.creo"')).toContain('disabled');
  });

  it('enables increment below the max', () => {
    store.entity.art_scores = [{ art: CREO, score: 4 }];
    const body = html();
    expect(tagContaining(body, 'data-testid="art-inc-art.creo"')).not.toContain('disabled');
  });
});

describe('ArtGrid effective-score badge', () => {
  it('renders no badge when the engine reports no bonus', () => {
    store.entity.art_scores = [{ art: CREO, score: 3 }];
    expect(html()).not.toContain('data-testid="art-eff-art.creo"');
  });

  it('renders the bought+bonus total when the engine reports a bonus', () => {
    store.entity.art_scores = [{ art: CREO, score: 3 }];
    store.effective = {
      art_bonuses: [{ art: CREO, bonus: 2 }],
    } as unknown as EffectiveScores;
    const body = html();
    expect(body).toContain('data-testid="art-eff-art.creo"');
    expect(clean(textOf(body, 'data-testid="art-eff-art.creo"'))).toContain('5');
  });
});
