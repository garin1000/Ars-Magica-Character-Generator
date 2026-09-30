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
// Short-Ranged Magic halves it for a spell whose Range is beyond Touch. The
// engine surfaces two `spell_level_caps` rows per Technique/Form pair, keyed by
// `range_beyond_touch`; the picker must read the row matching each candidate
// spell's own Range, not just its Technique/Form.
//
// CLAUDE.md: fixed taxonomies stay Rust enums, and the UI must not re-hardcode
// their values — the engine surfaces them (`Magnitude::points` →
// `Ruleset.magnitude_points`, `AbilityCategory::ALL` →
// `Ruleset.ability_category_order`). The "beyond Touch" whitelist follows the
// same rule: the picker reads `ruleset.ranges_beyond_touch`
// (`effective::range_beyond_touch` surfaced), never a Svelte-side list.
describe('SpellTab spell-level cap is range-aware (D28)', () => {
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
    store.effective!.spell_level_caps = [
      { technique: 'art.creo', form: 'art.animal', range_beyond_touch: false, cap: 20 },
      { technique: 'art.creo', form: 'art.animal', range_beyond_touch: true, cap: 9 },
    ];
  }

  it('greys only the beyond-Touch spell when the beyond-Touch cap is lower', () => {
    installRangedSpells();
    store.ruleset!.ruleset.ranges_beyond_touch = ['eye', 'voice', 'sight', 'arcane_connection'];
    const body = html();
    expect(outer(body, `add-${TOUCH_SPELL}`)).toMatch(/aria-disabled="false"/);
    expect(outer(body, `add-${EYE_SPELL}`)).toMatch(/aria-disabled="true"/);
  });

  it('follows the ruleset-surfaced set, not a hardcoded whitelist: a test ruleset naming Touch (not Eye) as beyond-Touch flips which spell is greyed', () => {
    installRangedSpells();
    // A surfaced set that disagrees with the real engine whitelist — if the
    // picker had its own hardcoded Eye/Voice/Sight/Arcane-Connection list
    // anywhere, this would still grey the Eye spell and this test would fail.
    store.ruleset!.ruleset.ranges_beyond_touch = ['touch'];
    const body = html();
    expect(outer(body, `add-${TOUCH_SPELL}`)).toMatch(/aria-disabled="true"/);
    expect(outer(body, `add-${EYE_SPELL}`)).toMatch(/aria-disabled="false"/);
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
});

// X10c (design-x10bc-save-format.md § 3, D73.2): "X10c covers the marker and
// the in-app totals" — a read-only per-spell Casting Total badge, selecting
// between the two figures `casting_totals` already computed via the pure
// `spellCastingTotal` selector (mirrors
// `crates/arm-rules/src/derived/casting.rs::spell_casting_total`). Labelled
// via the Fluent key `spell-casting-total-label` ("Casting Total").
describe('SpellTab in-app Casting Total (D73.2)', () => {
  function installDerived(formulaic: number, withinFocusFormulaic?: number): void {
    store.derived = {
      casting_totals: [
        {
          technique: 'art.creo',
          form: 'art.animal',
          formulaic,
          within_focus:
            withinFocusFormulaic == null
              ? null
              : {
                  focus_art: 0,
                  formulaic: withinFocusFormulaic,
                  ritual: 0,
                  spontaneous_fatiguing: 0,
                  spontaneous_non_fatiguing: 0,
                },
        },
      ],
    } as unknown as DerivedTotals;
  }

  it('shows the base formulaic Casting Total for a known spell, labelled via the Fluent key', () => {
    installDerived(29);
    const badge = outer(html(), `spell-casting-total-${SPELL}-0`);
    expect(badge).toContain('Casting Total');
    expect(badge).toContain('29');
  });

  it('shows the within-focus figure once the player claims the spell is within the Focus', () => {
    installDerived(29, 41);
    store.entity.spells = [{ spell: SPELL, mastery: 1, within_focus: true }];
    expect(outer(html(), `spell-casting-total-${SPELL}-0`)).toContain('41');
  });

  it('hides the total before any derived totals have been computed', () => {
    store.derived = null;
    expect(html()).not.toContain(`data-testid="spell-casting-total-${SPELL}-0"`);
  });
});
