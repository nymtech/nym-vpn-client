// Standard Jest type declarations - leverages @types/jest
import '@testing-library/jest-dom';

// Import Jest globals from @types/jest
import type {
  describe as jestDescribe,
  it as jestIt,
  test as jestTest,
  expect as jestExpect,
  beforeEach as jestBeforeEach,
  afterEach as jestAfterEach,
  beforeAll as jestBeforeAll,
  afterAll as jestAfterAll,
  jest as jestNamespace,
} from '@jest/globals';

// Make Jest globals available
declare global {
  const describe: typeof jestDescribe;
  const it: typeof jestIt;
  const test: typeof jestTest;
  const expect: typeof jestExpect;
  const beforeEach: typeof jestBeforeEach;
  const afterEach: typeof jestAfterEach;
  const beforeAll: typeof jestBeforeAll;
  const afterAll: typeof jestAfterAll;
  const jest: typeof jestNamespace;

  // Make Jest types available globally
  namespace jest {
    type MockedFunction<T extends (...args: any[]) => any> =
      import('jest-mock').MockedFunction<T>;
  }
}

// Module augmentation for Testing Library DOM matchers
declare module '@jest/expect' {
  interface Matchers<R> {
    toBeInTheDocument(): R;
    toHaveTextContent(text: string | RegExp): R;
    toHaveAttribute(attribute: string, value?: string): R;
    toHaveClass(...classes: string[]): R;
    toBeDisabled(): R;
    toBeEnabled(): R;
    toHaveFocus(): R;
    toBeVisible(): R;
    toBeEmptyDOMElement(): R;
    toHaveValue(value: string | string[] | number): R;
    toHaveDisplayValue(value: string | RegExp | (string | RegExp)[]): R;
    toBeChecked(): R;
    toBePartiallyChecked(): R;
    toHaveDescription(text?: string | RegExp): R;
    toHaveErrorMessage(text?: string | RegExp): R;
    toBeInvalid(): R;
    toBeValid(): R;
    toBeRequired(): R;
  }
}
