import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import type { AgingApplication, AgingProjection, PersistedSettings } from './ipc';
import type {
  Ability,
  AgingLogEntry,
  CreationPhase,
  Entity,
  EntityTypeProfile,
  EffectiveScores,
  House,
  LocalizedRuleset,
  PhaseRule,
  PointItem,
  Spell,
  ValidationIssue,
} from './types';

// The AppStore methods under test are synchronous; they only *schedule* a
// debounced revalidate via setTimeout, which calls into the Tauri IPC bridge.
// Mock that bridge so the timer (if it ever fires) is inert, and use fake
// timers so it never fires mid-assertion. We assert on entity state directly.
vi.mock('./ipc', () => ({
  loadRuleset: vi.fn(),
  validateEntity: vi.fn().mockResolvedValue({ issues: [] }),
  effectiveScores: vi.fn().mockResolvedValue({
    ability_bonuses: [],
    art_bonuses: [],
    characteristic_caps: {},
    characteristic_floors: {},
  }),
  derivedTotals: vi.fn().mockResolvedValue({
    hermetically_trained: false,
    lab_totals: [],
    casting_totals: [],
    spell_casting_totals: [],
    penetration: [],
    magic_resistance: [],
    combat: [],
    soak: { addends: [], total: 0 },
    encumbrance: { load: 0, burden: 0, total: 0 },
    fatigue: [],
    wounds: [],
    size: 0,
    decrepitude_score: 0,
    warping_score: 0,
    warping_points: 0,
    surfaced_modifiers: [],
  }),
  saveEntity: vi.fn(),
  loadEntity: vi.fn(),
  updateCloseGuard: vi.fn(),
  // C3b: New/Open confirm a discard through the SAME native dialog close/quit
  // uses. The default answer is "discard", so the many setup helpers below that
  // start from a possibly-dirty shared singleton reach their load unchanged; the
  // tests that care override it per case.
  confirmDiscard: vi.fn().mockResolvedValue(true),
  exportMarkdown: vi.fn(),
  exportLabelKeys: vi.fn(),
  applyChildhoodPackage: vi.fn(),
  agingPreview: vi.fn(),
  agingApply: vi.fn(),
  agingRevert: vi.fn(),
  // C4: the four persisted settings are read in ONE call and written with a patch
  // naming only what changed, so choosing one can never destroy another.
  readSettings: vi.fn().mockResolvedValue({
    default_saga_year: 1220,
    lang: null,
    theme: null,
    validation_mode: null,
  }),
  writeSettings: vi.fn().mockResolvedValue(undefined),
  deriveAge: vi.fn().mockResolvedValue({ age: 0, issues: [] }),
  deriveBirthYear: vi.fn().mockResolvedValue(0),
  // CV7: the removal/clear flow's engine round trip (design § 5.5). Defaults to
  // handing the SAME entity back unchanged; tests that care about the
  // conversion override this per case.
  unlinkAbilityParameters: vi.fn().mockImplementation((entity: Entity) => Promise.resolve(entity)),
}));

// Import the singleton after the mock is registered.
import * as ipc from './ipc';
import { store, defaultPickerFilters, DEFAULT_SAGA_YEAR, SCHEMA_VERSION } from './state.svelte';

// The screen the app boots on, captured at import time — before any test or
// `beforeEach` has touched the shared singleton, which is the only moment the
// initial value is still observable.
const bootView = store.view;

// --- Fixtures ---------------------------------------------------------------

function item(overrides: Partial<PointItem> & Pick<PointItem, 'id'>): PointItem {
  return {
    kind: 'virtue',
    magnitude: 'minor',
    categories: ['general'],
    classification: 'narrative',
    entity_kinds: ['character'],
    ...overrides,
  };
}

function ability(id: string, parameter?: string): Ability {
  return { id, category: 'general', ...(parameter ? { parameter } : {}) };
}

/** Build a localized ruleset from items + abilities and install it on the store. */
function installRuleset(
  items: PointItem[],
  abilities: Ability[] = [],
  profiles: LocalizedRuleset['ruleset']['type_profiles'] = {},
): LocalizedRuleset {
  const point_items: Record<string, PointItem> = {};
  for (const it of items) point_items[it.id] = it;
  const abilityMap: Record<string, Ability> = {};
  for (const a of abilities) abilityMap[a.id] = a;
  const ruleset: LocalizedRuleset = {
    ruleset: {
      id: 'test',
      version: '1',
      point_items,
      type_profiles: profiles,
      abilities: abilityMap,
      // Engine-derived taxonomy the real backend ships on every Ruleset payload.
      magnitude_points: { free: 0, minor: 1, major: 3 },
      ability_category_order: ['general', 'academic', 'arcane', 'martial', 'supernatural'],
      art_type_order: ['technique', 'form'],
    },
    i18n: {},
  };
  store.ruleset = ruleset;
  return ruleset;
}

// A profile that mandates The Gift + Hermetic Magus (both free, profile-declared).
// `createCharacter` seeds exactly this data-driven set of mandatory free traits.
const magusProfiles = {
  magus: {
    id: 'magus',
    budget: { virtue_points: 10, flaw_points: 10 },
    permitted_categories: [],
    forbidden_categories: [],
    required_traits: ['virtue.hermetic_magus'],
    gift_policy: 'required' as const,
    gift_id: 'virtue.the_gift',
    hermetically_trained: true,
    order_member: true,
    creation_phases: [],
  },
};
const gift = () => item({ id: 'virtue.the_gift', magnitude: 'free', categories: ['special'] });
const hermeticMagus = () =>
  item({ id: 'virtue.hermetic_magus', magnitude: 'free', categories: ['social_status'] });

/** Reset the shared singleton's entity to a clean character before each test. */
function resetEntity(): void {
  store.entity = {
    schema_version: 11,
    ruleset: { id: 'test', version: '1' },
    entity_kind: 'character',
    type_id: 'companion',
    selections: [],
    characteristics: {} as Entity['characteristics'],
    characteristic_descriptions: {},
    ability_scores: [],
    xp_pool: 0,
    // Present even on this deliberately old-schema fixture: the engine's load-time
    // migration folds the key onto every pre-16 save, so an entity that reaches the
    // store without it is a shape the app never produces.
    ability_funding: 'pool',
    // C8: document state, and required from schema 17 on. Present here for the same
    // reason `ability_funding` is — an entity reaching the store without it is a
    // shape the app never produces, and its presence is what makes the age ↔ birth
    // year derivation live, exactly as it is in a running app.
    saga_year: 1220,
    art_scores: [],
    personality_traits: [],
    reputations: [],
  };
}

/**
 * A persisted-settings payload as `read_settings` returns it: the default saga year
 * always resolved (its default is a rules value the engine applies), the other three
 * honestly `null` when never chosen, so their defaults stay in the one place each
 * already lives.
 */
function persisted(overrides: Partial<PersistedSettings> = {}): PersistedSettings {
  return {
    default_saga_year: 1220,
    lang: null,
    theme: null,
    validation_mode: null,
    ...overrides,
  };
}

beforeEach(() => {
  vi.useFakeTimers();
  // Almost every block below edits a character, which only exists in the editor —
  // and `revalidate()` is deliberately inert on the startup screen. Pin the editor
  // here so each test starts where it was written to run; the startup-screen
  // blocks set their own view.
  store.view = 'editor';
  resetEntity();
  installRuleset([]);
  // D10 surfaced a pre-existing leak: `effective` is a plain field on the
  // shared singleton, so a test that sets `store.effective` directly (rather
  // than through `revalidate()`) left its granted selections visible to every
  // later test in the file. That never mattered while an absent `max_total`
  // meant unlimited; once absent came to mean "capped at one", a stray
  // granted selection from an earlier test was enough to sit a later test's
  // fixture AT its cap before it ever added anything.
  store.effective = null;
});

afterEach(() => {
  // Drop any pending debounced revalidate without running it, then restore.
  vi.clearAllTimers();
  vi.useRealTimers();
});

// --- per-picker filter state (C7) -------------------------------------------

describe('picker filter state', () => {
  beforeEach(() => {
    // Isolate from other tests mutating the shared singleton's filters.
    store.filters = defaultPickerFilters();
  });

  it('defaults every picker to empty filters', () => {
    expect(store.filters.vf.virtue).toEqual({
      search: '',
      magnitude: '',
      category: '',
      taintedOnly: false,
    });
    expect(store.filters.vf.flaw).toEqual({
      search: '',
      magnitude: '',
      category: '',
      taintedOnly: false,
    });
    expect(store.filters.abilities).toEqual({ search: '', category: '' });
    expect(store.filters.spells).toEqual({
      search: '',
      technique: '',
      form: '',
      levelMin: null,
      levelMax: null,
    });
    expect(defaultPickerFilters().derivedArtPicker).toEqual({ technique: '', form: '' });
  });

  it('persists the Derived-Totals Technique/Form picker without dirtying the entity', () => {
    // The Totals tab unmounts on switch, so its Art picker lives on the store's
    // filters. It is not part of the saved entity snapshot, so setting it must
    // leave the dirty flag untouched (dirty derives from the entity alone).
    const dirtyBefore = store.dirty;
    store.filters.derivedArtPicker.technique = 'art.creo';
    store.filters.derivedArtPicker.form = 'art.ignem';

    // Reading it back on the store proves it survives a panel unmount.
    expect(store.filters.derivedArtPicker).toEqual({
      technique: 'art.creo',
      form: 'art.ignem',
    });
    expect(store.dirty).toBe(dirtyBefore);
  });

  it('retains a picker tab’s filter state independently of the others', () => {
    // A tab switch unmounts the picker components, so their filter state lives on
    // the store: mutating one picker leaves the values readable afterwards and
    // does not bleed into the sibling pickers.
    store.filters.vf.virtue.search = 'giant';
    store.filters.vf.virtue.magnitude = 'major';
    store.filters.vf.virtue.category = 'general';
    store.filters.vf.virtue.taintedOnly = true;
    store.filters.abilities.search = 'latin';
    store.filters.abilities.category = 'academic';

    // Returning to the V/F tab restores exactly what was set.
    expect(store.filters.vf.virtue).toEqual({
      search: 'giant',
      magnitude: 'major',
      category: 'general',
      taintedOnly: true,
    });
    // The Abilities tab keeps its own, independent state.
    expect(store.filters.abilities).toEqual({ search: 'latin', category: 'academic' });
    // The untouched flaw picker still holds its defaults.
    expect(store.filters.vf.flaw).toEqual({
      search: '',
      magnitude: '',
      category: '',
      taintedOnly: false,
    });
  });
});

// --- startup view + createCharacter() (M6a) ---------------------------------

describe('createCharacter', () => {
  beforeEach(() => {
    store.currentPath = null;
    store.filters = defaultPickerFilters();
  });

  it('boots on the start view', () => {
    expect(bootView).toBe('start');
  });

  it('creates a character of the chosen type and enters the editor', async () => {
    installRuleset([gift(), hermeticMagus()], [], magusProfiles);
    await store.createCharacter('magus');
    expect(store.entity.type_id).toBe('magus');
    expect(store.view).toBe('editor');
  });

  it("seeds the profile's mandatory free traits (The Gift + Hermetic Magus)", async () => {
    installRuleset([gift(), hermeticMagus()], [], magusProfiles);
    await store.createCharacter('magus');
    const refs = store.entity.selections!.map((s) => s.ref).sort();
    expect(refs).toEqual(['virtue.hermetic_magus', 'virtue.the_gift']);
  });

  it('discards the previous character entirely', async () => {
    installRuleset([item({ id: 'virtue.plain' })], [ability('ability.awareness')]);
    store.addSelection('virtue.plain');
    store.addAbility('ability.awareness');
    store.setIdentity('name', 'Marcus');

    await store.createCharacter('grog');

    expect(store.entity.type_id).toBe('grog');
    expect(store.entity.selections).toEqual([]);
    expect(store.entity.ability_scores).toEqual([]);
    expect(store.entity.name).toBeUndefined();
  });

  it('clears the current file and leaves the document not dirty', async () => {
    installRuleset([]);
    store.currentPath = '/tmp/marcus.armc';
    store.setIdentity('name', 'Marcus');

    await store.createCharacter('companion');

    expect(store.currentPath).toBeNull();
    expect(store.dirty).toBe(false);
  });

  it('resets the picker filters', async () => {
    installRuleset([]);
    store.filters.abilities.search = 'latin';
    store.filters.vf.virtue.magnitude = 'major';

    await store.createCharacter('companion');

    expect(store.filters).toEqual(defaultPickerFilters());
  });
});

// --- the guided wizard (M6b1b) -----------------------------------------------

describe('the guided wizard', () => {
  /** A magus-like profile with a real, ordered flow for the rail to walk. */
  const wizardProfiles = {
    magus: {
      ...magusProfiles.magus,
      creation_phases: [
        'concept',
        'characteristics',
        'house_specialisation',
        'virtues_flaws',
        'experience',
        'abilities',
      ] as CreationPhase[],
    },
    // A2/D56: a companion-shaped profile carrying the same conditional
    // Arts/Spells `PhaseRule`s the shipped `character_types.json` now does, for
    // the `phases_in_force`-respecting rail tests below.
    companion: {
      id: 'companion',
      budget: { virtue_points: 10, flaw_points: 10 },
      permitted_categories: [],
      forbidden_categories: [],
      creation_phases: [
        'concept',
        'characteristics',
        'virtues_flaws',
        'experience',
        'abilities',
        { phase: 'arts', when: { kind: 'hermetically_trained' } },
        { phase: 'spells', when: { kind: 'hermetically_trained' } },
      ] as PhaseRule[],
    },
  };

  /** Install a validation result the wizard's gating reads. */
  function issues(...list: { phase: CreationPhase; severity?: 'error' | 'warning' }[]): void {
    store.result = {
      issues: list.map(({ phase, severity }) => ({
        severity: severity ?? 'error',
        code: 'x',
        phase,
        args: {},
      })),
    };
  }

  /** A saved magus on disk, standing in for a file the user picks. */
  function savedMagus(overrides: Partial<Entity> = {}): Entity {
    return {
      schema_version: SCHEMA_VERSION,
      ruleset: { id: 'test', version: '1' },
      entity_kind: 'character',
      type_id: 'magus',
      name: 'Marcus of Bonisagus',
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
      ...overrides,
    };
  }

  /** Drive the open-into-wizard entry point with `entity` waiting on disk. */
  async function openIntoWizardWith(overrides: Partial<Entity> = {}): Promise<void> {
    vi.mocked(ipc.loadEntity).mockResolvedValue({
      path: '/tmp/marcus.armc',
      entity: savedMagus(overrides),
      migrated_aging_characteristics: [],
    });
    const opening = store.openIntoWizard();
    if (store.discardPromptOpen) store.resolveDiscardPrompt(true);
    await opening;
  }

  beforeEach(() => {
    installRuleset([gift(), hermeticMagus()], [], wizardProfiles);
    store.result = null;
  });

  describe('startWizard', () => {
    it('creates a real character of the chosen type and enters the wizard', async () => {
      await store.startWizard('magus');
      expect(store.view).toBe('wizard');
      expect(store.entity.type_id).toBe('magus');
    });

    it("seeds the profile's mandatory free traits, exactly as createCharacter does", async () => {
      await store.startWizard('magus');
      const refs = store.entity.selections!.map((s) => s.ref).sort();
      expect(refs).toEqual(['virtue.hermetic_magus', 'virtue.the_gift']);
    });

    it('starts a clean document at the first step', async () => {
      store.currentPath = '/tmp/old.armc';
      await store.startWizard('magus');
      expect(store.currentPath).toBeNull();
      expect(store.dirty).toBe(false);
      expect(store.wizardStep).toBe(0);
      expect(store.wizardPhase).toBe('concept');
    });

    // The wizard's entity is a real character, unlike the start screen's
    // placeholder — so validation must actually run, or nothing would ever gate.
    it('validates immediately rather than standing down like the start screen', async () => {
      vi.mocked(ipc.validateEntity).mockClear();
      await store.startWizard('magus');
      expect(ipc.validateEntity).toHaveBeenCalledTimes(1);
    });

    it("walks the profile's phases and then the wizard's own Review step", async () => {
      await store.startWizard('magus');
      expect(store.wizardPhases).toEqual([
        'concept',
        'characteristics',
        'house_specialisation',
        'virtues_flaws',
        'experience',
        'abilities',
        'review',
      ]);
    });

    // A2/D56: the companion profile's Arts/Spells `PhaseRule`s are conditional
    // on `hermetically_trained`. Shipping them unconditionally in the rail (the
    // engine-side fix's whole point) would put an Arts/Spells step in front of
    // every companion in the guided wizard, real training or not — the rail
    // must instead read the same resolved `phases_in_force` the direct-entry
    // tab list does (`App.svelte`), not the raw declared list.
    describe('the rail respects phases_in_force for a conditional phase', () => {
      /** Override the mocked `effectiveScores` response for the next call only. */
      function mockPhasesInForce(phases: string[] | undefined): void {
        vi.mocked(ipc.effectiveScores).mockResolvedValueOnce({
          ability_bonuses: [],
          art_bonuses: [],
          characteristic_caps: {},
          characteristic_floors: {},
          ...(phases === undefined ? {} : { phases_in_force: phases }),
        } as unknown as EffectiveScores);
      }

      it('hides Arts/Spells while the condition is unmet', async () => {
        mockPhasesInForce([
          'concept',
          'characteristics',
          'virtues_flaws',
          'experience',
          'abilities',
        ]);
        await store.startWizard('companion');
        expect(store.wizardPhases).toEqual([
          'concept',
          'characteristics',
          'virtues_flaws',
          'experience',
          'abilities',
          'review',
        ]);
      });

      it('shows Arts/Spells once phases_in_force resolves them', async () => {
        mockPhasesInForce([
          'concept',
          'characteristics',
          'virtues_flaws',
          'experience',
          'abilities',
          'arts',
          'spells',
        ]);
        await store.startWizard('companion');
        expect(store.wizardPhases).toEqual([
          'concept',
          'characteristics',
          'virtues_flaws',
          'experience',
          'abilities',
          'arts',
          'spells',
          'review',
        ]);
      });

      it('shows every declared phase before phases_in_force has ever arrived, rather than collapsing to just Review', async () => {
        mockPhasesInForce(undefined);
        await store.startWizard('companion');
        expect(store.wizardPhases).toEqual([
          'concept',
          'characteristics',
          'virtues_flaws',
          'experience',
          'abilities',
          'arts',
          'spells',
          'review',
        ]);
      });
    });

    /**
     * Owner request: guided creation should hand the player a character funded by
     * its life history, not a pool sitting at 0 — the wizard is where a rules-faithful
     * construction belongs, so it defaults to `life_stages` (unlike direct entry,
     * which stays on the typed pool: {@link createCharacter} is the "I already know
     * the numbers" mode).
     *
     * An EMPTY plan is allocated together with the mode — exactly like a manual
     * {@link store.setAbilityFunding} switch — because every engine read of the
     * plan (`LifeStageRules::budget`, `validate_life_stage_plan`) is gated on its
     * mere presence: without one, an age typed on `concept` before the player
     * even opens `experience` would raise no life-stage finding at all. The empty
     * plan does not itself finish the Experience step, though — see
     * `completeness.rs`'s `phase_is_engaged`, which now reads the plan's content
     * rather than its presence.
     */
    it('defaults new characters to life-stage funding, with an empty plan', async () => {
      await store.startWizard('magus');
      expect(store.entity.ability_funding).toBe('life_stages');
      expect(store.entity.life_stages).toEqual({});
    });
  });

  describe('navigation', () => {
    beforeEach(async () => {
      await store.startWizard('magus');
    });

    it('advances and raises the furthest-reached step', () => {
      store.wizardNext();
      expect(store.wizardStep).toBe(1);
      expect(store.wizardFurthest).toBe(1);
    });

    it('refuses to advance past an error in the current phase', () => {
      issues({ phase: 'concept' });
      store.wizardNext();
      expect(store.wizardStep).toBe(0);
      expect(store.wizardCanAdvance).toBe(false);
    });

    it('advances past a warning — an advisory is not an illegal state', () => {
      issues({ phase: 'concept', severity: 'warning' });
      store.wizardNext();
      expect(store.wizardStep).toBe(1);
    });

    it('goes back freely, even when the phase left behind is broken', () => {
      store.wizardNext();
      issues({ phase: 'characteristics' });
      store.wizardBack();
      expect(store.wizardStep).toBe(0);
    });

    it('never lowers the furthest-reached step by going back', () => {
      store.wizardNext();
      store.wizardNext();
      store.wizardBack();
      expect(store.wizardStep).toBe(1);
      expect(store.wizardFurthest).toBe(2);
    });

    it('is a no-op going back from the first step', () => {
      store.wizardBack();
      expect(store.wizardStep).toBe(0);
    });

    it('jumps to any already-visited step', () => {
      store.wizardNext();
      store.wizardNext();
      store.wizardBack();
      store.wizardGoTo(2);
      expect(store.wizardStep).toBe(2);
    });

    it('refuses to jump past the furthest step reached', () => {
      store.wizardGoTo(3);
      expect(store.wizardStep).toBe(0);
    });

    it('clamps a forward jump at a broken step in between', () => {
      store.wizardNext();
      store.wizardNext();
      store.wizardNext();
      store.wizardBack();
      store.wizardBack();
      store.wizardBack();
      issues({ phase: 'characteristics' }); // step 1, between 0 and 3
      store.wizardGoTo(3);
      expect(store.wizardStep).toBe(1);
    });

    // Slice 5 (#31): the counter is now mirrored onto the entity as a phase slug,
    // and `back()` must leave BOTH standing — that pairing is what keeps rail
    // browsing off the dirty flag while a deliberate Next records progress.
    it('next() raises furthest and back() does not lower it', () => {
      store.wizardNext();
      store.wizardNext();
      expect(store.wizardFurthest).toBe(2);
      store.wizardBack();
      expect(store.wizardStep).toBe(1);
      expect(store.wizardFurthest).toBe(2);
      expect(store.entity.wizard_furthest_phase).toBe(store.wizardPhases[2]);
    });

    // A slug, never an index: an index resolves against `creation_phases`, which is
    // ruleset data and has already changed twice in this review's slices.
    it('next() records the furthest phase as a slug on the entity', () => {
      expect(store.entity.wizard_furthest_phase).toBeUndefined();
      store.wizardNext();
      expect(store.entity.wizard_furthest_phase).toBe('characteristics');
      store.wizardNext();
      expect(store.entity.wizard_furthest_phase).toBe('house_specialisation');
    });

    // The departure step counts too: Next is blocked when the current phase is
    // broken, so a rail jump must not be a way around that same gate.
    it('clamps a forward jump at the step being left, when that step is broken', () => {
      store.wizardNext();
      store.wizardNext();
      store.wizardBack();
      issues({ phase: 'characteristics' }); // the step the user is standing on
      store.wizardGoTo(2);
      expect(store.wizardStep).toBe(1);
    });
  });

  describe('completeness', () => {
    /** Install an engine report of the phases nothing has been recorded for. */
    function untouched(...phases: CreationPhase[]): void {
      store.result = { issues: [], completeness: { incomplete_phases: phases } };
    }

    beforeEach(async () => {
      await store.startWizard('magus');
    });

    // The per-step `phaseIncomplete` getter went with the on-step notice
    // (manual-testing-findings #2). What is left is the list itself, which the rail's
    // per-step `.sr-only` marker and the Review step's outstanding list both read.
    it('reports the untouched phases the engine names, in rail order', () => {
      untouched('concept', 'abilities');
      expect(store.wizardIncompletePhases).toEqual(['concept', 'abilities']);
    });

    // The whole promise of the indicator: it says a step is empty, and changes
    // nothing about what the flow lets the player do.
    it('never gates: an untouched step can still be advanced past and finished on', () => {
      untouched('concept', 'characteristics', 'house_specialisation', 'experience');
      expect(store.wizardIncompletePhases).toContain(store.wizardPhase);
      expect(store.wizardCanAdvance).toBe(true);

      while (store.wizardPhase !== 'review') store.wizardNext();
      expect(store.wizardCanFinish).toBe(true);
      store.finishWizard();
      expect(store.view).toBe('editor');
    });

    it('reports nothing before the first validation result arrives', () => {
      store.result = null;
      expect(store.wizardIncompletePhases).toEqual([]);
    });
  });

  describe('finishing', () => {
    beforeEach(async () => {
      await store.startWizard('magus');
    });

    /** Walk to the terminal Review step. */
    function reachReview(): void {
      while (store.wizardPhase !== 'review') store.wizardNext();
    }

    it('lands in the editor changing nothing but the progress record', () => {
      store.setIdentity('name', 'Marcus');
      reachReview();
      // Snapshot AFTER the walk, not before it: since Slice 5 the walk itself
      // records the furthest phase reached (#31), so this is a claim about Finish
      // and never about the steps that led to it.
      //
      // And Finish clears that record, which is the one change it does make — see
      // the test below for why. So the comparison is against the snapshot with the
      // record removed, rather than a blanket "nothing changed": asserting the
      // weaker claim would stop this test noticing if Finish ever started editing
      // the character itself.
      const before = JSON.stringify(store.entity);
      const expected = JSON.stringify({ ...store.entity, wizard_furthest_phase: undefined });
      store.finishWizard();
      expect(store.view).toBe('editor');
      expect(JSON.stringify(store.entity)).toBe(expected);
      expect(JSON.stringify(store.entity)).not.toBe(before);
    });

    // Finishing ends the guided run, so the record of how far it got stops being
    // true and is cleared. Keeping it would gate a *completed* character: reopened
    // later it would take the restored branch, and if the player had meanwhile
    // introduced an error in the editor the clamp would lock them out of the very
    // steps past the break they needed to reach. The ungated branch exists for a
    // character that is not mid-run, and a finished one is not. Nothing is lost —
    // the clamp only stops skipping ahead, and a finished character has already been
    // everywhere (`canFinish` requires no errors anywhere).
    it('clears the recorded wizard progress, because the run is over', () => {
      reachReview();
      expect(store.entity.wizard_furthest_phase).toBeDefined();
      store.finishWizard();
      expect(store.entity.wizard_furthest_phase).toBeUndefined();
    });

    it('refuses to finish while any error remains, in any phase', () => {
      reachReview();
      issues({ phase: 'abilities' });
      expect(store.wizardCanFinish).toBe(false);
      store.finishWizard();
      expect(store.view).toBe('wizard');
    });

    // The Review step is the only place a finding no creation phase owns can be
    // seen, so it must not block a *step* — but it must block Finish.
    it('finishes over warnings, but not over a Review-phase error', () => {
      reachReview();
      issues({ phase: 'review', severity: 'warning' });
      expect(store.wizardCanFinish).toBe(true);
      issues({ phase: 'review' });
      expect(store.wizardCanFinish).toBe(false);
    });

    it('resets the rail, so the next wizard starts at the first step', () => {
      store.wizardNext();
      reachReview();
      store.finishWizard();
      expect(store.wizardStep).toBe(0);
      expect(store.wizardFurthest).toBe(0);
    });
  });

  describe('the unsaved-changes guard', () => {
    beforeEach(async () => {
      await store.startWizard('magus');
    });

    // Slice 5 (#31) changed this deliberately: Next now records the furthest phase
    // on the entity, so it dirties the document even on a step the user left empty
    // (`canAdvance` gates on errors only). That is the DECIDED trade — progress is
    // only persisted if it can be saved — and it is the reason the two cases below
    // exist as their own tests.
    it('advancing a wizard step dirties the document', () => {
      expect(store.dirty).toBe(false);
      store.wizardNext();
      expect(store.dirty).toBe(true);
      expect(store.closeGuardPayload().dirty).toBe(true);
    });

    it('rail navigation does not dirty the document', async () => {
      // A rail with steps already unlocked, reached WITHOUT an edit: a wizard-saved
      // character restores its furthest step from the file, so the clean baseline is
      // the file's own and every move below is pure browsing.
      await openIntoWizardWith({ wizard_furthest_phase: 'experience' });
      expect(store.dirty).toBe(false);

      store.wizardGoTo(0);
      store.wizardBack();
      store.wizardGoTo(store.wizardFurthest);

      expect(store.dirty).toBe(false);
      expect(store.closeGuardPayload().dirty).toBe(false);
    });

    // The case that distinguishes the rule from "every Next dirties". Next earns a
    // dirty flag by ACTIVATING a step that was not reachable before — that is a real
    // change to the document, because the rail's reach is now stored on it. Stepping
    // forward over ground already covered activates nothing and so changes nothing.
    // Without this, the two tests above are equally satisfied by a cruder rule that
    // dirties on any Next at all, and a player browsing back and forth with Next
    // would be told they had unsaved work they never did.
    it('does not dirty the document when Next re-treads an already-reached step', async () => {
      await openIntoWizardWith({ wizard_furthest_phase: 'experience' });
      expect(store.dirty).toBe(false);

      const furthest = store.wizardFurthest;
      store.wizardGoTo(0);
      store.wizardNext();
      store.wizardNext();

      // Moved, but only within ground already reached — so nothing was activated.
      expect(store.wizardStep).toBeGreaterThan(0);
      expect(store.wizardFurthest).toBe(furthest);
      expect(store.dirty).toBe(false);
      expect(store.closeGuardPayload().dirty).toBe(false);
    });

    it('dirties the document on a real edit, and mirrors that to the close guard', () => {
      store.setIdentity('name', 'Marcus');
      expect(store.dirty).toBe(true);
      expect(store.closeGuardPayload().dirty).toBe(true);
    });

    it('leaves the wizard for the start screen through newDocument, rail reset', async () => {
      store.wizardNext();
      const leaving = store.newDocument();
      if (store.discardPromptOpen) store.resolveDiscardPrompt(true);
      await leaving;
      expect(store.view).toBe('start');
      expect(store.wizardStep).toBe(0);
      expect(store.wizardFurthest).toBe(0);
    });

    // A save records no wizard progress, so an opened character is a finished
    // document: it belongs in the editor, never mid-flow.
    it('lands an opened character in the editor, not back in the wizard', async () => {
      store.wizardNext();
      vi.mocked(ipc.loadEntity).mockResolvedValue({
        path: '/tmp/marcus.armc',
        entity: {
          schema_version: 11,
          ruleset: { id: 'test', version: '1' },
          entity_kind: 'character',
          type_id: 'magus',
          name: 'Marcus',
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
        },
        migrated_aging_characteristics: [],
      });
      const opening = store.open();
      if (store.discardPromptOpen) store.resolveDiscardPrompt(true);
      await opening;
      expect(store.view).toBe('editor');
      expect(store.wizardStep).toBe(0);
    });
  });

  // Slice 5 (#31): the second entry point. A saved character is walked through the
  // guided flow without instantiating a blank one, and how far its rail opens is
  // decided by the phase slug the file carries (or does not).
  describe('opening a saved character into the wizard', () => {
    it('restores furthest from a stored phase slug', async () => {
      await openIntoWizardWith({ wizard_furthest_phase: 'experience' });

      expect(store.view).toBe('wizard');
      // It lands on the stored phase, and that phase is where the rail stops.
      expect(store.wizardPhase).toBe('experience');
      expect(store.wizardPhases[store.wizardFurthest]).toBe('experience');

      // Still clamped exactly as during the original run: nothing past the stored
      // phase was ever reached, so nothing past it may be jumped to.
      store.wizardGoTo(store.wizardFurthest + 1);
      expect(store.wizardPhase).toBe('experience');
    });

    // Slice 2 removed the `type` phase, so saves carrying it exist; an unknown slug
    // must degrade to the ungated branch rather than erroring or locking the rail.
    it('ignores a stored slug the profile does not declare', async () => {
      await openIntoWizardWith({ wizard_furthest_phase: 'type' });

      expect(store.view).toBe('wizard');
      expect(store.wizardPhase).toBe('concept');
      expect(store.wizardPhases[store.wizardFurthest]).toBe('review');
    });

    // Mechanism (a) of "ungated". Without it `furthest` stays at 0 and `goTo`'s own
    // guard (`step > furthest`) refuses every step, however unblocked they are.
    it('an absent slug leaves every step reachable', async () => {
      await openIntoWizardWith();

      const last = store.wizardPhases.length - 1;
      expect(store.wizardFurthest).toBe(last);
      store.wizardGoTo(last);
      expect(store.wizardPhase).toBe('review');
    });

    // Mechanism (b), and deliberately a SEPARATE test: (a) alone yields a rail that
    // looks reachable and still refuses to move, because a forward jump clamps at
    // the first blocking phase. A character built in the editor never passed those
    // gates, so it must not be held to them.
    it('an absent slug is exempt from the blocking clamp', async () => {
      await openIntoWizardWith();
      issues({ phase: 'characteristics' }); // a step in between, holding an error

      store.wizardGoTo(store.wizardPhases.length - 1);
      expect(store.wizardPhase).toBe('review');
      // Nor does that error gate the flow's own controls.
      expect(store.wizardCanAdvance).toBe(true);
      expect(store.wizardCanFinish).toBe(true);
    });

    // The complement of the two above: a wizard-saved character IS still clamped, so
    // the exemption is the absent-slug branch's and not a hole in the gate.
    it('keeps the clamp for a character whose slug restores', async () => {
      await openIntoWizardWith({ wizard_furthest_phase: 'abilities' });
      issues({ phase: 'characteristics' });

      store.wizardGoTo(0);
      store.wizardGoTo(store.wizardFurthest);
      expect(store.wizardPhase).toBe('characteristics');
    });

    it('opening a saved character into the wizard does not instantiate a blank one', async () => {
      await openIntoWizardWith({ wizard_furthest_phase: 'characteristics' });

      expect(store.view).toBe('wizard');
      expect(store.entity.name).toBe('Marcus of Bonisagus');
      expect(store.currentPath).toBe('/tmp/marcus.armc');
      // A just-loaded document is the saved one, so it is not dirty — an
      // instantiation would have replaced it and re-seeded a different baseline.
      expect(store.dirty).toBe(false);
    });

    it('does not offer the wizard for a type_id with no profile in the loaded ruleset', async () => {
      await openIntoWizardWith({ type_id: 'faerie_noble' });

      // No profile means no phases: a wizard with no rail is not a screen to enter.
      expect(store.canEnterWizard).toBe(false);
      expect(store.view).toBe('editor');
      store.enterWizard();
      expect(store.view).toBe('editor');
    });

    it('offers the wizard for a type the loaded ruleset does have a profile for', async () => {
      await openIntoWizardWith();
      expect(store.canEnterWizard).toBe(true);
    });

    it('stands down when the file dialog is cancelled', async () => {
      vi.mocked(ipc.loadEntity).mockResolvedValue(null);
      store.view = 'editor';
      const opening = store.openIntoWizard();
      if (store.discardPromptOpen) store.resolveDiscardPrompt(true);
      await opening;
      expect(store.view).toBe('editor');
    });
  });
});

