import * as React from 'react';
import { Collapsible } from '@base-ui-components/react';
import { Button } from '@headlessui/react';
import clsx from 'clsx';
import { MsIcon } from '../../../ui';

type FoldButtonProps = {
  html: React.HTMLAttributes<unknown>;
  state: Collapsible.Root.State;
};

const FoldButton = ({ html, state }: FoldButtonProps) => {
  return (
    <Button
      className={clsx(
        'group/fold-button',
        'flex h-12 w-12 items-center justify-center rounded-full',
        'text-baltic-sea/80 dark:text-white/80',
        'focus:outline-none',
      )}
      {...html}
    >
      <MsIcon
        icon="keyboard_arrow_down"
        className={clsx(
          'leading-none transition-transform duration-150',
          state.open && 'rotate-180',
          'group-hover/fold-button:text-primary',
        )}
      />
    </Button>
  );
};

export default FoldButton;
