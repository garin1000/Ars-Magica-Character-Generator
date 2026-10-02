import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { render } from 'svelte/server';

import type {
  CastingTotal,
  DerivedTotals,
  EffectiveScores,
  Entity,
  LocalizedRuleset,
} from '../types';

// The Spells tab reads the shared store singleton (ruleset catalogue + entity +
// engine-derived effective scores) and the Fluent bundle. The store schedules a
// debounced revalidate over the Tauri IPC bridge; mock the bridge so nothing
// reaches a backend. Harness mirrors SpellBudgetBar.test.ts.
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
import SpellTab from './SpellTab.svelte';

const SPELL = 'spell.true_rest_of_the_injured_brute';

/** A minimal localized ruleset: two Arts, one catalogue spell, the Ability
 * advancement table (which prices a Mastery score) and one mastery ability. */
function installRuleset(): void {
  store.ruleset = {
    ruleset: {
      id: 'test',
      version: '1',
      point_items: {},
      type_profiles: {},
      advancement: [
        { score: 1, total_xp: 5 },
        { score: 2, total_xp: 15 },
      ],
      arts: {
        'art.creo': { id: 'art.creo', art_type: 'technique' },
        'art.animal': { id: 'art.animal', art_type: 'form' },
      },
      spells: {
        [SPELL]: { id: SPELL, technique: 'art.creo', form: 'art.animal', level: 20 },
      },
      spell_mastery_abilities: {
        'spell_mastery_ability.quick_casting': {
          id: 'spell_mastery_ability.quick_casting',
          repeatable: true,
        },
      },
      magnitude_points: { free: 0, minor: 1, major: 3 },
      ability_category_order: ['general'],
      art_type_order: ['technique', 'form'],
      ritual_min_level: 20,
    },
    i18n: {
      'art.creo': { name: 'Creo', abbreviation: 'Cr' },
      'art.animal': { name: 'Animal', abbreviation: 'An' },
      [SPELL]: { name: 'True Rest of the Injured Brute' },
      'spell_mastery_ability.quick_casting': { name: 'Quick Casting' },
    },
  } as unknown as LocalizedRuleset;
}

/** A magus knowing the one catalogue spell, mastered at 1 (so the mastery
 * spinner AND the special-ability picker both render on the row). */
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
    saga_year: 1220,
    art_scores: [],
    personality_traits: [],
    reputations: [],
    spells: [{ spell: SPELL, mastery: 1 }],
  };
  store.effective = {
    ability_bonuses: [],
    art_bonuses: [],
    characteristic_caps: {},
    characteristic_floors: {},
    spell_levels_used: 20,
    spell_levels_budget: 120,
    spell_levels_profile_base: 120,
  } as unknown as EffectiveScores;
  store.result = null;
}

/** Render the tab to an HTML string (node env, no DOM). */
function html(): string {
  return render(SpellTab, { props: {} }).body;
}

/**
 * `app.css` as text. CSS is not observable through `render` from `svelte/server`
 * (no stylesheet is attached), and NOT importable as `./app.css?raw` either:
 * vitest stubs CSS modules by extension regardless of the query, so `?raw` would
 * yield `''` and every assertion would pass vacuously.
 */
const appCss = readFileSync(fileURLToPath(new URL('../../app.css', import.meta.url)), 'utf-8');

/** The full outer HTML of the element carrying a data-testid — nesting-aware, so
 * a container's own closing tag is not mistaken for a nested child's. */
function outer(body: string, testid: string): string {
  const open = new RegExp(`<([a-z]+)[^>]*data-testid="${testid}"[^>]*>`, 'i').exec(body);
  if (!open) throw new Error(`no element with data-testid="${testid}"`);
  const tags = new RegExp(`</?${open[1]}\\b[^>]*>`, 'gi');
  tags.lastIndex = open.index;
  let depth = 0;
  let tag: RegExpExecArray | null;
  while ((tag = tags.exec(body))) {
    depth += tag[0].startsWith('</') ? -1 : 1;
    if (depth === 0) return body.slice(open.index, tag.index + tag[0].length);
  }
  throw new Error(`unclosed element with data-testid="${testid}"`);
}

