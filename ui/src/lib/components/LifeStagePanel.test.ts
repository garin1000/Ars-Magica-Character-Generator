import { beforeEach, describe, expect, it, vi } from 'vitest';
import { render } from 'svelte/server';

import type {
  EffectiveScores,
  Entity,
  EntityTypeProfile,
  LifeStageBudget,
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
  applyChildhoodPackage: vi.fn(),
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
    // The block the Gauntlet-age placeholder reads its baseline off — "25 years old
    // and just out of apprenticeship" (`:1601`), which is what leaving the field
    // blank now means.
    apprenticeship: {
      default_gauntlet_age: 25,
      minimum_abilities: [],
      recommended_abilities: [],
      recommended_xp: 0,
      xp: 240,
      years: 15,
    },
    post_apprenticeship: {
      points_per_year: 30,
      lab_season_cost: 10,
      max_charged_lab_seasons_per_year: 3,
    },
  };
}

/** The same rules as a ruleset stating no baseline Gauntlet age ships them. */
function rulesWithoutDefaultGauntletAge(): LifeStageRules {
  const rules = lifeStageRules();
  delete rules.apprenticeship!.default_gauntlet_age;
  return rules;
}

/** The same rules as a ruleset predating the post-Gauntlet block ships them. */
function rulesWithoutPostApprenticeship(): LifeStageRules {
  const rules = lifeStageRules();
  delete rules.post_apprenticeship;
  return rules;
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
    ability_funding: 'pool',
    art_scores: [],
    personality_traits: [],
    reputations: [],
    spells: [],
  };
  store.effective = null;
}

/**
 * Put the entity in guided funding. Since schema 16 that takes BOTH the stored mode
 * and the plan: the panel reads `store.abilityFunding`, and a plan on its own is
 * inert data a pool-funded character may legitimately carry.
 */
function installPlan(plan: LifeStagePlan = {}): void {
  store.entity.ability_funding = 'life_stages';
  store.entity.life_stages = plan;
}

/** The engine's age→max-Ability-score cap, the panel's read-only echo. */
function setAgeCap(cap: number | null): void {
  store.effective = {
    ...(store.effective ?? {}),
    age_ability_cap: cap,
  } as unknown as EffectiveScores;
}

/**
 * The engine's life-stage budget, which the post-Gauntlet read-out reports. Shaped
 * as `LifeStageBudget::budget` derives it: `years × 30` less `seasons × 10`, split
 * into levels of spells and the experience left over.
 */
function setLifeStageBudget(budget: LifeStageBudget | null): void {
  store.effective = {
    ...(store.effective ?? {}),
    life_stage: budget,
  } as unknown as EffectiveScores;
}

/** A magus gauntleted at 25 and now `age`, with `seasons` charged lab seasons. */
function magusBudget(age: number, seasons = 0, spellLevels = 0): LifeStageBudget {
  const years = age - 25;
  const points = years * 30 - Math.min(seasons, 3 * years) * 10;
  return {
    childhood_native_xp: 75,
    childhood_spread_xp: 45,
    later_life_years: 5,
    later_life_rate: 15,
    later_life_xp: 75,
    apprenticeship_years: 15,
    apprenticeship_xp: 240,
    gauntlet_age: 25,
    post_gauntlet_years: years,
    post_gauntlet_points: points,
    post_gauntlet_spell_levels: spellLevels,
    post_gauntlet_xp: points - spellLevels,
  };
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

  // manual-testing-findings #21: the two explanatory hints under the radios are
  // gone, and — the half that matters for a11y — so is the `aria-describedby` that
  // pointed at them. A radio left describing a deleted id is a dangling reference.
  it('leaves neither hint node nor a description pointing at one', () => {
    const body = html();
    expect(has(body, 'ability-funding-pool-hint')).toBe(false);
    expect(has(body, 'ability-funding-life_stages-hint')).toBe(false);
    expect(element(body, 'ability-funding-pool').open).not.toMatch(/aria-describedby/);
    expect(element(body, 'ability-funding-life_stages').open).not.toMatch(/aria-describedby/);
  });
});

