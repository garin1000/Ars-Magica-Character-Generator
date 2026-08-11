import { beforeEach, describe, expect, it, vi } from 'vitest';
import { render } from 'svelte/server';

import type {
  EffectiveScores,
  Entity,
  EntityTypeProfile,
  LifeStagePlan,
  LifeStageRules,
  LocalizedRuleset,
} from '../types';

// The panel reads the shared store singleton (the entity's life-stage plan, the
// loaded ruleset's life-stage rules, the engine's age cap) and the Fluent bundle.
// The store schedules a debounced revalidate over the Tauri IPC bridge; mock the
// bridge so nothing reaches a backend. Harness mirrors XpBar.test.ts.
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
}));

import { SCHEMA_VERSION, store } from '../state.svelte';
import LifeStagePanel from './LifeStagePanel.svelte';

/** The life-stage rules as `rules/core/life_stages.json` ships them. */
function lifeStageRules(): LifeStageRules {
  return {
    childhood: {
      years: 5,
      native_language_ability: 'ability.living_language',
      native_language_xp: 75,
      spread_xp: 45,
      spread_abilities: ['ability.athletics', 'ability.awareness'],
    },
    later_life: { xp_per_year: 15 },
  };
}

function profile(id: string, isMagus: boolean): EntityTypeProfile {
  return {
    id,
    budget: { virtue_points: 10, flaw_points: 10 },
    is_magus: isMagus,
    gift_policy: isMagus ? 'required' : 'forbidden',
    creation_phases: [],
  } as unknown as EntityTypeProfile;
}

/**
 * Install a minimal localized ruleset. `rules` is `null` for a ruleset shipping no
 * life-stage file at all — the case the panel must render nothing for.
 */
function installRuleset(
  rules: LifeStageRules | null = lifeStageRules(),
  typeProfile: EntityTypeProfile = profile('companion', false),
): void {
  store.ruleset = {
    ruleset: {
      id: 'test',
      version: '1',
      point_items: {},
      type_profiles: { [typeProfile.id]: typeProfile },
      abilities: {},
      magnitude_points: { free: 0, minor: 1, major: 3 },
      ability_category_order: ['general', 'academic', 'arcane', 'martial', 'supernatural'],
      art_type_order: ['technique', 'form'],
      ...(rules ? { life_stages: rules } : {}),
    },
    i18n: {},
  } as unknown as LocalizedRuleset;
}

function resetEntity(typeId = 'companion'): void {
  store.entity = {
    schema_version: SCHEMA_VERSION,
    ruleset: { id: 'test', version: '1' },
    entity_kind: 'character',
    type_id: typeId,
    selections: [],
    characteristics: {} as Entity['characteristics'],
    characteristic_descriptions: {},
    ability_scores: [],
    xp_pool: 0,
    art_scores: [],
    personality_traits: [],
    reputations: [],
    spells: [],
  };
  store.effective = null;
}

/** Put the entity in guided funding: a plan present IS the switch. */
function installPlan(plan: LifeStagePlan = {}): void {
  store.entity.life_stages = plan;
}

/** The engine's age→max-Ability-score cap, the panel's read-only echo. */
function setAgeCap(cap: number | null): void {
  store.effective = { age_ability_cap: cap } as unknown as EffectiveScores;
}

/** Render the panel to an HTML string (node env, no DOM). */
function html(): string {
  return render(LifeStagePanel, { props: {} }).body;
}

/** The single element carrying a given data-testid, with its class attribute. */
function element(body: string, testid: string): { open: string; text: string } {
  const re = new RegExp(`<[^>]*data-testid="${testid}"[^>]*>([\\s\\S]*?)</`, 'i');
  const match = re.exec(body);
  if (!match) throw new Error(`no element with data-testid="${testid}"`);
  const openMatch = new RegExp(`<[^>]*data-testid="${testid}"[^>]*>`, 'i').exec(body);
  return { open: openMatch![0], text: match[1].replace(/<[^>]*>/g, '').trim() };
}

/** Whether any element carries the exact data-testid. */
function has(body: string, testid: string): boolean {
  return new RegExp(`data-testid="${testid}"`).test(body);
}

beforeEach(() => {
  vi.useFakeTimers();
  store.lang = 'en';
  installRuleset();
  resetEntity();
});

describe('LifeStagePanel without life-stage rules (slice 6b3b)', () => {
  it('renders nothing at all for a ruleset shipping no life-stage rules', () => {
    installRuleset(null);
    const body = html();
    // A funding mode the data cannot support must not appear as a dead control.
    expect(has(body, 'life-stage-panel')).toBe(false);
    expect(has(body, 'ability-funding-pool')).toBe(false);
    expect(has(body, 'ability-funding-life_stages')).toBe(false);
    expect(body.replace(/<!--[\s\S]*?-->/g, '').trim()).toBe('');
  });

  it('renders nothing before a ruleset is loaded', () => {
    store.ruleset = null;
    expect(has(html(), 'life-stage-panel')).toBe(false);
  });
});