beforeEach(() => {
  vi.useFakeTimers();
  store.lang = 'en';
  installRuleset();
  resetEntity();
});

// Spell names are long ("True Rest of the Injured Brute (CrAn 20)") and the row
// also carries a mastery spinner and a special-ability picker. Side by side the
// three squeeze the name down to one word per line, so the abilities picker takes
// a wrap line of its own below the controls and the name keeps the row's width.
// S24 (full-audit a11y): the free-text search box carries only a placeholder,
// which is not an accessible name — the same defect as S5 (AbilityTab).
describe('SpellTab search box (S24)', () => {
  it('gives the search box an accessible name via Fluent', () => {
    const body = html();
    const input = /<input[^>]*data-testid="spell-search"[^>]*>/.exec(body);
    expect(input).not.toBeNull();
    expect(input![0]).toContain('aria-label="Search…"');
  });
});

describe('SpellTab selected-row layout', () => {
  it('keeps the spell name ahead of the mastery controls in the row', () => {
    const body = html();
    expect(outer(body, `spell-mastery-${SPELL}-0`)).not.toContain('spell-name-');
    expect(body.indexOf(`data-testid="spell-name-${SPELL}-0"`)).toBeLessThan(
      body.indexOf(`data-testid="spell-mastery-${SPELL}-0"`),
    );
  });

  // guided-creation-review-2026-08 #17: the abilities picker (a label plus the "Add
  // special ability" select) used to be the score spinner's sibling inside a
  // content-width column block, so appearing at mastery 1 widened the block, the
  // elastic `.item-name` gave ground and the spinner — a repeated-click control —
  // slid sideways under the pointer. It is now a wrap line of the row itself.
  // SSR does no layout, so what is asserted here is the class hook the
  // `flex-basis: 100%` rule keys on (pinned in `app.css.test.ts`) and the row
  // structure that lets the line wrap below rather than beside.
  it('gives the mastery abilities their own full-width wrap line', () => {
    const body = html();
    const abilities = new RegExp(
      `<[^>]*data-testid="spell-mastery-abilities-${SPELL}-0"[^>]*>`,
    ).exec(body);
    expect(abilities).not.toBeNull();
    expect(abilities![0]).toContain('class="mastery-abilities"');

    // No content-width column wrapper is left to widen: the spinner and the wrap
    // line are both direct items of the row.
    expect(body).not.toContain('spell-mastery-block');
    expect(outer(body, `spell-mastery-${SPELL}-0`)).not.toContain('mastery-abilities');

    // Last in the row, after the remove button: a 100%-basis item claims the line
    // it starts, so anything after it would be pushed onto a third line.
    expect(body.indexOf(`data-testid="spell-remove-${SPELL}-0"`)).toBeLessThan(
      body.indexOf(`data-testid="spell-mastery-abilities-${SPELL}-0"`),
    );
  });
});

// --- 6b8c: who owns the budget bar ------------------------------------------

// Every other input surface takes its budget bar from whoever mounts it — the
// editor's tab in `App.svelte`, the wizard's step through `WizardStep`'s phase
// table. The Spells surface used to be the one exception, mounting
// `SpellBudgetBar` itself, which made the step table's `bar` column read "—" for
// `spells` alone and hid a whole-character budget inside the picker.
describe('SpellTab budget-bar ownership (slice 6b8c)', () => {
  it('mounts no budget bar of its own', () => {
    const body = html();
    expect(body).not.toContain('data-testid="spell-levels-used"');
    expect(body).not.toContain('data-testid="spell-levels-available"');
  });

  it('still renders both picker regions', () => {
    const body = html();
    expect(body).toContain('data-testid="available-title"');
    expect(body).toContain('data-testid="spell-list"');
  });
});

