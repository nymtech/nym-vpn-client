import globals from 'globals';
import eslint from '@eslint/js';
import tseslint from 'typescript-eslint';
import reactPlugin from 'eslint-plugin-react';
import hooksPlugin from 'eslint-plugin-react-hooks';
import prettierConfig from 'eslint-config-prettier';

// TODO add these plugins once support for ESLint 9 is added
// - react-plugin-import https://github.com/import-js/eslint-plugin-import/pull/3018
// - eslint-plugin-deprecation https://github.com/gund/eslint-plugin-deprecation/pull/79

export default [
  {
    files: ['**/*.{js,mjs,cjs,ts,jsx,tsx}'],
    ignores: ['*.config.js', '*.config.ts'],
  },
  eslint.configs.recommended,
  ...tseslint.configs.recommendedTypeChecked,
  ...tseslint.configs.stylisticTypeChecked,
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
  {
    languageOptions: {
      parserOptions: {
        ecmaFeatures: { jsx: true },
        project: true,
      },
      globals: globals.browser,
    },
    plugins: {
      'react-hooks': hooksPlugin,
    },
    settings: {
      react: {
        version: 'detect',
      },
    },
  },
  {
    rules: {
      ...hooksPlugin.configs.recommended.rules,
      'sort-imports': [
        'error',
        {
          ignoreDeclarationSort: true,
          allowSeparatedGroups: true,
        },
      ],
      'react/react-in-jsx-scope': 0,
      '@typescript-eslint/no-floating-promises': 0,
      '@typescript-eslint/prefer-nullish-coalescing': 0,
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
      // TODO enable these rules once ESLint 9 ready
      // 'import/first': 'error',
      // 'import/order': [
      //   'error',
      //   {
      //     groups: ['builtin', 'external', 'parent', 'sibling', 'index'],
      //   },
      // ],
      // 'import/extensions': [
      //   'error',
      //   'never',
      //   { json: 'always', svg: 'always' },
      // ],
      // 'deprecation/deprecation': 'warn',
    },
  },
  prettierConfig,
];
