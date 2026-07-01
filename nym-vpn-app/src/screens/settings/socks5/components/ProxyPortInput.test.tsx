import { describe, expect, it, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import ProxyPortInput from './ProxyPortInput';

// `ProxyPortInput` pulls `TextInput`/`Button` from the `../../../../ui` barrel,
// which loads modules reading `window._APP.devMode` and calling the Tauri OS
// plugin's `type()` at module-load time. `vi.hoisted`/`vi.mock` run before the
// imports so the global exists and the plugin is stubbed in time.
vi.hoisted(() => {
  (globalThis as unknown as { _APP: { devMode: boolean } })._APP = {
    devMode: false,
  };
});

vi.mock('@tauri-apps/plugin-os', () => ({
  type: () => 'linux',
  platform: () => 'linux',
}));

describe('ProxyPortInput', () => {
  it('renders the current value and a reset button', () => {
    render(
      <ProxyPortInput
        value="1080"
        defaultValue="1080"
        disabled={false}
        onChange={vi.fn()}
      />,
    );

    expect(screen.getByRole('textbox')).toHaveValue('1080');
    expect(
      screen.getByRole('button', { name: 'Reset to default' }),
    ).toBeInTheDocument();
  });

  it('reports a valid port on change', async () => {
    const onChange = vi.fn();
    render(
      <ProxyPortInput
        value=""
        defaultValue="1080"
        disabled={false}
        onChange={onChange}
      />,
    );

    await userEvent.type(screen.getByRole('textbox'), '8');

    expect(onChange).toHaveBeenCalledExactlyOnceWith('8', true);
    expect(screen.queryByText('Invalid port number')).not.toBeInTheDocument();
  });

  it('reports an invalid port and shows an error message', async () => {
    const onChange = vi.fn();
    render(
      <ProxyPortInput
        value=""
        defaultValue="1080"
        disabled={false}
        onChange={onChange}
      />,
    );

    await userEvent.type(screen.getByRole('textbox'), 'a');

    expect(onChange).toHaveBeenCalledExactlyOnceWith('a', false);
    expect(screen.getByText('Invalid port number')).toBeInTheDocument();
  });

  it('resets to the default value when reset is clicked', async () => {
    const onChange = vi.fn();
    render(
      <ProxyPortInput
        value="9999"
        defaultValue="1080"
        disabled={false}
        onChange={onChange}
      />,
    );

    await userEvent.click(
      screen.getByRole('button', { name: 'Reset to default' }),
    );

    expect(onChange).toHaveBeenCalledExactlyOnceWith('1080', true);
  });

  it('disables the input and reset button when disabled', () => {
    render(
      <ProxyPortInput
        value="1080"
        defaultValue="1080"
        disabled
        onChange={vi.fn()}
      />,
    );

    expect(screen.getByRole('textbox')).toBeDisabled();
    expect(
      screen.getByRole('button', { name: 'Reset to default' }),
    ).toBeDisabled();
  });
});
