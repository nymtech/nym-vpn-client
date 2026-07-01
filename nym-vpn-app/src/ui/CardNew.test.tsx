import { describe, expect, it, vi } from 'vitest';
import { screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';

import { renderWithProviders } from '../test/harness';
import {
  CardDataRow,
  CardDivider,
  CardHeaderSwitch,
  CardNew,
  CardNewBody,
  CardNewFooter,
  CardNewHeader,
} from './CardNew';

// `CardNew` imports `ButtonIcon`, which pulls the `./index` barrel and loads
// `DaemonDot` reading `window._APP.devMode` at module-load time; `vi.hoisted`
// runs before the static import below so the global exists in time.
vi.hoisted(() => {
  (globalThis as unknown as { _APP: { devMode: boolean } })._APP = {
    devMode: true,
  };
});

vi.mock('@tauri-apps/plugin-os', () => ({
  type: () => 'linux',
}));

describe('CardNew', () => {
  it('renders its children', () => {
    renderWithProviders(
      <CardNew>
        <span>card content</span>
      </CardNew>,
    );

    expect(screen.getByText('card content')).toBeInTheDocument();
  });

  it('renders header, body and footer subcomponents', () => {
    renderWithProviders(
      <CardNew>
        <CardNewHeader>header</CardNewHeader>
        <CardNewBody>body</CardNewBody>
        <CardNewFooter>footer</CardNewFooter>
      </CardNew>,
    );

    expect(screen.getByText('header')).toBeInTheDocument();
    expect(screen.getByText('body')).toBeInTheDocument();
    expect(screen.getByText('footer')).toBeInTheDocument();
  });

  it('renders a data row with its label and children', () => {
    renderWithProviders(
      <CardDataRow label="Address">
        <span>abc123</span>
      </CardDataRow>,
    );

    expect(screen.getByText('Address')).toBeInTheDocument();
    expect(screen.getByText('abc123')).toBeInTheDocument();
  });

  it('renders a divider element', () => {
    const { container } = renderWithProviders(<CardDivider />);

    expect(container.firstChild).toBeInTheDocument();
  });
});

describe('CardHeaderSwitch', () => {
  it('renders the header and optional subheader', () => {
    renderWithProviders(
      <CardHeaderSwitch
        header="Enable feature"
        subheader="Recommended"
        checked={false}
        onClick={vi.fn()}
      />,
    );

    expect(screen.getByText('Enable feature')).toBeInTheDocument();
    expect(screen.getByText('Recommended')).toBeInTheDocument();
  });

  it('calls onClick when the row is clicked', async () => {
    const user = userEvent.setup();
    const onClick = vi.fn();

    renderWithProviders(
      <CardHeaderSwitch header="Toggle" checked={false} onClick={onClick} />,
    );

    await user.click(screen.getByRole('button', { name: /Toggle/ }));

    expect(onClick).toHaveBeenCalled();
  });

  it('reflects the checked state on the switch', () => {
    renderWithProviders(
      <CardHeaderSwitch header="On" checked onClick={vi.fn()} />,
    );

    expect(screen.getByRole('switch')).toBeChecked();
  });
});
