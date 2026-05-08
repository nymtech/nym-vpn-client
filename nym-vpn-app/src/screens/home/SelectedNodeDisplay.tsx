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
        <div className="flex w-full flex-row items-center gap-3 overflow-hidden">
          <Skeleton className="h-7 w-7" rounded="full" />
          <div className="flex flex-col items-start justify-center gap-1">
            <Skeleton className="h-4 w-36" rounded />
            <Skeleton className="h-3 w-36" rounded />
          </div>
        </div>
      );
    }

    return (
      <div className="flex w-full flex-row items-center gap-3 overflow-hidden">
        {countryCode && <FlagIcon code={countryCode} alt={countryCode} />}
        {showFastest && (
          <MsIcon icon="casino" className="text-text-primary text-2xl" />
        )}
        <div className={clsx('flex flex-col items-start truncate')}>
          <div
            className={clsx([
              'truncate text-base',
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
                  className="text-text-secondary truncate text-sm"
                >
                  {subInfo}
                </motion.div>
              )}
            </AnimatePresence>
          ) : (
            <>
              {subInfo && (
                <div className="text-text-secondary truncate text-sm">
                  {subInfo}
                </div>
              )}
            </>
          )}
        </div>
        {(showQuic || showStreamOptimized) && (
          <div className="mr-1 flex flex-1 items-center justify-end gap-3">
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
