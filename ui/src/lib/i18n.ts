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

/**
 * Resolves a message key against a bundle, falling back to the key itself.
 *
 * **The `errors` array is load-bearing, not decoration.** `formatPattern` has two
 * modes: called with two arguments it THROWS a `ReferenceError` on any
 * resolution error (a missing variable, an unknown term), and called with an
 * errors array it collects them and returns its best-effort partial instead.
 * Every `store.t()` call site in the app is unguarded and many sit inside a
 * `$derived`, so the throwing form meant one message interpolating a variable
 * its caller had omitted would take down the entire render — a blank window
 * because a *label* could not be built. Collecting is strictly better: the
 * sentence still reads, missing its one slot, and the user keeps their
 * character on screen. (Round-1 audit; no reachable trigger was found in the
 * shipped data, so this is defence in depth.)
 *
 * The errors are deliberately not reported anywhere: the only ones reachable
 * would be authoring mistakes in the repo's own `.ftl`, which the i18n tests
 * catch at build time, and a desktop binary launched from a menu has no console
 * for a runtime warning to reach.
 */
export function translate(bundle: FluentBundle, key: string, args?: TranslateArgs): string {
  const message = bundle.getMessage(key);
  if (!message?.value) return key;
  const errors: Error[] = [];
  return bundle.formatPattern(message.value, args, errors);
}
