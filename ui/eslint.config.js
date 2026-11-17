import js from '@eslint/js';
import ts from 'typescript-eslint';
import svelte from 'eslint-plugin-svelte';
import globals from 'globals';

export default ts.config(
  { ignores: ['dist/', 'node_modules/', 'playwright-report/', 'test-results/'] },
  js.configs.recommended,
  ...ts.configs.recommended,
  ...svelte.configs['flat/recommended'],
  {
    languageOptions: {
      globals: { ...globals.browser },
    },
  },
  {
    // eslint-plugin-svelte 3 also routes `.svelte.js`/`.svelte.ts` rune
    // modules through svelte-eslint-parser (not just `.svelte` components),
    // so they need the same TS sub-parser or it chokes on TS syntax.
    files: ['**/*.svelte', '**/*.svelte.ts', '**/*.svelte.js'],
    languageOptions: {
      parserOptions: {
        parser: ts.parser,
      },
    },
    // TypeScript (svelte-check) already flags genuinely undefined references, and
    // `no-undef` cannot see the type parameter a `<script generics="T">` block
    // introduces — so it false-positives on generic components. Defer to TS here.
    //
    // `svelte/prefer-svelte-reactivity` is off too: every `new Map()`/`new Set()`
    // in this codebase is a function-local lookup table, built and consumed (or
    // returned wholesale by a pure/derived computation) within one call — never a
    // plain Map/Set held directly as `$state` and mutated across renders, which is
    // the actual hazard the rule guards against. Swapping to SvelteMap/SvelteSet
    // everywhere would add proxy overhead with no reactivity need anywhere it fires.
    rules: {
      'no-undef': 'off',
      'svelte/prefer-svelte-reactivity': 'off',
    },
  },
  {
    // WebdriverIO config + specs run under Node with Mocha globals.
    files: ['e2e/**/*.js'],
    languageOptions: {
      globals: { ...globals.node, ...globals.mocha },
    },
  },
);
