// Typed wrappers around the Tauri command bridge. Every backend call goes
// through here so the rest of the app never touches `invoke` directly.

import { invoke } from '@tauri-apps/api/core';
import type { Entity, LocalizedRuleset, ValidationMode, ValidationResult } from './types';

export function loadRuleset(lang: string): Promise<LocalizedRuleset> {
  return invoke('load_ruleset', { lang });
}

export function validateEntity(entity: Entity, mode: ValidationMode): Promise<ValidationResult> {
  return invoke('validate_entity', { entity, mode });
}

export function saveEntity(entity: Entity): Promise<string | null> {
  return invoke('save_entity', { entity });
}

export function loadEntity(): Promise<Entity | null> {
  return invoke('load_entity');
}
