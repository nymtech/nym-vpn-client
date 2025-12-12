import { memo } from 'react';
import { AnimatePresence, motion } from 'motion/react';
import clsx from 'clsx';
import { FlagIcon, MsIcon, Skeleton, countryCode } from '../../ui';
import { QuicTag } from '../node';

export type SelectedNodeDisplayProps = {
  countryCode?: countryCode;
  name: string;
  subInfo?: string | null;
  animate?: boolean;
  disabled?: boolean;
  showQuic?: boolean;
  showStreamOptimized?: boolean;
  showFastest?: boolean;
};

export const SelectedNodeDisplay = memo<SelectedNodeDisplayProps>(
  ({
    countryCode,
    name,
    subInfo,
    animate,
    disabled,
    showQuic,
    showStreamOptimized,
    showFastest,
  }) => {
    if (!countryCode && !showFastest) {
      return (
        <div className="flex flex-row items-center gap-3 overflow-hidden w-full">
          <Skeleton className="w-7 h-7" rounded="full" />
          <div className="flex flex-col items-start justify-center gap-1">
            <Skeleton className="w-36 h-4" rounded />
            <Skeleton className="w-36 h-3" rounded />
          </div>
        </div>
      );
    }

    return (
      <div className="flex flex-row items-center gap-3 overflow-hidden w-full">
        {countryCode && <FlagIcon code={countryCode} alt={countryCode} />}
        {showFastest && (
          <MsIcon
            icon="casino"
            className="text-2xl text-baltic-sea dark:text-white"
          />
        )}
        <div className={clsx('flex flex-col items-start truncate')}>
          <div
            className={clsx([
              'text-base truncate',
              disabled && 'cursor-default',
            ])}
          >
            {name}
          </div>
          {animate ? (
            <AnimatePresence>
              {subInfo && (
                <motion.div
                  initial={{ opacity: 0, x: '-1rem' }}
                  exit={{ opacity: 0, x: '1rem' }}
                  animate={{ opacity: 1, x: 0 }}
                  transition={{ duration: 0.2, ease: 'easeOut' }}
                  className="text-sm text-iron dark:text-bombay truncate"
                >
                  {subInfo}
                </motion.div>
              )}
            </AnimatePresence>
          ) : (
            <>
              {subInfo && (
                <div className="text-sm text-iron dark:text-bombay truncate">
                  {subInfo}
                </div>
              )}
            </>
          )}
        </div>
        {(showQuic || showStreamOptimized) && (
          <div className="flex items-center justify-end gap-3 flex-1 mr-1">
            {showStreamOptimized && (
              <MsIcon icon="smart_display" className="text-cornflower" />
            )}
            {showQuic && <QuicTag />}
          </div>
        )}
      </div>
    );
  },
);

SelectedNodeDisplay.displayName = 'SelectedNodeDisplay';