// VA2 (tmp/review/review-round-1-viktor-app.md): the Ritual level floor is read
// from the engine-surfaced `ruleset.ritual_min_level` (mirrors
// `crates/arm-rules/src/spell.rs`'s `RITUAL_MIN_LEVEL` constant), not a local
// literal. The fixture's `ritual_min_level: 20` above matches the shipped
// engine value, so most of these pin the real threshold; the dedicated
// "sources the ritual floor from the engine" test below sets a
// non-canonical value (25) specifically to prove the component tracks the
// engine's number rather than a hardcoded 20 that would happen to agree with
// it.
describe('SpellTab ritual minimum learnable level (VA2)', () => {
  const RITUAL = 'spell.test_general_ritual';
  const ORDINARY = 'spell.test_general_ordinary';

  function installGeneralSpells(): void {
    store.ruleset!.ruleset.spells = {
      ...store.ruleset!.ruleset.spells,
      [RITUAL]: { id: RITUAL, technique: 'art.creo', form: 'art.animal', ritual: true },
      [ORDINARY]: { id: ORDINARY, technique: 'art.creo', form: 'art.animal' },
    };
    store.ruleset!.i18n[RITUAL] = { name: 'Test General Ritual' };
    store.ruleset!.i18n[ORDINARY] = { name: 'Test General Ordinary' };
  }

  it('blocks a General Ritual when fewer than 20 levels remain', () => {
    installGeneralSpells();
    store.effective!.spell_levels_used = 100;
    store.effective!.spell_levels_budget = 119; // remaining = 19
    const body = html();
    expect(outer(body, `add-${RITUAL}`)).toMatch(/aria-disabled="true"/);
  });

  it('allows a General Ritual once exactly 20 levels remain', () => {
    installGeneralSpells();
    store.effective!.spell_levels_used = 100;
    store.effective!.spell_levels_budget = 120; // remaining = 20
    const body = html();
    expect(outer(body, `add-${RITUAL}`)).toMatch(/aria-disabled="false"/);
  });

  it('allows an ordinary General spell with only 1 level remaining', () => {
    installGeneralSpells();
    store.effective!.spell_levels_used = 119;
    store.effective!.spell_levels_budget = 120; // remaining = 1
    const body = html();
    expect(outer(body, `add-${ORDINARY}`)).toMatch(/aria-disabled="false"/);
  });

  it('blocks an ordinary General spell with 0 levels remaining', () => {
    installGeneralSpells();
    store.effective!.spell_levels_used = 120;
    store.effective!.spell_levels_budget = 120; // remaining = 0
    const body = html();
    expect(outer(body, `add-${ORDINARY}`)).toMatch(/aria-disabled="true"/);
  });

  it('sources the ritual floor from the engine, not a local constant', () => {
    installGeneralSpells();
    // A non-canonical floor: if the component still hardcoded 20, 24
    // remaining levels would satisfy it and this would render enabled.
    store.ruleset!.ruleset.ritual_min_level = 25;
    store.effective!.spell_levels_used = 100;
    store.effective!.spell_levels_budget = 124; // remaining = 24, below the engine's 25
    expect(outer(html(), `add-${RITUAL}`)).toMatch(/aria-disabled="true"/);

    store.effective!.spell_levels_budget = 125; // remaining = 25, meets the engine's floor
    expect(outer(html(), `add-${RITUAL}`)).toMatch(/aria-disabled="false"/);
  });
});

