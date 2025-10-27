import '@testing-library/jest-dom';
import React from 'react';
import { TextEncoder, TextDecoder } from 'util';

Object.assign(global, { TextDecoder, TextEncoder });

Object.defineProperty(window, '_APP', {
  value: {
    devMode: false,
    version: '1.0.0',
    platform: 'linux',
  },
  writable: true,
});

jest.mock('@tauri-apps/api/webviewWindow', () => ({
  getCurrentWebviewWindow: jest.fn(() => ({
    close: jest.fn(),
    minimize: jest.fn(),
    maximize: jest.fn(),
    isMaximized: jest.fn(() => Promise.resolve(false)),
    show: jest.fn(),
    hide: jest.fn(),
  })),
}));
jest.mock('@tauri-apps/plugin-os', () => ({
  type: jest.fn(() => 'linux'),
  platform: jest.fn(() => 'linux'),
  version: jest.fn(() => '1.0.0'),
}));

jest.mock('react-router', () => ({
  BrowserRouter: ({ children }: { children: React.ReactNode }) => (
    <>{children}</>
  ),
  useNavigate: () => jest.fn(),
  useLocation: () => ({
    pathname: '/',
    search: '',
    hash: '',
    state: null,
    key: 'default',
  }),
  Link: ({ children, to, ...props }: any) => (
    <a href={to} {...props}>
      {children}
    </a>
  ),
  NavLink: ({ children, to, ...props }: any) => (
    <a href={to} {...props}>
      {children}
    </a>
  ),
}));

jest.mock('@tauri-apps/api/core', () => ({
  invoke: jest.fn(),
}));

jest.mock('react-i18next', () => ({
  useTranslation: () => ({
    t: (key: string, options?: any) => {
      if (key.includes('count') && options?.count) {
        return `${key.replace('-', ' ')} ${options.count}`;
      }
      return key.replace(/[_-]/g, ' ');
    },
  }),
}));

jest.mock('../contexts', () => ({
  useMainState: jest.fn(),
  useMainDispatch: jest.fn(),
  useInAppNotify: jest.fn(),
  useDialog: jest.fn(() => ({
    show: jest.fn(),
    hide: jest.fn(),
  })),
  useNodeListState: jest.fn(() => ({
    reset: jest.fn(),
    entry: {
      expanded: [],
      focused: null,
      search: null,
    },
    exit: {
      expanded: [],
      focused: null,
      search: null,
    },
    setExpanded: jest.fn(),
    addToExpanded: jest.fn(),
    setFocused: jest.fn(),
    setSearch: jest.fn(),
  })),
}));

jest.mock('../hooks', () => ({
  useI18nError: jest.fn(),
  useI18nAccountState: jest.fn(),
  useI18nProgressMsg: jest.fn(),
  useNodesState: jest.fn(),
  useClipboard: jest.fn(),
}));

jest.mock('../router', () => ({
  routes: {
    root: '/',
    entryNodeLocation: '/entry-node',
    exitNodeLocation: '/exit-node',
    addCredential: '/add-credential',
  },
}));

// Global test setup
beforeEach(() => {
  // Clear all mocks before each test
  jest.clearAllMocks();
});