describe('LifeStagePanel funding switch (slice 6b3b)', () => {
  it('groups the two options in a real fieldset with a legend naming the switch', () => {
    const body = html();
    expect(has(body, 'life-stage-panel')).toBe(true);
    // Native grouping semantics, not a styled div: the legend is what a screen
    // reader announces before either option.
    expect(body).toMatch(/<fieldset/i);
    const legend = /<legend[^>]*>([\s\S]*?)<\/legend>/i.exec(body);
    expect(legend).not.toBeNull();
    expect(legend![1].replace(/<[^>]*>/g, '').trim()).toContain('Source of experience');
  });

  it('offers both funding modes as native radio inputs', () => {
    const body = html();
    expect(element(body, 'ability-funding-pool').open).toMatch(/type="radio"/);
    expect(element(body, 'ability-funding-life_stages').open).toMatch(/type="radio"/);
    // One group, so picking one deselects the other without any JS.
    expect(element(body, 'ability-funding-pool').open).toMatch(/name="[^"]+"/);
    expect(element(body, 'ability-funding-life_stages').open).toMatch(/name="[^"]+"/);
  });

  it('checks the pool radio when the entity carries no plan', () => {
    const body = html();
    expect(element(body, 'ability-funding-pool').open).toMatch(/checked/);
    expect(element(body, 'ability-funding-life_stages').open).not.toMatch(/checked/);
  });

  it('checks the life-stages radio when the entity carries a plan', () => {
    installPlan();
    const body = html();
    expect(element(body, 'ability-funding-life_stages').open).toMatch(/checked/);
    expect(element(body, 'ability-funding-pool').open).not.toMatch(/checked/);
  });

  it('labels each option through Fluent, never as a raw slug', () => {
    const body = html();
    expect(body).toContain('Experience pool');
    expect(body).toContain('Life stages');
    // The mode ids are testids only; nothing renders them as text.
    expect(body).not.toMatch(/>\s*life_stages\s*</);
  });

  it('points each radio at its own hint, so switching is explained before it happens', () => {
    const body = html();
    expect(element(body, 'ability-funding-pool').open).toMatch(
      /aria-describedby="[^"]*ability-funding-pool-hint/,
    );
    expect(element(body, 'ability-funding-life_stages').open).toMatch(
      /aria-describedby="[^"]*ability-funding-life_stages-hint/,
    );
    expect(element(body, 'ability-funding-pool-hint').open).toMatch(
      /id="ability-funding-pool-hint"/,
    );
    expect(element(body, 'ability-funding-life_stages-hint').open).toMatch(
      /id="ability-funding-life_stages-hint"/,
    );
    expect(element(body, 'ability-funding-pool-hint').text).toContain('Enter one total yourself');
  });
});

describe('LifeStagePanel magus restriction (slice 6b3b)', () => {
  it('disables the guided option for a magus and reaches its reason via aria-describedby', () => {
    installRuleset(lifeStageRules(), profile('magus', true));
    resetEntity('magus');
    const body = html();
    const guided = element(body, 'ability-funding-life_stages');
    // A magus earns experience in four periods; apprenticeship is not modelled yet,
    // so the engine refuses the combination and the UI must not offer it.
    expect(guided.open).toMatch(/disabled/);
    expect(guided.open).toMatch(/aria-describedby="[^"]*ability-funding-magus-reason/);
    const reason = element(body, 'ability-funding-magus-reason');
    expect(reason.open).toMatch(/id="ability-funding-magus-reason"/);
    // Never colour or absence alone: the reason is text a screen reader reads out.
    expect(reason.text).toContain('apprenticeship');
    // The pool option stays available — a magus is not locked out of Abilities.
    expect(element(body, 'ability-funding-pool').open).not.toMatch(/disabled/);
  });

  it('leaves the guided option enabled for a companion, with no magus reason node', () => {
    const body = html();
    expect(element(body, 'ability-funding-life_stages').open).not.toMatch(/disabled/);
    expect(has(body, 'ability-funding-magus-reason')).toBe(false);
  });
});

describe('LifeStagePanel guided fields (slice 6b3b)', () => {
  it('renders neither the age nor the native-language field in pool mode', () => {
    setAgeCap(5);
    const body = html();
    expect(has(body, 'life-stage-age-input')).toBe(false);
    expect(has(body, 'life-stage-age-cap')).toBe(false);
    expect(has(body, 'native-language-input')).toBe(false);
  });

  it('renders the age input carrying the entity age under a plan', () => {
    installPlan();
    store.entity.age = 25;
    const body = html();
    const age = element(body, 'life-stage-age-input');
    // Later life is (age - childhood years) × rate, so the guided flow needs the age.
    expect(age.open).toMatch(/type="number"/);
    expect(age.open).toMatch(/value="25"/);
    expect(body).toContain('Age');
  });

  it('renders the native-language field with its localized label and placeholder', () => {
    installPlan({ native_language: 'German' });
    const body = html();
    const input = element(body, 'native-language-input');
    expect(input.open).toMatch(/value="German"/);
    expect(input.open).toMatch(/placeholder="[^"]*German/);
    expect(body).toContain('Native language');
  });

  it('echoes the engine age cap with the shared age-cap wording, announced', () => {
    installPlan();
    store.entity.age = 25;
    setAgeCap(5);
    const cap = element(html(), 'life-stage-age-cap');
    // Reuses the Details tab's key — one wording for one rule, no second string.
    expect(cap.text).toContain('Max Ability score');
    expect(cap.text).toContain('5');
    // It arrives in response to the age edit, so its appearance must be announced.
    expect(cap.open).toMatch(/role="status"/);
  });

  it('omits the age-cap echo when the engine reports no cap', () => {
    installPlan();
    setAgeCap(null);
    expect(has(html(), 'life-stage-age-cap')).toBe(false);
  });
});
