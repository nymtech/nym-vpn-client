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
        'w-16 h-full flex justify-center items-center mr-3',
        'border-l-1 border-bombay dark:border-iron',
        'text-baltic-sea/80 dark:text-white/80',
        'hover:text-baltic-sea dark:hover:text-white',
        'focus:outline-none',
      )}
      data-testid="fold-button"
      {...html}
    >
      <MsIcon
        icon={state.open ? 'arrow_drop_up' : 'arrow_drop_down'}
        data-testid="fold-button-icon"
      />
    </Button>
  );
};

export default FoldButton;
