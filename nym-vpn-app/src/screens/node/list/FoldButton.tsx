import * as React from 'react';
import { Accordion } from '@base-ui-components/react';
import { Button } from '@headlessui/react';
import clsx from 'clsx';
import { MsIcon } from '../../../ui';

type FoldButtonProps = {
  html: React.HTMLAttributes<unknown>;
  state: Accordion.Item.State;
};

const FoldButton = ({ html, state }: FoldButtonProps) => {
  return (
    <Button
      className={clsx(
        'w-12 h-12 flex justify-center items-center rounded-full',
        'text-baltic-sea/80 dark:text-white/80',
        'hover:text-baltic-sea dark:hover:text-white',
        'hover:bg-mercury dark:hover:bg-mine-shaft',
        'focus:outline-none',
      )}
      {...html}
    >
      <MsIcon
        icon="keyboard_arrow_down"
        className={clsx(
          'transition-transform duration-150 leading-none',
          state.open && 'rotate-180',
        )}
      />
    </Button>
  );
};

export default FoldButton;
