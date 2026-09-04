import { beforeEach, describe, expect, it, vi } from 'vitest';
import { render } from 'svelte/server';

import type {
  EffectiveScores,
  Entity,
  LifeStageRules,
  LocalizedRuleset,
  MagusMinimumAbility,
} from '../types';

// The tab reads the shared store singleton (ruleset catalogue, entity rows, filter
// state) and the Fluent bundle. The store schedules a debounced revalidate over the
// Tauri IPC bridge; mock the bridge so nothing reaches a backend.
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
import AbilityTab from './AbilityTab.svelte';

/** The life-stage rules as `rules/core/life_stages.json` ships them. */
function lifeStageRules(): LifeStageRules {
  return {
    childhood: {
      years: 5,
      native_language_ability: 'ability.living_language',
      native_language_xp: 75,
      spread_xp: 45,
      spread_abilities: ['ability.athletics'],
    },
    later_life: { xp_per_year: 15 },
  };
}

/** Install a minimal localized ruleset; `rules` is null for one shipping no plan. */
function installRuleset(rules: LifeStageRules | null = lifeStageRules()): void {
  store.ruleset = {
    ruleset: {
      id: 'test',
      version: '1',
      point_items: {},
      type_profiles: {
        companion: {
          id: 'companion',
          budget: { virtue_points: 10, flaw_points: 10 },
          is_magus: false,
          gift_policy: 'forbidden',
          creation_phases: [],
        },
      },
      abilities: {
        'ability.athletics': { id: 'ability.athletics', category: 'general' },
        'ability.parma_magica': { id: 'ability.parma_magica', category: 'arcane' },
      },
      advancement: [],
      magnitude_points: { free: 0, minor: 1, major: 3 },
      ability_category_order: ['general', 'academic', 'arcane', 'martial', 'supernatural'],
      art_type_order: ['technique', 'form'],
      ...(rules ? { life_stages: rules } : {}),
    },
    i18n: { 'ability.parma_magica': { name: 'Parma Magica' } },
  } as unknown as LocalizedRuleset;
}

/** One unmet Hermetic minimum, as the engine sends it for a magus. */
function setChecklist(): void {
  const rows: MagusMinimumAbility[] = [
    {
      ability: 'ability.parma_magica',
      min_score: 1,
      score: 0,
      met: false,
      requirement: 'required',
    },
  ];
  store.effective = { magus_minimum_abilities: rows } as unknown as EffectiveScores;
}

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
  store.result = { issues: [] };
}

/** Render the tab to an HTML string (node env, no DOM). */
function html(): string {
  return render(AbilityTab, { props: {} }).body;
}

const VOID_TAGS = new Set([
  'area',
  'base',
  'br',
  'col',
  'embed',
  'hr',
  'img',
  'input',
  'link',
  'meta',
  'source',
  'track',
  'wbr',
]);

/**
 * Nesting depth of the first element whose start tag contains `marker` — 0 for a
 * root-level node. Walks the start/end tags, so it can tell a sibling from a child
 * without a DOM (these component tests render to a string).
 */
function depthOf(body: string, marker: string): number {
  const tags = body.matchAll(/<(\/?)([a-zA-Z][a-zA-Z0-9-]*)([^>]*)>/g);
  let depth = 0;
  for (const tag of tags) {
    const [, closing, name, attrs] = tag;
    if (closing) {
      depth -= 1;
      continue;
    }
    if (attrs.includes(marker)) return depth;
    if (!VOID_TAGS.has(name.toLowerCase()) && !attrs.trimEnd().endsWith('/')) depth += 1;
  }
  throw new Error(`no element whose start tag contains ${marker}`);
}

beforeEach(() => {
  vi.useFakeTimers();
  store.lang = 'en';
  installRuleset();
  resetEntity();
});

