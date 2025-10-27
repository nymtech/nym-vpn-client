import React, { ReactElement } from 'react';
import { render, RenderOptions, RenderResult } from '@testing-library/react';
import { BrowserRouter } from 'react-router';

const AllTheProviders: React.FC<{ children: React.ReactNode }> = ({
  children,
}) => {
  return <BrowserRouter>{children}</BrowserRouter>;
};

const customRender = (
  ui: ReactElement,
  options?: Omit<RenderOptions, 'wrapper'>,
): RenderResult => render(ui, { wrapper: AllTheProviders, ...options });

export * from '@testing-library/react';
export { customRender as render };

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

export const mockLoginContexts = (overrides = {}) => ({
  useMainState: jest.fn(() => ({
    daemonStatus: 'up',
    state: 'disconnected',
    ...overrides.mainState,
  })),
  useMainDispatch: jest.fn(() => jest.fn()),
  useInAppNotify: jest.fn(() => ({
    push: jest.fn(),
  })),
});

export const mockDialogProps = (overrides = {}) =>
  createMockProps({
    open: false,
    onClose: jest.fn(),
    children: 'Dialog content',
    ...overrides,
  });
