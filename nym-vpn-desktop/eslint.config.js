// @ts-check

import js from '@eslint/js';
import tseslint from 'typescript-eslint';
import reactRecommended from 'eslint-plugin-react/configs/recommended.js';
import jsxRuntime from 'eslint-plugin-react/configs/jsx-runtime.js';
import reactHooks from 'eslint-plugin-react-hooks';
import jsxA11y from 'eslint-plugin-jsx-a11y';
import globals from 'globals';

// TODO eslint v8 is still needed as v9 is not yet supported
// by typescript-eslint and other plugins

// TODO some plugins do not yet support eslint flat config
// * eslint-plugin-import + eslint-import-resolver-typescript
// for the following plugins we can work around
// * eslint-plugin-jsx-a11y
// * eslint-plugin-react-hooks

export default tseslint.config(
  js.configs.recommended,
  ...tseslint.configs.recommended,
  reactRecommended,
  jsxRuntime,
  {
    files: ['**/*.{js,jsx,mjs,cjs,ts,tsx}'],
    ignores: ['dist/', 'public/', 'src-tauri/'],
    plugins: {
      'react-hooks': reactHooks,
      'jsx-a11y': jsxA11y,
    },
    languageOptions: {
      ...reactRecommended.languageOptions,
      globals: {
        ...globals.browser,
      },
    },
    rules: {
      ...reactHooks.configs.recommended.rules,
      ...jsxA11y.configs.recommended.rules,
    },
  },
);

/* const legacyConfig = {
  extends: [
    'eslint:recommended',
    'plugin:@typescript-eslint/recommended',
    'plugin:react/recommended',
    'plugin:react/jsx-runtime',
    'plugin:jsx-a11y/recommended',
    'plugin:react-hooks/recommended',
    'plugin:import/typescript',
    'prettier',
  ],
  parser: '@typescript-eslint/parser',
  parserOptions: {
    project: './tsconfig.json',
  },
  plugins: ['@typescript-eslint', 'import'],
  root: true,
  settings: {
    react: {
      version: 'detect',
    },
    // https://github.com/import-js/eslint-import-resolver-typescript#configuration
    'import/parsers': {
      '@typescript-eslint/parser': ['.ts', '.tsx'],
    },
    'import/resolver': {
      typescript: {
        alwaysTryTypes: true, // always try to resolve types under `<root>@types` directory even it doesn't contain any source code, like `@types/unist`
        project: 'tsconfig.json',
      },
    },
  },
  rules: {
    'sort-imports': [
      'error',
      {
        ignoreDeclarationSort: true,
        allowSeparatedGroups: true,
      },
    ],
    'import/first': 'error',
    'import/order': [
      'error',
      {
        groups: ['builtin', 'external', 'parent', 'sibling', 'index'],
      },
    ],
    'import/extensions': ['error', 'never', { json: 'always', svg: 'always' }],
  },
}; */
