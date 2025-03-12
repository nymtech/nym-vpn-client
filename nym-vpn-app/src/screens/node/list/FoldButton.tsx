import { AccordionTriggerProps } from '@radix-ui/react-accordion';
import { Button } from '@headlessui/react';
import clsx from 'clsx';
import { MsIcon } from '../../../ui';

type FoldButtonProps = {
  'data-state'?: 'open' | 'closed';
} & AccordionTriggerProps;

const FoldButton = (props: FoldButtonProps) => (
  <Button
    className={clsx(
      'w-16 h-full flex justify-center items-center mr-3',
      'border-l-1 border-bombay dark:border-dim-gray',
      'text-baltic-sea/80 dark:text-white/80',
      'hover:text-baltic-sea dark:hover:text-white',
      'focus:outline-none',
    )}
    {...props}
  >
    <MsIcon
      icon={
        props['data-state'] === 'open' ? 'arrow_drop_up' : 'arrow_drop_down'
      }
    />
  </Button>
);

export default FoldButton;
