import { describe, expect, it, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import RadioGroup, { type RadioGroupOption } from './RadioGroup';

const options: RadioGroupOption<string>[] = [
  { key: 'fast', label: 'Fast', desc: 'Lower latency' },
  { key: 'anonymous', label: 'Anonymous', desc: 'Higher privacy' },
  { key: 'locked', label: 'Locked', disabled: true },
];

describe('RadioGroup', () => {
  it('renders one radio per option with its label', () => {
    render(<RadioGroup options={options} onChange={vi.fn()} />);

    const radios = screen.getAllByRole('radio');
    expect(radios).toHaveLength(3);
    expect(screen.getByText('Fast')).toBeInTheDocument();
    expect(screen.getByText('Anonymous')).toBeInTheDocument();
  });

  it('renders the root label', () => {
    render(
      <RadioGroup options={options} onChange={vi.fn()} rootLabel="Mode" />,
    );

    expect(screen.getByText('Mode')).toBeInTheDocument();
  });

  it('selects the first option by default', () => {
    render(<RadioGroup options={options} onChange={vi.fn()} />);

    expect(screen.getByRole('radio', { name: /Fast/ })).toBeChecked();
  });

  it('honours defaultValue', () => {
    render(
      <RadioGroup
        options={options}
        onChange={vi.fn()}
        defaultValue="anonymous"
      />,
    );

    expect(screen.getByRole('radio', { name: /Anonymous/ })).toBeChecked();
  });

  it('calls onChange with the selected key and updates selection', async () => {
    const onChange = vi.fn();
    render(<RadioGroup options={options} onChange={onChange} />);

    await userEvent.click(screen.getByRole('radio', { name: /Anonymous/ }));

    expect(onChange).toHaveBeenCalledExactlyOnceWith('anonymous');
    expect(screen.getByRole('radio', { name: /Anonymous/ })).toBeChecked();
  });

  it('does not select a disabled option', async () => {
    const onChange = vi.fn();
    render(<RadioGroup options={options} onChange={onChange} />);

    await userEvent.click(screen.getByRole('radio', { name: /Locked/ }));

    expect(onChange).not.toHaveBeenCalled();
  });
});