// Slice 2 (#11): the funding model and the whole life-stage plan moved to the new
// `experience` step (`ExperienceStep.svelte`), so this tab is only the lists again.
// The mirror image of `ExperienceStep.test.ts`: what left here has to arrive there.
describe('AbilityTab leaves the funding choice to the experience step (Slice 2)', () => {
  it('no longer renders the life-stage panel', () => {
    expect(html()).not.toContain('data-testid="life-stage-panel"');
  });

  it('still renders the Available and Selected regions, with the row at root level', () => {
    const body = html();
    // `.region-row` must stay the only `flex: 1` child of `.vf-tab`, which is the
    // whole point of the split: with no preamble above it, it now gets the height.
    expect(body).toContain('class="region-row"');
    expect(depthOf(body, 'class="region-row"')).toBe(0);
    expect(body).toContain('data-testid="ability-search"');
    expect(body).toContain('class="region region-source"');
    expect(body).toContain('class="region region-selected"');
  });

  it('still renders the region row for a ruleset with no life-stage rules', () => {
    installRuleset(null);
    const body = html();
    expect(body).toContain('class="region-row"');
    expect(body).not.toContain('data-testid="life-stage-panel"');
    expect(depthOf(body, 'class="region-row"')).toBe(0);
  });
});

describe('AbilityTab mounts the magus minimums checklist (slice 6b4)', () => {
  it('renders it above the Available/Selected row', () => {
    setChecklist();
    const body = html();
    const checklist = body.indexOf('data-testid="magus-minimums"');
    const row = body.indexOf('class="region-row"');
    // What the Order demands of a magus's Abilities, then the lists it is about.
    expect(checklist).toBeGreaterThanOrEqual(0);
    expect(checklist).toBeLessThan(row);
  });

  it('keeps it a root-level sibling, so the region row still owns the height', () => {
    setChecklist();
    const body = html();
    // An auto-height sibling, never a wrapper: `.region-row` must stay the only
    // `flex: 1` child of `.vf-tab` or the Available/Selected lists collapse.
    expect(depthOf(body, 'data-testid="magus-minimums"')).toBe(0);
    expect(depthOf(body, 'class="region-row"')).toBe(0);
  });

  it('renders it in both funding modes', () => {
    setChecklist();
    // Flat pool: `:2437` is unconditional, so a magus owes the minimums either way …
    expect(html()).toContain('data-testid="magus-minimums"');
    // … and under life-stage funding just the same. The mode is stored since schema
    // 16, so it is set alongside the plan.
    store.entity.ability_funding = 'life_stages';
    store.entity.life_stages = {};
    expect(html()).toContain('data-testid="magus-minimums"');
  });

  it('renders it for a ruleset shipping no life-stage rules', () => {
    installRuleset(null);
    setChecklist();
    const body = html();
    // The funding panel is gone with the rules it needs, but a magus still owes the
    // Order its minimums — which is why the checklist is not mounted inside the panel.
    expect(body).not.toContain('data-testid="life-stage-panel"');
    expect(body).toContain('data-testid="magus-minimums"');
    expect(depthOf(body, 'class="region-row"')).toBe(0);
  });

  it('renders nothing of it when the engine sends no rows', () => {
    // Empty for every type but a magus — the tab needs no is_magus test of its own.
    expect(html()).not.toContain('data-testid="magus-minimums"');
  });
});

// S6 (full-audit a11y): the ability-category filter `<select>` had no accessible
// name — a screen-reader user tabbing into it hears only "combo box". It has no
// visible label to associate with (the filter row is icon-less controls in a
// row), so a Fluent-sourced `aria-label` is the right fix, never the raw slug.
describe('AbilityTab category filter (S6)', () => {
  it('gives the category filter select an accessible name via Fluent', () => {
    const body = html();
    const select = /<select[^>]*data-testid="ability-category-filter"[^>]*>/.exec(body);
    expect(select).not.toBeNull();
    expect(select![0]).toContain('aria-label="Filter by ability category"');
  });
});