// D28 (docs/vf-audit/decisions.md): the spell-level cap is range-aware —
// Short-Ranged Magic halves it for a spell whose Range is beyond Touch. D81.5
// moved this folding fully server-side: the engine now surfaces ONE per-spell
// `spell_caps` row (`EffectiveScores.spell_caps`), keyed by spell id, that
// already has the Range-beyond-Touch halving baked in — the picker no longer
// keys a lookup by Technique/Form/range itself (see `derive.ts::nonTakeableReason`'s
// own doc comment). Two spells sharing a Te/Fo pair can therefore report
// different caps purely because the engine gave them different `spell_caps`
// rows, with no range-set machinery left in this component to prove.
describe('SpellTab per-spell cap gates the Add control (D81.5)', () => {
  const TOUCH_SPELL = 'spell.test_touch_range';
  const EYE_SPELL = 'spell.test_eye_range';

  function installRangedSpells(): void {
    store.ruleset!.ruleset.spells = {
      ...store.ruleset!.ruleset.spells,
      [TOUCH_SPELL]: {
        id: TOUCH_SPELL,
        technique: 'art.creo',
        form: 'art.animal',
        level: 10,
        range: 'touch',
      },
      [EYE_SPELL]: {
        id: EYE_SPELL,
        technique: 'art.creo',
        form: 'art.animal',
        level: 10,
        range: 'eye',
      },
    };
    store.ruleset!.i18n[TOUCH_SPELL] = { name: 'Test Touch-Range Spell' };
    store.ruleset!.i18n[EYE_SPELL] = { name: 'Test Eye-Range Spell' };
    // Same Te/Fo pair, different per-spell caps — exactly what the engine's
    // own Range-beyond-Touch halving produces for these two Ranges.
    store.effective!.spell_caps = [
      { spell: TOUCH_SPELL, cap: 20 },
      { spell: EYE_SPELL, cap: 9 },
    ];
  }

  it('greys only the spell whose own per-spell cap is below its level', () => {
    installRangedSpells();
    const body = html();
    expect(outer(body, `add-${TOUCH_SPELL}`)).toMatch(/aria-disabled="false"/);
    expect(outer(body, `add-${EYE_SPELL}`)).toMatch(/aria-disabled="true"/);
  });

  it('reads the cap by the SPELL id, not a shared Technique/Form figure', () => {
    // Both spells share "art.creo art.animal", so a Te/Fo-keyed lookup would
    // report the same cap for both — only a per-spell lookup can tell them
    // apart, which is the whole point of D81.5.
    installRangedSpells();
    const body = html();
    expect(outer(body, `add-${TOUCH_SPELL}`)).not.toBe(outer(body, `add-${EYE_SPELL}`));
  });
});

// D81.5: the picker's "add within focus" action — appears ONLY when a
// candidate's level exceeds its plain per-spell cap but fits the
// Magical-Focus-doubled `within_focus_cap`. Clicking it is covered end-to-end
// in `SpellTab.client.test.ts` (a real DOM click, which SSR cannot observe);
// this file only proves the markup's presence/absence, same split as every
// other interactive control in this suite.
describe('SpellTab "add within focus" action appears only within the focus-doubled cap (D81.5)', () => {
  const CANDIDATE = 'spell.test_focus_candidate';

  function installCandidate(): void {
    store.ruleset!.ruleset.spells = {
      ...store.ruleset!.ruleset.spells,
      [CANDIDATE]: { id: CANDIDATE, technique: 'art.creo', form: 'art.animal', level: 15 },
    };
    store.ruleset!.i18n[CANDIDATE] = { name: 'Test Focus Candidate' };
  }

  function actionTestid(): string {
    return `add-within-focus-${CANDIDATE}`;
  }

  it('is absent when the level is already within the plain cap', () => {
    installCandidate();
    store.effective!.spell_caps = [{ spell: CANDIDATE, cap: 20, within_focus_cap: 30 }];
    expect(html()).not.toContain(`data-testid="${actionTestid()}"`);
  });

  it('appears when the level exceeds the plain cap but fits the within-focus cap', () => {
    installCandidate();
    store.effective!.spell_caps = [{ spell: CANDIDATE, cap: 10, within_focus_cap: 20 }];
    const body = html();
    expect(body).toContain(`data-testid="${actionTestid()}"`);
    expect(outer(body, actionTestid())).toContain('Add within focus');
  });

  it('is absent when the level exceeds the within-focus cap too', () => {
    installCandidate();
    store.effective!.spell_caps = [{ spell: CANDIDATE, cap: 5, within_focus_cap: 8 }];
    expect(html()).not.toContain(`data-testid="${actionTestid()}"`);
  });

  it('is absent when the character holds no Magical Focus at all (no within_focus_cap)', () => {
    installCandidate();
    store.effective!.spell_caps = [{ spell: CANDIDATE, cap: 10 }];
    expect(html()).not.toContain(`data-testid="${actionTestid()}"`);
  });

  it('gives the action an accessible name naming the spell (not a generic label alone)', () => {
    installCandidate();
    store.effective!.spell_caps = [{ spell: CANDIDATE, cap: 10, within_focus_cap: 20 }];
    const tag = outer(html(), actionTestid());
    // Fluent wraps the interpolated $name in bidi isolate marks, so match
    // loosely rather than pinning the exact byte sequence around it.
    expect(tag).toMatch(/aria-label="Add .*Test Focus Candidate \(15\).*within focus"/);
  });
});

