import React, { ReactElement } from 'react';
import { render, RenderOptions, RenderResult } from '@testing-library/react';
import { BrowserRouter } from 'react-router';

// Mock providers that components might need
const AllTheProviders: React.FC<{ children: React.ReactNode }> = ({
  children,
}) => {
  return <BrowserRouter>{children}</BrowserRouter>;
};

// Custom render function that includes providers
const customRender = (
  ui: ReactElement,
  options?: Omit<RenderOptions, 'wrapper'>,
): RenderResult => render(ui, { wrapper: AllTheProviders, ...options });

// Re-export everything
export * from '@testing-library/react';
export { customRender as render };

// Helper functions for common test scenarios
export const createMockProps = <T extends Record<string, any>>(
  overrides: Partial<T> = {},
): T => {
  return {
    onClick: jest.fn(),
    onChange: jest.fn(),
    onSubmit: jest.fn(),
    ...overrides,
  } as T;
};

// Mock data generators
export const mockButtonProps = (overrides = {}) =>
  createMockProps({
    children: 'Test Button',
    onClick: jest.fn(),
    disabled: false,
    color: 'malachite' as const,
    ...overrides,
  });

export const mockTextInputProps = (overrides = {}) =>
  createMockProps({
    value: '',
    onChange: jest.fn(),
    placeholder: 'Test input',
    ...overrides,
  });

export const mockSwitchProps = (overrides = {}) =>
  createMockProps({
    checked: false,
    onChange: jest.fn(),
    ...overrides,
  });

export const mockDialogProps = (overrides = {}) =>
  createMockProps({
    open: false,
    onClose: jest.fn(),
    children: 'Dialog content',
    ...overrides,
  });

// Helper to wait for async operations
export const waitForNextTick = () =>
  new Promise((resolve) => setTimeout(resolve, 0));

// Helper to trigger keyboard events
export const pressKey = (key: string, element?: Element) => {
  const keyboardEvent = new KeyboardEvent('keydown', { key });
  (element || document).dispatchEvent(keyboardEvent);
};

// Helper to check if element has specific classes
export const hasClasses = (element: Element, classNames: string[]): boolean => {
  return classNames.every((className) => element.classList.contains(className));
};
