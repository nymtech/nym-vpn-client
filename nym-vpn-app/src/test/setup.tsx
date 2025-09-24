import '@testing-library/jest-dom';
import React from 'react';
import { TextEncoder, TextDecoder } from 'util';

// Polyfill for TextEncoder/TextDecoder
Object.assign(global, { TextDecoder, TextEncoder });

// Mock Tauri APIs
const mockTauriApi = {
  event: {
    listen: jest.fn(),
    emit: jest.fn(),
    once: jest.fn(),
  },
  window: {
    getCurrent: jest.fn(() => ({
      close: jest.fn(),
      minimize: jest.fn(),
      maximize: jest.fn(),
      unmaximize: jest.fn(),
      show: jest.fn(),
      hide: jest.fn(),
    })),
  },
  core: {
    invoke: jest.fn(),
  },
  path: {
    join: jest.fn(),
    resolve: jest.fn(),
    dirname: jest.fn(),
    basename: jest.fn(),
  },
  fs: {
    readTextFile: jest.fn(),
    writeTextFile: jest.fn(),
    exists: jest.fn(),
  },
};

// Mock all Tauri plugins
jest.mock('@tauri-apps/api', () => mockTauriApi);
jest.mock('@tauri-apps/plugin-autostart', () => ({}));
jest.mock('@tauri-apps/plugin-clipboard-manager', () => ({}));
jest.mock('@tauri-apps/plugin-dialog', () => ({}));
jest.mock('@tauri-apps/plugin-notification', () => ({}));
jest.mock('@tauri-apps/plugin-opener', () => ({}));
jest.mock('@tauri-apps/plugin-os', () => ({
  type: jest.fn(() => 'linux'),
  platform: jest.fn(() => 'linux'),
  version: jest.fn(() => '1.0.0'),
}));
jest.mock('@tauri-apps/plugin-process', () => ({}));
jest.mock('@tauri-apps/plugin-updater', () => ({}));
jest.mock('@tauri-apps/plugin-window-state', () => ({}));

// Mock contexts
jest.mock('../contexts', () => ({
  useMainState: jest.fn(() => ({
    uiTheme: 'light' as const,
  })),
  useDialog: jest.fn(() => ({
    show: jest.fn(),
    hide: jest.fn(),
  })),
}));

// Mock react-router for components that use navigation
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

// Mock lottie animations
jest.mock('@lottiefiles/dotlottie-react', () => ({
  DotLottieReact: ({ children, ...props }: any) => (
    <div {...props}>{children}</div>
  ),
}));

// Mock motion/framer-motion
jest.mock('motion', () => ({
  motion: {
    div: ({ children, ...props }: any) => <div {...props}>{children}</div>,
    span: ({ children, ...props }: any) => <span {...props}>{children}</span>,
    button: ({ children, ...props }: any) => (
      <button {...props}>{children}</button>
    ),
  },
  AnimatePresence: ({ children }: any) => children,
}));

// Global test setup
beforeEach(() => {
  // Clear all mocks before each test
  jest.clearAllMocks();
});

// Suppress console errors during tests unless needed
const originalError = console.error;
beforeAll(() => {
  console.error = (...args: any[]) => {
    if (
      typeof args[0] === 'string' &&
      args[0].includes('Warning: ReactDOM.render is deprecated')
    ) {
      return;
    }
    originalError.call(console, ...args);
  };
});

afterAll(() => {
  console.error = originalError;
});