// guided-creation-review-2026-08 #8: `SelectionList.svelte` opens a NEW `<ul>` per
// group, and the row separator was a `border-bottom` suppressed on `:last-child` —
// a selector scoped per PARENT, so the suppression fired once per group. A group
// that carries a header hides that (the next `<h3 class="category">` draws its own
// boundary), but this tab emits a HEADER-LESS trailing group for spells missing
// from the catalogue (no Technique/Form to label), which butted straight against
// the group above it with no divider at all.
describe('SpellTab separates a header-less group from the one above it (#8)', () => {
  const UNKNOWN = 'spell.not_in_catalogue';

  /** One catalogue spell (headed group) plus one absent from it (header-less). */
  function withUnknownSpell(): void {
    store.entity.spells = [{ spell: SPELL, mastery: 1 }, { spell: UNKNOWN }];
  }

  /** The selected side's markup, where the grouped `<ul>`s live. */
  function selectedRegion(body: string): string {
    const start = body.indexOf('class="region region-selected"');
    if (start < 0) throw new Error('no selected region');
    return body.slice(start);
  }

  it('emits the header-less group as a bare sibling list, with no heading to divide it', () => {
    withUnknownSpell();
    const region = selectedRegion(html());
    const lists = [...region.matchAll(/<ul\b[^>]*>/g)];
    expect(lists).toHaveLength(2);
    // Nothing between the two lists but markup-level noise: no `<h3>` boundary, so
    // the separator has to come from the list boundary itself.
    const between = region.slice(
      region.indexOf('</ul>'),
      region.indexOf('<ul', region.indexOf('</ul>')),
    );
    expect(between).not.toContain('<h3');
    expect(
      between
        .replace(/<!--[\s\S]*?-->/g, '')
        .replace(/<\/ul>/, '')
        .trim(),
    ).toBe('');
  });

  it('draws the separator at the list boundary rather than per-parent last-child', () => {
    // The mechanism, read from the stylesheet: CSS is not observable through
    // `render` from `svelte/server`.
    expect(appCss).toMatch(/^\.spell-list \+ \.spell-list.*\{[^}]*border-top:/ms);
    // And the broken selector is gone — any fix rebuilt on `:last-child` has the
    // same per-parent scoping bug.
    expect(appCss).not.toMatch(/^\.spell-list li:last-child/m);
  });
});

