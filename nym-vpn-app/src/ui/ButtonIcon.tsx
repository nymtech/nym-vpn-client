import { useTransition } from 'react';
import { motion } from 'motion/react';
import { Button as HuButton } from '@headlessui/react';
import clsx from 'clsx';
import { sleep } from '../util';
import { MsIcon } from './index';

export type ButtonIconProps = {
  icon: string;
  clickedIcon?: string;
  onClick: () => void;
  clickFeedback?: boolean;
  disabled?: boolean;
  className?: string;
  iconClassName?: string;
  clickedIconClassName?: string;
  clickDuration?: number;
};

function ButtonIcon({
  onClick,
  icon,
  clickedIcon = 'check',
  clickFeedback = false,
  disabled,
  className,
  iconClassName,
  clickedIconClassName,
  clickDuration = 500,
}: ButtonIconProps) {
  const [isClicked, click] = useTransition();

  const clickAnim = () => {
    click(async () => {
      await sleep(clickDuration);
    });
  };

  return (
    <HuButton
      className={clsx([
        'rounded-full w-10 h-10 min-w-10 min-h-10',
        'text-malachite-moss/80 data-hover:text-malachite-moss',
        'dark:text-malachite/80 data-hover:dark:text-malachite',
        'focus:outline-hidden',
        'transition data-disabled:opacity-60 data-active:ring-0',
        'cursor-default select-none',
        className && className,
      ])}
      onClick={() => {
        if (clickFeedback) {
          clickAnim();
        }
        onClick();
      }}
      disabled={disabled}
    >
      {isClicked ? (
        <motion.div
          initial={{ opacity: 0, scale: 0 }}
          animate={{ opacity: 1, scale: 1 }}
          transition={{
            duration: 0.15,
            scale: { type: 'spring', visualDuration: 0.2, bounce: 0.5 },
          }}
        >
          <MsIcon
            icon={clickedIcon}
            className={clsx([
              'text-2xl w-10 h-10 min-w-10 min-h-10',
              clickedIconClassName,
            ])}
          />
        </motion.div>
      ) : (
        <MsIcon
          icon={icon}
          className={clsx([
            'text-2xl w-10 h-10 min-w-10 min-h-10',
            iconClassName,
          ])}
        />
      )}
    </HuButton>
  );
}

export default ButtonIcon;
