import { describe, expect, it } from 'vitest';
import { render, screen } from '@testing-library/react';
import ProxyInfo from './ProxyInfo';

describe('ProxyInfo', () => {
  it('renders the given text', () => {
    render(<ProxyInfo text="Add this to your browser's proxy settings" />);

    expect(
      screen.getByText("Add this to your browser's proxy settings"),
    ).toBeInTheDocument();
  });
});