// X10c (design-x10bc-save-format.md § 3): a per-spell "within the focus"
// toggle, shown ONLY when the spell's own (Technique, Form) casting-total
// cell carries a within-focus figure — the same `within_focus != null` gate
// `DerivedLabCastingSection.svelte` already uses for its own column. Labelled
// via the `spell-within-focus-label` Fluent key. Red-checkpoint protocol,
// phase 1: `SpellTab.svelte`'s template is untouched, so every assertion
// below fails looking for an element that does not exist yet.
describe('SpellTab within-focus toggle (X10c)', () => {
  function castingTotal(withinFocus: boolean): CastingTotal {
    return {
      technique: 'art.creo',
      form: 'art.animal',
      addends: [],
      ritual_addends: [],
      casting_mod_addends: [],
      formulaic: 29,
      ritual: 29,
      spontaneous_fatiguing: 14,
      spontaneous_non_fatiguing: 14,
      within_focus: withinFocus
        ? {
            focus_art: 0,
            formulaic: 41,
            ritual: 41,
            spontaneous_fatiguing: 20,
            spontaneous_non_fatiguing: 20,
          }
        : null,
      non_standard: {
        voice_penalty: 0,
        gesture_penalty: 0,
        silent: 0,
        still: 0,
        silent_and_still: 0,
        deft_form: false,
      },
      deficient: false,
      unusable: false,
    };
  }

  function installDerived(withinFocus: boolean): void {
    store.derived = {
      casting_totals: [castingTotal(withinFocus)],
    } as unknown as DerivedTotals;
  }

  function toggleTag(body: string): string {
    const match = new RegExp(`<input[^>]*data-testid="spell-within-focus-${SPELL}-0"[^>]*>`).exec(
      body,
    );
    if (!match) throw new Error(`no within-focus toggle for ${SPELL}-0`);
    return match[0];
  }

  it('shows the toggle, labelled via the Fluent key, when the cell carries a within-focus figure', () => {
    installDerived(true);
    const tag = toggleTag(html());
    expect(tag).toContain('type="checkbox"');
    expect(tag).toContain('aria-label="Within focus"');
  });

  it('hides the toggle when the character holds no Magical Focus for this cell', () => {
    installDerived(false);
    expect(html()).not.toContain(`data-testid="spell-within-focus-${SPELL}-0"`);
  });

  it('hides the toggle before any derived totals have been computed', () => {
    store.derived = null;
    expect(html()).not.toContain(`data-testid="spell-within-focus-${SPELL}-0"`);
  });

  it("reflects the spell's own stored claim", () => {
    installDerived(true);
    store.entity.spells = [{ spell: SPELL, mastery: 1, within_focus: true }];
    expect(toggleTag(html())).toContain('checked');
  });

  // UI review 2026-09-30b #1 (HIGH): the checkbox carried only an aria-label,
  // with no sibling text a sighted user could read — every other checkbox in
  // the app (ParameterPicker's multi_ref options, LivingConditionsPicker) pairs
  // one. The aria-label stays (pinned above), so this is additive, not a
  // replacement of the accessible name.
  it('pairs the checkbox with a visible caption, not just an aria-label', () => {
    installDerived(true);
    const label = /<label class="checkbox inline within-focus-toggle">([\s\S]*?)<\/label>/.exec(
      html(),
    );
    expect(label, 'within-focus toggle <label>').not.toBeNull();
    expect(label![1]).toContain('<span>Within focus</span>');
  });
});

// D79 (docs/vf-audit/decisions.md): Potent Magic gets its OWN per-spell
// marker/toggle, independent of the Magical Focus one above — a character's
// Potent Magic field and Magical Focus descriptor need not be the same text.
// Mirrors the within-focus toggle block exactly, gated on
// `within_potent_field != null` instead of `within_focus != null`, labelled
// via `spell-within-potent-field-label`.
describe('SpellTab within-potent-field toggle (D79)', () => {
  function castingTotal(withinPotentField: boolean): CastingTotal {
    return {
      technique: 'art.creo',
      form: 'art.animal',
      addends: [],
      ritual_addends: [],
      casting_mod_addends: [],
      formulaic: 29,
      ritual: 29,
      spontaneous_fatiguing: 14,
      spontaneous_non_fatiguing: 14,
      within_focus: null,
      within_potent_field: withinPotentField
        ? {
            formulaic: 35,
            ritual: 35,
            spontaneous_fatiguing: 17,
            spontaneous_non_fatiguing: 17,
          }
        : null,
      non_standard: {
        voice_penalty: 0,
        gesture_penalty: 0,
        silent: 0,
        still: 0,
        silent_and_still: 0,
        deft_form: false,
      },
      deficient: false,
      unusable: false,
    };
  }

  function installDerived(withinPotentField: boolean): void {
    store.derived = {
      casting_totals: [castingTotal(withinPotentField)],
    } as unknown as DerivedTotals;
  }

  function toggleTag(body: string): string {
    const match = new RegExp(
      `<input[^>]*data-testid="spell-within-potent-field-${SPELL}-0"[^>]*>`,
    ).exec(body);
    if (!match) throw new Error(`no within-potent-field toggle for ${SPELL}-0`);
    return match[0];
  }

  it('shows the toggle, labelled via the Fluent key, when the cell carries a within-potent-field figure', () => {
    installDerived(true);
    const tag = toggleTag(html());
    expect(tag).toContain('type="checkbox"');
    expect(tag).toContain('aria-label="Within Potent Magic field"');
  });

  it('hides the toggle when the character holds no Potent Magic Virtue for this cell', () => {
    installDerived(false);
    expect(html()).not.toContain(`data-testid="spell-within-potent-field-${SPELL}-0"`);
  });

  it('hides the toggle before any derived totals have been computed', () => {
    store.derived = null;
    expect(html()).not.toContain(`data-testid="spell-within-potent-field-${SPELL}-0"`);
  });

  it("reflects the spell's own stored claim", () => {
    installDerived(true);
    store.entity.spells = [{ spell: SPELL, mastery: 1, within_potent_field: true }];
    expect(toggleTag(html())).toContain('checked');
  });

  it('pairs the checkbox with a visible caption, not just an aria-label', () => {
    installDerived(true);
    const label =
      /<label class="checkbox inline within-potent-field-toggle">([\s\S]*?)<\/label>/.exec(html());
    expect(label, 'within-potent-field toggle <label>').not.toBeNull();
    expect(label![1]).toContain('<span>Within Potent Magic field</span>');
  });
});