describe('open() and the startup view', () => {
  /**
   * A full, minimal character standing in for a loaded document. It carries a
   * name so the saved-baseline this load leaves behind stays distinguishable
   * from the blank entity `resetEntity()` installs for every later test — an
   * identical baseline would silently mark those tests' documents clean.
   */
  function loadedEntity(): Entity {
    return {
      schema_version: 11,
      ruleset: { id: 'test', version: '1' },
      entity_kind: 'character',
      type_id: 'companion',
      name: 'Marcus of Bonisagus',
      ability_funding: 'pool',
      saga_year: 1220,
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

  /** Drive open() to completion, confirming a discard prompt if one appears. */
  async function openAndConfirm(): Promise<void> {
    const opening = store.open();
    if (store.discardPromptOpen) store.resolveDiscardPrompt(true);
    await opening;
  }

  beforeEach(() => {
    vi.mocked(ipc.loadEntity).mockReset();
    // Opening a document must work from either screen, so arrange the start one.
    store.view = 'start';
  });

  it('enters the editor on a successful load', async () => {
    vi.mocked(ipc.loadEntity).mockResolvedValue({
      path: '/tmp/marcus.armc',
      entity: loadedEntity(),
      migrated_aging_characteristics: [],
    });

    await openAndConfirm();

    expect(store.view).toBe('editor');
  });

  it('leaves the view alone when the dialog is cancelled', async () => {
    vi.mocked(ipc.loadEntity).mockResolvedValue(null);

    await openAndConfirm();

    expect(store.view).toBe('start');
  });

  // The engine's canonical JSON omits `selections` when it is empty, and serde
  // re-defaults it Rust-side — JavaScript does not. So a saved character with no
  // Virtues or Flaws at all — a bare grog — arrives with the key absent, and an
  // unguarded read used to throw mid-render, aborting the editor's mount and
  // freezing the app on the previous screen. The guarantee is that such a save
  // opens and stays editable, not that the store back-fills the key.
  it('opens a saved character whose empty selections were omitted', async () => {
    installRuleset([item({ id: 'virtue.plain' })]);
    const sparse = loadedEntity() as Partial<Entity>;
    delete sparse.selections;
    vi.mocked(ipc.loadEntity).mockResolvedValue({
      path: '/tmp/rolf.armc',
      entity: sparse as Entity,
      migrated_aging_characteristics: [],
    });

    await openAndConfirm();

    expect(store.view).toBe('editor');
    store.addSelection('virtue.plain');
    expect(store.entity.selections).toEqual([{ ref: 'virtue.plain' }]);
  });

  // Sabine 3 (full-audit round 4). The picker filters are per-DOCUMENT, and the
  // other two document-replacement routes already say so: `newDocument()` and
  // `#instantiateCharacter()` both reset them, the latter under its own test
  // ("resets the picker filters"). `open()` was the one route that did not, which
  // made it the odd one out rather than a decision — nothing in `docs/` recorded
  // an intent for filters to outlive a document.
  //
  // The sharpest case is `derivedArtPicker`, which is not a filter at all but a
  // SELECTION: it chooses which Technique/Form pair the Derived tab computes Lab
  // and Casting Totals for. Carried across an open, the new magus's totals are
  // shown for a pair picked for the previous character — correctly computed by
  // the engine, but answering a question about somebody else.
  it('resets the picker filters, like every other document replacement', async () => {
    vi.mocked(ipc.loadEntity).mockResolvedValue({
      path: '/tmp/marcus.armc',
      entity: loadedEntity(),
      migrated_aging_characteristics: [],
    });
    store.filters.abilities.search = 'latin';
    store.filters.vf.virtue.magnitude = 'major';
    store.filters.derivedArtPicker.technique = 'art.creo';

    await openAndConfirm();

    expect(store.filters).toEqual(defaultPickerFilters());
  });

  it('leaves the filters alone when the open dialog is cancelled', async () => {
    // Nothing was replaced, so nothing is reset — the same rule `documentEpoch`
    // follows one describe below.
    vi.mocked(ipc.loadEntity).mockResolvedValue(null);
    store.filters.abilities.search = 'latin';

    await openAndConfirm();

    expect(store.filters.abilities.search).toBe('latin');
    store.filters = defaultPickerFilters();
  });

  // Slice 3 handoff: `derive.ts::agingMigrationNotice`, both locales' strings and
  // the `OpenedDocument` wire field all landed, but NOTHING carried the value
  // from `ipc.loadEntity()` to a surface — `#fileOps` is private, so no component
  // could reach a value only `open()` receives, and the notice was composed by
  // nobody.
  //
  // Why it is worth a surface at all: the migration is LOSSY. The engine
  // reconstructs the smallest Aging Point total that still reproduces the
  // recorded Characteristic scores, so whatever the original total was is gone —
  // and the next Save writes the reconstruction back as the document's own
  // figures. Silence makes that permanent without the user ever being asked.
  describe('the schema-migration notice', () => {
    it('composes the notice for a document the migration rewrote', async () => {
      vi.mocked(ipc.loadEntity).mockResolvedValue({
        path: '/tmp/marcus.armc',
        entity: loadedEntity(),
        migrated_aging_characteristics: ['com', 'sta'],
      });

      await openAndConfirm();

      const notice = store.migrationNotice;
      expect(notice).not.toBeNull();
      // Both Characteristics named by their localized label, never the enum id.
      for (const id of ['com', 'sta'] as const) {
        expect(notice).toContain(store.t(`characteristic-${id}`));
        expect(notice).not.toContain(`characteristic-${id}`);
      }
    });

    it('stays silent for a document that needed no migration', async () => {
      vi.mocked(ipc.loadEntity).mockResolvedValue({
        path: '/tmp/marcus.armc',
        entity: loadedEntity(),
        migrated_aging_characteristics: [],
      });

      await openAndConfirm();

      expect(store.migrationNotice).toBeNull();
    });

    it('re-composes in the new language when the UI language switches', async () => {
      // Stored as ids, not as a finished sentence: a notice frozen in the
      // language the document was opened in is a second place the language
      // switch fails to reach.
      vi.mocked(ipc.loadEntity).mockResolvedValue({
        path: '/tmp/marcus.armc',
        entity: loadedEntity(),
        migrated_aging_characteristics: ['com'],
      });

      await openAndConfirm();
      const english = store.migrationNotice;
      store.lang = 'de';
      const german = store.migrationNotice;

      expect(english).not.toBeNull();
      expect(german).not.toBeNull();
      expect(german).not.toBe(english);
      expect(german).toContain(store.t('characteristic-com'));
      store.lang = 'en';
    });

    it('clears the notice when the migrated document is closed', async () => {
      // The notice belongs to the document, not to the session: leaving it up
      // over a brand-new character would describe a rewrite that never happened.
      vi.mocked(ipc.loadEntity).mockResolvedValue({
        path: '/tmp/marcus.armc',
        entity: loadedEntity(),
        migrated_aging_characteristics: ['com'],
      });
      await openAndConfirm();
      expect(store.migrationNotice).not.toBeNull();

      await store.newDocument();

      expect(store.migrationNotice).toBeNull();
    });

    it('clears it when a fresh character replaces the migrated one', async () => {
      vi.mocked(ipc.loadEntity).mockResolvedValue({
        path: '/tmp/marcus.armc',
        entity: loadedEntity(),
        migrated_aging_characteristics: ['com'],
      });
      await openAndConfirm();

      await store.createCharacter('companion');

      expect(store.migrationNotice).toBeNull();
    });
  });

  // Sabine #1/#2 (full-audit round 3). A component mounted inside the editor is
  // NOT recreated when the document is replaced: `open()` leaves
  // `view === 'editor'` and never touches the active tab, so `App.svelte`'s
  // editor branch is unchanged and every instance inside it survives. Any
  // per-document UI state such an instance keeps locally — an aura clamp hint,
  // an unanswered destructive confirmation — therefore has no lifecycle of its
  // own and silently retargets at the next character.
  //
  // This counter is the single signal those components key off, so the store
  // gains ONE token rather than each feature gaining its own flag here. The
  // guarantee under test is that it advances on every route that replaces the
  // document, and only on those.
  describe('documentEpoch', () => {
    it('advances when a saved document replaces the open one', async () => {
      vi.mocked(ipc.loadEntity).mockResolvedValue({
        path: '/tmp/marcus.armc',
        entity: loadedEntity(),
        migrated_aging_characteristics: [],
      });
      const before = store.documentEpoch;

      await openAndConfirm();

      expect(store.documentEpoch).not.toBe(before);
    });

    it('does not advance when the open dialog is cancelled', async () => {
      vi.mocked(ipc.loadEntity).mockResolvedValue(null);
      const before = store.documentEpoch;

      await openAndConfirm();

      expect(store.documentEpoch).toBe(before);
    });

    it('advances when the document is discarded for the startup screen', async () => {
      const before = store.documentEpoch;

      const discarding = store.newDocument();
      if (store.discardPromptOpen) store.resolveDiscardPrompt(true);
      await discarding;

      expect(store.documentEpoch).not.toBe(before);
    });

    it('advances when a fresh character replaces the open one', async () => {
      const before = store.documentEpoch;

      await store.createCharacter('companion');

      expect(store.documentEpoch).not.toBe(before);
    });

    it('stays put while the same document is merely edited', () => {
      const before = store.documentEpoch;

      store.setAura(4);
      store.addDevice();

      expect(store.documentEpoch).toBe(before);
    });
  });
});

// Every selection mutator has to survive an entity whose empty `selections` the
// engine omitted, since that entity is what a loaded bare grog is. These drive
// the mutators straight off such an entity, without the open() path.
describe('editing a character whose omitted selections key left it absent', () => {
  /** The store's entity with no `selections` key at all, as a sparse save loads. */
  function sparseEntity(): Entity {
    const sparse = { ...store.entity } as Partial<Entity>;
    delete sparse.selections;
    return sparse as Entity;
  }

  it('adds a virtue/flaw selection', () => {
    installRuleset([item({ id: 'virtue.plain' })]);
    store.entity = sparseEntity();

    store.addSelection('virtue.plain');

    expect(store.entity.selections).toEqual([{ ref: 'virtue.plain' }]);
  });

  it('removes a selection without one to remove', () => {
    installRuleset([]);
    store.entity = sparseEntity();

    store.removeSelectionAt(0);

    expect(store.entity.selections).toEqual([]);
  });

  it('seeds a Mythic Companion type package', async () => {
    installRuleset([item({ id: 'flaw.blatant', kind: 'flaw', magnitude: 'major' })]);
    store.ruleset!.ruleset.mythic_companion_types = {
      'mythic_type.devil_child': {
        id: 'mythic_type.devil_child',
        required_virtues: [{ ref: 'virtue.might' }],
        required_flaws: [
          {
            default: { ref: 'flaw.blatant' },
            constraint: { kind: 'flaw', magnitude: 'major' },
          },
        ],
      },
    };
    store.entity = sparseEntity();

    await store.setMythicType('mythic_type.devil_child');

    expect(store.entity.selections).toEqual([{ ref: 'virtue.might' }, { ref: 'flaw.blatant' }]);
  });

  it('swaps a required Flaw', async () => {
    installRuleset([]);
    store.entity = sparseEntity();

    await store.setMythicRequiredFlaw('flaw.absent', 'flaw.other');

    expect(store.entity.selections).toEqual([{ ref: 'flaw.other' }]);
  });
});

// The startup screen holds no character, only a placeholder entity. Everything
// below pins what that means: an empty type nobody may read as a choice, no
// engine round trip for it, and nothing for the close/quit guard to warn about.
describe('the startup screen placeholder', () => {
  /** Drive newDocument() to completion, confirming a discard prompt if one appears. */
  async function newAndConfirm(): Promise<void> {
    const discarding = store.newDocument();
    if (store.discardPromptOpen) store.resolveDiscardPrompt(true);
    await discarding;
  }

  beforeEach(() => {
    store.currentPath = null;
    store.error = null;
    vi.mocked(ipc.validateEntity).mockClear();
    vi.mocked(ipc.effectiveScores).mockClear();
    vi.mocked(ipc.derivedTotals).mockClear();
  });

  afterEach(() => {
    vi.mocked(ipc.validateEntity).mockResolvedValue({ issues: [] });
    vi.mocked(ipc.saveEntity).mockReset();
    store.error = null;
  });

  it('newDocument() returns to the type choice, discarding the character', async () => {
    installRuleset([item({ id: 'virtue.plain' })]);
    store.addSelection('virtue.plain');
    store.setIdentity('name', 'Marcus');
    store.currentPath = '/tmp/marcus.armc';

    await newAndConfirm();

    expect(store.view).toBe('start');
    expect(store.entity.selections).toEqual([]);
    expect(store.entity.name).toBeUndefined();
    expect(store.currentPath).toBeNull();
  });

  it('leaves the placeholder without a type: no character means no type', async () => {
    await newAndConfirm();
    expect(store.entity.type_id).toBe('');
  });

  // The close/quit guard compares against the last save/load baseline. A
  // placeholder that read as dirty would make every quit from the startup screen
  // prompt about a character that does not exist.
  it('is not dirty, so quitting from the startup screen never prompts', async () => {
    await newAndConfirm();
    expect(store.dirty).toBe(false);
    expect(store.closeGuardPayload().dirty).toBe(false);
  });

  it('never sends the placeholder to the engine', async () => {
    store.view = 'start';

    await store.revalidate();

    expect(ipc.validateEntity).not.toHaveBeenCalled();
    expect(ipc.effectiveScores).not.toHaveBeenCalled();
    expect(ipc.derivedTotals).not.toHaveBeenCalled();
  });

  it('drops a debounced validate the discarded character left pending', async () => {
    store.setIdentity('name', 'Marcus');

    await newAndConfirm();
    vi.runAllTimers();
    await Promise.resolve();

    expect(ipc.validateEntity).not.toHaveBeenCalled();
  });

  it('discards a validate still in flight when the startup screen takes over', async () => {
    let finishStale: (result: { issues: [] }) => void = () => {};
    vi.mocked(ipc.validateEntity).mockReturnValueOnce(
      new Promise((resolve) => {
        finishStale = resolve;
      }) as ReturnType<typeof ipc.validateEntity>,
    );
    const stale = store.revalidate();
    store.result = null;

    store.view = 'start';
    await store.revalidate();

    finishStale({ issues: [] });
    await stale;
    expect(store.result).toBeNull();
  });

  it('retires a validation banner instead of stranding it on the startup screen', async () => {
    vi.mocked(ipc.validateEntity).mockRejectedValueOnce({ kind: 'invalid_entity' });
    await store.revalidate();
    expect(store.error).toEqual({ kind: 'invalid_entity' });

    store.view = 'start';
    await store.revalidate();

    expect(store.error).toBeNull();
  });

  // `error` is one shared banner channel: a failed save must stay visible, since
  // the document it names is still unwritten.
  it('keeps a file-operation banner it does not own', async () => {
    vi.mocked(ipc.saveEntity).mockRejectedValueOnce({ kind: 'io' });
    await store.saveAs();
    expect(store.error).toEqual({ kind: 'io' });

    store.view = 'start';
    await store.revalidate();

    expect(store.error).toEqual({ kind: 'io' });
  });
});

// --- addSelection() ---------------------------------------------------------

describe('addSelection', () => {
  it('dedups a plain (non-repeatable) item: adding twice yields one row', () => {
    installRuleset([item({ id: 'virtue.plain' })]);
    store.addSelection('virtue.plain');
    store.addSelection('virtue.plain');
    expect(store.entity.selections).toEqual([{ ref: 'virtue.plain' }]);
  });

  it('allows a parameterized item to appear several times', () => {
    // D10: a repeat across different targets is no longer the field's
    // default — the fixture must declare its own `max_total` explicitly, the
    // way every shipped repeater (Puissant Ability, Great Characteristic, …)
    // now does in `rules/core/virtues_flaws.json`.
    installRuleset([
      item({
        id: 'virtue.great',
        parameters: [{ key: 'characteristic', type: 'ref', domain: 'characteristic' }],
        max_total: 255,
      }),
    ]);
    store.addSelection('virtue.great');
    store.addSelection('virtue.great');
    expect(store.entity.selections).toEqual([{ ref: 'virtue.great' }, { ref: 'virtue.great' }]);
  });

  it('allows an item with max_per_target > 1 to appear several times', () => {
    // D10: with no parameter, `max_per_target` and `max_total` govern the same
    // set of copies, so `max_total` must explicitly match (or exceed)
    // `max_per_target` — exactly the fix `great_characteristic` /
    // `poor_characteristic` needed on the Rust side.
    installRuleset([item({ id: 'virtue.stacks', max_per_target: 3, max_total: 3 })]);
    store.addSelection('virtue.stacks');
    store.addSelection('virtue.stacks');
    expect(store.entity.selections).toHaveLength(2);
  });

  it('treats an unknown ref as non-repeatable (max one row)', () => {
    installRuleset([]);
    store.addSelection('virtue.unknown');
    store.addSelection('virtue.unknown');
    expect(store.entity.selections).toEqual([{ ref: 'virtue.unknown' }]);
  });

  // max_total slice: the store itself must not exceed an item's total ceiling,
  // even though VirtueFlawTab's disabled predicate is its only production
  // caller today — this is the model-level guard, not a UI-only one.
  describe('max_total ceiling', () => {
    function installCapped(maxTotal?: number): void {
      installRuleset([
        item({
          id: 'virtue.puissant_art',
          parameters: [{ key: 'art', type: 'ref', domain: 'art' }],
          ...(maxTotal !== undefined ? { max_total: maxTotal } : {}),
        }),
      ]);
    }

    it('allows adding up to max_total', () => {
      installCapped(2);
      store.addSelection('virtue.puissant_art');
      store.addSelection('virtue.puissant_art');
      expect(store.entity.selections).toHaveLength(2);
    });

    it('refuses a further bought copy once bought copies alone reach max_total', () => {
      installCapped(2);
      store.addSelection('virtue.puissant_art');
      store.addSelection('virtue.puissant_art');
      store.addSelection('virtue.puissant_art');
      expect(store.entity.selections).toHaveLength(2);
    });

    it('counts a granted copy toward the same cap as a bought one', () => {
      installCapped(2);
      store.effective = {
        granted_selections: [{ ref: 'virtue.puissant_art', params: { art: 'art.perdo' } }],
      } as unknown as EffectiveScores;
      // One bought copy + one granted copy already sits AT the cap of 2.
      store.addSelection('virtue.puissant_art');
      expect(store.entity.selections).toHaveLength(1);
      store.addSelection('virtue.puissant_art');
      expect(store.entity.selections).toHaveLength(1);
    });

    // D10: absent `max_total` now mirrors the engine's own default of 1
    // (once), not unlimited — see `derive.ts::atMaxTotalRefs`'s doc comment.
    it('caps an item with no stated max_total at one (D10 default)', () => {
      installCapped(undefined);
      store.addSelection('virtue.puissant_art');
      store.addSelection('virtue.puissant_art');
      store.addSelection('virtue.puissant_art');
      expect(store.entity.selections).toHaveLength(1);
    });
  });
});

// --- addSpell() / removeSpellAt() -------------------------------------------

describe('addSpell', () => {
  it('adds a fixed spell without a level', () => {
    store.addSpell('spell.pilum_of_fire');
    expect(store.entity.spells).toEqual([{ spell: 'spell.pilum_of_fire' }]);
  });

  it('dedups the same (spell, level) pair', () => {
    store.addSpell('spell.pilum_of_fire');
    store.addSpell('spell.pilum_of_fire');
    expect(store.entity.spells).toEqual([{ spell: 'spell.pilum_of_fire' }]);
  });

  it('stores the chosen level for a General spell', () => {
    store.addSpell('spell.aegis_of_the_hearth', 20);
    expect(store.entity.spells).toEqual([{ spell: 'spell.aegis_of_the_hearth', level: 20 }]);
  });

  it('lets the same General spell coexist at different levels', () => {
    store.addSpell('spell.aegis_of_the_hearth', 20);
    store.addSpell('spell.aegis_of_the_hearth', 25);
    expect(store.entity.spells).toHaveLength(2);
  });
});

// D81.5: the picker's "add within focus" action — same identity/dedupe rules
// as addSpell, but the new row starts with `within_focus: true` already set.
describe('addSpellWithinFocus', () => {
  it('adds a fixed spell already marked within_focus', () => {
    store.addSpellWithinFocus('spell.pilum_of_fire');
    expect(store.entity.spells).toEqual([{ spell: 'spell.pilum_of_fire', within_focus: true }]);
  });

  it('stores the chosen level for a General spell, already marked within_focus', () => {
    store.addSpellWithinFocus('spell.aegis_of_the_hearth', 20);
    expect(store.entity.spells).toEqual([
      { spell: 'spell.aegis_of_the_hearth', level: 20, within_focus: true },
    ]);
  });

  it('dedups the same (spell, level) pair exactly like addSpell', () => {
    store.addSpellWithinFocus('spell.pilum_of_fire');
    store.addSpellWithinFocus('spell.pilum_of_fire');
    expect(store.entity.spells).toEqual([{ spell: 'spell.pilum_of_fire', within_focus: true }]);
  });
});

describe('removeSpellAt', () => {
  it('removes only the row at the given index, keeping order', () => {
    store.addSpell('spell.a');
    store.addSpell('spell.b');
    store.addSpell('spell.c');
    store.removeSpellAt(1);
    expect((store.entity.spells ?? []).map((s) => s.spell)).toEqual(['spell.a', 'spell.c']);
  });
});

// --- adjustSpellMasteryAt() -------------------------------------------------

describe('adjustSpellMasteryAt', () => {
  it('raises the bought mastery of the spell at the given index', () => {
    store.addSpell('spell.pilum_of_fire');
    store.adjustSpellMasteryAt(0, 2, 5);
    expect(store.entity.spells?.[0].mastery).toBe(2);
  });

  it('clamps at 0 (never negative) and at the given max', () => {
    store.addSpell('spell.pilum_of_fire');
    store.adjustSpellMasteryAt(0, -3, 5); // floors at 0
    expect(store.entity.spells?.[0].mastery).toBe(0);
    store.adjustSpellMasteryAt(0, 99, 5); // clamps at max
    expect(store.entity.spells?.[0].mastery).toBe(5);
  });

  it('ignores an out-of-range index', () => {
    store.addSpell('spell.pilum_of_fire');
    store.adjustSpellMasteryAt(9, 1, 5);
    expect(store.entity.spells?.[0].mastery ?? 0).toBe(0);
  });
});

// --- addMasteryAbilityAt() / removeMasteryAbilityAt() -----------------------

describe('mastery special abilities', () => {
  it('appends a chosen mastery ability to the spell at the given index', () => {
    store.addSpell('spell.pilum_of_fire');
    store.addMasteryAbilityAt(0, 'spell_mastery_ability.penetration');
    expect(store.entity.spells?.[0].mastery_abilities).toEqual([
      'spell_mastery_ability.penetration',
    ]);
  });

  it('allows a repeatable ability to be added more than once (keeps duplicates)', () => {
    store.addSpell('spell.pilum_of_fire');
    store.addMasteryAbilityAt(0, 'spell_mastery_ability.quiet_casting');
    store.addMasteryAbilityAt(0, 'spell_mastery_ability.quiet_casting');
    expect(store.entity.spells?.[0].mastery_abilities).toEqual([
      'spell_mastery_ability.quiet_casting',
      'spell_mastery_ability.quiet_casting',
    ]);
  });

  it('removes exactly one instance at the given position', () => {
    store.addSpell('spell.pilum_of_fire');
    store.addMasteryAbilityAt(0, 'spell_mastery_ability.quiet_casting');
    store.addMasteryAbilityAt(0, 'spell_mastery_ability.penetration');
    store.addMasteryAbilityAt(0, 'spell_mastery_ability.quiet_casting');
    store.removeMasteryAbilityAt(0, 0);
    expect(store.entity.spells?.[0].mastery_abilities).toEqual([
      'spell_mastery_ability.penetration',
      'spell_mastery_ability.quiet_casting',
    ]);
  });
});

// --- setSpellLevelAt() ------------------------------------------------------

describe('setSpellLevelAt', () => {
  it('sets the level on the indexed spell, leaving others', () => {
    store.addSpell('spell.aegis_of_the_hearth', 5);
    store.addSpell('spell.wind_at_back', 5);
    store.setSpellLevelAt(0, 200);
    expect(store.entity.spells?.[0].level).toBe(200);
    expect(store.entity.spells?.[1].level).toBe(5);
  });

  it('marks the document dirty', () => {
    store.addSpell('spell.aegis_of_the_hearth', 5);
    store.setSpellLevelAt(0, 30);
    expect(store.dirty).toBe(true);
  });

  it('ignores an out-of-range index', () => {
    store.addSpell('spell.aegis_of_the_hearth', 5);
    store.setSpellLevelAt(9, 200);
    expect(store.entity.spells?.[0].level).toBe(5);
  });
});

// --- Phase 7: age, personality traits, reputations -------------------------

describe('setAge', () => {
  it('stores a positive integer and clears on null', () => {
    store.setAge(30);
    expect(store.entity.age).toBe(30);
    store.setAge(null);
    expect(store.entity.age).toBe(null);
  });

  it('clamps non-positive/non-finite to null', () => {
    store.setAge(0);
    expect(store.entity.age).toBe(null);
  });
});

// --- Slice 12 (#25): the saga year, and age ↔ birth year as two views ------

describe('the saga year and the age ↔ birth-year link', () => {
  beforeEach(async () => {
    vi.mocked(ipc.readSettings).mockResolvedValue(persisted({ default_saga_year: 1220 }));
    vi.mocked(ipc.writeSettings).mockResolvedValue(undefined);
    vi.mocked(ipc.deriveAge).mockClear();
    vi.mocked(ipc.deriveBirthYear).mockClear();
    await store.loadSettings();
  });

  afterEach(() => {
    store.sagaIssues = [];
    vi.mocked(ipc.deriveAge).mockReset().mockResolvedValue({ age: 0, issues: [] });
    vi.mocked(ipc.deriveBirthYear).mockReset().mockResolvedValue(0);
  });

  it('loads the persisted default rather than assuming one', async () => {
    // The default lives in the engine (a rules value) and is applied by the settings
    // reader in `arm-app`; the frontend only ever reports what it was told. Since C8
    // it seeds new documents rather than governing every open one.
    vi.mocked(ipc.readSettings).mockResolvedValue(persisted({ default_saga_year: 1230 }));
    await store.loadSettings();
    expect(store.defaultSagaYear).toBe(1230);
  });

  it('derives the age from a typed birth year, against the saga year', async () => {
    vi.mocked(ipc.deriveAge).mockResolvedValue({ age: 30, issues: [] });
    store.setBirthYear(1190);
    await vi.runAllTimersAsync();

    expect(vi.mocked(ipc.deriveAge)).toHaveBeenCalledWith(1220, 1190);
    expect(store.entity.birth_year).toBe(1190);
    expect(store.entity.age).toBe(30);
  });

  it('derives the birth year from a typed age, against the saga year', async () => {
    vi.mocked(ipc.deriveBirthYear).mockResolvedValue(1190);
    store.setAge(30);
    await vi.runAllTimersAsync();

    expect(vi.mocked(ipc.deriveBirthYear)).toHaveBeenCalledWith(1220, 30);
    expect(store.entity.age).toBe(30);
    expect(store.entity.birth_year).toBe(1190);
  });

  it('derives nothing from an emptied field, rather than dating anything to year 0', async () => {
    store.setBirthYear(null);
    store.setAge(null);
    await vi.runAllTimersAsync();
    expect(vi.mocked(ipc.deriveAge)).not.toHaveBeenCalled();
    expect(vi.mocked(ipc.deriveBirthYear)).not.toHaveBeenCalled();
  });

  it('clamps the age to zero and warns when the saga year precedes the birth year', async () => {
    // The clamp and the advisory are the engine's, not the store's: `birth_year` is
    // i32 and `age` is u32, so the subtraction has exactly one correct home.
    vi.mocked(ipc.deriveAge).mockResolvedValue({
      age: 0,
      issues: [
        {
          severity: 'warning',
          code: 'saga_year_before_birth_year',
          phase: 'concept',
          args: { saga_year: '1220', birth_year: '1250' },
        },
      ],
    });
    store.setBirthYear(1250);
    await vi.runAllTimersAsync();

    expect(store.entity.age).toBe(0);
    expect(store.sagaIssues.map((issue) => issue.code)).toEqual(['saga_year_before_birth_year']);

    // And it clears again once the pair becomes possible.
    vi.mocked(ipc.deriveAge).mockResolvedValue({ age: 30, issues: [] });
    store.setBirthYear(1190);
    await vi.runAllTimersAsync();
    expect(store.sagaIssues).toEqual([]);
  });

  it('dirties the document when either half of the stored pair is edited', async () => {
    vi.mocked(ipc.deriveBirthYear).mockResolvedValue(1190);
    await store.createCharacter('companion');
    expect(store.dirty).toBe(false);
    store.setAge(30);
    await vi.runAllTimersAsync();
    expect(store.dirty).toBe(true);

    vi.mocked(ipc.deriveAge).mockResolvedValue({ age: 30, issues: [] });
    await store.createCharacter('companion');
    expect(store.dirty).toBe(false);
    store.setBirthYear(1190);
    await vi.runAllTimersAsync();
    expect(store.dirty).toBe(true);
  });

  // C8 DELETED a test here: "changes no stored value and does not dirty the document
  // when only the saga year moves". Half of it was a contract this slice reverses —
  // the saga year IS a stored value now, so moving it MUST dirty the document — and
  // the surviving half (D3.3: it rewrites neither the age nor the birth year) is
  // asserted, against the new contract, by "dirties the document when the saga year
  // moves, and rewrites nothing" in the C8 block below. Keeping a renamed copy here
  // would have been a second assertion of the same thing.

  it('uses the new saga year for the next edit, and only then', async () => {
    store.setSagaYear(1230);
    await vi.runAllTimersAsync();

    vi.mocked(ipc.deriveAge).mockResolvedValue({ age: 40, issues: [] });
    store.setBirthYear(1190);
    await vi.runAllTimersAsync();
    expect(vi.mocked(ipc.deriveAge)).toHaveBeenCalledWith(1230, 1190);
  });
});

// --- C8: the saga year belongs to the saga, not the installation -------------

describe('the saga year is document state, and settings keep only a default', () => {
  /** Open a saved document with `overrides` applied, so the baseline is clean. */
  async function openSaved(overrides: Partial<Entity>): Promise<void> {
    vi.mocked(ipc.loadEntity).mockResolvedValue({
      path: '/tmp/saga.armc',
      entity: { ...store.entity, ...overrides },
      migrated_aging_characteristics: [],
    });
    const opening = store.open();
    if (store.discardPromptOpen) store.resolveDiscardPrompt(true);
    await opening;
  }

  beforeEach(async () => {
    installRuleset([]);
    vi.mocked(ipc.readSettings).mockResolvedValue(persisted({ default_saga_year: 1197 }));
    vi.mocked(ipc.writeSettings).mockClear();
    await store.loadSettings();
  });

  afterEach(async () => {
    vi.mocked(ipc.readSettings).mockResolvedValue(persisted());
    await store.loadSettings();
    vi.mocked(ipc.deriveAge).mockReset().mockResolvedValue({ age: 0, issues: [] });
    vi.mocked(ipc.deriveBirthYear).mockReset().mockResolvedValue(0);
  });

  it('reads the stored default under its own key', () => {
    expect(store.defaultSagaYear).toBe(1197);
  });

  it('stamps the configured default onto a new character', async () => {
    // The whole point of the slice: a storyguide who runs an Iberia saga gets an
    // Iberia character, and the number travels WITH the document from then on.
    await store.createCharacter('companion');
    expect(store.entity.saga_year).toBe(1197);
  });

  it('derives the age against the DOCUMENT year, not the setting', async () => {
    await store.createCharacter('companion');
    store.setSagaYear(1220);
    vi.mocked(ipc.deriveAge).mockResolvedValue({ age: 30, issues: [] });
    store.setBirthYear(1190);
    await vi.runAllTimersAsync();

    // 1220 — the document's — even though the installation's default is 1197.
    expect(vi.mocked(ipc.deriveAge)).toHaveBeenCalledWith(1220, 1190);
  });

  it('dirties the document when the saga year moves, and rewrites nothing', async () => {
    // A just-loaded document IS the saved one — the only way to a clean baseline
    // that already carries both halves of the pair.
    await openSaved({ age: 30, birth_year: 1167, saga_year: 1197 });
    expect(store.dirty).toBe(false);

    store.setSagaYear(1220);
    await vi.runAllTimersAsync();

    expect(store.entity.saga_year).toBe(1220);
    // It IS a stored value now, so the guard must see it — the year is part of what
    // an unsaved document would lose.
    expect(store.dirty).toBe(true);
    expect(store.closeGuardPayload().dirty).toBe(true);
    // D3.3 still holds: the year is the reference the pair is measured against, never
    // a rewrite of it. A silent recompute would fabricate ages that skipped their
    // aging rolls.
    expect(store.entity.age).toBe(30);
    expect(store.entity.birth_year).toBe(1167);
    // And it is not a preference, so nothing is written to the settings file.
    expect(vi.mocked(ipc.writeSettings)).not.toHaveBeenCalled();
  });

  it('persists the default without touching the open document', async () => {
    await openSaved({ saga_year: 1197 });

    store.setDefaultSagaYear(1230);
    await vi.runAllTimersAsync();

    expect(store.defaultSagaYear).toBe(1230);
    // A patch naming ONLY the new key: the Rust side is read-modify-write, so this
    // cannot disturb the theme or the language sharing the file.
    expect(vi.mocked(ipc.writeSettings)).toHaveBeenCalledWith({ default_saga_year: 1230 });
    // The open document was built for another saga and keeps its own year.
    expect(store.entity.saga_year).toBe(1197);
    expect(store.dirty).toBe(false);
  });

  it('keeps the year an opened save carries', async () => {
    await openSaved({ saga_year: 1220 });

    // The installation's default is 1197; the document's own year wins and the
    // document is not dirtied by having been opened.
    expect(store.entity.saga_year).toBe(1220);
    expect(store.dirty).toBe(false);
  });
});

describe('personality traits', () => {
  it('adds, names, values (clamped to ±6), and removes by index', () => {
    store.addPersonalityTrait();
    store.setPersonalityTraitName(0, 'Brave');
    store.setPersonalityTraitValue(0, 9);
    expect(store.entity.personality_traits).toEqual([{ name: 'Brave', value: 6 }]);
    store.addPersonalityTrait();
    store.removePersonalityTraitAt(0);
    expect(store.entity.personality_traits).toHaveLength(1);
  });
});

describe('reputations', () => {
  it('adds from a grant (kind + score), edits content, removes by index', () => {
    store.addReputation('local', 4);
    store.setReputationContent(0, 'dragon slayer');
    expect(store.entity.reputations).toEqual([
      { kind: 'local', score: 4, content: 'dragon slayer' },
    ]);
    store.removeReputationAt(0);
    expect(store.entity.reputations).toEqual([]);
  });

  it('writes the description a granted slot is filled with in one go', () => {
    // The panel has no add button: the first keystroke in an empty granted slot
    // is what creates the row, description and all.
    store.addReputation('local', 4, 'dragon slayer');
    expect(store.entity.reputations).toEqual([
      { kind: 'local', score: 4, content: 'dragon slayer' },
    ]);
  });

  it('deletes the row when its description is emptied', () => {
    // An empty row is not a choice — and `Entity::normalize` sorts on content, so
    // a surviving empty row would re-sort ahead of every described one on reload.
    store.addReputation('local', 4, 'dragon slayer');
    store.setReputationContent(0, '');
    expect(store.entity.reputations).toEqual([]);
  });

  it('sets the kind a wildcard slot was resolved to, keeping the description', () => {
    // Famous fixes no type, so picking one IS a choice and persists on its own.
    store.addReputation('local', 4, 'dragon slayer');
    store.setReputationKind(0, 'hermetic');
    expect(store.entity.reputations).toEqual([
      { kind: 'hermetic', score: 4, content: 'dragon slayer' },
    ]);
  });
});

describe('magic possessions', () => {
  it('sets a signed aura and clears to 0 on null', () => {
    store.setAura(-3);
    expect(store.entity.aura).toBe(-3);
    store.setAura(null);
    expect(store.entity.aura).toBe(0);
  });

  it('adds, edits (name + non-negative level) and removes devices by index', () => {
    store.addDevice();
    store.setDeviceName(0, 'Wand');
    store.setDeviceLevel(0, -5);
    expect(store.entity.devices).toEqual([{ name: 'Wand', level: 0 }]);
    store.setDeviceLevel(0, 20);
    store.addDevice();
    store.removeDeviceAt(1);
    expect(store.entity.devices).toEqual([{ name: 'Wand', level: 20 }]);
  });

  it('adds a familiar, edits name and cords, and removes it', () => {
    store.addFamiliar();
    store.setFamiliarName('Corax');
    store.setFamiliarCord('bronze', 3);
    store.setFamiliarCord('gold', -2);
    expect(store.entity.familiar).toEqual({
      name: 'Corax',
      animal: '',
      might: null,
      characteristics: {},
      size: 0,
      personality_traits: [],
      cord_gold: 0,
      cord_silver: 0,
      cord_bronze: 3,
      powers: [],
    });
    store.removeFamiliar();
    expect(store.entity.familiar).toBeNull();
  });

  it('seeds a familiar with the full empty statblock', () => {
    store.addFamiliar();
    expect(store.entity.familiar).toEqual({
      name: '',
      animal: '',
      might: null,
      characteristics: {},
      size: 0,
      personality_traits: [],
      cord_gold: 0,
      cord_silver: 0,
      cord_bronze: 0,
      powers: [],
    });
  });

  it('edits the familiar animal and its signed Size without clamping', () => {
    store.addFamiliar();
    store.setFamiliarAnimal('raven');
    store.setFamiliarSize(-4);
    expect(store.entity.familiar?.animal).toBe('raven');
    expect(store.entity.familiar?.size).toBe(-4);
    // Size is signed and truncated, never clamped at 0 — a raven is smaller than 0.
    store.setFamiliarSize(2.7);
    expect(store.entity.familiar?.size).toBe(2);
  });

  it("sets, scores and clears the familiar's own Magic Might", () => {
    store.addFamiliar();
    store.setFamiliarMightRealm('magic');
    expect(store.entity.familiar?.might).toEqual({ realm: 'magic', score: 0 });
    store.setFamiliarMightScore(10);
    expect(store.entity.familiar?.might).toEqual({ realm: 'magic', score: 10 });
    store.setFamiliarMightScore(-3);
    expect(store.entity.familiar?.might).toEqual({ realm: 'magic', score: 0 });
    store.setFamiliarMightRealm('faerie');
    expect(store.entity.familiar?.might).toEqual({ realm: 'faerie', score: 0 });
    store.clearFamiliarMight();
    expect(store.entity.familiar?.might).toBeNull();
  });

  it('sets a familiar Characteristic and deletes it at zero', () => {
    store.addFamiliar();
    store.setFamiliarCharacteristic('int', -3);
    store.setFamiliarCharacteristic('qik', 4);
    expect(store.entity.familiar?.characteristics).toEqual({ int: -3, qik: 4 });
    store.setFamiliarCharacteristic('qik', 0);
    expect(store.entity.familiar?.characteristics).toEqual({ int: -3 });
  });

  it("adds, edits and removes the familiar's Personality Traits", () => {
    store.addFamiliar();
    store.addFamiliarPersonalityTrait();
    store.setFamiliarPersonalityTraitName(0, 'Loyal (Marcus)');
    store.setFamiliarPersonalityTraitValue(0, 3);
    expect(store.entity.familiar?.personality_traits).toEqual([
      { name: 'Loyal (Marcus)', value: 3 },
    ]);
    store.addFamiliarPersonalityTrait();
    store.removeFamiliarPersonalityTraitAt(1);
    expect(store.entity.familiar?.personality_traits).toHaveLength(1);
  });

  it("adds, edits and removes the familiar's bond-invested powers", () => {
    store.addFamiliar();
    store.addFamiliarPower();
    store.setFamiliarPowerName(0, 'Mental communication');
    store.setFamiliarPowerLevel(0, 15);
    expect(store.entity.familiar?.powers).toEqual([{ name: 'Mental communication', level: 15 }]);
    store.setFamiliarPowerLevel(0, -5);
    expect(store.entity.familiar?.powers).toEqual([{ name: 'Mental communication', level: 0 }]);
    store.addFamiliarPower();
    store.removeFamiliarPowerAt(1);
    expect(store.entity.familiar?.powers).toHaveLength(1);
  });

  it('ignores every familiar statblock edit when there is no familiar', () => {
    expect(store.entity.familiar).toBeUndefined();
    store.setFamiliarName('Corvus');
    store.setFamiliarAnimal('raven');
    store.setFamiliarSize(-4);
    store.setFamiliarCord('gold', 3);
    store.setFamiliarMightRealm('magic');
    store.setFamiliarMightScore(10);
    store.clearFamiliarMight();
    store.setFamiliarCharacteristic('int', -3);
    store.addFamiliarPersonalityTrait();
    store.setFamiliarPersonalityTraitName(0, 'Loyal');
    store.setFamiliarPersonalityTraitValue(0, 3);
    store.removeFamiliarPersonalityTraitAt(0);
    store.addFamiliarPower();
    store.setFamiliarPowerName(0, 'Speech');
    store.setFamiliarPowerLevel(0, 20);
    store.removeFamiliarPowerAt(0);
    expect(store.entity.familiar).toBeFalsy();
  });

  it("keeps the familiar's statblock out of the character's own fields", () => {
    store.addFamiliar();
    store.setFamiliarMightRealm('faerie');
    store.setFamiliarMightScore(25);
    store.setFamiliarCharacteristic('int', 3);
    store.addFamiliarPower();
    store.setFamiliarPowerLevel(0, 400);
    store.addFamiliarPersonalityTrait();
    // The magus's own Might / powers / Characteristics / traits are untouched: the
    // familiar is a separate creature, not part of the character's point-buy.
    expect(store.entity.might).toBeFalsy();
    expect(store.entity.powers ?? []).toEqual([]);
    expect(store.entity.characteristics ?? {}).toEqual({});
    expect(store.entity.personality_traits ?? []).toEqual([]);
  });

  it('adds a talisman with its identity, and removes it', () => {
    store.addTalisman();
    expect(store.entity.talisman).toEqual({ description: '', attunements: [], effects: [] });
    store.setTalismanDescription('An ash staff shod with silver');
    expect(store.entity.talisman?.description).toBe('An ash staff shod with silver');
    store.removeTalisman();
    expect(store.entity.talisman).toBeNull();
  });

  it('adds, edits and removes talisman attunements by index', () => {
    store.addTalisman();
    store.addTalismanAttunement();
    store.setTalismanAttunementDescription(0, 'Attuned to fire');
    store.setTalismanAttunementBonus(0, 5);
    expect(store.entity.talisman?.attunements).toEqual([
      { description: 'Attuned to fire', bonus: 5 },
    ]);
    store.removeTalismanAttunementAt(0);
    expect(store.entity.talisman?.attunements).toEqual([]);
  });

  it('adds, edits (name + non-negative level) and removes instilled effects', () => {
    store.addTalisman();
    store.addTalismanEffect();
    store.setTalismanEffectName(0, 'Wielding the Invisible Sling');
    store.setTalismanEffectLevel(0, -5);
    expect(store.entity.talisman?.effects).toEqual([
      { name: 'Wielding the Invisible Sling', level: 0 },
    ]);
    store.setTalismanEffectLevel(0, 15);
    store.addTalismanEffect();
    store.removeTalismanEffectAt(1);
    expect(store.entity.talisman?.effects).toEqual([
      { name: 'Wielding the Invisible Sling', level: 15 },
    ]);
  });

  it('ignores every talisman mutator while there is no talisman', () => {
    // The panel only renders these controls with a talisman present, but a guard
    // per mutator keeps a stray call from conjuring one out of nothing.
    store.setTalismanDescription('An ash staff');
    store.addTalismanAttunement();
    store.setTalismanAttunementDescription(0, 'Warding');
    store.setTalismanAttunementBonus(0, 5);
    store.removeTalismanAttunementAt(0);
    store.addTalismanEffect();
    store.setTalismanEffectName(0, 'Lamp Without Flame');
    store.setTalismanEffectLevel(0, 10);
    store.removeTalismanEffectAt(0);
    expect(store.entity.talisman ?? null).toBeNull();
  });

  it('sets Might realm + non-negative score, and clears Might', () => {
    store.setMightRealm('infernal');
    expect(store.entity.might).toEqual({ realm: 'infernal', score: 0 });
    store.setMightScore(-4);
    expect(store.entity.might).toEqual({ realm: 'infernal', score: 0 });
    store.setMightScore(5);
    expect(store.entity.might).toEqual({ realm: 'infernal', score: 5 });
    store.clearMight();
    expect(store.entity.might ?? null).toBeNull();
  });

  it('adds, edits (name + non-negative level) and removes supernatural powers', () => {
    store.addPower();
    store.setPowerName(0, 'Curse');
    store.setPowerLevel(0, -5);
    expect(store.entity.powers).toEqual([{ name: 'Curse', level: 0 }]);
    store.setPowerLevel(0, 20);
    store.addPower();
    store.removePowerAt(1);
    expect(store.entity.powers).toEqual([{ name: 'Curse', level: 20 }]);
  });

  // Penetration is bought with levels out of the SAME budget as the level
  // (ArMDE:4019), so it has to be editable or the budget bar charges for
  // something the player cannot enter. Clamped to u16 like the level itself.
  it('edits a power Penetration and clamps it to u16', () => {
    store.addPower();
    store.setPowerName(0, 'Wolf Shape');
    store.setPowerLevel(0, 20);
    store.setPowerPenetration(0, 20);
    expect(store.entity.powers).toEqual([{ name: 'Wolf Shape', level: 20, penetration: 20 }]);
    store.setPowerPenetration(0, -5);
    expect(store.entity.powers?.[0].penetration).toBe(0);
    store.setPowerPenetration(0, 70000);
    expect(store.entity.powers?.[0].penetration).toBe(65535);
  });

  // Focus Power's pool is a SECOND currency (ArMDE:3899): 2 points per level of
  // effect, 1 per point of Penetration, out of 25 per copy of the Virtue. Its
  // level is a ceiling ("the maximum level of effect"), so it lives in its own
  // list rather than among the level-budget powers.
  it('adds, edits and removes focus powers, clamping both numbers to u16', () => {
    store.addFocusPower();
    store.setFocusPowerName(0, 'Wolves');
    store.setFocusPowerMaxLevel(0, 10);
    store.setFocusPowerPenetration(0, 5);
    expect(store.entity.focus_powers).toEqual([{ name: 'Wolves', max_level: 10, penetration: 5 }]);

    store.setFocusPowerMaxLevel(0, -5);
    expect(store.entity.focus_powers?.[0].max_level).toBe(0);
    store.setFocusPowerPenetration(0, 70000);
    expect(store.entity.focus_powers?.[0].penetration).toBe(65535);

    // The ordinary power list is a different list and is untouched.
    expect(store.entity.powers ?? []).toEqual([]);

    store.addFocusPower();
    store.removeFocusPowerAt(1);
    expect(store.entity.focus_powers).toHaveLength(1);
  });

  it('adds a longevity ritual with nothing entered yet', () => {
    store.addLongevityRitual('self_made');
    expect(store.entity.longevity_ritual).toEqual({ source: 'self_made', bonus: null, focus: '' });
    store.removeLongevityRitual();
    expect(store.entity.longevity_ritual).toBeNull();
  });

  it('enters a bonus and a focus for a self-made ritual', () => {
    store.addLongevityRitual('self_made');
    store.setLongevityBonus(7);
    store.setLongevityFocus('A draught of quicksilver');
    expect(store.entity.longevity_ritual).toEqual({
      source: 'self_made',
      bonus: 7,
      focus: 'A draught of quicksilver',
    });
  });

  it('clearing the bonus returns the ritual to not-entered', () => {
    store.addLongevityRitual('self_made');
    store.setLongevityBonus(7);
    // Emptying the input passes null, NOT the 0 that `Number('')` yields — a
    // deliberate 0 is a claim ("the ritual grants nothing"), which is not what
    // clearing the field means. Only null restores "not entered".
    store.setLongevityBonus(null);
    expect(store.entity.longevity_ritual?.bonus).toBeNull();
    // A deliberate 0 is still storable, and is a different state from null.
    store.setLongevityBonus(0);
    expect(store.entity.longevity_ritual?.bonus).toBe(0);
    // An unparseable value (mid-typing "-") is "not entered", never a stored 0.
    store.setLongevityBonus(Number.NaN);
    expect(store.entity.longevity_ritual?.bonus).toBeNull();
  });

  it('keeps the entered bonus when switching source', () => {
    store.addLongevityRitual('self_made');
    store.setLongevityBonus(4);
    store.setLongevityFocus('An amulet of hawthorn');
    store.setLongevitySource('external');
    expect(store.entity.longevity_ritual).toEqual({
      source: 'external',
      bonus: 4,
      focus: 'An amulet of hawthorn',
    });
    store.setLongevitySource('self_made');
    expect(store.entity.longevity_ritual).toEqual({
      source: 'self_made',
      bonus: 4,
      focus: 'An amulet of hawthorn',
    });
  });

  it('ignores longevity edits when there is no ritual', () => {
    store.setLongevitySource('external');
    store.setLongevityBonus(3);
    store.setLongevityFocus('nothing');
    expect(store.entity.longevity_ritual ?? null).toBeNull();
  });
});

// Every number typed into a panel input ends up in a fixed-width Rust integer
// field. An out-of-range value makes serde reject the whole payload at the Tauri
// boundary, so `validate` / `effective_scores` / `derived_totals` all fail at once
// and every read-out freezes on stale numbers while looking current. The store
// clamps instead, so the engine always receives a representable value.
describe('integer clamps at the Tauri boundary', () => {
  it('clamps the familiar Size and Characteristics to i8', () => {
    store.addFamiliar();
    store.setFamiliarSize(200);
    expect(store.entity.familiar?.size).toBe(127);
    store.setFamiliarSize(-200);
    expect(store.entity.familiar?.size).toBe(-128);
    store.setFamiliarCharacteristic('int', 200);
    expect(store.entity.familiar?.characteristics?.int).toBe(127);
    store.setFamiliarCharacteristic('qik', -200);
    expect(store.entity.familiar?.characteristics?.qik).toBe(-128);
  });

  it('clamps the u8-backed familiar Might Score', () => {
    store.addFamiliar();
    store.setFamiliarMightRealm('magic');
    store.setFamiliarMightScore(900);
    expect(store.entity.familiar?.might?.score).toBe(255);
  });

  // A cord is bounded by the RULE, not by the serde width: "rated from 0 to +5 …
  // a score of +5 (the maximum)" (ArMDE:10836). Storing a 6 gave the
  // engine's three cord consumers a value they read inconsistently (one clamps at
  // 5, two do not), so one entered number produced three contradictory read-outs.
  it('clamps a familiar cord to the rules maximum of +5, not to u8', () => {
    store.addFamiliar();
    store.setFamiliarCord('gold', 5);
    expect(store.entity.familiar?.cord_gold).toBe(5);
    store.setFamiliarCord('silver', 6);
    expect(store.entity.familiar?.cord_silver).toBe(5);
    store.setFamiliarCord('bronze', 900);
    expect(store.entity.familiar?.cord_bronze).toBe(5);
    store.setFamiliarCord('gold', -1);
    expect(store.entity.familiar?.cord_gold).toBe(0);
  });

  it('clamps u16-backed device and character power levels', () => {
    store.addDevice();
    store.setDeviceLevel(0, 70000);
    expect(store.entity.devices?.[0].level).toBe(65535);
    store.setDeviceLevel(0, -5);
    expect(store.entity.devices?.[0].level).toBe(0);
    store.addPower();
    store.setPowerLevel(0, 70000);
    expect(store.entity.powers?.[0].level).toBe(65535);
    store.setPowerLevel(0, -5);
    expect(store.entity.powers?.[0].level).toBe(0);
  });

  it("clamps the u8-backed base Might Score and a Characteristic's aging points", () => {
    store.setMightRealm('magic');
    store.setMightScore(900);
    expect(store.entity.might?.score).toBe(255);
    store.setAgingPoints('str', 900);
    expect(store.entity.aging_points?.str).toBe(255);
  });

  // Slice 1 handoff. This block's own premise — a value the serde width cannot
  // represent — is not the aura's real problem: clamping to i32 is clamping to
  // nothing, because no aura a player can plausibly type falls outside it. So a
  // typed 999 was stored as 999 and every derived read-out was computed from it,
  // producing a Casting Total ~989 too high until the next save+reload silently
  // replaced it with the engine's clamp. Two different answers for one document
  // is the product-integrity failure CLAUDE.md rates highest, and it outranks the
  // typing-ergonomics nicety the previous behaviour preserved.
  //
  // The bound is the RULE's, read from the engine payload, exactly as the two
  // aura INPUTS already read it (`MagicPossessions`, `DerivedAuraField`) — the
  // same reason a familiar cord clamps at +5 rather than at u8 above.
  it('clamps the aura to the engine-surfaced rules range, not to i32', () => {
    store.ruleset!.ruleset.aura_modifier_min = -50;
    store.ruleset!.ruleset.aura_modifier_max = 10;

    store.setAura(999);
    expect(store.entity.aura).toBe(10);
    store.setAura(-999);
    expect(store.entity.aura).toBe(-50);
    // A legal value is untouched, at both ends of the range and inside it.
    for (const legal of [-50, -7, 0, 3, 10]) {
      store.setAura(legal);
      expect(store.entity.aura).toBe(legal);
    }
  });

  it('reads the aura bound from the payload rather than a hardcoded pair', () => {
    store.ruleset!.ruleset.aura_modifier_min = -7;
    store.ruleset!.ruleset.aura_modifier_max = 4;

    store.setAura(999);
    expect(store.entity.aura).toBe(4);
    store.setAura(-999);
    expect(store.entity.aura).toBe(-7);
  });

  // The old assertion, kept for the path where it is still the right one: a
  // ruleset payload predating `aura_modifier_min/max` constrains nothing, so the
  // store falls back to the serde width — the same defensive "no constraint"
  // sentinel both aura inputs use. The clamp must still keep the value
  // REPRESENTABLE there, or serde rejects the whole payload.
  it('falls back to the i32 range for a ruleset payload carrying no aura bound', () => {
    delete store.ruleset!.ruleset.aura_modifier_min;
    delete store.ruleset!.ruleset.aura_modifier_max;

    store.setAura(3e9);
    expect(store.entity.aura).toBe(2147483647);
    store.setAura(-3e9);
    expect(store.entity.aura).toBe(-2147483648);
  });

  it('clamps the u32-backed Warping Points and XP pool', () => {
    store.setWarpingPoints(5e9);
    expect(store.entity.warping_points).toBe(4294967295);
    store.setXpPool(5e9);
    expect(store.entity.xp_pool).toBe(4294967295);
  });

  // A user mid-typing a lone "-" in a signed field sends `Number('-')` — NaN.
  // Without the non-finite guard the clamp propagates NaN, serde rejects the
  // payload and every read-out freezes on stale numbers that still look current.
  it('treats a non-finite value as 0 in signed and unsigned fields alike', () => {
    store.addFamiliar();
    store.setFamiliarSize(Number('-'));
    expect(store.entity.familiar?.size).toBe(0);
    store.setFamiliarCord('gold', Number('-'));
    expect(store.entity.familiar?.cord_gold).toBe(0);
    store.addTalisman();
    store.addTalismanEffect();
    store.setTalismanEffectLevel(0, Number('-'));
    expect(store.entity.talisman?.effects?.[0].level).toBe(0);
    store.addTalismanAttunement();
    store.setTalismanAttunementBonus(0, Number('-'));
    expect(store.entity.talisman?.attunements?.[0].bonus).toBe(0);
  });

  it('clamps u16-backed power and instilled-effect levels', () => {
    store.addFamiliar();
    store.addFamiliarPower();
    store.setFamiliarPowerLevel(0, 99999);
    expect(store.entity.familiar?.powers?.[0].level).toBe(65535);
    store.addTalisman();
    store.addTalismanEffect();
    store.setTalismanEffectLevel(0, 99999);
    expect(store.entity.talisman?.effects?.[0].level).toBe(65535);
  });

  it('clamps the i8-backed longevity bonus and talisman attunement bonus', () => {
    store.addLongevityRitual('self_made');
    store.setLongevityBonus(200);
    expect(store.entity.longevity_ritual?.bonus).toBe(127);
    store.setLongevityBonus(-200);
    expect(store.entity.longevity_ritual?.bonus).toBe(-128);
    store.addTalisman();
    store.addTalismanAttunement();
    store.setTalismanAttunementBonus(0, 200);
    expect(store.entity.talisman?.attunements?.[0].bonus).toBe(127);
    store.setTalismanAttunementBonus(0, -200);
    expect(store.entity.talisman?.attunements?.[0].bonus).toBe(-128);
  });

  it('clamps a bought Characteristic to i8', () => {
    store.setCharacteristic('str', 200);
    expect(store.entity.characteristics?.str).toBe(127);
    store.setCharacteristic('sta', -200);
    expect(store.entity.characteristics?.sta).toBe(-128);
  });

  // A General spell's level is the entity's tightest numeric field (u8), and one
  // stray digit on a 25 or 50 lands past it.
  it('clamps the u8-backed General spell level, keeping the minimum of 1', () => {
    store.addSpell('spell.aegis_of_the_hearth', 5);
    store.setSpellLevelAt(0, 256);
    expect(store.entity.spells?.[0].level).toBe(255);
    store.setSpellLevelAt(0, 0);
    expect(store.entity.spells?.[0].level).toBe(1);
    store.setSpellLevelAt(0, Number('-'));
    expect(store.entity.spells?.[0].level).toBe(1);
  });

  it('clamps the u32-backed age, apparent age and spell-levels override', () => {
    store.setAge(5e9);
    expect(store.entity.age).toBe(4294967295);
    store.setApparentAge(5e9);
    expect(store.entity.apparent_age).toBe(4294967295);
    store.setSpellLevelsOverride(5e9);
    expect(store.entity.spell_levels_override).toBe(4294967295);
    // The non-positive value still clears the field instead of clamping to 0.
    store.setAge(0);
    expect(store.entity.age).toBeNull();
    store.setApparentAge(-3);
    expect(store.entity.apparent_age).toBeNull();
    store.setSpellLevelsOverride(-3);
    expect(store.entity.spell_levels_override).toBeNull();
  });

  it('clamps the i32-backed birth year and aging-log year at both ends', () => {
    store.setBirthYear(3e9);
    expect(store.entity.birth_year).toBe(2147483647);
    store.setBirthYear(-3e9);
    expect(store.entity.birth_year).toBe(-2147483648);
    // An empty field still clears it rather than clamping to 0.
    store.setBirthYear(null);
    expect(store.entity.birth_year).toBeNull();
    store.addAgingLogEntry();
    store.setAgingLogEntryYear(0, 3e9);
    expect(store.entity.aging_log?.[0].year).toBe(2147483647);
    store.setAgingLogEntryYear(0, -3e9);
    expect(store.entity.aging_log?.[0].year).toBe(-2147483648);
    // A non-number (a lone minus sign mid-typing) is not a year; it clears.
    store.setAgingLogEntryYear(0, Number('-'));
    expect(store.entity.aging_log?.[0].year).toBeNull();
  });
});

// A rejected validate must not latch the error banner on for the rest of the
// session: once the user corrects the offending value and a pass succeeds, the
// banner has to go. Only a *current* pass may clear it, so a stale response that
// lands after a newer edit clears nothing.
describe('revalidate error latching', () => {
  afterEach(() => {
    vi.mocked(ipc.validateEntity).mockResolvedValue({ issues: [] });
    vi.mocked(ipc.saveEntity).mockReset();
    store.error = null;
  });

  it('clears a latched error once a validate succeeds', async () => {
    vi.mocked(ipc.validateEntity).mockRejectedValueOnce({ kind: 'invalid_entity' });

    await store.revalidate();
    expect(store.error).toEqual({ kind: 'invalid_entity' });

    await store.revalidate();
    expect(store.error).toBeNull();
  });

  it('does not let a stale response clear a current error', async () => {
    // A slow first call resolves only after a second (rejecting) call has already
    // bumped the sequence — its success must not wipe the newer failure.
    let finishStale: (result: { issues: [] }) => void = () => {};
    vi.mocked(ipc.validateEntity).mockReturnValueOnce(
      new Promise((resolve) => {
        finishStale = resolve;
      }) as ReturnType<typeof ipc.validateEntity>,
    );
    const stale = store.revalidate();

    vi.mocked(ipc.validateEntity).mockRejectedValueOnce({ kind: 'invalid_entity' });
    await store.revalidate();
    expect(store.error).toEqual({ kind: 'invalid_entity' });

    finishStale({ issues: [] });
    await stale;
    expect(store.error).toEqual({ kind: 'invalid_entity' });
  });

  // The mirror case: a direct `revalidate()` does not cancel a pending debounced
  // one, so a rejection from the pass the user already corrected can land after
  // the corrected pass succeeded. It must not raise a banner for a payload that
  // no longer exists.
  it('does not let a stale rejection raise an error after a newer pass succeeded', async () => {
    let failStale: (reason: unknown) => void = () => {};
    vi.mocked(ipc.validateEntity).mockReturnValueOnce(
      new Promise((_resolve, reject) => {
        failStale = reject;
      }) as ReturnType<typeof ipc.validateEntity>,
    );
    const stale = store.revalidate();

    await store.revalidate();
    expect(store.error).toBeNull();

    failStale({ kind: 'invalid_entity' });
    await stale;
    expect(store.error).toBeNull();
  });

  // `error` is one shared banner channel — file operations publish to it too. A
  // succeeding validate may only retire an error the validation path itself
  // raised: a failed save leaves the document unsaved, so its banner must stay up
  // until the user resolves it, even though every later validate succeeds.
  it('does not let a succeeding validate clear a save failure', async () => {
    // Dirty this document explicitly. The assertion at the end — a failed save leaves
    // the document unsaved — is only meaningful against edits this test made; it used
    // to ride on whatever the blocks above happened to leave behind in the shared
    // singleton, which made it hostage to their ordering (C8 changed that ordering and
    // the test went green-for-nothing until this line was added).
    store.entity.name = 'Unsaved by a failed save';
    expect(store.dirty).toBe(true);

    vi.mocked(ipc.saveEntity).mockRejectedValueOnce({ kind: 'io' });
    await store.saveAs();
    expect(store.error).toEqual({ kind: 'io' });

    await store.revalidate();

    expect(store.error).toEqual({ kind: 'io' });
    expect(store.dirty).toBe(true);
  });
});

describe('aged / warped state + identity', () => {
  it('sets per-Characteristic aging points and prunes zeros', () => {
    store.setAgingPoints('str', 7);
    store.setAgingPoints('qik', -3); // clamps to 0 → removed
    expect(store.entity.aging_points).toEqual({ str: 7 });
    store.setAgingPoints('str', 0); // back to 0 → removed
    expect(store.entity.aging_points).toEqual({});
  });

  it('sets non-negative stored Warping Points', () => {
    store.setWarpingPoints(15);
    expect(store.entity.warping_points).toBe(15);
    store.setWarpingPoints(-4);
    expect(store.entity.warping_points).toBe(0);
  });

  it('sets and clears an owed Warping fill by choice_key', () => {
    store.setWarpingChoice('warping.minor_flaw.0', { ref: 'flaw.clumsy' });
    expect(store.entity.warping_choices).toEqual({
      'warping.minor_flaw.0': { ref: 'flaw.clumsy' },
    });
    // A second slot coexists without clobbering the first.
    store.setWarpingChoice('warping.supernatural_virtue.0', { ref: 'virtue.second_sight' });
    expect(store.entity.warping_choices?.['warping.minor_flaw.0']).toEqual({ ref: 'flaw.clumsy' });
    // Passing null clears just that slot.
    store.setWarpingChoice('warping.minor_flaw.0', null);
    expect(store.entity.warping_choices).toEqual({
      'warping.supernatural_virtue.0': { ref: 'virtue.second_sight' },
    });
  });

  it('keeps the chosen parameters of a parameterized owed Warping fill', () => {
    // A fill may be a parameterized item ("Master of (Form) Creatures"); the
    // chosen parameter travels with the pick, and re-picking the same slot
    // replaces the whole Selection (so a stale param cannot linger).
    store.setWarpingChoice('warping.supernatural_virtue.0', {
      ref: 'virtue.master_of_form_creatures',
      params: { form: 'art.ignem' },
    });
    expect(store.entity.warping_choices?.['warping.supernatural_virtue.0']).toEqual({
      ref: 'virtue.master_of_form_creatures',
      params: { form: 'art.ignem' },
    });
    store.setWarpingChoice('warping.supernatural_virtue.0', { ref: 'virtue.second_sight' });
    expect(store.entity.warping_choices?.['warping.supernatural_virtue.0']).toEqual({
      ref: 'virtue.second_sight',
    });
  });

  it('adds, edits and removes Twilight Scars by index', () => {
    store.addTwilightScar();
    store.setTwilightScarDescription(0, 'Silver streak in the hair');
    expect(store.entity.twilight_scars).toEqual([{ description: 'Silver streak in the hair' }]);
    store.addTwilightScar();
    store.removeTwilightScarAt(0);
    expect(store.entity.twilight_scars).toEqual([{ description: '' }]);
  });

  it('sets and clears the apparent age', () => {
    store.setApparentAge(45);
    expect(store.entity.apparent_age).toBe(45);
    store.setApparentAge(0);
    expect(store.entity.apparent_age).toBeNull();
    store.setApparentAge(50);
    store.setApparentAge(null);
    expect(store.entity.apparent_age).toBeNull();
  });

  it('sets the free-text warping and decrepitude effect fields', () => {
    // The warping-effect field is a multi-line textarea, so it must round-trip
    // embedded newlines verbatim.
    const multiLineWarping =
      'A faint aura of ozone clings to him.\nHis eyes glow faintly at night.';
    store.setWarpingEffect(multiLineWarping);
    store.setDecrepitudeEffect('Stooped and hard of hearing');
    expect(store.entity.warping_effect).toBe(multiLineWarping);
    expect(store.entity.decrepitude_effect).toBe('Stooped and hard of hearing');
  });

  it('adds, edits and removes aging-log entries by index', () => {
    // A fresh row names no year: `AgingLogEntry.year` is optional, so a blank
    // entry claims nothing rather than claiming the year 0.
    store.addAgingLogEntry();
    expect(store.entity.aging_log).toEqual([{ year: null, effect: '' }]);
    store.setAgingLogEntryYear(0, 1215);
    store.setAgingLogEntryEffect(0, 'Lost a point of Stamina');
    expect(store.entity.aging_log).toEqual([{ year: 1215, effect: 'Lost a point of Stamina' }]);
    // Clearing the field un-dates the entry instead of dating it to year 0.
    store.setAgingLogEntryYear(0, null);
    expect(store.entity.aging_log).toEqual([{ year: null, effect: 'Lost a point of Stamina' }]);
    store.addAgingLogEntry();
    store.removeAgingLogEntryAt(0);
    expect(store.entity.aging_log).toEqual([{ year: null, effect: '' }]);
  });

  it('keeps a resolved entry’s recorded roll when only its free text is edited', () => {
    store.entity.aging_log = [
      {
        year: 1220,
        age: 40,
        effect: '',
        die: 11,
        total: 15,
        points: { sta: 1 },
        apparent_age_increased: true,
      },
    ];
    store.setAgingLogEntryEffect(0, 'Weathered it');
    expect(store.entity.aging_log?.[0]).toEqual({
      year: 1220,
      age: 40,
      effect: 'Weathered it',
      die: 11,
      total: 15,
      points: { sta: 1 },
      apparent_age_increased: true,
    });
  });

  it('writes the chosen Living Conditions sorted, and prunes the key when the last goes', () => {
    // Canonical serialization: the ids go out sorted whatever order they were
    // ticked in, so the save's diff is zero-noise.
    store.setLivingCondition('living_condition.work_in_a_mine', true);
    store.setLivingCondition('living_condition.leper', true);
    expect(store.entity.living_conditions).toEqual([
      'living_condition.leper',
      'living_condition.work_in_a_mine',
    ]);
    // Ticking the same row twice is idempotent, not a duplicate.
    store.setLivingCondition('living_condition.leper', true);
    expect(store.entity.living_conditions).toEqual([
      'living_condition.leper',
      'living_condition.work_in_a_mine',
    ]);
    store.setLivingCondition('living_condition.leper', false);
    expect(store.entity.living_conditions).toEqual(['living_condition.work_in_a_mine']);
    // The last one off leaves no key: an empty set is the engine's default (the
    // table's own "Average peasant 0"), so a sparse save is the canonical one.
    store.setLivingCondition('living_condition.work_in_a_mine', false);
    expect(store.entity.living_conditions).toBeUndefined();
  });

  it('sets free-text identity fields and birth year', () => {
    store.setIdentity('name', 'Marcus');
    store.setIdentity('description', 'Knight of the Teutonic Order');
    store.setIdentity('concept', 'A grim knight turned magus.');
    store.setIdentity('sigil', 'the smell of ozone');
    store.setBirthYear(1194);
    expect(store.entity.name).toBe('Marcus');
    expect(store.entity.description).toBe('Knight of the Teutonic Order');
    expect(store.entity.concept).toBe('A grim knight turned magus.');
    expect(store.entity.sigil).toBe('the smell of ozone');
    expect(store.entity.birth_year).toBe(1194);
    store.setBirthYear(null);
    expect(store.entity.birth_year).toBeNull();
  });
});

// --- removeSelectionAt() ----------------------------------------------------

describe('removeSelectionAt', () => {
  it('removes only the row at the given index, keeping order', () => {
    installRuleset([
      item({ id: 'v.a', max_per_target: 5 }),
      item({ id: 'v.b', max_per_target: 5 }),
      item({ id: 'v.c', max_per_target: 5 }),
    ]);
    store.addSelection('v.a');
    store.addSelection('v.b');
    store.addSelection('v.c');
    store.removeSelectionAt(1);
    expect(store.entity.selections!.map((s) => s.ref)).toEqual(['v.a', 'v.c']);
  });

  it('is a no-op for an out-of-range index', () => {
    installRuleset([item({ id: 'v.a' })]);
    store.addSelection('v.a');
    store.removeSelectionAt(5);
    expect(store.entity.selections!.map((s) => s.ref)).toEqual(['v.a']);
  });
});

// --- setParamAt() -----------------------------------------------------------

describe('setParamAt', () => {
  it('sets a param on the targeted row only', () => {
    // D10: max_total must explicitly match max_per_target when there is no
    // parameter to separate the two axes (same fix as great_characteristic).
    installRuleset([item({ id: 'v.great', max_per_target: 5, max_total: 5 })]);
    store.addSelection('v.great');
    store.addSelection('v.great');
    store.setParamAt(0, 'characteristic', 'characteristic.per');
    expect(store.entity.selections![0]).toEqual({
      ref: 'v.great',
      params: { characteristic: 'characteristic.per' },
    });
    expect(store.entity.selections![1]).toEqual({ ref: 'v.great' });
  });

  it('merges into existing params rather than replacing them', () => {
    installRuleset([item({ id: 'v.x' })]);
    store.addSelection('v.x');
    store.setParamAt(0, 'a', '1');
    store.setParamAt(0, 'b', '2');
    expect(store.entity.selections![0].params).toEqual({ a: '1', b: '2' });
  });

  it('trims the value, so a padded descriptor is the same target as an unpadded one', () => {
    // Row 10: parameter values establish selection identity — the engine's
    // duplicate key is the whole params map — so 'Wolf Shape ' would otherwise be
    // a second, distinct power. `ParameterPicker`'s own text handler already
    // trimmed; this is the store method every other write path goes through, and
    // the two must not disagree. Case is left alone: two powers deliberately
    // capitalised differently are the player's business.
    installRuleset([item({ id: 'flaw.lesser_power' })]);
    store.addSelection('flaw.lesser_power');
    store.setParamAt(0, 'power', '  Wolf Shape \t');
    expect(store.entity.selections![0].params).toEqual({ power: 'Wolf Shape' });

    store.setParamAt(0, 'power', 'wolf shape');
    expect(store.entity.selections![0].params).toEqual({ power: 'wolf shape' });
  });
});

// --- setMultiParamAt() (C5b, D9 part 3) --------------------------------------

// RED-CHECKPOINT: `store.setMultiParamAt` does not exist yet (phase 1 shipped
// no store/workflow method at all, since the picker's multi_ref branch was an
// empty stub with nothing to call it). Expected to fail — at runtime, calling
// an undefined method — until phase 2 adds `SelectionWorkflow.setMultiParamAt`
// and the `AppStore` passthrough.
describe('setMultiParamAt (C5b)', () => {
  beforeEach(() => {
    installRuleset([
      item({
        id: 'flaw.corrupted_spells_probe',
        kind: 'flaw',
        // D10: a repeatable item's own `max_total` — otherwise the default of
        // 1 blocks the second row the "targeted row only" test below adds.
        max_total: 2,
        parameters: [{ key: 'targets', type: 'multi_ref', domain: 'spell' }],
      }),
    ]);
    store.addSelection('flaw.corrupted_spells_probe');
  });

  it('writes a sorted, deduplicated array — the canonical form § 8 requires', () => {
    store.setMultiParamAt(0, 'targets', [
      'spell.pilum_of_fire',
      'spell.aegis_of_the_hearth',
      'spell.pilum_of_fire',
    ]);
    expect(store.entity.selections![0].params).toEqual({
      targets: ['spell.aegis_of_the_hearth', 'spell.pilum_of_fire'],
    });
  });

  it('writes an empty array (not an absent key) when handed no values', () => {
    store.setMultiParamAt(0, 'targets', ['spell.pilum_of_fire']);
    store.setMultiParamAt(0, 'targets', []);
    expect(store.entity.selections![0].params).toEqual({ targets: [] });
  });

  it('sets the param on the targeted row only', () => {
    store.addSelection('flaw.corrupted_spells_probe');
    store.setMultiParamAt(0, 'targets', ['spell.pilum_of_fire']);
    expect(store.entity.selections![1]).toEqual({ ref: 'flaw.corrupted_spells_probe' });
  });
});

// --- setAbilityBonusTarget() ------------------------------------------------

describe('setAbilityBonusTarget', () => {
  beforeEach(() => {
    installRuleset(
      [item({ id: 'virtue.puissant', max_per_target: 5 })],
      [ability('ability.awareness'), ability('ability.area_lore', 'area')],
    );
    store.addSelection('virtue.puissant');
  });

  it('stores the instance key for a parameterized ability', () => {
    store.setAbilityBonusTarget(0, 'ability.area_lore', 'Rhine');
    expect(store.entity.selections![0].params).toEqual({
      ability: 'ability.area_lore',
      area: 'Rhine',
    });
  });

  it('trims the instance value, which is free text the player types', () => {
    // The area/language discriminator is part of the ability-bonus target's
    // identity (`usedAbilityTargets` composes `ability + SEP + instance`), so a
    // padded 'Rhine ' would read as a second instance of the same Lore.
    store.setAbilityBonusTarget(0, 'ability.area_lore', ' Rhine ');
    expect(store.entity.selections![0].params).toEqual({
      ability: 'ability.area_lore',
      area: 'Rhine',
    });
  });

  it('stores just the ability for a plain ability (no instance key)', () => {
    store.setAbilityBonusTarget(0, 'ability.awareness');
    expect(store.entity.selections![0].params).toEqual({ ability: 'ability.awareness' });
  });

  it('drops the stale instance key when switching to a plain ability', () => {
    // First point it at a parameterized instance...
    store.setAbilityBonusTarget(0, 'ability.area_lore', 'Rhine');
    expect(store.entity.selections![0].params).toHaveProperty('area', 'Rhine');
    // ...then switch the target to a plain ability: the params object is
    // rebuilt, so the stale `area` key is gone, not merged.
    store.setAbilityBonusTarget(0, 'ability.awareness');
    expect(store.entity.selections![0].params).toEqual({ ability: 'ability.awareness' });
    expect(store.entity.selections![0].params).not.toHaveProperty('area');
  });

  it('omits the instance key when a parameterized ability has no parameter value', () => {
    store.setAbilityBonusTarget(0, 'ability.area_lore', null);
    expect(store.entity.selections![0].params).toEqual({ ability: 'ability.area_lore' });
  });
});

// --- setCharacteristic() ----------------------------------------------------

describe('setCharacteristic', () => {
  it('stores a non-zero score', () => {
    store.setCharacteristic('int', 3);
    expect(store.entity.characteristics).toEqual({ int: 3 });
  });

  it('stores a negative score', () => {
    store.setCharacteristic('per', -2);
    expect(store.entity.characteristics).toEqual({ per: -2 });
  });

  it('removes the entry when the score is set to 0', () => {
    store.setCharacteristic('int', 3);
    store.setCharacteristic('per', 1);
    store.setCharacteristic('int', 0);
    expect(store.entity.characteristics).toEqual({ per: 1 });
    expect(store.entity.characteristics).not.toHaveProperty('int');
  });
});

// --- setXpPool() ------------------------------------------------------------

describe('setXpPool', () => {
  it('stores a positive integer', () => {
    store.setXpPool(45);
    expect(store.entity.xp_pool).toBe(45);
  });

  it('floors a fractional value', () => {
    store.setXpPool(45.9);
    expect(store.entity.xp_pool).toBe(45);
  });

  it('clamps non-positive and non-finite values to 0', () => {
    store.setXpPool(-5);
    expect(store.entity.xp_pool).toBe(0);
    store.setXpPool(0);
    expect(store.entity.xp_pool).toBe(0);
    store.setXpPool(Number.NaN);
    expect(store.entity.xp_pool).toBe(0);
  });
});

// --- the Ability funding mode (M6b3b) ---------------------------------------

describe('ability funding mode', () => {
  /** A loadable character carrying a life-stage plan (a guided-mode save). */
  function entityWithPlan(): Entity {
    return {
      schema_version: SCHEMA_VERSION,
      ruleset: { id: 'test', version: '1' },
      entity_kind: 'character',
      type_id: 'companion',
      selections: [],
      characteristics: {} as Entity['characteristics'],
      characteristic_descriptions: {},
      ability_scores: [],
      xp_pool: 0,
      saga_year: 1220,
      art_scores: [],
      personality_traits: [],
      reputations: [],
      // Since schema 16 the mode is STORED, not inferred from the plan: a save the
      // engine's migration folded carries both.
      ability_funding: 'life_stages',
      life_stages: { native_language: 'German' },
    };
  }

  beforeEach(() => {
    vi.mocked(ipc.validateEntity).mockClear();
  });

  it('reports the typed pool for an entity carrying no plan', () => {
    expect(store.abilityFunding).toBe('pool');
  });

  it('reads the stored field, not the presence of a plan', () => {
    // The case that could not exist before schema 16, and the whole point of it: a
    // pool-funded character that still carries the plan it typed earlier. While the
    // mode was inferred from the plan, DELETING the plan was how "pool" got
    // recorded — so nothing could stop the destruction until the field was stored.
    store.entity.ability_funding = 'pool';
    store.entity.life_stages = { native_language: 'German', gauntlet_age: 25 };
    expect(store.abilityFunding).toBe('pool');

    // And the other direction: the field alone flips the mode, plan or no plan.
    store.entity.ability_funding = 'life_stages';
    expect(store.abilityFunding).toBe('life_stages');
  });

  it('preserves a typed xp_pool when switching to life stages', async () => {
    store.setXpPool(45);
    await store.setAbilityFunding('life_stages');
    // A hand-typed total is the player's work: the two funding sides now coexist on
    // the entity and the inactive one is merely inert, so nothing is zeroed.
    expect(store.entity.xp_pool).toBe(45);
    expect(store.entity.life_stages).toEqual({});
    expect(store.abilityFunding).toBe('life_stages');
  });

  it('records the mode on the entity, so the value Rust receives is never inferred', async () => {
    // Tauri commands take an entity through plain serde, NOT through
    // `load_entity_migrating`, so the load-time fold never runs on an IPC payload.
    // An omitted key would default to `Pool` and every life-stage pool would
    // silently vanish (`LifeStageRules::budget` returns `None`).
    await store.setAbilityFunding('life_stages');
    expect(store.entity.ability_funding).toBe('life_stages');
    await store.setAbilityFunding('pool');
    expect(store.entity.ability_funding).toBe('pool');
  });

  it('validates immediately rather than through the debounce', async () => {
    await store.setAbilityFunding('life_stages');
    // No timer advance: a discrete action revalidates on the spot.
    expect(vi.mocked(ipc.validateEntity)).toHaveBeenCalledTimes(1);
  });

  it('leaves bought ability rows alone in both directions', async () => {
    // An over-spend must surface as `not_enough_xp` — visible and fixable —
    // rather than being silently wiped by the switch.
    installRuleset([], [ability('ability.awareness')]);
    store.addAbility('ability.awareness');
    store.adjustAbilityAt(0, 3, 10);

    await store.setAbilityFunding('life_stages');
    expect(store.entity.ability_scores).toEqual([{ ability: 'ability.awareness', score: 3 }]);
    await store.setAbilityFunding('pool');
    expect(store.entity.ability_scores).toEqual([{ ability: 'ability.awareness', score: 3 }]);
  });

  it('preserves the life-stage plan when switching to pool', async () => {
    const plan = {
      native_language: 'German',
      childhood_package: 'childhood.traveling',
      gauntlet_age: 25,
      post_gauntlet_lab_seasons: 6,
      post_gauntlet_spell_levels: 120,
    };
    store.entity.ability_funding = 'life_stages';
    store.entity.life_stages = { ...plan };

    await store.setAbilityFunding('pool');

    // A deliberate departure from the sparse-save principle: the save now keeps data
    // the active mode ignores, because discarding it silently destroyed a native
    // language, a Gauntlet age, lab seasons, spell levels and a childhood package.
    expect(store.entity.life_stages).toEqual(plan);
    expect(store.abilityFunding).toBe('pool');
  });

  it('leaves both the typed pool and the plan intact across a pool → life stages → pool round trip', async () => {
    store.setXpPool(240);
    await store.setAbilityFunding('life_stages');
    store.setNativeLanguage('German');
    store.setGauntletAge(25);

    await store.setAbilityFunding('pool');

    expect(store.entity.xp_pool).toBe(240);
    expect(store.entity.life_stages).toEqual({ native_language: 'German', gauntlet_age: 25 });
    expect(store.abilityFunding).toBe('pool');

    // And back again: the plan is picked up where it was left, not restarted empty.
    await store.setAbilityFunding('life_stages');
    expect(store.entity.xp_pool).toBe(240);
    expect(store.entity.life_stages).toEqual({ native_language: 'German', gauntlet_age: 25 });
  });

  it('still clears the childhood draft when leaving life stages', async () => {
    await store.setAbilityFunding('life_stages');
    store.setChildhoodDraftPackage('childhood.traveling');
    store.childhoodRejections = [
      { severity: 'error', code: 'childhood_slot_unfilled', phase: 'abilities', args: {} },
    ];

    await store.setAbilityFunding('pool');

    // The plan and the pool stop being destroyed; the DRAFTS do not. One blanket
    // rule: an un-submitted draft never outlives a change to how the document is
    // built.
    expect(store.childhoodDraft).toEqual({ packageId: null, slots: {} });
    expect(store.childhoodRejections).toEqual([]);
  });

  it('reports life stages for a loaded save that already carries a plan', async () => {
    vi.mocked(ipc.loadEntity).mockResolvedValue({
      path: '/tmp/marcus.armc',
      entity: entityWithPlan(),
      migrated_aging_characteristics: [],
    });
    const opening = store.open();
    if (store.discardPromptOpen) store.resolveDiscardPrompt(true);
    await opening;

    expect(store.abilityFunding).toBe('life_stages');
    // And the mode needs no reconciliation: asking for it again is a no-op.
    vi.mocked(ipc.validateEntity).mockClear();
    await store.setAbilityFunding('life_stages');
    expect(vi.mocked(ipc.validateEntity)).not.toHaveBeenCalled();
  });

  it('is a no-op when the requested funding is already active', async () => {
    await store.setAbilityFunding('pool');
    expect(vi.mocked(ipc.validateEntity)).not.toHaveBeenCalled();
  });

  it('makes the document dirty', async () => {
    await store.createCharacter('companion');
    expect(store.dirty).toBe(false);
    await store.setAbilityFunding('life_stages');
    expect(store.dirty).toBe(true);
  });

  it('is carried by a freshly instantiated character', async () => {
    await store.createCharacter('companion');
    // The key must be PRESENT, not merely defaulted on the Rust side: a Tauri
    // command deserializes the payload with plain serde, so an absent key silently
    // reads as `Pool`.
    expect('ability_funding' in store.entity).toBe(true);
    expect(store.entity.ability_funding).toBe('pool');

    // The guided wizard defaults to the OTHER mode (owner request, M6/D2): a
    // guided character is built through its life history from the start, while
    // direct entry stays on the flat pool it always used.
    await store.startWizard('companion');
    expect('ability_funding' in store.entity).toBe(true);
    expect(store.entity.ability_funding).toBe('life_stages');
  });

  it('still carries life_stages after a save/load round trip through the store', async () => {
    await store.createCharacter('companion');
    await store.setAbilityFunding('life_stages');
    store.setNativeLanguage('German');
    vi.advanceTimersByTime(200);

    vi.mocked(ipc.saveEntity).mockResolvedValue('/tmp/marcus.armc');
    await store.saveAs();
    const written = vi.mocked(ipc.saveEntity).mock.calls[0][0];
    expect(written.ability_funding).toBe('life_stages');

    // Reload exactly the bytes that were handed to Rust — the mode must survive,
    // or every life-stage pool evaporates on the next validate.
    vi.mocked(ipc.loadEntity).mockResolvedValue({
      path: '/tmp/marcus.armc',
      entity: written,
      migrated_aging_characteristics: [],
    });
    const opening = store.open();
    if (store.discardPromptOpen) store.resolveDiscardPrompt(true);
    await opening;

    expect(store.entity.ability_funding).toBe('life_stages');
    expect(store.abilityFunding).toBe('life_stages');
    expect(store.entity.life_stages).toEqual({ native_language: 'German' });
    vi.mocked(ipc.saveEntity).mockReset();
    vi.mocked(ipc.loadEntity).mockReset();
  });
});

describe('setNativeLanguage', () => {
  beforeEach(async () => {
    vi.mocked(ipc.validateEntity).mockClear();
    await store.setAbilityFunding('life_stages');
    vi.mocked(ipc.validateEntity).mockClear();
  });

  it('stores the trimmed language on the plan', () => {
    store.setNativeLanguage('  German  ');
    expect(store.entity.life_stages).toEqual({ native_language: 'German' });
  });

  it('validates through the debounce, like the other typed fields', () => {
    store.setNativeLanguage('German');
    expect(vi.mocked(ipc.validateEntity)).not.toHaveBeenCalled();
    vi.advanceTimersByTime(200);
    expect(vi.mocked(ipc.validateEntity)).toHaveBeenCalledTimes(1);
  });

  it('deletes the key for a blank or whitespace-only value', () => {
    store.setNativeLanguage('German');
    store.setNativeLanguage('  ');
    // The engine reads blank as unset, and a sparse save is the canonical one.
    expect(store.entity.life_stages).toEqual({});
    expect(store.entity.life_stages).not.toHaveProperty('native_language');
  });

  it('keeps the rest of the plan intact', () => {
    store.entity.life_stages = { childhood_package: 'childhood.traveling' };
    store.setNativeLanguage('German');
    expect(store.entity.life_stages).toEqual({
      childhood_package: 'childhood.traveling',
      native_language: 'German',
    });
  });

  it('is a no-op when no plan exists, so no surface can conjure one', async () => {
    // The guard is the plan's PRESENCE, not the funding mode: since schema 16 a
    // switch to pool funding preserves the plan, so this case has to be built by
    // removing the plan rather than by switching mode. (M6/D2: the wizard's own
    // default now allocates a plan too, alongside `setAbilityFunding` — see
    // `#instantiateCharacter` — so this remains the only way to reach "no plan".)
    await store.setAbilityFunding('pool');
    delete store.entity.life_stages;
    vi.mocked(ipc.validateEntity).mockClear();

    store.setNativeLanguage('German');

    expect('life_stages' in store.entity).toBe(false);
    vi.advanceTimersByTime(200);
    expect(vi.mocked(ipc.validateEntity)).not.toHaveBeenCalled();
  });
});

// --- the post-Gauntlet choices (M6b5) ---------------------------------------

describe('the post-Gauntlet plan fields', () => {
  beforeEach(async () => {
    await store.setAbilityFunding('pool');
    await store.setAbilityFunding('life_stages');
    vi.mocked(ipc.validateEntity).mockClear();
  });

  describe('setGauntletAge', () => {
    it('stores the age the apprenticeship ended at', () => {
      store.setGauntletAge(25);
      expect(store.entity.life_stages).toEqual({ gauntlet_age: 25 });
    });

    it('deletes the key for a blank or zero age, so a magus at its Gauntlet saves nothing', () => {
      store.setNativeLanguage('German');
      store.setGauntletAge(25);
      store.setGauntletAge(null);
      // Absent means "standing at the Gauntlet" — exactly the pre-6b5 shape.
      expect(store.entity.life_stages).toEqual({ native_language: 'German' });

      store.setGauntletAge(25);
      store.setGauntletAge(0);
      expect(store.entity.life_stages).toEqual({ native_language: 'German' });
    });

    it('clamps to the u32 range the engine field is', () => {
      store.setGauntletAge(9999999999);
      expect(store.entity.life_stages?.gauntlet_age).toBe(4294967295);
      store.setGauntletAge(25.7);
      expect(store.entity.life_stages?.gauntlet_age).toBe(25);
    });

    it('validates through the debounce, like the other typed fields', () => {
      store.setGauntletAge(25);
      expect(vi.mocked(ipc.validateEntity)).not.toHaveBeenCalled();
      vi.advanceTimersByTime(200);
      expect(vi.mocked(ipc.validateEntity)).toHaveBeenCalledTimes(1);
    });

    it('is a no-op when no plan exists, so no surface can conjure one', async () => {
      // Built by removing the plan, not by switching mode: a switch preserves it.
      await store.setAbilityFunding('pool');
      delete store.entity.life_stages;
      vi.mocked(ipc.validateEntity).mockClear();

      store.setGauntletAge(25);

      expect('life_stages' in store.entity).toBe(false);
      vi.advanceTimersByTime(200);
      expect(vi.mocked(ipc.validateEntity)).not.toHaveBeenCalled();
    });
  });

  describe('setPostGauntletLabSeasons', () => {
    it('stores the charged lab seasons', () => {
      store.setPostGauntletLabSeasons(6);
      expect(store.entity.life_stages).toEqual({ post_gauntlet_lab_seasons: 6 });
    });

    it('deletes the key for a blank or zero count', () => {
      store.setPostGauntletLabSeasons(6);
      store.setPostGauntletLabSeasons(0);
      expect(store.entity.life_stages).toEqual({});
      store.setPostGauntletLabSeasons(6);
      store.setPostGauntletLabSeasons(null);
      expect(store.entity.life_stages).toEqual({});
    });

    it('clamps to the u32 range the engine field is', () => {
      store.setPostGauntletLabSeasons(9999999999);
      expect(store.entity.life_stages?.post_gauntlet_lab_seasons).toBe(4294967295);
    });

    it('validates through the debounce', () => {
      store.setPostGauntletLabSeasons(6);
      expect(vi.mocked(ipc.validateEntity)).not.toHaveBeenCalled();
      vi.advanceTimersByTime(200);
      expect(vi.mocked(ipc.validateEntity)).toHaveBeenCalledTimes(1);
    });

    it('is a no-op when no plan exists', async () => {
      await store.setAbilityFunding('pool');
      delete store.entity.life_stages;
      vi.mocked(ipc.validateEntity).mockClear();

      store.setPostGauntletLabSeasons(6);

      expect('life_stages' in store.entity).toBe(false);
      vi.advanceTimersByTime(200);
      expect(vi.mocked(ipc.validateEntity)).not.toHaveBeenCalled();
    });
  });

  describe('setPostGauntletSpellLevels', () => {
    it('stores the levels taken as spells rather than experience', () => {
      store.setPostGauntletSpellLevels(120);
      expect(store.entity.life_stages).toEqual({ post_gauntlet_spell_levels: 120 });
    });

    it('deletes the key for a blank or zero split', () => {
      store.setPostGauntletSpellLevels(120);
      store.setPostGauntletSpellLevels(0);
      expect(store.entity.life_stages).toEqual({});
      store.setPostGauntletSpellLevels(120);
      store.setPostGauntletSpellLevels(null);
      expect(store.entity.life_stages).toEqual({});
    });

    it('clamps to the u32 range the engine field is', () => {
      store.setPostGauntletSpellLevels(9999999999);
      expect(store.entity.life_stages?.post_gauntlet_spell_levels).toBe(4294967295);
    });

    it('validates through the debounce', () => {
      store.setPostGauntletSpellLevels(120);
      expect(vi.mocked(ipc.validateEntity)).not.toHaveBeenCalled();
      vi.advanceTimersByTime(200);
      expect(vi.mocked(ipc.validateEntity)).toHaveBeenCalledTimes(1);
    });

    it('is a no-op when no plan exists', async () => {
      await store.setAbilityFunding('pool');
      delete store.entity.life_stages;
      vi.mocked(ipc.validateEntity).mockClear();

      store.setPostGauntletSpellLevels(120);

      expect('life_stages' in store.entity).toBe(false);
      vi.advanceTimersByTime(200);
      expect(vi.mocked(ipc.validateEntity)).not.toHaveBeenCalled();
    });
  });

  it('keeps all three when guided funding is left, plan and all', async () => {
    store.setGauntletAge(40);
    store.setPostGauntletLabSeasons(6);
    store.setPostGauntletSpellLevels(120);

    await store.setAbilityFunding('pool');

    // They used to go with the plan the switch deleted. Three typed numbers is
    // exactly the kind of work #29 was about losing: the plan is now inert, not gone.
    expect(store.entity.life_stages).toEqual({
      gauntlet_age: 40,
      post_gauntlet_lab_seasons: 6,
      post_gauntlet_spell_levels: 120,
    });
  });
});

// --- the Sample Childhood draft (M6b3b) -------------------------------------

/**
 * Install a ruleset carrying two Sample Childhood packages. They share the
 * `language` slot and differ elsewhere, which is what makes pruning on a package
 * switch observable.
 */
function installChildhoods(): void {
  const localized = installRuleset([]);
  localized.ruleset.childhoods = {
    'childhood.athletic': {
      id: 'childhood.athletic',
      entries: [
        { ability: 'ability.living_language', score: 5, slot: 'language', native: true },
        { ability: 'ability.athletics', score: 2 },
      ],
    },
    'childhood.traveling': {
      id: 'childhood.traveling',
      entries: [
        { ability: 'ability.living_language', score: 5, slot: 'language', native: true },
        { ability: 'ability.area_lore', score: 1, slot: 'area_a' },
        { ability: 'ability.area_lore', score: 1, slot: 'area_b' },
      ],
    },
  };
  store.ruleset = localized;
}

describe('the childhood package draft', () => {
  beforeEach(() => {
    installChildhoods();
    // Isolate from other tests mutating the shared singleton's draft.
    store.setChildhoodDraftPackage(null);
    vi.mocked(ipc.validateEntity).mockClear();
  });

  it('starts with nothing selected and no slot values', () => {
    expect(store.childhoodDraft).toEqual({ packageId: null, slots: {} });
  });

  it('records the selected package', () => {
    store.setChildhoodDraftPackage('childhood.traveling');
    expect(store.childhoodDraft.packageId).toBe('childhood.traveling');
  });

  it('stores a trimmed slot value under its slot key', () => {
    store.setChildhoodDraftPackage('childhood.traveling');
    store.setChildhoodDraftSlot('area_a', '  Rhine  ');
    expect(store.childhoodDraft.slots).toEqual({ area_a: 'Rhine' });
  });

  it('prunes the slot values the newly selected package does not declare', () => {
    store.setChildhoodDraftPackage('childhood.traveling');
    store.setChildhoodDraftSlot('language', 'German');
    store.setChildhoodDraftSlot('area_a', 'Rhine');
    store.setChildhoodDraftSlot('area_b', 'Provence');

    store.setChildhoodDraftPackage('childhood.athletic');

    // The shared `language` slot survives; the Area Lore slots the athletic
    // package never asks for are dropped (the #prunedHouseChoices precedent).
    expect(store.childhoodDraft).toEqual({
      packageId: 'childhood.athletic',
      slots: { language: 'German' },
    });
  });

  it('clears both the selection and every slot value for null', () => {
    store.setChildhoodDraftPackage('childhood.traveling');
    store.setChildhoodDraftSlot('area_a', 'Rhine');

    store.setChildhoodDraftPackage(null);

    expect(store.childhoodDraft).toEqual({ packageId: null, slots: {} });
  });

  it('deletes a slot key for a blank or whitespace-only value', () => {
    store.setChildhoodDraftPackage('childhood.traveling');
    store.setChildhoodDraftSlot('area_a', 'Rhine');
    store.setChildhoodDraftSlot('area_a', '   ');
    // An unanswered slot is absent, never present-but-empty.
    expect(store.childhoodDraft.slots).toEqual({});
    expect(store.childhoodDraft.slots).not.toHaveProperty('area_a');
  });

  it('never touches the entity, so drafting cannot dirty the document', async () => {
    await store.createCharacter('companion');
    expect(store.dirty).toBe(false);

    store.setChildhoodDraftPackage('childhood.traveling');
    store.setChildhoodDraftSlot('area_a', 'Rhine');

    // The draft is UI state: the entity records only the package actually taken.
    expect(store.dirty).toBe(false);
    expect(store.entity.life_stages).toBeUndefined();
  });

  it('never validates: nothing in a draft is checked until it is applied', () => {
    store.setChildhoodDraftPackage('childhood.traveling');
    store.setChildhoodDraftSlot('area_a', 'Rhine');
    vi.advanceTimersByTime(200);
    expect(vi.mocked(ipc.validateEntity)).not.toHaveBeenCalled();
  });

  it('is reset by createCharacter', async () => {
    store.setChildhoodDraftPackage('childhood.traveling');
    store.setChildhoodDraftSlot('area_a', 'Rhine');

    await store.createCharacter('companion');

    expect(store.childhoodDraft).toEqual({ packageId: null, slots: {} });
  });

  it('is reset by startWizard', async () => {
    store.setChildhoodDraftPackage('childhood.traveling');
    store.setChildhoodDraftSlot('area_a', 'Rhine');

    await store.startWizard('companion');

    expect(store.childhoodDraft).toEqual({ packageId: null, slots: {} });
  });

  it('is reset by newDocument, like the picker filters', async () => {
    store.setChildhoodDraftPackage('childhood.traveling');
    const discarding = store.newDocument();
    if (store.discardPromptOpen) store.resolveDiscardPrompt(true);
    await discarding;

    expect(store.childhoodDraft).toEqual({ packageId: null, slots: {} });
  });

  it('is reset by a load: a recorded package is history, not a draft', async () => {
    store.setChildhoodDraftPackage('childhood.traveling');
    store.setChildhoodDraftSlot('area_a', 'Rhine');
    vi.mocked(ipc.loadEntity).mockResolvedValue({
      path: '/tmp/marcus.armc',
      entity: { ...store.entity, life_stages: { childhood_package: 'childhood.traveling' } },
      migrated_aging_characteristics: [],
    });

    const opening = store.open();
    if (store.discardPromptOpen) store.resolveDiscardPrompt(true);
    await opening;

    expect(store.childhoodDraft).toEqual({ packageId: null, slots: {} });
    vi.mocked(ipc.loadEntity).mockReset();
  });

  it('is pruned when guided funding is left, together with its rejections', async () => {
    await store.setAbilityFunding('life_stages');
    store.setChildhoodDraftPackage('childhood.traveling');
    store.setChildhoodDraftSlot('area_a', 'Rhine');
    store.childhoodRejections = [
      { severity: 'error', code: 'childhood_slot_unfilled', phase: 'abilities', args: {} },
    ];

    await store.setAbilityFunding('pool');

    // The plan itself survives the switch now (schema 16), but the DRAFT does not:
    // an un-submitted draft never outlives a change to how the document is built,
    // and a survivor would show stale slot faults for a decision nobody has made.
    expect(store.childhoodDraft).toEqual({ packageId: null, slots: {} });
    expect(store.childhoodRejections).toEqual([]);
  });

  it('survives entering guided funding, so toggling the radio keeps typed slots', async () => {
    await store.setAbilityFunding('pool');
    store.setChildhoodDraftPackage('childhood.traveling');
    store.setChildhoodDraftSlot('area_a', 'Rhine');

    await store.setAbilityFunding('life_stages');

    // Deliberately asymmetric to the prune above: entering the mode adds a plan, so
    // there is nothing stale about the draft — and clearing it here would destroy
    // typed answers on a stray double toggle.
    expect(store.childhoodDraft).toEqual({
      packageId: 'childhood.traveling',
      slots: { area_a: 'Rhine' },
    });
  });
});

describe('applyChildhoodPackage', () => {
  /** A plain deep copy, for comparing an entity against its own earlier state. */
  function plain(entity: Entity): Entity {
    return JSON.parse(JSON.stringify(entity)) as Entity;
  }

  /** The entity the engine hands back on a successful application. */
  function appliedEntity(): Entity {
    return {
      ...plain(store.entity),
      life_stages: { native_language: 'German', childhood_package: 'childhood.traveling' },
      ability_scores: [
        { ability: 'ability.living_language', score: 5, parameter: { text: 'German' } },
        { ability: 'ability.area_lore', score: 1, parameter: { text: 'Rhine' } },
      ],
    };
  }

  /** The engine's "you left a slot unanswered" rejection. */
  function slotUnfilled(): ValidationIssue {
    return {
      severity: 'error',
      code: 'childhood_slot_unfilled',
      phase: 'abilities',
      args: { ability: 'ability.area_lore', key: 'area', slot: 'area_b' },
    };
  }

  beforeEach(() => {
    installChildhoods();
    store.setChildhoodDraftPackage(null);
    store.error = null;
    vi.mocked(ipc.applyChildhoodPackage).mockReset();
    vi.mocked(ipc.validateEntity).mockClear();
  });

  afterEach(() => {
    store.error = null;
  });

  it('sends the entity snapshot, the drafted package and its slot answers', async () => {
    vi.mocked(ipc.applyChildhoodPackage).mockResolvedValue({
      status: 'applied',
      entity: appliedEntity(),
    });
    store.setChildhoodDraftPackage('childhood.traveling');
    store.setChildhoodDraftSlot('language', 'German');
    store.setChildhoodDraftSlot('area_a', 'Rhine');
    const sent = plain(store.entity);

    await store.applyChildhoodPackage();

    expect(vi.mocked(ipc.applyChildhoodPackage)).toHaveBeenCalledWith(sent, 'childhood.traveling', {
      language: 'German',
      area_a: 'Rhine',
    });
  });

  it('replaces the entity with the applied one and revalidates once', async () => {
    const applied = appliedEntity();
    vi.mocked(ipc.applyChildhoodPackage).mockResolvedValue({ status: 'applied', entity: applied });
    store.setChildhoodDraftPackage('childhood.traveling');

    await store.applyChildhoodPackage();

    expect(plain(store.entity)).toEqual(applied);
    expect(vi.mocked(ipc.validateEntity)).toHaveBeenCalledTimes(1);
  });

  it('clears a previous rejection when applying again', async () => {
    vi.mocked(ipc.applyChildhoodPackage).mockResolvedValue({
      status: 'rejected',
      issues: [slotUnfilled()],
    });
    store.setChildhoodDraftPackage('childhood.traveling');
    await store.applyChildhoodPackage();
    expect(store.childhoodRejections).toHaveLength(1);

    vi.mocked(ipc.applyChildhoodPackage).mockResolvedValue({
      status: 'applied',
      entity: appliedEntity(),
    });
    await store.applyChildhoodPackage();

    expect(store.childhoodRejections).toEqual([]);
  });

  it('leaves the entity untouched and exposes the issues on a rejection', async () => {
    const issues = [slotUnfilled()];
    vi.mocked(ipc.applyChildhoodPackage).mockResolvedValue({ status: 'rejected', issues });
    store.result = { issues: [] };
    store.setChildhoodDraftPackage('childhood.traveling');
    const before = plain(store.entity);

    await store.applyChildhoodPackage();

    expect(plain(store.entity)).toEqual(before);
    expect(store.childhoodRejections).toEqual(issues);
    // A rejection describes the command input, not the entity's state, so it stays
    // out of the validation results and needs no engine round trip.
    expect(store.result?.issues).toEqual([]);
    expect(vi.mocked(ipc.validateEntity)).not.toHaveBeenCalled();
  });

  it('clears a stale rejection when the drafted slot answer changes', async () => {
    vi.mocked(ipc.applyChildhoodPackage).mockResolvedValue({
      status: 'rejected',
      issues: [slotUnfilled()],
    });
    store.setChildhoodDraftPackage('childhood.traveling');
    await store.applyChildhoodPackage();

    store.setChildhoodDraftSlot('area_b', 'Provence');

    // A rejection pointing at a field the user has just fixed is worse than none.
    expect(store.childhoodRejections).toEqual([]);
  });

  it('clears a stale rejection when a different package is drafted', async () => {
    vi.mocked(ipc.applyChildhoodPackage).mockResolvedValue({
      status: 'rejected',
      issues: [slotUnfilled()],
    });
    store.setChildhoodDraftPackage('childhood.traveling');
    await store.applyChildhoodPackage();

    store.setChildhoodDraftPackage('childhood.athletic');

    expect(store.childhoodRejections).toEqual([]);
  });

  it('does nothing at all without a drafted package', async () => {
    await store.applyChildhoodPackage();
    expect(vi.mocked(ipc.applyChildhoodPackage)).not.toHaveBeenCalled();
  });

  it('surfaces a thrown ipc failure on the shared error banner', async () => {
    vi.mocked(ipc.applyChildhoodPackage).mockRejectedValueOnce({ kind: 'invalid_entity' });
    store.setChildhoodDraftPackage('childhood.traveling');
    const before = plain(store.entity);

    await store.applyChildhoodPackage();

    expect(store.error).toEqual({ kind: 'invalid_entity' });
    expect(plain(store.entity)).toEqual(before);
  });
});

// --- Art actions ------------------------------------------------------------

describe('art actions', () => {
  it('upserts an Art score by id, clamping to [0, max]', () => {
    // First raise creates the entry; all 15 Arts are always present in the UI, so
    // there is no separate "add" step.
    store.adjustArt('art.ignem', 3, 20);
    expect(store.entity.art_scores).toEqual([{ art: 'art.ignem', score: 3 }]);
    store.adjustArt('art.ignem', 99, 20); // clamps at max
    expect(store.entity.art_scores?.[0].score).toBe(20);
  });

  it('drops an Art entry when its score returns to 0 (sparse save)', () => {
    store.adjustArt('art.creo', 2, 20);
    expect(store.entity.art_scores).toEqual([{ art: 'art.creo', score: 2 }]);
    store.adjustArt('art.creo', -5, 20); // clamps at 0 and removes the entry
    expect(store.entity.art_scores).toEqual([]);
  });

  it('tracks several Arts independently', () => {
    store.adjustArt('art.creo', 2, 20);
    store.adjustArt('art.ignem', 4, 20);
    expect(store.entity.art_scores).toEqual([
      { art: 'art.creo', score: 2 },
      { art: 'art.ignem', score: 4 },
    ]);
  });

  it('points a Puissant Art selection at an Art by id', () => {
    installRuleset([
      item({
        id: 'virtue.puissant_art',
        categories: ['hermetic'],
        parameters: [{ key: 'art', type: 'ref', domain: 'art' }],
        effects: [{ type: 'art_bonus', param: 'art', amount: 3 }],
      }),
    ]);
    store.addSelection('virtue.puissant_art');
    store.setArtBonusTarget(0, 'art', 'art.ignem');
    expect(store.entity.selections![0].params).toEqual({ art: 'art.ignem' });
  });

  it('writes the Art under the parameter key that declared it', () => {
    // An Art-domain parameter need not be keyed "art": Master of (Form)
    // Creatures declares `form` over the Art catalogue, and storing the pick
    // under "art" would leave `form` missing (and "art" unexpected).
    installRuleset([
      item({
        id: 'virtue.master_of_form_creatures',
        categories: ['supernatural'],
        parameters: [{ key: 'form', type: 'ref', domain: 'art' }],
      }),
    ]);
    store.addSelection('virtue.master_of_form_creatures');
    store.setArtBonusTarget(0, 'form', 'art.ignem');
    expect(store.entity.selections![0].params).toEqual({ form: 'art.ignem' });
  });
});

// --- House actions ----------------------------------------------------------

// Houses exercising each grant kind: a fixed Virtue (Tytalus), a Choice between
// two Puissant Arts (Flambeau), and an open Minor Virtue (Jerbiton).
const HOUSES: House[] = [
  {
    id: 'house.tytalus',
    lineage_type: 'societas',
    grants: [{ kind: 'fixed', item: 'virtue.self_confident' }],
  },
  {
    id: 'house.flambeau',
    lineage_type: 'societas',
    grants: [
      {
        kind: 'choice',
        choice_key: 'flambeau_puissant',
        options: [
          { ref: 'virtue.puissant_art', params: { art: 'art.perdo' } },
          { ref: 'virtue.puissant_art', params: { art: 'art.ignem' } },
        ],
      },
    ],
  },
  {
    id: 'house.jerbiton',
    lineage_type: 'societas',
    grants: [
      {
        kind: 'open',
        choice_key: 'jerbiton_virtue',
        constraint: { kind: 'virtue', magnitude: 'minor' },
      },
    ],
  },
];

/** Install a House catalogue on the already-installed ruleset. */
function installHouses(houses: House[]): void {
  const map: Record<string, House> = {};
  for (const h of houses) map[h.id] = h;
  store.ruleset!.ruleset.houses = map;
}

describe('setHouse', () => {
  beforeEach(() => {
    installRuleset([]);
    installHouses(HOUSES);
  });

  it('sets the chosen house id', async () => {
    await store.setHouse('house.flambeau');
    expect(store.entity.house).toBe('house.flambeau');
  });

  it('clears the house and all its choices with null', async () => {
    await store.setHouse('house.flambeau');
    store.setHouseChoice('flambeau_puissant', {
      ref: 'virtue.puissant_art',
      params: { art: 'art.ignem' },
    });
    await store.setHouse(null);
    expect(store.entity.house).toBeNull();
    expect(store.entity.house_choices).toEqual({});
  });

  it('drops choices whose choice_key the new house no longer defines', async () => {
    await store.setHouse('house.flambeau');
    store.setHouseChoice('flambeau_puissant', {
      ref: 'virtue.puissant_art',
      params: { art: 'art.ignem' },
    });
    // Tytalus grants only a fixed Virtue — no choice keys — so the stale
    // Flambeau pick must not linger.
    await store.setHouse('house.tytalus');
    expect(store.entity.house).toBe('house.tytalus');
    expect(store.entity.house_choices).toEqual({});
  });

  it('is a no-op when the house is unchanged', async () => {
    await store.setHouse('house.jerbiton');
    const before = store.entity.house_choices;
    await store.setHouse('house.jerbiton');
    // Same reference: the second call returned early rather than re-pruning.
    expect(store.entity.house_choices).toBe(before);
  });
});

describe('setHouseChoice', () => {
  it('stores a pick under the choice key', () => {
    const pick = { ref: 'virtue.puissant_art', params: { art: 'art.ignem' } };
    store.setHouseChoice('flambeau_puissant', pick);
    expect(store.entity.house_choices).toEqual({ flambeau_puissant: pick });
  });

  it('replaces an existing pick for the same key', () => {
    store.setHouseChoice('flambeau_puissant', {
      ref: 'virtue.puissant_art',
      params: { art: 'art.perdo' },
    });
    store.setHouseChoice('flambeau_puissant', {
      ref: 'virtue.puissant_art',
      params: { art: 'art.ignem' },
    });
    expect(store.entity.house_choices).toEqual({
      flambeau_puissant: { ref: 'virtue.puissant_art', params: { art: 'art.ignem' } },
    });
  });

  it('keeps picks for other keys independently', () => {
    store.setHouseChoice('a', { ref: 'virtue.x' });
    store.setHouseChoice('b', { ref: 'virtue.y' });
    expect(store.entity.house_choices).toEqual({
      a: { ref: 'virtue.x' },
      b: { ref: 'virtue.y' },
    });
  });
});

// --- addAbility() -----------------------------------------------------------

describe('addAbility', () => {
  it('adds a plain ability at score 0 and dedups a second add', () => {
    installRuleset([], [ability('ability.awareness')]);
    store.addAbility('ability.awareness');
    store.addAbility('ability.awareness');
    expect(store.entity.ability_scores).toEqual([{ ability: 'ability.awareness', score: 0 }]);
  });

  it('allows a parameterized ability to be added several times', () => {
    installRuleset([], [ability('ability.area_lore', 'area')]);
    store.addAbility('ability.area_lore');
    store.addAbility('ability.area_lore');
    expect(store.entity.ability_scores).toEqual([
      { ability: 'ability.area_lore', score: 0 },
      { ability: 'ability.area_lore', score: 0 },
    ]);
  });
});

// --- removeAbilityAt() ------------------------------------------------------

describe('removeAbilityAt', () => {
  it('removes only the row at the given index, keeping order', () => {
    store.entity.ability_scores = [
      { ability: 'ability.a', score: 1 },
      { ability: 'ability.b', score: 2 },
      { ability: 'ability.c', score: 3 },
    ];
    store.removeAbilityAt(1);
    expect(store.entity.ability_scores).toEqual([
      { ability: 'ability.a', score: 1 },
      { ability: 'ability.c', score: 3 },
    ]);
  });

  it('is a no-op for an out-of-range index', () => {
    store.entity.ability_scores = [{ ability: 'ability.a', score: 1 }];
    store.removeAbilityAt(5);
    expect(store.entity.ability_scores).toEqual([{ ability: 'ability.a', score: 1 }]);
  });
});

// --- adjustAbilityAt() ------------------------------------------------------

describe('adjustAbilityAt', () => {
  beforeEach(() => {
    store.entity.ability_scores = [
      { ability: 'ability.a', score: 2 },
      { ability: 'ability.b', score: 5 },
    ];
  });

  it('raises the score by a positive delta within range', () => {
    store.adjustAbilityAt(0, 2, 10);
    expect(store.entity.ability_scores![0].score).toBe(4);
    // other rows untouched
    expect(store.entity.ability_scores![1].score).toBe(5);
  });

  it('clamps to 0 when the delta would push below 0', () => {
    store.adjustAbilityAt(0, -5, 10);
    expect(store.entity.ability_scores![0].score).toBe(0);
  });

  it('clamps to max when the delta would push above max', () => {
    store.adjustAbilityAt(1, 99, 7);
    expect(store.entity.ability_scores![1].score).toBe(7);
  });

  it('is a no-op for an out-of-range index', () => {
    store.adjustAbilityAt(9, 3, 10);
    expect(store.entity.ability_scores!.map((a) => a.score)).toEqual([2, 5]);
  });
});

// --- setCharacteristicDescription() -----------------------------------------

describe('setCharacteristicDescription', () => {
  it('stores the (untrimmed) text when it is non-blank', () => {
    // The trim() is only an emptiness guard; the raw text is stored verbatim.
    store.setCharacteristicDescription('int', '  Sharp wit  ');
    expect(store.entity.characteristic_descriptions).toEqual({ int: '  Sharp wit  ' });
  });

  it('removes the entry when the text is blank or whitespace-only', () => {
    store.setCharacteristicDescription('int', 'Sharp wit');
    store.setCharacteristicDescription('per', 'Keen eyes');
    store.setCharacteristicDescription('int', '   ');
    expect(store.entity.characteristic_descriptions).toEqual({ per: 'Keen eyes' });
    expect(store.entity.characteristic_descriptions).not.toHaveProperty('int');
  });
});

// --- setAbilitySpecialtyAt() ------------------------------------------------

describe('setAbilitySpecialtyAt', () => {
  beforeEach(() => {
    store.entity.ability_scores = [
      { ability: 'ability.a', score: 1 },
      { ability: 'ability.b', score: 2 },
    ];
  });

  it('stores a trimmed specialty on the targeted row only', () => {
    store.setAbilitySpecialtyAt(0, '  Birds  ');
    expect(store.entity.ability_scores![0]).toEqual({
      ability: 'ability.a',
      score: 1,
      specialty: 'Birds',
    });
    expect(store.entity.ability_scores![1]).toEqual({ ability: 'ability.b', score: 2 });
  });

  it('sets specialty to undefined for blank/whitespace input', () => {
    store.setAbilitySpecialtyAt(0, 'Birds');
    store.setAbilitySpecialtyAt(0, '   ');
    expect(store.entity.ability_scores![0].specialty).toBeUndefined();
  });
});

// --- setAbilityParameterAt() ------------------------------------------------

describe('setAbilityParameterAt', () => {
  beforeEach(() => {
    store.entity.ability_scores = [
      { ability: 'ability.area_lore', score: 1 },
      { ability: 'ability.area_lore', score: 2 },
    ];
  });

  it('stores a trimmed parameter on the targeted row only', () => {
    store.setAbilityParameterAt(1, '  Rhine  ');
    expect(store.entity.ability_scores![1]).toEqual({
      ability: 'ability.area_lore',
      score: 2,
      parameter: { text: 'Rhine' },
    });
    expect(store.entity.ability_scores![0]).toEqual({ ability: 'ability.area_lore', score: 1 });
  });

  it('sets parameter to undefined for blank input', () => {
    store.setAbilityParameterAt(0, 'Rhine');
    store.setAbilityParameterAt(0, '   ');
    expect(store.entity.ability_scores![0].parameter).toBeUndefined();
  });
});

// --- setAbilityParameterValueAt() (CV7) -------------------------------------

describe('setAbilityParameterValueAt', () => {
  /** A loaded document (dirty === false) already carrying one ability row —
   *  `store.open()` establishes both the row and a clean baseline in one
   *  step, unlike a direct `entity.ability_scores = […]` write, which would
   *  itself already differ from the freshly-reset baseline. */
  async function openWithOrgLoreRow(): Promise<void> {
    vi.mocked(ipc.loadEntity).mockResolvedValue({
      path: '/tmp/saga.armc',
      entity: {
        ...store.entity,
        ability_scores: [{ ability: 'ability.organization_lore', score: 1 }],
      },
      migrated_aging_characteristics: [],
    });
    const opening = store.open();
    if (store.discardPromptOpen) store.resolveDiscardPrompt(true);
    await opening;
  }

  beforeEach(async () => {
    await openWithOrgLoreRow();
  });

  it('writes a Catalogued value chosen from the combo box', () => {
    store.setAbilityParameterValueAt(0, { id: 'language.latin' });
    expect(store.entity.ability_scores![0].parameter).toEqual({ id: 'language.latin' });
  });

  it('writes a Linked value chosen from the combo box', () => {
    store.setAbilityParameterValueAt(0, {
      item: 'virtue.craft_guild_training',
      param: 'guild',
    });
    expect(store.entity.ability_scores![0].parameter).toEqual({
      item: 'virtue.craft_guild_training',
      param: 'guild',
    });
  });

  it('replaces a Linked value with a Catalogued one — never keeps both', () => {
    store.setAbilityParameterValueAt(0, { item: 'virtue.craft_guild_training', param: 'guild' });
    store.setAbilityParameterValueAt(0, { id: 'language.latin' });
    expect(store.entity.ability_scores![0].parameter).toEqual({ id: 'language.latin' });
  });

  it('dirties the document', () => {
    expect(store.dirty).toBe(false);
    store.setAbilityParameterValueAt(0, { id: 'language.latin' });
    expect(store.dirty).toBe(true);
  });
});

// --- CV7: the removal/clear flow calls the engine's unlink operation --------

describe('removeSelectionAt calls unlink_ability_parameters before removing (CV7, design § 5.5)', () => {
  beforeEach(() => {
    vi.mocked(ipc.unlinkAbilityParameters).mockClear();
    store.entity.selections = [{ ref: 'virtue.craft_guild_training' }];
    store.entity.ability_scores = [
      {
        ability: 'ability.organization_lore',
        score: 1,
        parameter: { item: 'virtue.craft_guild_training', param: 'guild' },
      },
    ];
    // The removal flow only pays for the round trip when the removed item is
    // a currently-offered link source (a latency optimization over the
    // ordinary, non-linking removal) — mirrors what the real
    // `ability_parameter_options` derived output would report.
    store.effective = {
      ability_parameter_options: [
        {
          ability: 'ability.organization_lore',
          catalogued: [],
          linked: [
            {
              item: 'virtue.craft_guild_training',
              param: 'guild',
              resolved: "Smiths' Guild of Verdi",
            },
          ],
          hint: false,
        },
      ],
    } as unknown as EffectiveScores;
  });

  it('converts a Linked Ability score to Text before splicing the selection out', async () => {
    vi.mocked(ipc.unlinkAbilityParameters).mockResolvedValueOnce({
      ...store.entity,
      ability_scores: [
        {
          ability: 'ability.organization_lore',
          score: 1,
          parameter: { text: "Smiths' Guild of Verdi" },
        },
      ],
    });

    await store.removeSelectionAt(0);

    expect(ipc.unlinkAbilityParameters).toHaveBeenCalledWith(
      expect.anything(),
      'virtue.craft_guild_training',
    );
    expect(store.entity.ability_scores![0].parameter).toEqual({
      text: "Smiths' Guild of Verdi",
    });
    expect(store.entity.selections).toEqual([]);
  });
});

describe('setParamAt calls unlink_ability_parameters when clearing a Bound source to empty (CV7, design § 5.5)', () => {
  beforeEach(() => {
    vi.mocked(ipc.unlinkAbilityParameters).mockClear();
    store.entity.selections = [
      { ref: 'virtue.craft_guild_training', params: { guild: "Smiths' Guild of Verdi" } },
    ];
    store.entity.ability_scores = [
      {
        ability: 'ability.organization_lore',
        score: 1,
        parameter: { item: 'virtue.craft_guild_training', param: 'guild' },
      },
    ];
    store.effective = {
      ability_parameter_options: [
        {
          ability: 'ability.organization_lore',
          catalogued: [],
          linked: [
            {
              item: 'virtue.craft_guild_training',
              param: 'guild',
              resolved: "Smiths' Guild of Verdi",
            },
          ],
          hint: false,
        },
      ],
    } as unknown as EffectiveScores;
  });

  it('converts a Linked Ability score to Text before the clear is applied', async () => {
    vi.mocked(ipc.unlinkAbilityParameters).mockResolvedValueOnce({
      ...store.entity,
      ability_scores: [
        {
          ability: 'ability.organization_lore',
          score: 1,
          parameter: { text: "Smiths' Guild of Verdi" },
        },
      ],
    });

    await store.setParamAt(0, 'guild', '');

    expect(ipc.unlinkAbilityParameters).toHaveBeenCalledWith(
      expect.anything(),
      'virtue.craft_guild_training',
    );
    expect(store.entity.ability_scores![0].parameter).toEqual({
      text: "Smiths' Guild of Verdi",
    });
  });

  it('does not call it for a non-empty edit — a live link must keep tracking a rename', async () => {
    await store.setParamAt(0, 'guild', "Smiths' Guild of Fenster");
    expect(ipc.unlinkAbilityParameters).not.toHaveBeenCalled();
  });
});

// --- Mythic Companion type selection ----------------------------------------

describe('setMythicType / required package', () => {
  // A mythic profile plus a Devil-Child-like type: a fixed status grant, a
  // `choice` free-Minor, two required Virtues (one parameterized), and one
  // required Flaw with a substitute constraint.
  function installMythic(): void {
    installRuleset(
      [
        item({ id: 'virtue.status', magnitude: 'free', categories: ['social_status'] }),
        item({ id: 'virtue.min_a', categories: ['supernatural'] }),
        item({ id: 'virtue.min_b', categories: ['supernatural'] }),
        item({ id: 'virtue.req_major', magnitude: 'major', categories: ['supernatural'] }),
        item({
          id: 'virtue.puissant',
          categories: ['general'],
          parameters: [{ key: 'ability', type: 'ref', domain: 'ability' }],
        }),
        item({ id: 'flaw.default_major', kind: 'flaw', magnitude: 'major', categories: ['story'] }),
        item({ id: 'flaw.other_major', kind: 'flaw', magnitude: 'major', categories: ['story'] }),
      ],
      [],
      {
        mythic_companion: {
          id: 'mythic_companion',
          budget: { virtue_points: 20, flaw_points: 10, virtue_points_per_flaw_point: 2 },
          permitted_categories: [],
          forbidden_categories: [],
          has_mythic_type: true,
          creation_phases: [],
        },
      },
    );
    store.ruleset!.ruleset.mythic_companion_types = {
      'mythic_type.devil': {
        id: 'mythic_type.devil',
        grants: [
          { kind: 'fixed', item: 'virtue.status' },
          {
            kind: 'choice',
            choice_key: 'free_minor',
            options: [{ ref: 'virtue.min_a' }, { ref: 'virtue.min_b' }],
          },
        ],
        required_virtues: [
          { ref: 'virtue.req_major' },
          { ref: 'virtue.puissant', params: { ability: 'ability.guile' } },
        ],
        required_flaws: [
          {
            default: { ref: 'flaw.default_major' },
            constraint: { kind: 'flaw', magnitude: 'major', require_categories: ['story'] },
          },
        ],
      },
      'mythic_type.other': {
        id: 'mythic_type.other',
        grants: [{ kind: 'fixed', item: 'virtue.status' }],
        required_virtues: [{ ref: 'virtue.req_major' }],
      },
    };
    store.entity.type_id = 'mythic_companion';
  }

  beforeEach(installMythic);

  it('seeds the required package (with params) and defaults the free-Minor choice', async () => {
    await store.setMythicType('mythic_type.devil');
    expect(store.entity.mythic_type).toBe('mythic_type.devil');
    // Required Virtues (incl. the parameterized Puissant) + the default Flaw are seeded.
    expect(store.entity.selections).toContainEqual({ ref: 'virtue.req_major' });
    expect(store.entity.selections).toContainEqual({
      ref: 'virtue.puissant',
      params: { ability: 'ability.guile' },
    });
    expect(store.entity.selections).toContainEqual({ ref: 'flaw.default_major' });
    // The free status/Minor Virtues are grants, never bought selections.
    expect(store.entity.selections!.some((s) => s.ref === 'virtue.status')).toBe(false);
    // The `choice` free-Minor defaults to its first option.
    expect(store.entity.mythic_choices?.free_minor).toEqual({ ref: 'virtue.min_a' });
  });

  it('swaps the package when the type changes and drops it when cleared', async () => {
    await store.setMythicType('mythic_type.devil');
    await store.setMythicType('mythic_type.other');
    // Devil-only rows (Puissant, the default Flaw) are dropped; the shared
    // req_major stays; the stale free_minor choice is pruned.
    expect(store.entity.selections!.some((s) => s.ref === 'virtue.puissant')).toBe(false);
    expect(store.entity.selections!.some((s) => s.ref === 'flaw.default_major')).toBe(false);
    expect(store.entity.selections).toContainEqual({ ref: 'virtue.req_major' });
    expect(store.entity.mythic_choices?.free_minor).toBeUndefined();

    await store.setMythicType(null);
    expect(store.entity.mythic_type).toBeNull();
    expect(store.entity.selections!.some((s) => s.ref === 'virtue.req_major')).toBe(false);
  });

  it('swaps a required Flaw for a substitute', async () => {
    await store.setMythicType('mythic_type.devil');
    await store.setMythicRequiredFlaw('flaw.default_major', 'flaw.other_major');
    expect(store.entity.selections!.some((s) => s.ref === 'flaw.default_major')).toBe(false);
    expect(store.entity.selections).toContainEqual({ ref: 'flaw.other_major' });
  });
});

// --- unsaved-changes tracking (close/quit guard) ----------------------------

describe('unsaved-changes tracking', () => {
  /** A full, minimal character used as a freshly-loaded (clean) baseline. */
  function cleanEntity(): Entity {
    return {
      schema_version: 11,
      ruleset: { id: 'test', version: '1' },
      entity_kind: 'character',
      type_id: 'companion',
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
    };
  }

  /** Drive a real open so the store captures a clean saved-baseline. */
  async function loadClean(path = '/tmp/marcus.armc'): Promise<void> {
    vi.mocked(ipc.loadEntity).mockResolvedValue({
      path,
      entity: cleanEntity(),
      migrated_aging_characteristics: [],
    });
    const opening = store.open();
    // The shared singleton may be dirty from a prior test; open() then shows the
    // discard prompt. Confirm it so this setup helper always reaches the load.
    if (store.discardPromptOpen) store.resolveDiscardPrompt(true);
    await opening;
  }

  it('is not dirty once a file has just been loaded', async () => {
    await loadClean();
    expect(store.dirty).toBe(false);
  });

  it('becomes dirty after any mutation', async () => {
    await loadClean();
    store.setIdentity('name', 'Marcus of Bonisagus');
    expect(store.dirty).toBe(true);
  });

  // E3 (round-1 audit): every other case in this block passes equally well under
  // a naive one-way `dirty` boolean that latches true on the first edit and never
  // reconsiders. Only a genuine snapshot-compare (CLAUDE.md: "a snapshot-compare
  // against the last save/load baseline") clears `dirty` again once the edited
  // field is set BACK to its original, saved value — this is the case that tells
  // the two implementations apart.
  it('is a true snapshot compare: editing a field back to its saved baseline clears dirty again', async () => {
    const original = cleanEntity();
    original.name = 'Original Name';
    vi.mocked(ipc.loadEntity).mockResolvedValue({
      path: '/tmp/marcus.armc',
      entity: original,
      migrated_aging_characteristics: [],
    });
    const opening = store.open();
    if (store.discardPromptOpen) store.resolveDiscardPrompt(true);
    await opening;
    expect(store.dirty).toBe(false);

    store.setIdentity('name', 'Changed Name');
    expect(store.dirty).toBe(true);

    store.setIdentity('name', 'Original Name');
    expect(store.dirty).toBe(false);
  });

  it('becomes dirty after an aging/warping annotation edit', async () => {
    await loadClean();
    store.setWarpingEffect('A stigmatic scar');
    expect(store.dirty).toBe(true);
  });

  it('clears dirty after a successful save (non-null path)', async () => {
    await loadClean();
    store.setIdentity('name', 'Marcus');
    expect(store.dirty).toBe(true);

    vi.mocked(ipc.saveEntity).mockResolvedValue('/tmp/marcus.armc');
    await store.save();
    expect(store.dirty).toBe(false);
  });

  // S3 (full-audit round 2): the schema-migration notice closes with "Saving
  // will keep the rebuilt values" — a future-tense instruction that stayed on
  // screen, untouched and undismissible, for the rest of the session after the
  // save it describes had already happened.
  it('retires the schema-migration notice once the save it asks for has happened', async () => {
    vi.mocked(ipc.loadEntity).mockResolvedValue({
      path: '/tmp/migrated.armc',
      entity: cleanEntity(),
      migrated_aging_characteristics: ['str'],
    });
    const opening = store.open();
    if (store.discardPromptOpen) store.resolveDiscardPrompt(true);
    await opening;

    expect(store.migrationNotice).not.toBeNull();

    vi.mocked(ipc.saveEntity).mockResolvedValue('/tmp/migrated.armc');
    await store.save();

    expect(store.migrationNotice).toBeNull();
  });

  it('keeps the migration notice when the save was cancelled', async () => {
    vi.mocked(ipc.loadEntity).mockResolvedValue({
      path: '/tmp/migrated.armc',
      entity: cleanEntity(),
      migrated_aging_characteristics: ['str'],
    });
    const opening = store.open();
    if (store.discardPromptOpen) store.resolveDiscardPrompt(true);
    await opening;
    // No current file, so save() routes to Save As, which can be cancelled.
    store.currentPath = null;

    vi.mocked(ipc.saveEntity).mockResolvedValue(null);
    await store.save();

    // Nothing was written, so the advice has not been consumed.
    expect(store.migrationNotice).not.toBeNull();
  });

  it('stays dirty after a cancelled save (null return)', async () => {
    await loadClean();
    // Clear the current file so save() routes to Save As (which prompts and can
    // be cancelled). A cancelled prompt returns null: nothing was written.
    store.currentPath = null;
    store.setIdentity('name', 'Marcus');

    vi.mocked(ipc.saveEntity).mockResolvedValue(null);
    await store.save();
    expect(store.dirty).toBe(true);
  });

  it('stays dirty after a language reload that keeps the edited entity', async () => {
    await loadClean();
    store.setIdentity('name', 'Marcus');
    expect(store.dirty).toBe(true);

    // setLang → #reloadRuleset(false): keeps the entity, must NOT clear dirty.
    vi.mocked(ipc.loadRuleset).mockResolvedValue(installRuleset([]));
    await store.setLang('de');
    expect(store.dirty).toBe(true);
  });

  it('keeps edits made while the save dialog is open marked dirty', async () => {
    await loadClean();
    store.setIdentity('name', 'Marcus');

    // Model a slow native dialog: capture the snapshot at call time, resolve later.
    let resolveSave: (path: string | null) => void = () => {};
    vi.mocked(ipc.saveEntity).mockReturnValue(
      new Promise<string | null>((resolve) => {
        resolveSave = resolve;
      }),
    );
    const saving = store.save();
    // User keeps typing while the dialog is open.
    store.setIdentity('name', 'Marcus of Bonisagus');
    resolveSave('/tmp/marcus.armc');
    await saving;

    // The write captured the earlier state, so the newer edit is still unsaved.
    expect(store.dirty).toBe(true);
  });

  it('reports dirty + localized labels to the backend guard', async () => {
    await loadClean();
    let payload = store.closeGuardPayload();
    expect(payload.dirty).toBe(false);
    expect(payload.labels.title).toBe(store.t('close-unsaved-title'));
    expect(payload.labels.message).toBe(store.t('close-unsaved-message'));
    expect(payload.labels.discard).toBe(store.t('close-unsaved-discard'));
    expect(payload.labels.cancel).toBe(store.t('close-unsaved-cancel'));

    store.setIdentity('name', 'Marcus');
    payload = store.closeGuardPayload();
    expect(payload.dirty).toBe(true);
  });
});

// --- document file model (current file, Save vs Save As, New, in-flight) -----

describe('document file model', () => {
  function cleanEntity(): Entity {
    return {
      schema_version: 11,
      ruleset: { id: 'test', version: '1' },
      entity_kind: 'character',
      type_id: 'companion',
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
    };
  }

  /** Open a file with a known path so the store tracks it as the current file. */
  async function openFile(path = '/tmp/marcus.armc'): Promise<void> {
    vi.mocked(ipc.loadEntity).mockResolvedValue({
      path,
      entity: cleanEntity(),
      migrated_aging_characteristics: [],
    });
    const opening = store.open();
    // The shared singleton may be dirty from a prior test; confirm the discard
    // prompt so this setup helper always reaches the load.
    if (store.discardPromptOpen) store.resolveDiscardPrompt(true);
    await opening;
  }

  beforeEach(() => {
    vi.mocked(ipc.saveEntity).mockReset();
    vi.mocked(ipc.loadEntity).mockReset();
    store.currentPath = null;
    store.filters = defaultPickerFilters();
  });

  it('open() sets the current file path from the loaded document', async () => {
    await openFile('/tmp/aelius.armc');
    expect(store.currentPath).toBe('/tmp/aelius.armc');
    expect(store.currentFileName).toBe('aelius.armc');
  });

  it('save() writes directly to the current file without prompting', async () => {
    await openFile('/tmp/marcus.armc');
    store.setIdentity('name', 'Marcus');
    vi.mocked(ipc.saveEntity).mockResolvedValue('/tmp/marcus.armc');

    await store.save();

    // Passed the known path (no prompt) — second arg is the current file.
    expect(ipc.saveEntity).toHaveBeenCalledTimes(1);
    expect(vi.mocked(ipc.saveEntity).mock.calls[0][1]).toBe('/tmp/marcus.armc');
    expect(store.dirty).toBe(false);
  });

  it('save() with no current file falls back to Save As (prompt)', async () => {
    store.setIdentity('name', 'Marcus');
    vi.mocked(ipc.saveEntity).mockResolvedValue('/tmp/new.armc');

    await store.save();

    // Prompted: path arg is null so the backend opens the dialog.
    expect(vi.mocked(ipc.saveEntity).mock.calls[0][1]).toBeNull();
    expect(store.currentPath).toBe('/tmp/new.armc');
  });

  it('saveAs() always prompts and updates the current file on success', async () => {
    await openFile('/tmp/marcus.armc');
    store.setIdentity('name', 'Marcus');
    vi.mocked(ipc.saveEntity).mockResolvedValue('/tmp/renamed.armc');

    await store.saveAs();

    expect(vi.mocked(ipc.saveEntity).mock.calls[0][1]).toBeNull();
    expect(store.currentPath).toBe('/tmp/renamed.armc');
    expect(store.dirty).toBe(false);
  });

  it('saveAs() cancel leaves the current file unchanged and stays dirty', async () => {
    await openFile('/tmp/marcus.armc');
    store.setIdentity('name', 'Marcus');
    vi.mocked(ipc.saveEntity).mockResolvedValue(null);

    await store.saveAs();

    expect(store.currentPath).toBe('/tmp/marcus.armc');
    expect(store.dirty).toBe(true);
  });

  it('save() failure surfaces an error and keeps the document dirty', async () => {
    await openFile('/tmp/marcus.armc');
    store.setIdentity('name', 'Marcus');
    vi.mocked(ipc.saveEntity).mockRejectedValue({ kind: 'io' });

    await store.save();

    expect(store.error).toEqual({ kind: 'io' });
    expect(store.dirty).toBe(true);
    expect(store.currentPath).toBe('/tmp/marcus.armc');
  });

  it('a second save while one is in flight is a no-op (guarded)', async () => {
    await openFile('/tmp/marcus.armc');
    store.setIdentity('name', 'Marcus');
    // A save that stays pending models a still-open write; the second call must
    // not re-enter while the first is unresolved.
    let finishFirst: (path: string | null) => void = () => {};
    vi.mocked(ipc.saveEntity).mockReturnValue(
      new Promise<string | null>((resolve) => {
        finishFirst = resolve;
      }),
    );

    const first = store.save();
    void store.save();

    expect(ipc.saveEntity).toHaveBeenCalledTimes(1);

    // Let the first write finish so the in-flight guard clears for later tests.
    finishFirst('/tmp/marcus.armc');
    await first;
  });

  it('newDocument() resets the current file, filters, and dirty baseline', async () => {
    await openFile('/tmp/marcus.armc');
    store.filters.abilities.search = 'latin';
    // Clean document, so no discard prompt is needed.
    await store.newDocument();

    expect(store.currentPath).toBeNull();
    expect(store.dirty).toBe(false);
    expect(store.filters.abilities.search).toBe('');
    expect(store.entity.name).toBeUndefined();
  });

  it('newDocument() on a dirty document waits for the discard confirmation', async () => {
    await openFile('/tmp/marcus.armc');
    store.setIdentity('name', 'Marcus');

    // Cancelling aborts: the edited document is kept.
    vi.mocked(ipc.confirmDiscard).mockResolvedValueOnce(false);
    await store.newDocument();
    expect(store.entity.name).toBe('Marcus');
    expect(store.currentPath).toBe('/tmp/marcus.armc');

    // Confirming discards and resets to a fresh document.
    vi.mocked(ipc.confirmDiscard).mockResolvedValueOnce(true);
    await store.newDocument();
    expect(store.entity.name).toBeUndefined();
    expect(store.currentPath).toBeNull();
    expect(store.dirty).toBe(false);
  });

  it('open() on a dirty document honors a cancelled discard confirmation', async () => {
    await openFile('/tmp/marcus.armc');
    store.setIdentity('name', 'Marcus');

    // Ignore the setup helper's load; only the cancelled open below matters.
    vi.mocked(ipc.loadEntity).mockClear();
    vi.mocked(ipc.loadEntity).mockResolvedValue({
      path: '/tmp/other.armc',
      entity: cleanEntity(),
      migrated_aging_characteristics: [],
    });
    vi.mocked(ipc.confirmDiscard).mockResolvedValueOnce(false);
    await store.open();
    // Cancelled: the current file is unchanged and loadEntity was never called.
    expect(store.currentPath).toBe('/tmp/marcus.armc');
    expect(ipc.loadEntity).not.toHaveBeenCalled();
  });

  // --- C3b: one discard confirmation, and it is the native one --------------

  /**
   * Let the awaited `confirm_discard` round-trip settle. Microtasks only: this
   * file runs on fake timers, so a `setTimeout(0)` tick would never fire.
   */
  async function settleConfirmation(): Promise<void> {
    for (let i = 0; i < 5; i++) await Promise.resolve();
  }

  it('asks the backend to confirm the discard, with the localized discard labels', async () => {
    await openFile('/tmp/marcus.armc');
    store.setIdentity('name', 'Marcus');
    vi.mocked(ipc.confirmDiscard).mockClear();

    await store.newDocument();

    expect(ipc.confirmDiscard).toHaveBeenCalledTimes(1);
    // No user-facing string is authored in Rust: every word arrives resolved
    // from Fluent, exactly as `closeGuardPayload()` supplies the close dialog's.
    expect(vi.mocked(ipc.confirmDiscard).mock.calls[0][0]).toEqual({
      title: store.t('discard-changes-title'),
      message: store.t('discard-changes-message'),
      discard: store.t('discard-changes-confirm'),
      cancel: store.t('discard-changes-cancel'),
    });
  });

  it('never confirms a discard on a clean document', async () => {
    await openFile('/tmp/marcus.armc');
    vi.mocked(ipc.confirmDiscard).mockClear();

    await store.newDocument();

    expect(ipc.confirmDiscard).not.toHaveBeenCalled();
  });

  // TRAP 2. `CloseGuardState::confirmed` (Rust) is a one-shot LATCH that is
  // never reset until the frontend reports a fresh dirty state, and it exists so
  // a re-issued *quit* does not prompt twice. A non-terminal action must not
  // ride on it: if it did, the first confirmed New would silence every later
  // one, and the second New would destroy the player's work with no prompt at
  // all. So this test confirms TWICE in one session and insists the second New
  // asked again.
  it('asks again on the second discard of the same session (no confirmation latch)', async () => {
    await openFile('/tmp/marcus.armc');
    store.setIdentity('name', 'Marcus');
    vi.mocked(ipc.confirmDiscard).mockClear();
    vi.mocked(ipc.confirmDiscard).mockResolvedValue(true);

    await store.newDocument();
    expect(ipc.confirmDiscard).toHaveBeenCalledTimes(1);
    expect(store.entity.name).toBeUndefined();

    // A second document, dirtied again, discarded again — the case the latch
    // would have swallowed.
    await openFile('/tmp/aelius.armc');
    store.setIdentity('name', 'Aelius');
    expect(store.dirty).toBe(true);

    await store.newDocument();
    expect(ipc.confirmDiscard).toHaveBeenCalledTimes(2);
    expect(store.entity.name).toBeUndefined();
  });

  it('falls back to the in-app prompt when the build offers no native dialog', async () => {
    await openFile('/tmp/marcus.armc');
    store.setIdentity('name', 'Marcus');
    // What the `e2e-testing` build answers: no native dialog here, so the
    // frontend must raise the one WebDriver can click.
    vi.mocked(ipc.confirmDiscard).mockResolvedValueOnce(null);

    const pending = store.newDocument();
    await settleConfirmation();
    expect(store.discardPromptOpen).toBe(true);

    store.resolveDiscardPrompt(false);
    await pending;
    expect(store.entity.name).toBe('Marcus');
    expect(store.discardPromptOpen).toBe(false);
  });

  // A rejected IPC call must never read as "yes, discard". The in-app prompt is
  // the safety net for a broken bridge, not only for the e2e build: losing an
  // unsaved character to a failed `invoke` is the highest-severity class of bug
  // this project has (CLAUDE.md).
  it('falls back to the in-app prompt when the confirmation call fails', async () => {
    await openFile('/tmp/marcus.armc');
    store.setIdentity('name', 'Marcus');
    vi.mocked(ipc.confirmDiscard).mockRejectedValueOnce({ kind: 'io' });

    const pending = store.newDocument();
    await settleConfirmation();
    expect(store.discardPromptOpen).toBe(true);

    store.resolveDiscardPrompt(false);
    await pending;
    expect(store.entity.name).toBe('Marcus');
  });

  it('blocks a second New while a discard confirmation is already pending', async () => {
    await openFile('/tmp/marcus.armc');
    store.setIdentity('name', 'Marcus');
    vi.mocked(ipc.confirmDiscard).mockClear();
    // The setup helper above already loaded once; only the blocked Open counts.
    vi.mocked(ipc.loadEntity).mockClear();
    // A confirmation that has not answered yet models a dialog still on screen.
    let answer: (discard: boolean | null) => void = () => {};
    vi.mocked(ipc.confirmDiscard).mockReturnValueOnce(
      new Promise<boolean | null>((resolve) => {
        answer = resolve;
      }),
    );

    const first = store.newDocument();
    expect(store.discardConfirmPending).toBe(true);

    void store.newDocument();
    void store.open();

    expect(ipc.confirmDiscard).toHaveBeenCalledTimes(1);
    expect(ipc.loadEntity).not.toHaveBeenCalled();

    // Answer it so the flag does not leak into the next test.
    answer(false);
    await first;
    expect(store.discardConfirmPending).toBe(false);
  });
});

// --- Markdown export (M5.6c) -------------------------------------------------

describe('exportMarkdown', () => {
  /** A profile skeleton: only its id matters here — it names the `type-<id>` key. */
  function profile(id: string): EntityTypeProfile {
    return {
      id,
      budget: { virtue_points: 10, flaw_points: 10 },
      permitted_categories: [],
      forbidden_categories: [],
      creation_phases: ['concept'],
    };
  }

  /**
   * A ruleset that exercises all three key families the engine composes from
   * catalogue data and therefore cannot enumerate: two character-type profiles, a
   * parameter key declared on each of the three parameterized catalogues
   * (Virtue/Flaw, Ability, spell), and two distinct point-item categories.
   */
  function installExportRuleset(): void {
    const localized = installRuleset(
      [
        item({
          id: 'virtue.puissant_ability',
          categories: ['hermetic'],
          parameters: [{ key: 'ability', type: 'ref', domain: 'ability' }],
        }),
        item({ id: 'flaw.optimistic', kind: 'flaw', categories: ['personality'] }),
        // A dual-category item, so the composed `category-<id>` family has to
        // cover a category no item lists first.
        item({ id: 'flaw.visions', kind: 'flaw', categories: ['story', 'supernatural'] }),
      ],
      [ability('ability.area_lore', 'area')],
      { magus: profile('magus'), grog: profile('grog') },
    );
    const spell: Spell = {
      id: 'spell.wizards_boost',
      technique: 'art.muto',
      form: 'art.vim',
      parameters: [{ key: 'form', type: 'ref', domain: 'form' }],
    };
    localized.ruleset.spells = { [spell.id]: spell };
  }

  /** The label map the store handed the export command on its last call. */
  function sentLabels(): Record<string, string> {
    return vi.mocked(ipc.exportMarkdown).mock.calls[0][1];
  }

  /** The current-save-file hint the store handed the export command, for the prefill. */
  function sentCurrentPath(): string | null | undefined {
    return vi.mocked(ipc.exportMarkdown).mock.calls[0][3];
  }

  beforeEach(() => {
    vi.mocked(ipc.exportMarkdown).mockReset();
    vi.mocked(ipc.exportLabelKeys).mockReset();
    vi.mocked(ipc.saveEntity).mockReset();
    vi.mocked(ipc.exportMarkdown).mockResolvedValue('/tmp/marcus.md');
    vi.mocked(ipc.exportLabelKeys).mockResolvedValue(['identity-name', 'abilities-title']);
    installExportRuleset();
    store.lang = 'en';
    store.error = null;
    store.currentPath = null;
  });

  it('sends the entity and a resolved label for every key the engine asks for', async () => {
    await store.exportMarkdown();

    expect(ipc.exportMarkdown).toHaveBeenCalledTimes(1);
    expect(vi.mocked(ipc.exportMarkdown).mock.calls[0][0].type_id).toBe(store.entity.type_id);
    const labels = sentLabels();
    expect(labels['identity-name']).toBe('Name');
    expect(labels['abilities-title']).toBe('Abilities');
  });

  it('adds the three composed families the engine cannot enumerate', async () => {
    await store.exportMarkdown();

    const labels = sentLabels();
    // `type-<profile id>`: one per shipped profile, read off the ruleset.
    expect(labels['type-magus']).toBe('Magus');
    expect(labels['type-grog']).toBe('Grog');
    // `param-label-<parameter key>`: declared by a Virtue, an Ability and a spell.
    expect(labels['param-label-ability']).toBe('Ability');
    expect(labels['param-label-area']).toBe('Area');
    expect(labels['param-label-form']).toBe('Form');
    // `category-<item category>`: the Type cell of an exported Virtue/Flaw row —
    // one per distinct category the point-item catalogue uses.
    expect(labels['category-hermetic']).toBe('Hermetic');
    expect(labels['category-personality']).toBe('Personality');
  });

  // The exporter prints EVERY category of a Virtue/Flaw in its Type cell, so the
  // label family must cover a secondary category too — collecting only the
  // primary would ship `supernatural` as its own label in the sheet.
  it('includes a category no item carries as its primary', async () => {
    await store.exportMarkdown();

    const labels = sentLabels();
    expect(labels['category-story']).toBe('Story');
    expect(labels['category-supernatural']).toBe('Supernatural');
  });

  it('never echoes a key back as its own label, in either language', async () => {
    // Fluent's miss behavior is to return the key, so an unresolved composed key
    // would silently ship a raw slug into the exported sheet.
    for (const lang of ['en', 'de'] as const) {
      vi.mocked(ipc.exportMarkdown).mockClear();
      store.lang = lang;
      await store.exportMarkdown();
      for (const [key, text] of Object.entries(sentLabels())) {
        expect(text, `'${key}' is unresolved in '${lang}'`).not.toBe(key);
      }
    }
    store.lang = 'en';
  });

  it('leaves the document dirty: an export is not a save', async () => {
    store.currentPath = '/tmp/marcus.armc';
    store.setIdentity('name', 'Marcus');
    expect(store.dirty).toBe(true);

    await store.exportMarkdown();

    expect(store.dirty).toBe(true);
    expect(ipc.saveEntity).not.toHaveBeenCalled();
  });

  it('never adopts the written Markdown file as the current file', async () => {
    store.currentPath = '/tmp/marcus.armc';

    await store.exportMarkdown();

    expect(store.currentPath).toBe('/tmp/marcus.armc');
  });

  it('passes the current save file so the dialog can prefill from it', async () => {
    // A save records the current file; the export dialog then defaults to that
    // file's name and directory rather than the generic kind default.
    vi.mocked(ipc.saveEntity).mockResolvedValue('/tmp/gerhard.armc');
    await store.saveAs();

    await store.exportMarkdown();

    expect(sentCurrentPath()).toBe('/tmp/gerhard.armc');
  });

  it('passes no current save file for a document that was never saved', async () => {
    expect(store.currentPath).toBeNull();

    await store.exportMarkdown();

    expect(sentCurrentPath()).toBeNull();
  });

  it('surfaces a failed export in the error banner and clears the busy flag', async () => {
    vi.mocked(ipc.exportMarkdown).mockRejectedValue({ kind: 'io' });

    await store.exportMarkdown();

    expect(store.error).toEqual({ kind: 'io' });
    expect(store.busy).toBe(false);
  });

  it('is a no-op while another file operation is in flight', async () => {
    store.currentPath = '/tmp/marcus.armc';
    let finishSave: (path: string | null) => void = () => {};
    vi.mocked(ipc.saveEntity).mockReturnValue(
      new Promise<string | null>((resolve) => {
        finishSave = resolve;
      }),
    );

    const saving = store.save();
    await store.exportMarkdown();

    expect(ipc.exportMarkdown).not.toHaveBeenCalled();

    // Let the save finish so the in-flight guard clears for later tests.
    finishSave('/tmp/marcus.armc');
    await saving;
  });

  // MOVED HERE BY C3c from `SaveLoadBar.test.ts`, which asserted it of the
  // Export toolbar button that no longer exists. It was always a claim about the
  // store action rather than the button: exporting must not write the save file,
  // or "export a copy" would quietly become "save".
  it('exports without writing the document through the save path', async () => {
    vi.mocked(ipc.saveEntity).mockReset();

    await store.exportMarkdown();

    expect(ipc.exportMarkdown).toHaveBeenCalledTimes(1);
    expect(ipc.saveEntity).not.toHaveBeenCalled();
  });
});

// --- the aging roll draft (M6/6b6c) -----------------------------------------

describe('the aging roll draft', () => {
  beforeEach(() => {
    store.clearAgingDraft();
    vi.mocked(ipc.agingPreview).mockClear();
  });

  it('keeps the typed aging die off the entity, so the calculator never dirties the document', async () => {
    await store.createCharacter('companion');
    expect(store.dirty).toBe(false);

    store.setAgingYear(40);
    store.setAgingDie(9);
    store.setAgingDistribution('sta', 2);

    // The die is player input the entity must never store: it is not a choice the
    // character records, and `dirty` is a snapshot compare of the entity, so
    // holding it here is what makes "the calculator does not persist" true.
    expect(store.dirty).toBe(false);
    const serialized = JSON.stringify(store.entity);
    expect(serialized).not.toContain('die');
    expect(serialized).not.toContain('aging_log');
    expect(store.agingDraft).toEqual({
      age: 40,
      die: 9,
      distribution: { sta: 2 },
      crisisDie: null,
    });
  });

  it('drops the aging draft when a new character is created', async () => {
    store.setAgingYear(40);
    store.setAgingDie(9);

    await store.createCharacter('companion');

    expect(store.agingDraft).toEqual({
      age: null,
      die: null,
      distribution: {},
      crisisDie: null,
    });
    expect(store.agingPreview).toBeNull();
  });

  it('keeps the Crisis die off the entity too, and asks the engine again for each', async () => {
    // The Simple Die of `ArMDE:16621` is player input exactly as the stress die is —
    // the engine has no `rand` dependency and rolls neither — so it lives in the
    // draft and the document stays clean.
    await store.createCharacter('companion');
    vi.mocked(ipc.agingPreview).mockResolvedValue({
      status: 'previewed',
      total: { total: 13 },
      outcome: { total: 13, crisis: true },
      crisis: { total: { total: 15 }, row: 'crisis.minor_illness' },
    } as unknown as AgingProjection);

    store.setAgingYear(40);
    store.setAgingDie(9);
    store.setAgingCrisisDie(10);
    expect(store.dirty).toBe(false);
    expect(JSON.stringify(store.entity)).not.toContain('crisis');
    expect(store.agingDraft.crisisDie).toBe(10);

    // Placing the points changes the CRISIS TOTAL, because those points ARE the
    // Decrepitude increase `ArMDE:16619` puts first — so the reading has to be asked
    // for again, not left standing.
    vi.mocked(ipc.agingPreview).mockClear();
    store.setAgingDistribution('sta', 5);
    await store.previewAgingRoll();
    expect(vi.mocked(ipc.agingPreview).mock.lastCall?.slice(1)).toEqual([40, 9, { sta: 5 }, 10]);
    expect(store.agingPreview?.crisis?.total.total).toBe(15);
  });

  it('reports what the applied year spent, and forgets it with the draft', async () => {
    await store.createCharacter('companion');
    vi.mocked(ipc.agingApply).mockResolvedValue({
      status: 'applied',
      entity: $state.snapshot(store.entity),
      total: { total: 13 },
      outcome: { total: 13, crisis: true },
      notes: [{ kind: 'longevity_ritual_spent' }],
    } as unknown as AgingApplication);

    store.setAgingYear(40);
    store.setAgingDie(9);
    store.setAgingCrisisDie(10);
    store.setAgingDistribution('sta', 5);
    await store.applyAgingRoll();

    // The Crisis die crosses with the rest of the form, or the engine records the
    // Crisis as owed and unrolled however carefully the player rolled it.
    expect(vi.mocked(ipc.agingApply).mock.lastCall?.slice(1)).toEqual([40, 9, { sta: 5 }, 10]);
    // "its power is spent, and the focal ritual must be performed again"
    // (`ArMDE:16573`) — the entity keeps the ritual, so only the note can say this.
    expect(store.agingNotes).toEqual([{ kind: 'longevity_ritual_spent' }]);

    store.clearAgingDraft();
    expect(store.agingNotes).toEqual([]);
  });

  it('never lets a stale preview overwrite a newer one', async () => {
    /** A previewed projection carrying only the field this test reads. */
    const previewed = (total: number) =>
      ({
        status: 'previewed',
        total: { total },
        outcome: { total },
      }) as unknown as AgingProjection;

    let finishFirst: (value: AgingProjection) => void = () => {};
    vi.mocked(ipc.agingPreview)
      .mockReturnValueOnce(
        new Promise<AgingProjection>((resolve) => {
          finishFirst = resolve;
        }),
      )
      .mockResolvedValueOnce(previewed(14));

    store.setAgingYear(40);
    store.setAgingDie(9);
    const stale = store.previewAgingRoll();
    store.setAgingDie(10);
    await store.previewAgingRoll();

    // The newer answer is in place; the older one lands afterwards and is dropped.
    finishFirst(previewed(13));
    await stale;

    expect(store.agingPreview?.total.total).toBe(14);
  });
});

// --- the aging log's remove button (manual-testing-findings #23) ------------

describe('the aging log’s remove button', () => {
  /**
   * The row the engine writes when a year is resolved: the age it was rolled for
   * (the key `aging::revert_year` addresses entries by), the points it placed and
   * whether it advanced the appearance. Deleting such a row on its own leaves
   * every one of those effects standing on the character with nothing left to
   * explain them — which is the corruption these tests pin.
   */
  function recorded(age: number, year: number): AgingLogEntry {
    return {
      year,
      age,
      effect: '',
      die: 9,
      total: 13,
      points: { sta: 1 },
      apparent_age_increased: true,
    };
  }

  /** A hand-written row: free text and no age, so there is nothing mechanical to
   *  undo and `aging.rs::revert_year` cannot reach it. */
  const handWritten: AgingLogEntry = { year: 1219, effect: 'A hard winter' };

  /** The engine's refusal for an age nothing records (`AgingError::YearNotRecorded`). */
  const notRecorded: ValidationIssue = {
    severity: 'error',
    code: 'aging_year_not_recorded',
    phase: 'aging',
    args: { age: '40' },
  };

  beforeEach(() => {
    store.clearAgingDraft();
    store.error = null;
    vi.mocked(ipc.agingRevert).mockReset();
  });

  it('takes an engine-recorded year off the character, not just off the log', async () => {
    // One resolved year: 1 Aging Point in Stamina and a year of apparent age.
    store.entity.aging_points = { sta: 1 };
    store.entity.apparent_age = 41;
    store.entity.aging_log = [recorded(40, 1220)];

    // What `aging::revert_year` hands back for age 40: the character exactly as
    // he stood before that year was applied.
    const unaged: Entity = {
      ...($state.snapshot(store.entity) as Entity),
      aging_points: {},
      apparent_age: null,
      aging_log: [],
    };
    vi.mocked(ipc.agingRevert).mockResolvedValue({ status: 'reverted', entity: unaged });

    await store.removeAgingLogEntryAt(0);

    // The year's effects come back off with its row. A plain array filter left
    // both applied and deleted the only record of where they came from.
    expect(store.entity.aging_points).toEqual({});
    expect(store.entity.apparent_age).toBeNull();
    expect(store.entity.aging_log).toEqual([]);
    // Addressed by the entry's age, which is the engine's key.
    expect(vi.mocked(ipc.agingRevert).mock.lastCall?.[1]).toBe(40);
  });

  it('takes back the row that was clicked, by its age and not by its index', async () => {
    // A hand-written row first, so the clicked row's index (1) is neither of the
    // recorded ages (40, 41) — sending the index would revert the wrong year, or
    // no year at all.
    store.entity.aging_points = { sta: 2 };
    store.entity.apparent_age = 42;
    store.entity.aging_log = [handWritten, recorded(40, 1220), recorded(41, 1221)];

    const without40: Entity = {
      ...($state.snapshot(store.entity) as Entity),
      aging_points: { sta: 1 },
      apparent_age: 41,
      aging_log: [handWritten, recorded(41, 1221)],
    };
    vi.mocked(ipc.agingRevert).mockResolvedValue({ status: 'reverted', entity: without40 });

    await store.removeAgingLogEntryAt(1);

    expect(vi.mocked(ipc.agingRevert).mock.lastCall?.[1]).toBe(40);
    expect(store.entity.aging_points).toEqual({ sta: 1 });
    expect(store.entity.apparent_age).toBe(41);
    expect(store.entity.aging_log).toEqual([handWritten, recorded(41, 1221)]);
  });

  it('drops a hand-written row without asking the engine', async () => {
    // No age, so nothing mechanical was ever applied and there is nothing to
    // undo: the ordinary log editor is the right home for it.
    store.entity.aging_points = { sta: 1 };
    store.entity.apparent_age = 41;
    store.entity.aging_log = [handWritten, recorded(40, 1220)];

    await store.removeAgingLogEntryAt(0);

    expect(ipc.agingRevert).not.toHaveBeenCalled();
    expect(store.entity.aging_log).toEqual([recorded(40, 1220)]);
    // And the recorded year it sat next to is untouched.
    expect(store.entity.aging_points).toEqual({ sta: 1 });
    expect(store.entity.apparent_age).toBe(41);
  });

  it('keeps the row and reports the refusal when the engine will not take the year back', async () => {
    store.entity.aging_points = { sta: 1 };
    store.entity.apparent_age = 41;
    store.entity.aging_log = [recorded(40, 1220)];
    vi.mocked(ipc.agingRevert).mockResolvedValue({ status: 'rejected', issues: [notRecorded] });

    await store.removeAgingLogEntryAt(0);

    // A × that deleted the row anyway would claim an undo that never happened,
    // and the effects it did not take off would have no record left.
    expect(store.entity.aging_log).toEqual([recorded(40, 1220)]);
    expect(store.entity.aging_points).toEqual({ sta: 1 });
    expect(store.agingRejections).toEqual([notRecorded]);
  });

  it('surfaces a failed undo in the error banner rather than dropping the row', async () => {
    store.entity.aging_points = { sta: 1 };
    store.entity.aging_log = [recorded(40, 1220)];
    vi.mocked(ipc.agingRevert).mockRejectedValue({ kind: 'io' });

    await store.removeAgingLogEntryAt(0);

    expect(store.error).toEqual({ kind: 'io' });
    expect(store.entity.aging_log).toEqual([recorded(40, 1220)]);
    expect(store.entity.aging_points).toEqual({ sta: 1 });
  });
});

// The palette the app paints with is chosen HERE, not in CSS: `app.css` declares
// a dark `:root` and a light `:root[data-theme='light']`, and the store decides
// which of the two `<html>` names. Putting the decision in one place is what lets
// `auto` and an explicit choice take the same path — a `prefers-color-scheme`
// media query would have needed a second copy of the whole light palette, and
// two copies drift.
describe('the active theme', () => {
  afterEach(() => {
    store.theme = 'auto';
    store.osPrefersLight = false;
  });

  it('follows the OS by default, so a correct palette needs no settings UI', () => {
    expect(store.theme).toBe('auto');
  });

  it('resolves auto against what the OS currently prefers', () => {
    store.theme = 'auto';

    store.osPrefersLight = false;
    expect(store.resolvedTheme).toBe('dark');

    // The resolved value is what `<html data-theme>` is set to, so it is never
    // 'auto': CSS has no rule for a theme that has not been decided.
    store.osPrefersLight = true;
    expect(store.resolvedTheme).toBe('light');
  });

  it('lets an explicit choice override the OS in either direction', () => {
    store.osPrefersLight = true;
    store.theme = 'dark';
    expect(store.resolvedTheme).toBe('dark');

    store.osPrefersLight = false;
    store.theme = 'light';
    expect(store.resolvedTheme).toBe('light');
  });

  // The store module is imported by the whole SSR suite, which runs under `node`
  // with no `window` at all — and the shipped app itself evaluates this module
  // before the webview has done anything. Reaching for `matchMedia` unguarded
  // would take the store's import down with it, so the watcher degrades to a
  // no-op instead and the app simply stays on its default palette.
  it('degrades to a no-op where no media-query API exists at all', () => {
    expect(() => store.watchSystemTheme()()).not.toThrow();
    expect(store.resolvedTheme).toBe('dark');
  });
});

// --- C4: the four settings that survive a restart --------------------------
describe('persisted settings', () => {
  /** A ruleset payload `loadRuleset` can resolve with, tagged by language. */
  function localized(lang: string): LocalizedRuleset {
    return {
      ruleset: {
        id: `test-${lang}`,
        version: '1',
        point_items: {},
        type_profiles: {},
        abilities: {},
        magnitude_points: { free: 0, minor: 1, major: 3 },
        ability_category_order: ['general'],
        art_type_order: ['technique', 'form'],
      },
      i18n: {},
    };
  }

  beforeEach(() => {
    vi.mocked(ipc.readSettings).mockReset().mockResolvedValue(persisted());
    vi.mocked(ipc.writeSettings).mockReset().mockResolvedValue(undefined);
    vi.mocked(ipc.loadRuleset)
      .mockReset()
      .mockImplementation((lang: string) => Promise.resolve(localized(lang)));
  });

  afterEach(() => {
    // Put the shared singleton back to the state every other block was written
    // against: English, dark-by-OS, enforced, and the published setting's year as
    // the default new documents start at.
    store.lang = 'en';
    store.theme = 'auto';
    store.mode = 'enforced';
    store.defaultSagaYear = DEFAULT_SAGA_YEAR;
    store.closeSettings();
    vi.mocked(ipc.loadRuleset).mockReset();
  });

  // THE ORDERING TRAP. `init()` used to run the ruleset load and the saga-year read
  // concurrently, which was correct precisely because neither needed the other. A
  // persisted LANGUAGE destroys that premise: rules display text is per-language, so
  // the language has to be known BEFORE the load. Loading first and reloading after
  // would have shown a German user a screen of English on first paint, and a second
  // whole-ruleset round trip to replace it.
  describe('the launch order', () => {
    it('loads the ruleset in the persisted language, with no flash of English', async () => {
      vi.mocked(ipc.readSettings).mockResolvedValue(persisted({ lang: 'de' }));

      await store.init();

      expect(vi.mocked(ipc.loadRuleset).mock.calls).toEqual([['de']]);
      expect(store.lang).toBe('de');
      // One load, not two: the English ruleset is never fetched, so there is no
      // first paint to flash and nothing to throw away.
      expect(store.ruleset?.ruleset.id).toBe('test-de');
    });

    it('reads the settings before it asks for the ruleset at all', async () => {
      const order: string[] = [];
      vi.mocked(ipc.readSettings).mockImplementation(async () => {
        order.push('settings');
        return persisted({ lang: 'de' });
      });
      vi.mocked(ipc.loadRuleset).mockImplementation(async (lang: string) => {
        order.push(`ruleset:${lang}`);
        return localized(lang);
      });

      await store.init();

      expect(order).toEqual(['settings', 'ruleset:de']);
    });

    // Serializing the launch exposed something the concurrent version hid. A
    // ruleset load clears `error` on its way in, to retire whatever a previous
    // failed load had said — but `error` is ONE shared banner that the menu build,
    // the file operations and validation all write to. While `init()` started the
    // load synchronously, that clear happened before anything else could raise
    // anything; with the settings read in front of it, the clear lands a microtask
    // later and wipes whatever was raised in between. The native menu build is
    // exactly that: `App.svelte` pushes the menu from an `$effect` that runs right
    // after mount, and a window system that refuses reports `AppError::Menu` — which
    // would then vanish with no diagnostic at all.
    //
    // The rule is the one `revalidate` already follows: a succeeding pass may retire
    // ONLY its own error.
    it('retires its own error on a successful load, and nobody else’s', async () => {
      const fromTheMenu = { kind: 'menu', message: 'the window system refused' };
      store.error = fromTheMenu as typeof store.error;

      await store.init();

      expect(store.error).toEqual(fromTheMenu);
      store.error = null;
    });

    it('does retire the error its own previous failure raised', async () => {
      vi.mocked(ipc.loadRuleset).mockRejectedValueOnce({ kind: 'ruleset' });
      await store.init();
      expect(store.error).toEqual({ kind: 'ruleset' });

      await store.init();

      expect(store.error).toBeNull();
    });

    it('still starts when the settings cannot be read at all', async () => {
      // A broken bridge must not stop the app launching: the settings fall back to
      // their own defaults and the ruleset still loads.
      vi.mocked(ipc.readSettings).mockRejectedValue({ kind: 'io' });

      await store.init();

      expect(store.lang).toBe('en');
      expect(vi.mocked(ipc.loadRuleset).mock.calls).toEqual([['en']]);
    });
  });

  describe('reading what was stored', () => {
    it('applies every persisted setting', async () => {
      vi.mocked(ipc.readSettings).mockResolvedValue({
        default_saga_year: 1230,
        lang: 'de',
        theme: 'light',
        validation_mode: 'advisory',
      });

      await store.loadSettings();

      expect(store.lang).toBe('de');
      expect(store.defaultSagaYear).toBe(1230);
      expect(store.theme).toBe('light');
      expect(store.mode).toBe('advisory');
    });

    it('keeps its own default for a setting that was never chosen', async () => {
      await store.loadSettings();

      // `auto` is the theme default and it lives HERE — not in `arm-rules`, which is
      // a rules engine and has no business knowing what a palette is, and not in
      // `arm-app`, which would only be a second answer to drift from this one.
      expect(store.theme).toBe('auto');
      expect(store.lang).toBe('en');
      expect(store.mode).toBe('enforced');
    });

    it('ignores a stored value this build cannot use', async () => {
      // The settings file is hand-editable, so it is the trust boundary. A theme of
      // `purple` has no palette and a language of `fr` has no bundle; taking either
      // at face value would paint an unstyled app or an untranslated one.
      vi.mocked(ipc.readSettings).mockResolvedValue({
        default_saga_year: 1220,
        lang: 'fr',
        theme: 'purple',
        validation_mode: null,
      });

      await store.loadSettings();

      expect(store.theme).toBe('auto');
      expect(store.lang).toBe('en');
    });
  });

  describe('writing what was chosen', () => {
    it('persists the theme, naming only the theme', async () => {
      await store.setTheme('light');

      expect(store.theme).toBe('light');
      expect(vi.mocked(ipc.writeSettings)).toHaveBeenCalledWith({ theme: 'light' });
    });

    it('persists the language, naming only the language', async () => {
      await store.setLang('de');

      expect(store.lang).toBe('de');
      expect(vi.mocked(ipc.writeSettings)).toHaveBeenCalledWith({ lang: 'de' });
    });

    it('persists the validation mode, naming only the validation mode', async () => {
      await store.setMode('silent');

      expect(store.mode).toBe('silent');
      expect(vi.mocked(ipc.writeSettings)).toHaveBeenCalledWith({ validation_mode: 'silent' });
    });

    it('dirties no document — none of the four is character state', async () => {
      const before = store.dirty;
      await store.setTheme('dark');
      await store.setMode('advisory');
      expect(store.dirty).toBe(before);
    });

    it('surfaces a failed write rather than letting the choice silently not stick', async () => {
      vi.mocked(ipc.writeSettings).mockRejectedValue({ kind: 'io' });

      await store.setTheme('light');
      await vi.runAllTimersAsync();

      expect(store.error).toEqual({ kind: 'io' });
      store.error = null;
    });
  });

  // The settings surface itself. C3a left `settings` permanently disabled with an
  // explicit no-op arm, because a menu item that opens nothing is worse than a
  // greyed-out one; it has a screen now, so the placeholder is over.
  describe('the settings screen', () => {
    it('is reachable from every view, now that it has a screen to open', () => {
      for (const view of ['start', 'editor', 'wizard'] as const) {
        store.view = view;
        expect(store.documentActionEnabled('settings'), view).toBe(true);
      }
    });

    it('opens from the menu action and closes again', async () => {
      expect(store.settingsOpen).toBe(false);

      await store.runDocumentAction('settings');
      expect(store.settingsOpen).toBe(true);

      store.closeSettings();
      expect(store.settingsOpen).toBe(false);
    });

    it('starts closed, so a launch lands on the app and not on its preferences', () => {
      store.closeSettings();
      expect(store.settingsOpen).toBe(false);
    });
  });
});

// C3a: the native menu can reach neither of the two mechanisms that gated the
// document actions before it. The toolbar buttons carried `disabled`, and
// `App.svelte`'s window-level keydown handler repeated the same check by hand
// (`inert` does not reach a window listener) — a menu item obeys neither. So
// "may this action run right now?" moved into the store as ONE predicate, and
// the shortcut handler, the menu's enabled state and the dispatcher below all
// read that single answer instead of each carrying a copy.
describe('the document-action gate', () => {
  /**
   * Hold the store in its busy state — a native Save dialog left open — and
   * return the release. Save As is the cleanest way in: it goes straight to
   * the write path, with no unsaved-changes prompt in front of it.
   */
  function holdBusy(): () => Promise<void> {
    let release: (value: string | null) => void = () => {};
    vi.mocked(ipc.saveEntity).mockReturnValue(
      new Promise((resolve) => {
        release = resolve;
      }),
    );
    const saving = store.saveAs();
    return async () => {
      release(null);
      await saving;
    };
  }

  beforeEach(() => {
    store.view = 'editor';
  });

  it('offers every document action while a character is being edited', () => {
    for (const action of ['new', 'open', 'save', 'saveAs', 'export'] as const) {
      expect(store.documentActionEnabled(action), action).toBe(true);
    }
  });

  // MOVED HERE BY C3c, from `App.test.ts`'s "keeps the banner and the document
  // controls", which asserted a `save-button` was rendered on the wizard screen.
  // With the toolbar gone that claim has no element to make itself about: the
  // menu expresses availability as an ENABLED ITEM, which WebDriver and a
  // server-rendered string can both see nothing of. The predicate is the honest
  // subject, and it is the one all three callers read.
  //
  // The claim itself is load-bearing rather than incidental: the unsaved-changes
  // guard (CLAUDE.md) offers the user a chance to SAVE rather than lose the
  // work, and the wizard is a screen they can be holding unsaved work on. A
  // wizard that could not save would make that offer a lie.
  it('offers the document-writing actions in the guided wizard too', () => {
    store.view = 'wizard';

    for (const action of ['new', 'open', 'save', 'saveAs', 'export'] as const) {
      expect(store.documentActionEnabled(action), action).toBe(true);
    }
  });

  // Save/Save As/Export write the document being edited, and the startup screen
  // has none — only the placeholder entity. The menu bar is on screen there like
  // everywhere else and the shortcuts are live, so both have to say no
  // themselves; there is no longer a toolbar whose simple absence said it for
  // them.
  it('withholds the document-writing actions on the startup screen', () => {
    store.view = 'start';

    expect(store.documentActionEnabled('new')).toBe(true);
    expect(store.documentActionEnabled('open')).toBe(true);
    for (const action of ['save', 'saveAs', 'export'] as const) {
      expect(store.documentActionEnabled(action), action).toBe(false);
    }
  });

  // UPDATED BY C4, from "keeps Settings inert until it has a screen to open". That
  // claim was deliberately about a placeholder: C3a shipped the item disabled
  // everywhere because a menu entry that opens nothing is worse than a greyed-out
  // one. C4 gave it a screen, so the placeholder is over and the claim inverts —
  // and unlike Save/Save As/Export the preferences are not about a document, so
  // they are offered on the startup screen too.
  it('offers Settings on every screen, document or not', () => {
    for (const view of ['start', 'editor', 'wizard'] as const) {
      store.view = view;
      expect(store.documentActionEnabled('settings'), view).toBe(true);
    }
  });

  it('withholds every action while a native file dialog is open', async () => {
    const release = holdBusy();
    expect(store.busy).toBe(true);

    for (const action of ['new', 'open', 'save', 'saveAs', 'export', 'settings'] as const) {
      expect(store.documentActionEnabled(action), action).toBe(false);
    }

    await release();
  });

  it('runs the very store method that owns each file operation', async () => {
    const calls: string[] = [];
    const spies = [
      vi.spyOn(store, 'newDocument').mockImplementation(async () => void calls.push('new')),
      vi.spyOn(store, 'open').mockImplementation(async () => {
        calls.push('open');
        return true;
      }),
      vi.spyOn(store, 'save').mockImplementation(async () => void calls.push('save')),
      vi.spyOn(store, 'saveAs').mockImplementation(async () => void calls.push('saveAs')),
      vi
        .spyOn(store, 'exportMarkdown')
        .mockImplementation(async () => void calls.push('exportMarkdown')),
    ];

    for (const action of ['new', 'open', 'save', 'saveAs', 'export'] as const) {
      await store.runDocumentAction(action);
    }

    expect(calls).toEqual(['new', 'open', 'save', 'saveAs', 'exportMarkdown']);
    for (const spy of spies) spy.mockRestore();
  });

  it('runs nothing the gate has withheld', async () => {
    store.view = 'start';
    const save = vi.spyOn(store, 'save').mockResolvedValue(undefined);

    // `settings` used to stand beside `save` here, as the other withheld action.
    // C4 gave it a screen, so the gate no longer withholds it and it has moved to
    // the "reachable from every view" claim in the persisted-settings block.
    await store.runDocumentAction('save');

    expect(save).not.toHaveBeenCalled();
    save.mockRestore();
  });

  it('reports the same answer to the menu as it gives the dispatcher', () => {
    for (const view of ['start', 'editor'] as const) {
      store.view = view;
      expect(store.menuFlags()).toEqual({
        new: store.documentActionEnabled('new'),
        open: store.documentActionEnabled('open'),
        save: store.documentActionEnabled('save'),
        saveAs: store.documentActionEnabled('saveAs'),
        export: store.documentActionEnabled('export'),
        settings: store.documentActionEnabled('settings'),
      });
    }
  });
});

// --- Choosing a language is not a document edit (S1, full-audit round 2) -----
//
// `setLang` reloads the ruleset because rules display text is per-language. That
// reload used to re-stamp `entity.ruleset` with the freshly loaded identity,
// which is a no-op in the ordinary case and destructive in exactly the one case
// the engine's `ruleset_mismatch` warning exists to report: a save written under
// a different ruleset. `ssr` project — plain store state, no component, no
// `$effect`.
describe('setLang leaves the document provenance alone', () => {
  const savedProvenance = { id: 'house-rules', version: '3' };
  let langBefore: typeof store.lang;

  /** A localized ruleset payload announcing `id`/`version` as its identity. */
  function loadedRuleset(id: string, version: string): LocalizedRuleset {
    return {
      ruleset: {
        id,
        version,
        point_items: {},
        type_profiles: {},
        abilities: {},
        magnitude_points: { free: 0, minor: 1, major: 3 },
        ability_category_order: ['general'],
        art_type_order: ['technique', 'form'],
      },
      i18n: {},
    } as LocalizedRuleset;
  }

  beforeEach(() => {
    langBefore = store.lang;
    vi.mocked(ipc.loadEntity).mockReset();
    vi.mocked(ipc.loadRuleset).mockReset();
    store.view = 'start';
  });

  afterEach(() => {
    store.lang = langBefore;
  });

  it('keeps the saved ruleset identity, and the document clean, across a language switch', async () => {
    // A save written against a house-ruled `rules/` directory beside the binary
    // — a supported layout — opened under the shipped catalogue. The mismatch is
    // the point: it is what the Validation panel warns about.
    const saved: Entity = {
      schema_version: 11,
      ruleset: { ...savedProvenance },
      entity_kind: 'character',
      type_id: 'companion',
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
    };
    vi.mocked(ipc.loadEntity).mockResolvedValue({
      path: '/tmp/magus.armc',
      entity: saved,
      migrated_aging_characteristics: [],
    });
    const opening = store.open();
    if (store.discardPromptOpen) store.resolveDiscardPrompt(true);
    await opening;

    expect(store.entity.ruleset).toEqual(savedProvenance);
    expect(store.dirty).toBe(false);

    vi.mocked(ipc.loadRuleset).mockResolvedValue(loadedRuleset('arm5-core', '2024.1'));
    await store.setLang(store.lang === 'de' ? 'en' : 'de');

    // The language changed; what the character was built against did not. A
    // re-stamp here would erase the `ruleset_mismatch` warning with no user
    // action addressing it, write a false provenance on the next save, and
    // dirty a document nobody edited.
    expect(store.entity.ruleset).toEqual(savedProvenance);
    expect(store.dirty).toBe(false);
    // The reload itself still happened — the rules text really is per-language.
    expect(store.ruleset?.ruleset.id).toBe('arm5-core');
  });
});
