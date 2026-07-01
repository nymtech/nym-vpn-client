import { describe, expect, it } from 'vitest';
import { render, screen } from '@testing-library/react';
import DataCard, { type DataCardProps } from './DataCard';

const rows: DataCardProps['rows'] = [
  { key: 'a', row: <span>first row</span> },
  { key: 'b', row: <span>second row</span> },
];

describe('DataCard', () => {
  it('renders one list item per valid row', () => {
    render(<DataCard rows={rows} />);

    expect(screen.getByText('first row')).toBeInTheDocument();
    expect(screen.getByText('second row')).toBeInTheDocument();
    expect(screen.getAllByRole('listitem')).toHaveLength(2);
  });

  it('filters out falsy rows', () => {
    render(
      <DataCard
        rows={[{ key: 'a', row: <span>only row</span> }, false, null, '']}
      />,
    );

    expect(screen.getAllByRole('listitem')).toHaveLength(1);
    expect(screen.getByText('only row')).toBeInTheDocument();
  });

  it('renders a footer when provided', () => {
    render(<DataCard rows={rows} footer={<span>footer text</span>} />);

    expect(screen.getByText('footer text')).toBeInTheDocument();
  });

  it('renders no footer node when none is given', () => {
    render(<DataCard rows={rows} />);

    expect(screen.queryByText('footer text')).not.toBeInTheDocument();
  });
});