// D79: the two toggles are gated INDEPENDENTLY — a character may hold a
// Magical Focus, Potent Magic, both, or neither, and this spell's own cell may
// carry either figure, both, or neither, independently of the other. The four
// combinations below are the full gate matrix.
describe('SpellTab focus/potent-field toggle gating is independent (D79)', () => {
  function castingTotal(withinFocus: boolean, withinPotentField: boolean): CastingTotal {
    return {
      technique: 'art.creo',
      form: 'art.animal',
      addends: [],
      ritual_addends: [],
      casting_mod_addends: [],
      formulaic: 29,
      ritual: 29,
      spontaneous_fatiguing: 14,
      spontaneous_non_fatiguing: 14,
      within_focus: withinFocus
        ? {
            focus_art: 0,
            formulaic: 41,
            ritual: 41,
            spontaneous_fatiguing: 20,
            spontaneous_non_fatiguing: 20,
          }
        : null,
      within_potent_field: withinPotentField
        ? {
            formulaic: 35,
            ritual: 35,
            spontaneous_fatiguing: 17,
            spontaneous_non_fatiguing: 17,
          }
        : null,
      non_standard: {
        voice_penalty: 0,
        gesture_penalty: 0,
        silent: 0,
        still: 0,
        silent_and_still: 0,
        deft_form: false,
      },
      deficient: false,
      unusable: false,
    };
  }

  function install(withinFocus: boolean, withinPotentField: boolean): void {
    store.derived = {
      casting_totals: [castingTotal(withinFocus, withinPotentField)],
    } as unknown as DerivedTotals;
  }

  const focusToggle = `data-testid="spell-within-focus-${SPELL}-0"`;
  const potentToggle = `data-testid="spell-within-potent-field-${SPELL}-0"`;

  it('shows only the focus toggle when only a Magical Focus matches this cell (focus-only)', () => {
    install(true, false);
    const body = html();
    expect(body).toContain(focusToggle);
    expect(body).not.toContain(potentToggle);
  });

  it('shows only the potent-field toggle when only Potent Magic matches this cell (potent-only)', () => {
    install(false, true);
    const body = html();
    expect(body).not.toContain(focusToggle);
    expect(body).toContain(potentToggle);
  });

  it('shows both toggles when both match this cell (both)', () => {
    install(true, true);
    const body = html();
    expect(body).toContain(focusToggle);
    expect(body).toContain(potentToggle);
  });

  it('shows neither toggle when neither matches this cell (neither)', () => {
    install(false, false);
    const body = html();
    expect(body).not.toContain(focusToggle);
    expect(body).not.toContain(potentToggle);
  });
});