describe('LifeStagePanel offers both modes to a magus (slice 6b4)', () => {
  /** Install the magus profile and a magus entity — the type 6b4 admits. */
  function installMagus(): void {
    installRuleset(lifeStageRules(), profile('magus', true));
    resetEntity('magus');
  }

  it('offers the guided option to a magus, undisabled and with no refusal', () => {
    installMagus();
    const body = html();
    const guided = element(body, 'ability-funding-life_stages');
    // Apprenticeship is modelled now, so a magus with a life-stage plan is legal and
    // the option must be live — the engine no longer refuses the combination.
    expect(guided.open).not.toMatch(/disabled/);
    // The reason node is gone, and so is the hint that replaced it: the radio's own
    // label is the whole of what it says now.
    expect(guided.open).not.toMatch(/aria-describedby/);
    expect(element(body, 'ability-funding-pool').open).not.toMatch(/disabled/);
  });

  it('carries no magus refusal node for any character type', () => {
    // Neither for a companion, which never had one …
    expect(has(html(), 'ability-funding-magus-reason')).toBe(false);
    // … nor for the magus, which no longer does.
    installMagus();
    expect(has(html(), 'ability-funding-magus-reason')).toBe(false);
  });

  // manual-testing-findings #21: the paragraph explaining what a magus's two ages
  // mean is gone for every character type and every funding mode. The two fields
  // still carry their own labels, which is what the panel is for.
  it('shows the Gauntlet explanation to nobody, in either funding mode', () => {
    installMagus();
    installPlan();
    const body = html();
    expect(has(body, 'life-stage-gauntlet-note')).toBe(false);
    // Both age controls survive it — this is a prose removal, not a field removal.
    expect(has(body, 'age-readout')).toBe(true);
    expect(has(body, 'life-stage-gauntlet-age-input')).toBe(true);

    installMagus();
    expect(has(html(), 'life-stage-gauntlet-note')).toBe(false);
  });
});

describe('LifeStagePanel guided fields (slice 6b3b)', () => {
  it('renders neither the age nor the native-language field in pool mode', () => {
    setAgeCap(5);
    const body = html();
    expect(has(body, 'age-readout')).toBe(false);
    expect(has(body, 'life-stage-age-input')).toBe(false);
    expect(has(body, 'life-stage-age-cap')).toBe(false);
    expect(has(body, 'native-language-input')).toBe(false);
  });

  it('shows the entity age read-only under a plan, and offers no second input', () => {
    installPlan();
    store.entity.age = 25;
    const body = html();
    // Later life is (age - childhood years) × rate, so the guided flow has to show
    // the age it is priced from. Slice 12 (#24) made that a read-out: the one
    // editable field lives with the identity, and this panel had been the second
    // place the same `entity.age` could be changed from.
    const age = element(body, 'age-readout');
    expect(age.open).not.toMatch(/<input/i);
    expect(age.text).toContain('25');
    expect(body).toContain('Age');
    expect(has(body, 'life-stage-age-input')).toBe(false);
    expect(has(body, 'age-input')).toBe(false);
  });

  it('renders the native-language field with its localized label and placeholder', () => {
    installPlan({ native_language: 'German' });
    const body = html();
    const input = element(body, 'native-language-input');
    expect(input.open).toMatch(/value="German"/);
    expect(input.open).toMatch(/placeholder="[^"]*German/);
    expect(body).toContain('Native language');
  });

  it('echoes no age cap at all — it has one home, beside the Ability lists', () => {
    // Slice 12 (#24): `age-cap-note` used to render on two surfaces at once, neither
    // of which shows an Ability score. It renders only on the Abilities surface now
    // (`AbilityTab`), which is the list the cap actually constrains.
    installPlan();
    store.entity.age = 25;
    setAgeCap(5);
    const body = html();
    expect(has(body, 'life-stage-age-cap')).toBe(false);
    expect(has(body, 'age-cap-note')).toBe(false);
  });
});

