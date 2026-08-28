import { beforeEach, describe, expect, it, vi } from 'vitest';
import { render } from 'svelte/server';

import type {
  ChildhoodPackage,
  Entity,
  LifeStagePlan,
  LifeStageRules,
  LocalizedRuleset,
  ValidationIssue,
} from '../types';

// The picker reads the shared store singleton: the loaded ruleset's childhood
// catalogue, the entity's plan (the recorded package and the native language) and
// the UI-only draft. Mock the Tauri bridge so nothing reaches a backend; harness
// mirrors LifeStagePanel.test.ts.
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
import ChildhoodPackagePicker from './ChildhoodPackagePicker.svelte';

/** The life-stage rules as `rules/core/life_stages.json` ships them. */
function lifeStageRules(): LifeStageRules {
  return {
    childhood: {
      years: 5,
      native_language_ability: 'ability.living_language',
      native_language_xp: 75,
      spread_xp: 45,
      spread_abilities: ['ability.area_lore', 'ability.living_language'],
    },
    later_life: { xp_per_year: 15 },
  };
}

/** Traveling Childhood: two Area Lore slots (so labels need ordinals) and a language slot. */
const TRAVELING: ChildhoodPackage = {
  id: 'childhood.traveling',
  entries: [
    { ability: 'ability.area_lore', score: 1, slot: 'area_a' },
    { ability: 'ability.area_lore', score: 1, slot: 'area_b' },
    { ability: 'ability.folk_ken', score: 2 },
    { ability: 'ability.living_language', score: 5, native: true },
    { ability: 'ability.living_language', score: 1, slot: 'language' },
    { ability: 'ability.survival', score: 2 },
  ],
};

/** Athletic Childhood: nothing to ask, so no slot input and Apply is free to be enabled. */
const ATHLETIC: ChildhoodPackage = {
  id: 'childhood.athletic',
  entries: [
    { ability: 'ability.athletics', score: 2 },
    { ability: 'ability.living_language', score: 5, native: true },
  ],
};

/**
 * Install a minimal localized ruleset. `packages` is `null` for a ruleset shipping
 * no childhood catalogue at all — the case the picker must render nothing for.
 */
function installRuleset(packages: ChildhoodPackage[] | null = [TRAVELING, ATHLETIC]): void {
  const childhoods: Record<string, ChildhoodPackage> = {};
  for (const pkg of packages ?? []) childhoods[pkg.id] = pkg;
  store.ruleset = {
    ruleset: {
      id: 'test',
      version: '1',
      point_items: {},
      type_profiles: {},
      abilities: {
        'ability.area_lore': { id: 'ability.area_lore', category: 'general', parameter: 'area' },
        'ability.athletics': { id: 'ability.athletics', category: 'general' },
        'ability.folk_ken': { id: 'ability.folk_ken', category: 'general' },
        'ability.living_language': {
          id: 'ability.living_language',
          category: 'general',
          parameter: 'language',
        },
        'ability.survival': { id: 'ability.survival', category: 'general' },
      },
      magnitude_points: { free: 0, minor: 1, major: 3 },
      ability_category_order: ['general', 'academic', 'arcane', 'martial', 'supernatural'],
      art_type_order: ['technique', 'form'],
      life_stages: lifeStageRules(),
      ...(packages ? { childhoods } : {}),
    },
    i18n: {
      'ability.area_lore': { name: '{area} Lore' },
      'ability.athletics': { name: 'Athletics' },
      'ability.folk_ken': { name: 'Folk Ken' },
      'ability.living_language': { name: '{language}' },
      'ability.survival': { name: 'Survival' },
      'childhood.athletic': { name: 'Athletic Childhood' },
      'childhood.traveling': { name: 'Traveling Childhood' },
    },
  } as unknown as LocalizedRuleset;
}

function resetEntity(plan: LifeStagePlan | null = { native_language: 'German' }): void {
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
    ...(plan ? { life_stages: plan } : {}),
  };
}

/** Render the picker to an HTML string (node env, no DOM). */
function html(): string {
  return render(ChildhoodPackagePicker, { props: {} }).body;
}

/** Whether any element carries the exact data-testid. */
function has(body: string, testid: string): boolean {
  return new RegExp(`data-testid="${testid}"`).test(body);
}

/** The opening tag of the single element carrying a data-testid. */
function open(body: string, testid: string): string {
  const match = new RegExp(`<[^>]*data-testid="${testid}"[^>]*>`, 'i').exec(body);
  if (!match) throw new Error(`no element with data-testid="${testid}"`);
  return match[0];
}

