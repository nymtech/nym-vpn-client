import { describe, expect, it, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import TextInput from './TextInput';

// `TextInput` renders `ButtonIcon`, which pulls `MsIcon` from the `./index`
// barrel; that loads modules reading `window._APP.devMode` and calling the
// Tauri OS plugin's `type()` at module-load time. `vi.hoisted`/`vi.mock` are
// hoisted above the imports so the global exists and the plugin is stubbed
// before that code runs.
vi.hoisted(() => {
  (globalThis as unknown as { _APP: { devMode: boolean } })._APP = {
    devMode: true,
  };
});

vi.mock('@tauri-apps/plugin-os', () => ({
  type: () => 'linux',
}));

describe('TextInput', () => {
  it('renders an input with the given value and placeholder', () => {
    render(
      <TextInput value="hello" onChange={vi.fn()} placeholder="Enter text" />,
    );

    const input = screen.getByPlaceholderText('Enter text');
    expect(input).toHaveValue('hello');
  });

  it('calls onChange with each typed character', async () => {
    const onChange = vi.fn();
    render(<TextInput value="" onChange={onChange} placeholder="Enter text" />);

    await userEvent.type(screen.getByPlaceholderText('Enter text'), 'ab');

    expect(onChange).toHaveBeenCalledTimes(2);
    expect(onChange).toHaveBeenNthCalledWith(1, 'a');
    expect(onChange).toHaveBeenNthCalledWith(2, 'b');
  });

  it('is disabled when the disabled prop is set', async () => {
    const onChange = vi.fn();
    render(
      <TextInput
        value=""
        onChange={onChange}
        placeholder="Enter text"
        disabled
      />,
    );

    const input = screen.getByPlaceholderText('Enter text');
    expect(input).toBeDisabled();
    await userEvent.type(input, 'a');

    expect(onChange).not.toHaveBeenCalled();
  });

  it('renders a clear button when clearable with a value and clears on click', async () => {
    const onChange = vi.fn();
    render(
      <TextInput
        value="text"
        onChange={onChange}
        placeholder="Enter text"
        clearable
      />,
    );

    await userEvent.click(screen.getByTestId('button-icon'));

    expect(onChange).toHaveBeenCalledExactlyOnceWith('');
  });

  it('does not render the clear button when the value is empty', () => {
    render(
      <TextInput
        value=""
        onChange={vi.fn()}
        placeholder="Enter text"
        clearable
      />,
    );

    expect(screen.queryByTestId('button-icon')).not.toBeInTheDocument();
  });

  it('renders a left icon when leftIcon is provided', () => {
    render(
      <TextInput
        value=""
        onChange={vi.fn()}
        placeholder="Enter text"
        leftIcon="search"
      />,
    );

    expect(screen.getByPlaceholderText('Enter text')).toHaveAttribute(
      'data-test-has-left-icon',
      'true',
    );
    expect(screen.getByTestId('icon-search')).toBeInTheDocument();
  });
});
