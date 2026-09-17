import globals from 'globals';
import { globalIgnores } from 'eslint/config';
import eslint from '@eslint/js';
import tseslint from 'typescript-eslint';
import eslintReact from '@eslint-react/eslint-plugin';
import reactHooks from 'eslint-plugin-react-hooks';
import prettierConfig from 'eslint-config-prettier';
import importPlugin from 'eslint-plugin-import';

export default [
  globalIgnores(['src/types/tauri.ts']),
  {
    files: ['**/*.{js,mjs,cjs,ts,jsx,tsx}'],
    ignores: ['*.config.js', '*.config.ts'],
  },
  eslint.configs.recommended,
  ...tseslint.configs.recommendedTypeChecked,
  ...tseslint.configs.stylisticTypeChecked,
  importPlugin.flatConfigs.recommended,
  importPlugin.flatConfigs.typescript,
  {
    languageOptions: {
      parserOptions: {
        projectService: true,
        tsconfigRootDir: import.meta.dirname,
      },
    },
  },
  {
    files: ['**/*.js'],
    ...tseslint.configs.disableTypeChecked,
  },
  eslintReact.configs['recommended-typescript'],
  {
    // `eslint-plugin-react-hooks` stays the source of truth for hooks rules,
    // so turn off the overlapping `@eslint-react` ones to avoid duplicate reports
    rules: {
      '@eslint-react/error-boundaries': 0,
      '@eslint-react/exhaustive-deps': 0,
      '@eslint-react/purity': 0,
      '@eslint-react/rules-of-hooks': 0,
      '@eslint-react/set-state-in-effect': 0,
      '@eslint-react/set-state-in-render': 0,
      '@eslint-react/static-components': 0,
      '@eslint-react/unsupported-syntax': 0,
      '@eslint-react/use-memo': 0,
    },
  },
  reactHooks.configs.flat.recommended,
  {
    languageOptions: {
      parserOptions: {
        ecmaFeatures: { jsx: true },
      },
      globals: globals.browser,
    },
  },
  {
    rules: {
      // TODO all of these will need to be fixed eventually
      //  disable them for now
      'react-hooks/set-state-in-effect': 0,
      'react-hooks/static-components': 0,
      'react-hooks/preserve-manual-memoization': 0,
      'react-hooks/refs': 0,

      'sort-imports': [
        'error',
        {
          ignoreDeclarationSort: true,
          allowSeparatedGroups: true,
        },
      ],
      'no-empty': 0,
      'import/no-unresolved': 0,
      '@eslint-react/no-nested-component-definitions': 0,
      '@eslint-react/no-array-index-key': 0,
      '@eslint-react/web-api-no-leaked-timeout': 0,
      'import/no-named-as-default': 0,
      '@typescript-eslint/no-floating-promises': 0,
      '@typescript-eslint/prefer-nullish-coalescing': 0,
      '@typescript-eslint/no-deprecated': 'warn',
      // disable this rule as it produces false positives with i18next `t` function
      '@typescript-eslint/restrict-template-expressions': 0,
      '@typescript-eslint/use-unknown-in-catch-callback-variable': 'error',
      '@typescript-eslint/consistent-type-definitions': ['error', 'type'],
      '@typescript-eslint/no-misused-promises': [
        'error',
        {
          checksVoidReturn: false,
        },
      ],
      'import/first': 'error',
      'import/order': [
        'error',
        {
          groups: ['builtin', 'external', 'parent', 'sibling', 'index'],
        },
      ],
      'import/extensions': [
        'error',
        'never',
        { json: 'always', svg: 'always' },
      ],
      'import/no-named-as-default-member': 0,
    },
  },
  prettierConfig,
];