/**
 * The visible text of the element carrying a data-testid, tags stripped. Fluent
 * wraps interpolated values in bidi isolation marks, so those come out too.
 */
function text(body: string, testid: string): string {
  const match = new RegExp(`<[^>]*data-testid="${testid}"[^>]*>([\\s\\S]*?)</[a-z]+>`, 'i').exec(
    body,
  );
  if (!match) throw new Error(`no element with data-testid="${testid}"`);
  return stripMarks(match[1].replace(/<[^>]*>/g, ' '))
    .replace(/\s+/g, ' ')
    .trim();
}

/** Everything the user can actually read: all markup stripped, isolation marks too. */
function visibleText(body: string): string {
  return stripMarks(body.replace(/<[^>]*>/g, ' '));
}

function stripMarks(s: string): string {
  return s.replace(/[⁨⁩]/g, '');
}

/** The whole `<select>` block, so option-level assertions cannot stray. */
function selectBlock(body: string): string {
  const match = /<select[\s\S]*?<\/select>/i.exec(body);
  if (!match) throw new Error('no <select> rendered');
  return match[0];
}

/** The `<li>` rows of the element carrying a data-testid. */
function rows(body: string, testid: string): string[] {
  const block = new RegExp(`<[^>]*data-testid="${testid}"[^>]*>([\\s\\S]*?)</ul>`, 'i').exec(body);
  if (!block) throw new Error(`no list with data-testid="${testid}"`);
  return [...block[1].matchAll(/<li[^>]*>([\s\S]*?)<\/li>/g)].map((m) =>
    stripMarks(m[1].replace(/<[^>]*>/g, ' '))
      .replace(/\s+/g, ' ')
      .trim(),
  );
}

beforeEach(() => {
  vi.useFakeTimers();
  store.lang = 'en';
  installRuleset();
  resetEntity();
  store.setChildhoodDraftPackage(null);
  store.childhoodRejections = [];
});

describe('ChildhoodPackagePicker without a catalogue (slice 6b3b)', () => {
  it('renders nothing when the ruleset ships no childhood packages', () => {
    installRuleset(null);
    const body = html();
    // A choice the data cannot offer must not appear as an empty dropdown.
    expect(has(body, 'childhood-package-select')).toBe(false);
    expect(body.replace(/<!--[\s\S]*?-->/g, '').trim()).toBe('');
  });

  it('renders nothing when the catalogue is empty', () => {
    installRuleset([]);
    expect(has(html(), 'childhood-package-select')).toBe(false);
  });

  it('renders nothing before a ruleset is loaded', () => {
    store.ruleset = null;
    expect(has(html(), 'childhood-package-select')).toBe(false);
  });
});

describe('ChildhoodPackagePicker draft selection (slice 6b3b)', () => {
  it('offers every package by its localized name, alphabetically, never as an id', () => {
    const body = html();
    const options = [...selectBlock(body).matchAll(/<option[^>]*>([\s\S]*?)<\/option>/g)].map((m) =>
      stripMarks(m[1]).trim(),
    );
    // Option 0 is the first-class choice of dividing the experience yourself, not
    // an opt-out: "Note that you can spend the 45 experience points for yourself".
    // Source: Ars Magica - Definitive Edition (Core Rules).md:2382
    expect(options[0]).toContain('Spend');
    expect(options.slice(1)).toEqual(['Athletic Childhood', 'Traveling Childhood']);
    // Package ids are option values only; no id is ever readable.
    expect(visibleText(body)).not.toContain('childhood.');
  });

  it('labels the select through Fluent', () => {
    expect(visibleText(html())).toContain('Sample Childhood');
  });

  it('shows neither a preview, a slot nor an Apply button while no package is drafted', () => {
    const body = html();
    expect(has(body, 'childhood-package-preview')).toBe(false);
    expect(has(body, 'childhood-slot-area_a')).toBe(false);
    expect(has(body, 'childhood-apply')).toBe(false);
  });
});

describe('ChildhoodPackagePicker preview (slice 6b3b)', () => {
  beforeEach(() => {
    store.setChildhoodDraftPackage('childhood.traveling');
  });

  it('lists one row per package entry, headed by its label', () => {
    const body = html();
    expect(visibleText(body)).toContain('Package preview');
    expect(rows(body, 'childhood-package-preview')).toEqual([
      '(Area) Lore 1',
      '(Area) Lore 1',
      'Folk Ken 2',
      'German 5',
      '(Language) 1',
      'Survival 2',
    ]);
  });

  it('marks the drafted package as the selected option', () => {
    expect(selectBlock(html())).toMatch(/value="childhood\.traveling"[^>]*selected/);
  });

  it('reads a filled slot back into its row', () => {
    store.setChildhoodDraftSlot('area_a', 'Rhine');
    expect(rows(html(), 'childhood-package-preview')[0]).toBe('Rhine Lore 1');
  });

  it('never leaves a parameter token or an ability id in a row', () => {
    const preview = rows(html(), 'childhood-package-preview').join(' | ');
    expect(preview).not.toContain('{');
    expect(preview).not.toContain('ability.');
  });
});

