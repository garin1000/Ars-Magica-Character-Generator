import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { render } from 'svelte/server';

import type { CreationPhase, Entity, LocalizedRuleset } from '../types';

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
import WizardShell from './WizardShell.svelte';

const PHASES: CreationPhase[] = ['concept', 'type', 'characteristics', 'virtues_flaws'];

function installFlow(phases: CreationPhase[] = PHASES): void {
  store.ruleset = {
    ruleset: {
      id: 'test',
      version: '1',
      point_items: {},
      type_profiles: {
        magus: {
          id: 'magus',
          budget: { virtue_points: 10, flaw_points: 10 },
          is_magus: true,
          gift_policy: 'required',
          creation_phases: phases,
        },
      },
      abilities: {},
      characteristic_rules: {
        start_points: 7,
        base_max: 3,
        base_min: -3,
        effective_max: 5,
        effective_min: -5,
        costs: [],
      },
      magnitude_points: { free: 0, minor: 1, major: 3 },
      ability_category_order: ['general'],
      art_type_order: ['technique', 'form'],
    },
    i18n: {},
  } as unknown as LocalizedRuleset;
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
    art_scores: [],
    personality_traits: [],
    reputations: [],
  };
}

function html(): string {
  return render(WizardShell).body;
}

/** The opening tag of the single element carrying `testid`. */
function tag(body: string, testid: string): string {
  const open = new RegExp(`<[^>]*data-testid="${testid}"[^>]*>`, 'i').exec(body);
  if (!open) throw new Error(`no element with data-testid="${testid}"`);
  return open[0];
}

/** Every rail step test id, in document order. */
function railSteps(body: string): string[] {
  return [...body.matchAll(/data-testid="(wizard-step-[^"]+)"/g)].map((m) => m[1]);
}

function text(body: string, testid: string): string {
  const whole = new RegExp(`<[^>]*data-testid="${testid}"[^>]*>([\\s\\S]*?)</`, 'i').exec(body)!;
  return whole[1]
    .replace(/<[^>]*>/g, '')
    .replace(/[⁨⁩]/g, '')
    .trim();
}

beforeEach(() => {
  vi.useFakeTimers();
  store.lang = 'en';
  store.mode = 'enforced';
  installFlow();
  store.result = { issues: [] };
  store.view = 'wizard';
  store.wizardStep = 0;
  store.wizardFurthest = 0;
});

afterEach(() => {
  vi.clearAllTimers();
  vi.useRealTimers();
  store.result = null;
  store.view = 'editor';
  store.wizardStep = 0;
  store.wizardFurthest = 0;
});

describe('WizardShell', () => {
  it("lists the profile's phases in their declared order, then Review", () => {
    expect(railSteps(html())).toEqual([
      'wizard-step-concept',
      'wizard-step-type',
      'wizard-step-characteristics',
      'wizard-step-virtues_flaws',
      'wizard-step-review',
    ]);
  });

  // The magus order is the load-bearing case: the House step comes before Virtues
  // & Flaws because its free Virtue lands in that budget.
  it('follows the ruleset rather than a fixed order', () => {
    installFlow(['house_specialisation', 'virtues_flaws']);
    expect(railSteps(html())).toEqual([
      'wizard-step-house_specialisation',
      'wizard-step-virtues_flaws',
      'wizard-step-review',
    ]);
  });

  it('labels each step through its phase-<slug> key, never the raw slug', () => {
    const label = text(html(), 'wizard-step-virtues_flaws');
    expect(label).toContain('Virtues');
    expect(label).not.toContain('virtues_flaws');
  });

  it('marks the current step for assistive tech', () => {
    store.wizardStep = 1;
    expect(tag(html(), 'wizard-step-type')).toContain('aria-current="step"');
  });

  it('gives the rail an accessible name', () => {
    expect(tag(html(), 'wizard-rail')).toMatch(/aria-label="[^"]+"/);
  });

  it('reports progress as a localized step count, not a bare index', () => {
    store.wizardStep = 1;
    expect(text(html(), 'wizard-progress')).toBe('Step 2 of 5');
  });

  it('offers no way back from the first step', () => {
    expect(tag(html(), 'wizard-back')).toContain('disabled');
  });

  it('offers Back once the flow has moved on', () => {
    store.wizardStep = 1;
    store.wizardFurthest = 1;
    expect(tag(html(), 'wizard-back')).not.toContain('disabled');
  });

  it('locks steps the flow has not reached yet', () => {
    const body = html();
    expect(tag(body, 'wizard-step-concept')).not.toContain('disabled');
    expect(tag(body, 'wizard-step-characteristics')).toContain('disabled');
  });

  it('unlocks a step once it has been visited', () => {
    store.wizardStep = 2;
    store.wizardFurthest = 2;
    expect(tag(html(), 'wizard-step-characteristics')).not.toContain('disabled');
  });

  it('blocks Next while the current step holds an error', () => {
    store.result = {
      issues: [{ severity: 'error', code: 'x', phase: 'concept', args: {} }],
    };
    expect(tag(html(), 'wizard-next')).toContain('disabled');
  });

  it('allows Next over a warning — an advisory is not an illegal state', () => {
    store.result = {
      issues: [{ severity: 'warning', code: 'x', phase: 'concept', args: {} }],
    };
    expect(tag(html(), 'wizard-next')).not.toContain('disabled');
  });

  // `data-blocked` styles the rail, but a screen reader needs to be told; the
  // marker text carries that.
  it('announces a blocked step in the rail, not just styles it', () => {
    store.result = {
      issues: [{ severity: 'error', code: 'x', phase: 'concept', args: {} }],
    };
    const step = tag(html(), 'wizard-step-concept');
    expect(step).toContain('data-blocked="true"');
    expect(step).toMatch(/aria-describedby="[^"]+"/);
  });

  it('explains why Next is blocked, and names the mode that lifts the gate', () => {
    store.result = {
      issues: [{ severity: 'error', code: 'x', phase: 'concept', args: {} }],
    };
    expect(html()).toContain('data-testid="wizard-blocked-hint"');
  });

  it('says that nothing is gated while validation is not enforced', () => {
    store.mode = 'silent';
    expect(html()).toContain('data-testid="wizard-unchecked-hint"');
  });

  it('replaces Next with Finish on the closing step', () => {
    store.wizardStep = 4;
    store.wizardFurthest = 4;
    const body = html();
    expect(body).not.toContain('data-testid="wizard-next"');
    expect(body).toContain('data-testid="wizard-finish"');
  });

  it('holds Finish shut while any error remains anywhere', () => {
    store.wizardStep = 4;
    store.wizardFurthest = 4;
    store.result = {
      issues: [{ severity: 'error', code: 'x', phase: 'virtues_flaws', args: {} }],
    };
    expect(tag(html(), 'wizard-finish')).toContain('disabled');
  });

  it('shows the current step body', () => {
    store.wizardStep = 1;
    expect(html()).toContain('data-testid="type-step-name"');
  });

  it("docks a validation panel showing only this step's findings", () => {
    store.result = {
      issues: [
        { severity: 'error', code: 'unknown_equipment', phase: 'review', args: { item: 'w.x' } },
      ],
    };
    // On `concept`, a review-phase finding belongs to the closing step, not here.
    expect(html()).not.toContain('data-code="unknown_equipment"');
  });

  it('localizes the chrome to German', () => {
    store.lang = 'de';
    const body = html();
    expect(text(body, 'wizard-back')).toBe('Zurück');
    expect(text(body, 'wizard-progress')).toBe('Schritt 1 von 5');
  });
});
