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
    files: ['**/*.svelte'],
    languageOptions: {
      parserOptions: {
        parser: ts.parser,
      },
    },
    // TypeScript (svelte-check) already flags genuinely undefined references, and
    // `no-undef` cannot see the type parameter a `<script generics="T">` block
    // introduces — so it false-positives on generic components. Defer to TS here.
    rules: {
      'no-undef': 'off',
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