describe('ChildhoodPackagePicker slot inputs (slice 6b3b)', () => {
  beforeEach(() => {
    store.setChildhoodDraftPackage('childhood.traveling');
  });

  it('asks for one value per slotted entry, ordinal-labelled where an Ability repeats', () => {
    const body = html();
    expect(has(body, 'childhood-slot-area_a')).toBe(true);
    expect(has(body, 'childhood-slot-area_b')).toBe(true);
    expect(has(body, 'childhood-slot-language')).toBe(true);
    // Two Area Lores are indistinguishable without the ordinal; the lone language
    // slot must not carry one. Slot keys are data, so no label may show one.
    const visible = visibleText(body);
    expect(visible).toContain('(Area) Lore (1)');
    expect(visible).toContain('(Area) Lore (2)');
    expect(visible).toContain('(Language)');
    expect(visible).not.toContain('area_a');
    expect(visible).not.toContain('area_b');
  });

  it('carries the drafted answer back into the input', () => {
    store.setChildhoodDraftSlot('area_b', 'Provence');
    expect(open(html(), 'childhood-slot-area_b')).toMatch(/value="Provence"/);
  });

  it('marks an empty slot invalid and points at its own spoken reason', () => {
    const body = html();
    const input = open(body, 'childhood-slot-area_a');
    expect(input).toMatch(/aria-invalid="true"/);
    expect(input).toMatch(/aria-describedby="childhood-slot-area_a-reason"/);
    const reason = open(body, 'childhood-slot-area_a-reason');
    expect(reason).toMatch(/id="childhood-slot-area_a-reason"/);
    // Never colour alone: the fault is text a screen reader reads out.
    expect(text(body, 'childhood-slot-area_a-reason')).toContain('Fill in');
  });

  it('marks both slots of one Ability answered alike, naming the merge', () => {
    store.setChildhoodDraftSlot('area_a', 'Rhine');
    store.setChildhoodDraftSlot('area_b', 'Rhine');
    store.setChildhoodDraftSlot('language', 'Italian');
    const body = html();
    expect(open(body, 'childhood-slot-area_a')).toMatch(/aria-invalid="true"/);
    expect(open(body, 'childhood-slot-area_b')).toMatch(/aria-invalid="true"/);
    expect(text(body, 'childhood-slot-area_a-reason')).toContain('merge');
    expect(open(body, 'childhood-slot-language')).not.toMatch(/aria-invalid/);
  });

  it('marks a childhood language repeating the native language', () => {
    store.setChildhoodDraftSlot('area_a', 'Rhine');
    store.setChildhoodDraftSlot('area_b', 'Provence');
    store.setChildhoodDraftSlot('language', 'German');
    const body = html();
    expect(open(body, 'childhood-slot-language')).toMatch(/aria-invalid="true"/);
    expect(text(body, 'childhood-slot-language-reason')).toContain('native language');
    expect(open(body, 'childhood-slot-area_a')).not.toMatch(/aria-invalid/);
  });

  it('leaves a fully and distinctly answered set of slots unflagged', () => {
    store.setChildhoodDraftSlot('area_a', 'Rhine');
    store.setChildhoodDraftSlot('area_b', 'Provence');
    store.setChildhoodDraftSlot('language', 'Italian');
    const body = html();
    expect(body).not.toMatch(/aria-invalid/);
    expect(has(body, 'childhood-slot-area_a-reason')).toBe(false);
  });
});

