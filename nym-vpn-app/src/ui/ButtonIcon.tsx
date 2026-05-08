import { useTransition } from 'react';
import { AnimatePresence, motion } from 'motion/react';
import { Button as HuButton } from '@headlessui/react';
import clsx from 'clsx';
import { Button } from '@base-ui/react';
import { sleep } from '../util';
import { MsIcon } from './index';

export type ButtonIconNewProps = {
  onClick: () => void;
  icon: string;
  size?: 'small' | 'base';
  className?: string;
  initialAnimation?: boolean;
  noDefaultSize?: boolean;
  clickFeedback?: boolean;
};

export function ButtonIconNew({
  onClick,
  icon,
  className,
  size = 'base',
  initialAnimation = false,
  noDefaultSize = false,
  clickFeedback = false,
}: ButtonIconNewProps) {
  const [isClicked, click] = useTransition();

  const clickAnim = () => {
    click(async () => {
      await sleep(500);
    });
  };
  return (
    <Button
      className={clsx([
        'flex items-center justify-center rounded-full transition-colors',
        'text-iron hover:text-baltic-sea',
        'dark:text-bombay dark:hover:text-white',
        !noDefaultSize && 'hover:bg-baltic-sea/20 dark:hover:bg-baltic-sea',
        !noDefaultSize && 'h-10 w-10',
        className && className,
      ])}
      onClick={() => {
        if (clickFeedback) {
          clickAnim();
        }
        onClick();
      }}
    >
      <AnimatePresence mode="wait" initial={initialAnimation}>
        {isClicked ? (
          <motion.div
            className="flex"
            initial={{ opacity: 0, scale: 0 }}
            animate={{ opacity: 1, scale: 1 }}
            transition={{
              duration: 0.15,
              scale: { type: 'spring', visualDuration: 0.2, bounce: 0.5 },
            }}
          >
            <MsIcon
              icon="check"
              className={clsx([
                'text-primary leading-none',
                size === 'small' && 'text-2xl',
                size === 'base' && 'text-3xl',
              ])}
            />
          </motion.div>
        ) : (
          <motion.span
            key={icon}
            initial={{ opacity: 0, rotate: 90 }}
            animate={{ opacity: 1, rotate: 0 }}
            exit={{ opacity: 0, rotate: -90 }}
            transition={{ duration: 0.1 }}
            className={clsx([
              'h-[1em] w-[1em] leading-none',
              'font-icon inline-block select-none rtl:-scale-x-100',
              !noDefaultSize && size === 'small' && 'text-2xl',
              !noDefaultSize && size === 'base' && 'text-3xl',
            ])}
          >
            {icon}
          </motion.span>
        )}
      </AnimatePresence>
    </Button>
  );
}

export type ButtonIconProps = {
  icon: string;
  color?: 'malachite' | 'chalk';
  clickedIcon?: string;
  onClick: () => void;
  clickFeedback?: boolean;
  disabled?: boolean;
  className?: string;
  iconClassName?: string;
  clickedIconClassName?: string;
  clickDuration?: number;
  noDefaultSize?: boolean;
  'data-testid'?: string;
};

function ButtonIcon({
  onClick,
  icon,
  color = 'malachite',
  clickedIcon = 'check',
  clickFeedback = false,
  disabled,
  className,
  iconClassName,
  clickedIconClassName,
  clickDuration = 500,
  noDefaultSize,
  ...rest
}: ButtonIconProps) {
  const [isClicked, click] = useTransition();
  const testId = rest['data-testid'] || 'button-icon';

  const clickAnim = () => {
    click(async () => {
      await sleep(clickDuration);
    });
  };

  return (
    <HuButton
      className={clsx([
        'flex items-center justify-center rounded-full',
        color === 'malachite' && [
          'text-malachite-moss/80 data-hover:text-malachite-moss',
          'dark:text-malachite/80 data-hover:dark:text-malachite',
        ],
        color === 'chalk' && [
          'text-baltic-sea data-hover:text-baltic-sea/70',
          'dark:text-white data-hover:dark:text-white/80',
        ],
        'focus:outline-hidden',
        'transition data-active:ring-0 data-disabled:opacity-60',
        'cursor-default select-none',
        className && className,
        !noDefaultSize && 'h-10 min-h-10 w-10 min-w-10',
      ])}
      onClick={() => {
        if (clickFeedback) {
          clickAnim();
        }
        onClick();
      }}
      disabled={disabled}
      data-testid={testId}
      data-test-disabled={disabled ? 'true' : 'false'}
      data-test-clicked={isClicked ? 'true' : 'false'}
    >
      {isClicked ? (
        <motion.div
          className="flex"
          initial={{ opacity: 0, scale: 0 }}
          animate={{ opacity: 1, scale: 1 }}
          transition={{
            duration: 0.15,
            scale: { type: 'spring', visualDuration: 0.2, bounce: 0.5 },
          }}
          data-testid={`${testId}-clicked-container`}
        >
          <MsIcon
            icon={clickedIcon}
            className={clsx([
              'text-primary text-2xl',
              !noDefaultSize && 'h-10 min-h-10 w-10 min-w-10',
              clickedIconClassName,
            ])}
            data-testid={`${testId}-clicked-icon`}
          />
        </motion.div>
      ) : (
        <MsIcon
          icon={icon}
          className={clsx([
            'text-2xl',
            !noDefaultSize && 'h-10 min-h-10 w-10 min-w-10',
            iconClassName,
          ])}
          data-testid={`${testId}-icon`}
        />
      )}
    </HuButton>
  );
}

export default ButtonIcon;
