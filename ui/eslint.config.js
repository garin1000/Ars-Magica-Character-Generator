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
  },
  {
    // WebdriverIO config + specs run under Node with Mocha globals.
    files: ['e2e/**/*.js'],
    languageOptions: {
      globals: { ...globals.node, ...globals.mocha },
    },
  },
);
