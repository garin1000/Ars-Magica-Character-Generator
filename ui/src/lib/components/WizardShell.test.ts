import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
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
  applyChildhoodPackage: vi.fn(),
}));

import { SCHEMA_VERSION, store } from '../state.svelte';
import WizardShell from './WizardShell.svelte';

const PHASES: CreationPhase[] = ['concept', 'experience', 'characteristics', 'virtues_flaws'];

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
    ability_funding: 'pool',
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
      'wizard-step-experience',
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
    expect(tag(html(), 'wizard-step-experience')).toContain('aria-current="step"');
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

  // S22 (full-audit a11y): a rail step's `aria-describedby` used to reference a
  // SHARED id (`wizard-blocked-hint`) that only rendered for the CURRENT step's
  // own block reason — so a step other than the current one, blocked while the
  // current step is not, pointed at an id absent from the document. Each
  // blocked step must now carry its own self-contained hint.
  it('never lets a blocked step reference a hint id absent from the document', () => {
    store.wizardStep = 1;
    store.wizardFurthest = 1;
    // `concept` (not the current step) is blocked; `experience` (the current
    // step) is not — so the shared nav hint (tied to the CURRENT step) never
    // renders at all, reproducing the dangling reference.
    store.result = {
      issues: [{ severity: 'error', code: 'x', phase: 'concept', args: {} }],
    };
    const body = html();
    expect(body).not.toContain('data-testid="wizard-blocked-hint"');

    const step = tag(body, 'wizard-step-concept');
    const described = /aria-describedby="([^"]+)"/.exec(step);
    expect(described).not.toBeNull();
    for (const id of described![1].split(/\s+/)) {
      expect(body).toContain(`id="${id}"`);
    }
  });

  it("gives each blocked step its own hint text, not just the current step's", () => {
    store.wizardStep = 1;
    store.wizardFurthest = 1;
    store.result = {
      issues: [{ severity: 'error', code: 'x', phase: 'concept', args: {} }],
    };
    const body = html();
    const step = tag(body, 'wizard-step-concept');
    const id = /aria-describedby="([^"]+)"/.exec(step)![1];
    const hint = new RegExp(`id="${id}"[^>]*>([\\s\\S]*?)<`).exec(body);
    expect(hint).not.toBeNull();
    expect(hint![1].trim()).not.toBe('');
  });

  // manual-testing-findings-2026-09-03 #5: Puissant Ability names a target that is
  // bought on the LATER Abilities step, so its dangling-target error is filed there.
  // The V/F step must therefore let the user through, and the pending work must stay
  // visible on the rail — announced, not by colour alone — so it is not silently
  // forgotten the moment the step is left.
  it('carries a pending finding forward to the rail step that owns it', () => {
    installFlow(['virtues_flaws', 'abilities']);
    store.wizardStep = 0;
    store.wizardFurthest = 0;
    store.result = {
      issues: [
        {
          severity: 'error',
          code: 'ability_bonus_dangling_target',
          phase: 'abilities',
          args: { item: 'virtue.puissant_ability', ability: 'ability.awareness', parameter: '' },
        },
      ],
    };
    const body = html();
    // Standing on virtues_flaws: nothing on this step blocks, so the flow moves on.
    expect(tag(body, 'wizard-next')).not.toContain('disabled');
    // And the step that owns the fix is marked, and says so in words.
    expect(tag(body, 'wizard-step-abilities')).toContain('data-blocked="true"');
    expect(text(body, 'wizard-blocked-hint-abilities')).not.toBe('');
  });

  // manual-testing-findings #4c: Improved Characteristics grants 3 more points, and
  // the engine's `characteristic_points_unspent` warning carries NO context, so it
  // cannot ride the docked panel's item trace the way #4a/#4b do. The rail is the
  // only surface left that can keep "you still have points to spend" visible from
  // the Virtues & Flaws step onward.
  function unspentPointsBehind(): void {
    store.wizardStep = 3;
    store.wizardFurthest = 3;
    store.result = {
      issues: [
        {
          severity: 'warning',
          code: 'characteristic_points_unspent',
          phase: 'characteristics',
          args: { cost: '4', points: '10' },
        },
      ],
    };
  }

  it('marks a reached step still holding an open warning', () => {
    unspentPointsBehind();
    expect(tag(html(), 'wizard-step-characteristics')).toContain('data-pending="true"');
  });

  it('says the pending warning in words, not by styling alone', () => {
    unspentPointsBehind();
    const label = text(html(), 'wizard-pending-characteristics');
    expect(label).not.toBe('');
    expect(label).not.toContain('characteristics');

    // A different statement from the blocked marker on the very same step: "still
    // open here" must not sound like "you cannot leave until this is fixed".
    store.result = {
      issues: [
        { severity: 'error', code: 'characteristic_overspent', phase: 'characteristics', args: {} },
      ],
    };
    expect(label).not.toBe(text(html(), 'wizard-blocked-hint-characteristics'));
  });

  // The noise check this gate exists for. Measured against the shipped catalogue
  // (`core_type_conformance.rs::a_fresh_wizard_magus_…`): an untouched magus already
  // warns on 4 of its 11 steps — Virtues & Flaws, House, Abilities and Spells — so an
  // ungated marker would light more than a third of the rail on step one, on steps
  // the player has never opened. Only steps already reached are marked, which is
  // exactly the set the rail lets you click.
  it('leaves a step the flow has not reached yet unmarked, however it validates', () => {
    unspentPointsBehind();
    store.wizardStep = 0;
    store.wizardFurthest = 0;
    expect(tag(html(), 'wizard-step-characteristics')).not.toContain('data-pending');
  });

  it('says the stronger thing only when a step holds an error as well', () => {
    unspentPointsBehind();
    store.result = {
      issues: [
        ...store.result!.issues,
        { severity: 'error', code: 'characteristic_overspent', phase: 'characteristics', args: {} },
      ],
    };
    const step = tag(html(), 'wizard-step-characteristics');
    expect(step).toContain('data-blocked="true"');
    expect(step).not.toContain('data-pending');
  });

  it('leaves Next enabled over a pending warning elsewhere', () => {
    unspentPointsBehind();
    expect(tag(html(), 'wizard-next')).not.toContain('disabled');
  });

  // `::after` content never reaches server-rendered markup, so the glyphs are read
  // from the stylesheet. Two different GLYPHS, not two hues: the pending marker has
  // to be tellable from the blocking one in greyscale as well as in words.
  it('gives the pending marker its own glyph, different from the blocked one', () => {
    const appCss = readFileSync(fileURLToPath(new URL('../../app.css', import.meta.url)), 'utf-8');
    const glyph = (attr: string): string => {
      const rule = new RegExp(
        `\\.wizard-rail-step\\[data-${attr}='true'\\]::after\\s*{([^}]*)}`,
      ).exec(appCss);
      expect(rule).not.toBeNull();
      return /content:\s*'([^']*)'/.exec(rule![1])![1];
    };
    expect(glyph('pending')).not.toBe(glyph('blocked'));
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

  // Legal is not finished: every step of this flow takes a choice, so an untouched
  // character leaves all of them marked — the engine reports which.
  it('marks a step the player has recorded nothing for', () => {
    store.result = { issues: [], completeness: { incomplete_phases: ['characteristics'] } };
    const body = html();
    expect(tag(body, 'wizard-step-characteristics')).toContain('data-incomplete="true"');
    expect(tag(body, 'wizard-step-concept')).not.toContain('data-incomplete');
  });

  // `data-incomplete` is a CSS hook; a screen reader needs the marker in words.
  it('says in words that a step is untouched, not only in styling', () => {
    store.result = { issues: [], completeness: { incomplete_phases: ['concept'] } };
    const label = text(html(), 'wizard-incomplete-concept');
    expect(label).not.toBe('');
    expect(label).not.toContain('wizard-step-incomplete');
    expect(label).not.toContain('concept');
  });

  // guided-creation-review-2026-08 #10 (requested change): the flag behind this
  // marker is the engine's `completeness.incomplete_phases`, NOT "step not opened
  // yet" — so on a fresh character EVERY rail step carried the words "not started"
  // at once, which is noise on the one screen that has to stay scannable (and
  // Slice 11 adds warnings of its own to the same surface).
  //
  // The words go, the announcement stays: deleting the span would leave
  // `data-incomplete` as a style hook only, making the marker a colour/styling-only
  // channel (WCAG 1.4.1). `.sr-only` is the right utility here — this marker must be
  // ANNOUNCED and must take no space.
  it('keeps the incomplete marker present but visually hidden', () => {
    store.result = { issues: [], completeness: { incomplete_phases: ['concept'] } };
    const body = html();
    const marker = tag(body, 'wizard-incomplete-concept');
    // Still in the DOM, still inside the button, so it is part of the accessible
    // name the rail step announces.
    expect(marker).toContain('sr-only');
    expect(text(body, 'wizard-incomplete-concept')).not.toBe('');
    // Not hidden from assistive tech, and not hidden by a mechanism that removes
    // it from the accessibility tree.
    expect(marker).not.toContain('aria-hidden');
    expect(marker).not.toContain('hidden-reserved');
  });

  // The mark says nothing about legality, so it must not touch the gate.
  it('leaves Next enabled over an untouched step', () => {
    store.result = { issues: [], completeness: { incomplete_phases: ['concept'] } };
    expect(tag(html(), 'wizard-next')).not.toContain('disabled');
  });

  // manual-testing-findings #2: the on-step "nothing has been recorded here yet"
  // notice is gone in BOTH states, and with it the reserved line it kept above every
  // step body. The rail marker asserted above is the whole of what remains, so the
  // shell must not have grown a replacement in either state.
  it('says nothing on the step itself about an empty step', () => {
    store.result = { issues: [], completeness: { incomplete_phases: ['concept'] } };
    expect(html()).not.toContain('data-testid="wizard-incomplete-hint"');

    store.result = { issues: [], completeness: { incomplete_phases: ['characteristics'] } };
    expect(html()).not.toContain('data-testid="wizard-incomplete-hint"');
  });

  it('shows the current step body', () => {
    store.wizardStep = 2;
    expect(html()).toContain('data-testid="characteristic-points"');
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