describe('LifeStagePanel post-Gauntlet fields (slice 6b5)', () => {
  /** A guided magus against a ruleset carrying the post-apprenticeship block. */
  function installGuidedMagus(plan: LifeStagePlan = {}): void {
    installRuleset(lifeStageRules(), profile('magus', true));
    resetEntity('magus');
    installPlan(plan);
  }

  const inputs = [
    'life-stage-gauntlet-age-input',
    'life-stage-lab-seasons-input',
    'life-stage-spell-levels-input',
  ];

  it('offers all three number inputs to a guided magus', () => {
    installGuidedMagus();
    const body = html();
    for (const testid of inputs) {
      expect(element(body, testid).open).toMatch(/type="number"/);
    }
  });

  it('offers none of them to a guided companion', () => {
    installPlan();
    const body = html();
    // Only a magus serves a Gauntlet, so only a magus has years past one.
    for (const testid of inputs) expect(has(body, testid)).toBe(false);
    expect(has(body, 'life-stage-post-gauntlet-summary')).toBe(false);
  });

  it('offers none of them to a magus in pool mode', () => {
    installRuleset(lifeStageRules(), profile('magus', true));
    resetEntity('magus');
    const body = html();
    for (const testid of inputs) expect(has(body, testid)).toBe(false);
  });

  it('offers none of them when the ruleset ships no post-apprenticeship block', () => {
    installRuleset(rulesWithoutPostApprenticeship(), profile('magus', true));
    resetEntity('magus');
    installPlan();
    const body = html();
    // The rate is data: with no block there is no number to grant, so the fields
    // would be dead controls.
    for (const testid of inputs) expect(has(body, testid)).toBe(false);
    expect(has(body, 'life-stage-post-gauntlet-summary')).toBe(false);
    // The rest of the guided panel is untouched — the age read-out included.
    expect(has(body, 'age-readout')).toBe(true);
  });

  it('carries the plan values, and offers the ruleset default as the Gauntlet placeholder', () => {
    installGuidedMagus({
      gauntlet_age: 25,
      post_gauntlet_lab_seasons: 6,
      post_gauntlet_spell_levels: 40,
    });
    store.entity.age = 40;
    const body = html();
    const gauntlet = element(body, 'life-stage-gauntlet-age-input');
    expect(gauntlet.open).toMatch(/value="25"/);
    // The placeholder says what blank MEANS, and blank no longer means "this magus
    // stands at its Gauntlet" — it means the ruleset's baseline magus, gauntleted at
    // `apprenticeship.default_gauntlet_age`. Showing the age here (it used to show
    // 40) told the player the opposite of what the engine would do.
    expect(gauntlet.open).toMatch(/placeholder="25"/);
    expect(element(body, 'life-stage-lab-seasons-input').open).toMatch(/value="6"/);
    expect(element(body, 'life-stage-spell-levels-input').open).toMatch(/value="40"/);
  });

  it('clamps the Gauntlet placeholder to a magus younger than the default', () => {
    installGuidedMagus();
    store.entity.age = 22;
    // The engine clamps the baseline to the character's own age, so a magus of 22
    // does stand at its Gauntlet — and the placeholder has to say 22, not 25, or it
    // would name an age the character has not reached.
    expect(element(html(), 'life-stage-gauntlet-age-input').open).toMatch(/placeholder="22"/);
  });

  it('falls back to the age for a ruleset stating no default Gauntlet age', () => {
    installRuleset(rulesWithoutDefaultGauntletAge(), profile('magus', true));
    resetEntity('magus');
    installPlan();
    store.entity.age = 40;
    // No baseline in the data means the older reading stands: blank is "stands at its
    // Gauntlet", and the placeholder still says so.
    expect(element(html(), 'life-stage-gauntlet-age-input').open).toMatch(/placeholder="40"/);
  });

  it('leaves every field empty for a magus that has entered nothing', () => {
    installGuidedMagus();
    store.entity.age = 25;
    const body = html();
    // Empty VALUES — nothing is written into the plan on the player's behalf, so the
    // save records no Gauntlet age and the dirty flag is untouched. What blank means
    // is the placeholder's job (above), not a prefill's.
    for (const testid of inputs) expect(element(body, testid).open).toMatch(/value=""/);
  });

  it('labels each field through Fluent, never as a raw slug', () => {
    installGuidedMagus();
    const body = html();
    expect(body).toContain('Gauntlet age');
    expect(body).toContain('Lab seasons');
    expect(body).toContain('Levels of spells');
    expect(body).not.toMatch(/>\s*post_gauntlet_lab_seasons\s*</);
    expect(body).not.toMatch(/>\s*gauntlet_age\s*</);
  });

  // manual-testing-findings #21: all three per-field hints are gone, together with
  // the `aria-describedby` that named them — the dangling-reference half of the
  // removal, and the one an a11y regression would hide in. In the ordinary state
  // (years to spend) the fields carry no description at all; the read-only state
  // below is the sole exception and has its own suite.
  it('leaves no per-field hint and no description naming one', () => {
    installGuidedMagus();
    const body = html();
    for (const testid of inputs) {
      expect(has(body, testid.replace('-input', '-hint'))).toBe(false);
      expect(element(body, testid).open).not.toMatch(/aria-describedby/);
    }
  });

  it('reads out the engine years, points and split, announced', () => {
    installGuidedMagus({
      gauntlet_age: 25,
      post_gauntlet_lab_seasons: 6,
      post_gauntlet_spell_levels: 40,
    });
    store.entity.age = 40;
    setLifeStageBudget(magusBudget(40, 6, 40));
    const summary = element(html(), 'life-stage-post-gauntlet-summary');
    // 15 years × 30 = 450, less 6 charged seasons × 10 = 390 points, 40 of them
    // taken as levels of spells.
    expect(summary.open).toMatch(/role="status"/);
    expect(summary.text).toContain('15');
    expect(summary.text).toContain('390');
    expect(summary.text).toContain('350');
    expect(summary.text).toContain('40');
    // Engine numbers through a Fluent key, never a slug and never a U+2212.
    expect(summary.text).not.toContain('post_gauntlet');
    expect(summary.text).not.toContain('−');
  });

  it('omits the read-out until the engine has a budget', () => {
    installGuidedMagus({ gauntlet_age: 25 });
    setLifeStageBudget(null);
    expect(has(html(), 'life-stage-post-gauntlet-summary')).toBe(false);
  });

  describe('with no years as a magus (guided-creation-review-2026-08 #11)', () => {
    /** The two fields whose ceiling is a multiple of the post-Gauntlet years. */
    const scaled = ['life-stage-lab-seasons-input', 'life-stage-spell-levels-input'];

    it('takes no lab seasons or levels of spells while the span is empty', () => {
      installGuidedMagus({ gauntlet_age: 25 });
      store.entity.age = 25;
      setLifeStageBudget(magusBudget(25));
      const body = html();
      // `readonly`, not `disabled`: the fields stay in the tab order and keep their
      // label, so a screen-reader user reaches them and is told why they take
      // nothing — a disabled control is skipped and explains itself to nobody.
      for (const testid of scaled) {
        expect(element(body, testid).open).toMatch(/readonly/);
      }
      // The Gauntlet age is the field that ENDS this state, so it stays editable.
      expect(element(body, 'life-stage-gauntlet-age-input').open).not.toMatch(/readonly/);
    });

    it('says why, in words, and points both fields at the explanation', () => {
      installGuidedMagus({ gauntlet_age: 25 });
      store.entity.age = 25;
      setLifeStageBudget(magusBudget(25));
      const body = html();
      const note = element(body, 'life-stage-post-gauntlet-no-years-note');
      expect(note.open).toMatch(/id="life-stage-post-gauntlet-no-years-note"/);
      // Localized prose, never the raw key.
      expect(note.text).toContain('as a magus');
      expect(note.text).not.toContain('life-stage-post-gauntlet-no-years-note');
      // manual-testing-findings #21 kept the first sentence — a read-only state the
      // controls cannot state for themselves — and cut the second, which taught the
      // player where those years come from.
      expect(note.text).not.toContain('run from the Gauntlet age');
      for (const testid of scaled) {
        expect(element(body, testid).open).toMatch(/life-stage-post-gauntlet-no-years-note/);
      }
    });

    it('opens both fields again as soon as one year is earned', () => {
      installGuidedMagus({ gauntlet_age: 25 });
      store.entity.age = 26;
      setLifeStageBudget(magusBudget(26));
      const body = html();
      for (const testid of scaled) {
        expect(element(body, testid).open).not.toMatch(/readonly/);
      }
      expect(has(body, 'life-stage-post-gauntlet-no-years-note')).toBe(false);
    });

    it('leaves the fields editable while the engine has sent no budget yet', () => {
      installGuidedMagus({ gauntlet_age: 25 });
      store.entity.age = 25;
      setLifeStageBudget(null);
      const body = html();
      // No budget is "not computed yet", not "no years": locking the fields on the
      // first paint would fight the player for the round trip's duration.
      for (const testid of scaled) {
        expect(element(body, testid).open).not.toMatch(/readonly/);
      }
      expect(has(body, 'life-stage-post-gauntlet-no-years-note')).toBe(false);
    });
  });
});

describe('LifeStagePanel childhood picker (slice 6b3b)', () => {
  /** A ruleset that also ships a Sample Childhood catalogue for the picker to offer. */
  function installCatalogue(): void {
    installRuleset();
    store.ruleset!.ruleset.childhoods = {
      'childhood.athletic': {
        id: 'childhood.athletic',
        entries: [
          { ability: 'ability.athletics', score: 2 },
          { ability: 'ability.living_language', score: 5, native: true },
        ],
      },
    };
    store.ruleset!.i18n['childhood.athletic'] = { name: 'Athletic Childhood' };
  }

  it('mounts the childhood picker under the guided fields', () => {
    installCatalogue();
    installPlan({ native_language: 'German' });
    const body = html();
    expect(has(body, 'childhood-package-select')).toBe(true);
    // Childhood is part of the guided flow, so it belongs below the native
    // language it depends on.
    expect(body.indexOf('native-language-input')).toBeLessThan(
      body.indexOf('childhood-package-select'),
    );
  });

  it('offers no childhood package in pool mode', () => {
    installCatalogue();
    // No plan: the character's Abilities are funded by the typed pool, and
    // childhood is not one of its blocks.
    expect(has(html(), 'childhood-package-select')).toBe(false);
  });
});
