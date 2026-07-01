import { afterEach, describe, expect, it, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import { type } from '@tauri-apps/plugin-os';
import Spinner from './Spinner';

// `Spinner` calls `type()` from the Tauri OS plugin, which is unavailable in
// jsdom; mock it so the platform-dependent border branch can be exercised.
vi.mock('@tauri-apps/plugin-os', () => ({ type: vi.fn(() => 'windows') }));

const mockType = vi.mocked(type);

describe('Spinner', () => {
  afterEach(() => {
    mockType.mockReturnValue('windows');
  });

  it('renders the loader element with its testid', () => {
    render(<Spinner />);
    const spinner = screen.getByTestId('button-spinner');
    expect(spinner).toBeInTheDocument();
    expect(spinner).toHaveClass('loader');
  });

  it('adds a thicker border on non-linux platforms', () => {
    mockType.mockReturnValue('windows');
    render(<Spinner />);
    expect(screen.getByTestId('button-spinner')).toHaveClass('border-4');
  });

  it('omits the thick border on linux', () => {
    mockType.mockReturnValue('linux');
    render(<Spinner />);
    expect(screen.getByTestId('button-spinner')).not.toHaveClass('border-4');
  });

  it('forwards a custom className', () => {
    render(<Spinner className="custom-spinner" />);
    expect(screen.getByTestId('button-spinner')).toHaveClass('custom-spinner');
  });
});
