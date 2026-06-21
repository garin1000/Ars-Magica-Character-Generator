// Fluent integration for UI chrome. The `.ftl` files live in the repo-level
// `locales/` directory and are bundled by Vite as raw text. Rules display text
// is NOT handled here — it arrives from the backend via `load_ruleset`.

import { FluentBundle, FluentResource } from '@fluent/bundle';

export const AVAILABLE_LANGS = ['en', 'de'] as const;
export type Lang = (typeof AVAILABLE_LANGS)[number];

// Eagerly import every locale's main.ftl as raw text, keyed by file path.
const ftlSources = import.meta.glob('../../../locales/*/main.ftl', {
  query: '?raw',
  import: 'default',
  eager: true,
}) as Record<string, string>;

function sourceForLang(lang: Lang): string {
  const entry = Object.entries(ftlSources).find(([path]) => path.includes(`/locales/${lang}/`));
  if (!entry) throw new Error(`missing locale: ${lang}`);
  return entry[1];
}

/** Builds a Fluent bundle for the given language. */
export function buildBundle(lang: Lang): FluentBundle {
  const bundle = new FluentBundle(lang);
  bundle.addResource(new FluentResource(sourceForLang(lang)));
  return bundle;
}

export type TranslateArgs = Record<string, string | number>;

/** Resolves a message key against a bundle, falling back to the key itself. */
export function translate(bundle: FluentBundle, key: string, args?: TranslateArgs): string {
  const message = bundle.getMessage(key);
  if (!message?.value) return key;
  return bundle.formatPattern(message.value, args);
}