// S7 (full-audit a11y): `.invalid-selection` (a row an error-severity issue
// points at, e.g. a supernatural Ability whose granting Virtue was removed)
// carried the fact by colour alone. WCAG 1.4.1 requires a non-colour channel —
// mirrors ValidationPanel's own fix (S2 in that panel's history): a visible
// glyph, `.sr-only` text naming it in words, and `aria-invalid="true"` on the
// row's editable fields (specialty, parameter) so assistive tech is told too —
// never on the spinner/remove buttons, which the ARIA spec does not permit the
// attribute on.
describe('AbilityTab invalid selection (S7)', () => {
  beforeEach(() => {
    store.entity.ability_scores = [{ ability: 'ability.athletics', score: 3 }];
  });

  it('marks an invalid row with a visible glyph, sr-only text, and aria-invalid', () => {
    store.result = {
      issues: [
        {
          severity: 'error',
          code: 'x',
          phase: 'abilities',
          context: 'ability.athletics',
          args: {},
        },
      ],
    };
    const body = html();
    // Non-colour visible marker, not merely the CSS class.
    expect(body).toMatch(/<span class="invalid-glyph" aria-hidden="true">[^<]+<\/span>/);
    // Named in words for a screen reader, not just implied by the glyph.
    expect(body).toContain('class="sr-only"');
    // aria-invalid goes on the row's actual form fields — never on the
    // spinner/remove buttons, which the ARIA spec does not permit it on
    // (role="button" does not support aria-invalid; svelte-check's a11y lint
    // flags it, and it would fail the required `npm run check` gate).
    const specialty = /<input[^>]*data-testid="ability-specialty-ability.athletics-0"[^>]*>/.exec(
      body,
    );
    expect(specialty![0]).toContain('aria-invalid="true"');
    const dec = /<button[^>]*data-testid="ability-dec-ability.athletics-0"[^>]*>/.exec(body);
    expect(dec![0]).not.toContain('aria-invalid');
  });

  it('leaves a valid row unmarked', () => {
    store.result = { issues: [] };
    const body = html();
    expect(body).not.toContain('invalid-glyph');
    const specialty = /<input[^>]*data-testid="ability-specialty-ability.athletics-0"[^>]*>/.exec(
      body,
    );
    expect(specialty![0]).not.toContain('aria-invalid');
  });

  it('does not mark a row a WARNING-severity issue points at (advisory only)', () => {
    store.result = {
      issues: [
        {
          severity: 'warning',
          code: 'x',
          phase: 'abilities',
          context: 'ability.athletics',
          args: {},
        },
      ],
    };
    const body = html();
    expect(body).not.toContain('invalid-glyph');
  });
});

// S5 (full-audit a11y): the free-text search box carries only a placeholder,
// which is not an accessible name — a screen-reader user tabbing into it hears
// only "text box, search". Mirrors the S6 fix for the sibling <select> above.
describe('AbilityTab search box (S5)', () => {
  it('gives the search box an accessible name via Fluent', () => {
    const body = html();
    const input = /<input[^>]*data-testid="ability-search"[^>]*>/.exec(body);
    expect(input).not.toBeNull();
    expect(input![0]).toContain('aria-label="Search…"');
  });
});

