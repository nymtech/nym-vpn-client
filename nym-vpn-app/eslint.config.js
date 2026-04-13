import globals from 'globals';
import { globalIgnores } from 'eslint/config';
import eslint from '@eslint/js';
import tseslint from 'typescript-eslint';
import reactPlugin from 'eslint-plugin-react';
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
  reactPlugin.configs.flat.recommended,
  reactHooks.configs.flat.recommended,
  {
    languageOptions: {
      parserOptions: {
        ecmaFeatures: { jsx: true },
      },
      globals: globals.browser,
    },
    settings: {
      react: {
        version: 'detect',
      },
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
      'react/react-in-jsx-scope': 0,
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
      'react/prop-types': 0,
    },
  },
  prettierConfig,
];
