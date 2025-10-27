// Import Jest DOM matchers - this automatically extends Jest matchers
import '@testing-library/jest-dom';

// Import Jest globals and make them available globally
import * as jestGlobals from '@jest/globals';

// Make Jest globals available in the global scope
declare global {
  const describe: typeof jestGlobals.describe;
  const it: typeof jestGlobals.it;
  const test: typeof jestGlobals.test;
  const expect: typeof jestGlobals.expect;
  const beforeEach: typeof jestGlobals.beforeEach;
  const afterEach: typeof jestGlobals.afterEach;
  const beforeAll: typeof jestGlobals.beforeAll;
  const afterAll: typeof jestGlobals.afterAll;
  const jest: typeof jestGlobals.jest;

  // Make Jest types available globally
  namespace jest {
    type MockedFunction<T extends (...args: any[]) => any> =
      import('jest-mock').MockedFunction<T>;
  }
}

// Extend Jest matchers with Testing Library DOM matchers
declare global {
  namespace jest {
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
}

// Also extend @jest/expect matchers
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