// Issue 17: the Selected list is built from BOUGHT rows, so a Puissant Ability (or
// a virtue-granted floor) whose target has no bought score had nothing to hang its
// badge on — the character genuinely held Puissant Magic Theory and this tab said
// nothing. `ArtGrid` never had the problem: all 15 Arts are always rendered. The
// fix is a display-only row, testid-suffixed `unbought` rather than an array index,
// because there is no entity row behind it to address.
describe('AbilityTab unbought bonus rows (#17)', () => {
  /** The engine's effective scores as they arrive on the store. */
  function setModifiers(
    bonuses: EffectiveScores['ability_bonuses'],
    floors: EffectiveScores['ability_score_floors'] = [],
  ): void {
    store.effective = {
      ability_bonuses: bonuses,
      ability_score_floors: floors,
    } as unknown as EffectiveScores;
  }

  it('renders a row for a bonus whose target has no bought score', () => {
    setModifiers([{ ability: 'ability.parma_magica', bonus: 2 }]);
    const body = html();
    expect(body).toContain('data-testid="ability-score-ability.parma_magica-unbought"');
    // Score 0 — the bought score, which is what it is — and effective 0 + 2.
    const score = /data-testid="ability-score-ability.parma_magica-unbought"[^>]*>([\s\S]*?)</.exec(
      body,
    );
    expect(score![1].trim()).toBe('0');
    const badge = /data-testid="ability-eff-ability.parma_magica-unbought"[^>]*>([\s\S]*?)</.exec(
      body,
    );
    expect(badge![1]).toContain('2');
  });

  it('renders one for a granted free starting score too', () => {
    // `ability_score_floors` already iterated effects rather than bought rows, so
    // the floor path had the same hole: Second Sight granted but not yet bought.
    setModifiers([], [{ ability: 'ability.parma_magica', floor: 1 }]);
    const body = html();
    const badge = /data-testid="ability-eff-ability.parma_magica-unbought"[^>]*>([\s\S]*?)</.exec(
      body,
    );
    expect(badge![1]).toContain('1');
  });

  it('offers no edits on it — nothing exists yet to raise, name or remove', () => {
    setModifiers([{ ability: 'ability.parma_magica', bonus: 2 }]);
    const body = html();
    // The Available picker stays the one way to buy it, so both stepper buttons
    // are inert rather than silently conjuring an entity row.
    const dec = /<button[^>]*data-testid="ability-dec-ability.parma_magica-unbought"[^>]*>/.exec(
      body,
    );
    expect(dec![0]).toContain('disabled');
    const inc = /<button[^>]*data-testid="ability-inc-ability.parma_magica-unbought"[^>]*>/.exec(
      body,
    );
    expect(inc![0]).toContain('disabled');
    // No specialty box, no parameter box, no remove button: there is no stored row
    // to edit, and removing the bonus means removing the Virtue on its own tab.
    expect(body).not.toContain('data-testid="ability-specialty-ability.parma_magica-unbought"');
    expect(body).not.toContain('data-testid="remove-ability.parma_magica-unbought"');
    // Told in words, not by the greyed controls alone (WCAG 1.4.1).
    expect(body).toContain('Not bought');
  });

  it('shows the row instead of the "nothing selected" message', () => {
    setModifiers([{ ability: 'ability.parma_magica', bonus: 2 }]);
    const body = html();
    expect(body).toContain('data-testid="ability-score-ability.parma_magica-unbought"');
    expect(body).not.toContain('class="empty"');
  });

  it('sorts it into its own category group, where the bought row will appear', () => {
    // Athletics is general, Parma Magica arcane; `ability_category_order` puts
    // general first, so the unbought arcane row lands below it under its own header.
    store.entity.ability_scores = [{ ability: 'ability.athletics', score: 3 }];
    setModifiers([{ ability: 'ability.parma_magica', bonus: 2 }]);
    const body = html();
    expect(body.indexOf('data-testid="ability-score-ability.athletics-0"')).toBeLessThan(
      body.indexOf('data-testid="ability-score-ability.parma_magica-unbought"'),
    );
  });

  it('adds no row when the target already has a bought row to carry the badge', () => {
    store.entity.ability_scores = [{ ability: 'ability.parma_magica', score: 1 }];
    setModifiers([{ ability: 'ability.parma_magica', bonus: 2 }]);
    const body = html();
    expect(body).not.toContain('ability-score-ability.parma_magica-unbought');
    expect(body).toContain('data-testid="ability-eff-ability.parma_magica-0"');
  });
});

// --- Slice 12 (#24): the age cap note's one home -----------------------------
//
// This component is BOTH surfaces — the editor's Abilities tab and the wizard's
// `abilities` step mount it — so covering it here covers both, which the shared
// components' blast radius requires. `WizardStep.test.ts` counts it across the whole
// wizard and asserts exactly one; this is the "and it is here" half.
describe('AbilityTab age cap note', () => {
  /** The engine's age → max-Ability-score cap, as it arrives on the store. */
  function setAgeCap(cap: number | null): void {
    store.effective = { age_ability_cap: cap } as unknown as EffectiveScores;
  }

  it('reads the cap beside the lists it constrains, from the engine', () => {
    store.entity.age = 25;
    setAgeCap(5);
    const body = html();
    const note = /data-testid="age-cap-note"[^>]*>([\s\S]*?)<\//.exec(body);
    expect(note).not.toBeNull();
    expect(note![1]).toContain('5');
    // Fluent, never the engine's field name.
    expect(body).not.toContain('age_ability_cap');
    // A sibling of the region row, like the checklist above it: the row must stay the
    // only `flex: 1` child of `.vf-tab`, or both ability lists collapse.
    expect(depthOf(body, 'data-testid="age-cap-note"')).toBe(0);
    // And announced — the age it reflects is typed on another surface entirely.
    expect(/<[^>]*data-testid="age-cap-note"[^>]*>/.exec(body)![0]).toMatch(/role="status"/);
  });

  it('renders no note when the engine reports no cap', () => {
    // A ruleset shipping no age bands cannot cap anything, and an empty echo would be
    // a dead line taken out of the lists' height.
    setAgeCap(null);
    expect(html()).not.toContain('data-testid="age-cap-note"');
  });
});