describe('ChildhoodPackagePicker apply (slice 6b3b)', () => {
  it('offers Apply with its hint spoken, for a package that asks nothing', () => {
    store.setChildhoodDraftPackage('childhood.athletic');
    const body = html();
    const button = open(body, 'childhood-apply');
    expect(button).not.toMatch(/disabled/);
    expect(button).toMatch(/aria-describedby="[^"]*childhood-apply-hint/);
    // The scores are filled in, not frozen — say so before the click.
    expect(text(body, 'childhood-apply-hint')).toContain('editable');
    expect(has(body, 'childhood-apply-reason')).toBe(false);
  });

  it('refuses Apply while a slot is unanswered, with the reason reachable', () => {
    store.setChildhoodDraftPackage('childhood.traveling');
    const body = html();
    const button = open(body, 'childhood-apply');
    expect(button).toMatch(/disabled/);
    expect(button).toMatch(/aria-describedby="[^"]*childhood-apply-reason/);
    expect(text(body, 'childhood-apply-reason')).toContain('Fill in');
  });

  it('refuses Apply for a duplicated answer, naming that fault', () => {
    store.setChildhoodDraftPackage('childhood.traveling');
    store.setChildhoodDraftSlot('area_a', 'Rhine');
    store.setChildhoodDraftSlot('area_b', 'Rhine');
    store.setChildhoodDraftSlot('language', 'Italian');
    const body = html();
    expect(open(body, 'childhood-apply')).toMatch(/disabled/);
    expect(text(body, 'childhood-apply-reason')).toContain('merge');
  });

  it('refuses Apply for a childhood language equal to the native language', () => {
    store.setChildhoodDraftPackage('childhood.traveling');
    store.setChildhoodDraftSlot('area_a', 'Rhine');
    store.setChildhoodDraftSlot('area_b', 'Provence');
    store.setChildhoodDraftSlot('language', 'German');
    const body = html();
    expect(open(body, 'childhood-apply')).toMatch(/disabled/);
    expect(text(body, 'childhood-apply-reason')).toContain('native language');
  });

  it('allows Apply once every slot is answered distinctly', () => {
    store.setChildhoodDraftPackage('childhood.traveling');
    store.setChildhoodDraftSlot('area_a', 'Rhine');
    store.setChildhoodDraftSlot('area_b', 'Provence');
    store.setChildhoodDraftSlot('language', 'Italian');
    const body = html();
    expect(open(body, 'childhood-apply')).not.toMatch(/disabled/);
    expect(has(body, 'childhood-apply-reason')).toBe(false);
  });
});

describe('ChildhoodPackagePicker record vs draft (slice 6b3b)', () => {
  it('names the package the character has taken, read-only', () => {
    resetEntity({ native_language: 'German', childhood_package: 'childhood.traveling' });
    const body = html();
    expect(text(body, 'childhood-taken')).toBe('Childhood taken: Traveling Childhood');
    // A record, not a control: it is text, not an input the player can retype.
    expect(open(body, 'childhood-taken')).toMatch(/^<span/);
  });

  it('leaves the draft select unselected even with a package recorded', () => {
    resetEntity({ native_language: 'German', childhood_package: 'childhood.traveling' });
    const body = html();
    // A recorded package is history, not a draft: pre-filling it would show
    // spurious empty-slot faults after a reload, since slot drafts are not saved.
    expect(selectBlock(body)).not.toMatch(/value="childhood\.traveling"[^>]*selected/);
    // Positively: the "divide it yourself" prompt is what stands selected.
    expect(selectBlock(body)).toMatch(/<option value=""[^>]*selected/);
    expect(has(body, 'childhood-package-preview')).toBe(false);
    expect(has(body, 'childhood-slot-area_a')).toBe(false);
  });

  it('says nothing about a taken package when none is recorded', () => {
    expect(has(html(), 'childhood-taken')).toBe(false);
  });

  it('says nothing about a taken package without a plan at all', () => {
    resetEntity(null);
    expect(has(html(), 'childhood-taken')).toBe(false);
  });
});

describe('ChildhoodPackagePicker engine rejections (slice 6b3b)', () => {
  const rejection: ValidationIssue = {
    severity: 'error',
    code: 'childhood_slot_unfilled',
    phase: 'abilities',
    args: { ability: 'ability.area_lore', key: 'area', slot: 'area_a' },
  };

  it('renders a rejection localized through the shared issue path', () => {
    store.setChildhoodDraftPackage('childhood.traveling');
    store.childhoodRejections = [rejection];
    const body = html();
    const rejections = rows(body, 'childhood-rejections');
    expect(rejections).toHaveLength(1);
    // The engine is authoritative once Apply is pressed; its findings localize
    // through `issue-<code>` with the args resolved, exactly as ValidationPanel does.
    expect(rejections[0]).toContain('Fill in the Area for (Area) Lore');
    expect(rejections[0]).not.toContain('ability.area_lore');
    expect(body).toMatch(/data-code="childhood_slot_unfilled"/);
  });

  it('renders no rejection list when the engine has said nothing', () => {
    store.setChildhoodDraftPackage('childhood.traveling');
    expect(has(html(), 'childhood-rejections')).toBe(false);
  });
});