// X10c (design-x10bc-save-format.md § 3, D73.2) + D79: a read-only per-spell
// Casting Total badge. D79 moved this off a client-side selector over the
// three grid figures (base / within-focus / within-potent-field) onto the
// engine's own per-row computation: `DerivedTotals.spell_casting_totals`,
// index-aligned with `entity.spells` (`derived/casting.rs::spell_casting_total`
// — combines both markers directly, which a client-side reconstruction cannot
// do correctly under Deficient-Art halving, since `halve(a) + halve(b) !=
// halve(a + b)`). The badge is now a plain index lookup, nothing more.
// Labelled via the Fluent key `spell-casting-total` ("Casting Total: { $total }").
describe('SpellTab in-app Casting Total (D73.2, D79)', () => {
  function installDerived(totals: Array<number | null>): void {
    store.derived = {
      spell_casting_totals: totals,
    } as unknown as DerivedTotals;
  }

  it('shows the engine-computed Casting Total for a known spell, labelled via the Fluent key', () => {
    installDerived([29]);
    const badge = outer(html(), `spell-casting-total-${SPELL}-0`);
    expect(badge).toContain('Casting Total');
    expect(badge).toContain('29');
  });

  // D79: the engine combines focus doubling AND the Potent Magic bonus
  // together for a spell marked under both — the badge must show WHATEVER
  // figure the engine computed for this row, never re-derive it client-side.
  it('shows whatever combined figure the engine computed for the row, regardless of which markers are claimed', () => {
    installDerived([47]);
    store.entity.spells = [
      { spell: SPELL, mastery: 1, within_focus: true, within_potent_field: true },
    ];
    expect(outer(html(), `spell-casting-total-${SPELL}-0`)).toContain('47');
  });

  it('hides the total before any derived totals have been computed', () => {
    store.derived = null;
    expect(html()).not.toContain(`data-testid="spell-casting-total-${SPELL}-0"`);
  });

  it('hides the total when the row carries no figure (e.g. a spell absent from the catalogue)', () => {
    installDerived([null]);
    expect(html()).not.toContain(`data-testid="spell-casting-total-${SPELL}-0"`);
  });
});

// D81.8 (docs/vf-audit/decisions.md): a spell touching one of a held
// Incompatible Arts Flaw's two barred combinations — directly or only through a
// requisite — shows a marker NEXT TO its Casting Total badge (the figure itself
// stays visible; D81.15 separately raises a creation-time ERROR for this, via the
// existing generic `invalidSelectionIds` row-styling, not a second UI surface).
describe('SpellTab unusable-spell marker (D81.8)', () => {
  function installDerived(totals: Array<number | null>, unusable: boolean[]): void {
    store.derived = {
      spell_casting_totals: totals,
      spell_casting_unusable: unusable,
    } as unknown as DerivedTotals;
  }

  it('shows the unusable marker next to the Casting Total badge when the spell is flagged', () => {
    installDerived([29], [true]);
    const badge = outer(html(), `spell-casting-total-${SPELL}-0`);
    expect(badge).toContain('29');
    expect(badge).toContain(store.t('derived-unusable'));
  });

  it('shows no marker when the spell is not flagged', () => {
    installDerived([29], [false]);
    const badge = outer(html(), `spell-casting-total-${SPELL}-0`);
    expect(badge).not.toContain(store.t('derived-unusable'));
  });

  it('shows no marker when the engine has not computed any flags at all', () => {
    installDerived([29], []);
    const badge = outer(html(), `spell-casting-total-${SPELL}-0`);
    expect(badge).not.toContain(store.t('derived-unusable'));
  });
});

// UI review 2026-09-30b #5 (MEDIUM): the ": " between the label and the value
// was a literal in the template, not part of a Fluent message — every other
// composite label in this locale family (`spell-mastery-pool`,
// `spell-mastery-xp`) embeds its value via a placeable instead. A source scan,
// not a rendered-output check: "Casting Total: 29" reads identically either
// way, so only reading the template itself can tell the fixed template from
// the composed message.
describe('SpellTab Casting Total composed via one Fluent placeable (Sabine review #5)', () => {
  const source = readFileSync(
    fileURLToPath(new URL('./SpellTab.svelte', import.meta.url)),
    'utf-8',
  );

  it('does not concatenate a hardcoded ": " separator around the label in the template', () => {
    expect(source).not.toMatch(/casting-total-label'\)\s*}\s*:\s*\{total\}/);
  });

  it('interpolates the total into a single Fluent message', () => {
    expect(source).toMatch(/store\.t\('spell-casting-total',\s*\{\s*total:/);
  });
});
