import { ReactNode } from 'react';
import { motion } from 'motion/react';
import clsx from 'clsx';

export type SegmentedToggleItem<T extends string> = {
  id: T;
  label: string;
  icon?: ReactNode;
  'data-testid'?: string;
};

export type SegmentedToggleProps<T extends string> = {
  items: readonly SegmentedToggleItem<T>[];
  value: T;
  onChange: (id: T) => void;
  // Must be unique per rendered instance so the sliding pill animates within
  // this control only (motion shares layout across identical layoutIds).
  layoutId: string;
  className?: string;
  'aria-label'?: string;
  'data-testid'?: string;
};

// Pill-style segmented single-select control. Shared look with the home
// "Fast | Mixnet" mode toggle.
function SegmentedToggle<T extends string>({
  items,
  value,
  onChange,
  layoutId,
  className,
  ...rest
}: SegmentedToggleProps<T>) {
  return (
    <div
      className={clsx(
        'bg-surface-bg relative flex items-center gap-2 rounded-full p-0.5',
        className,
      )}
      aria-label={rest['aria-label']}
      data-testid={rest['data-testid']}
    >
      {items.map((item) => {
        const isSelected = value === item.id;
        return (
          <button
            key={item.id}
            type="button"
            aria-pressed={isSelected}
            data-testid={item['data-testid']}
            onClick={() => onChange(item.id)}
            className={clsx(
              'relative flex flex-1 cursor-default items-center justify-center gap-1.5 rounded-full px-4.5 py-2.5 text-sm font-bold transition-colors',
              isSelected
                ? 'text-primary'
                : 'text-text-secondary hover:bg-surface-elev',
            )}
          >
            {isSelected && (
              <motion.div
                layoutId={layoutId}
                className="bg-surface-elev absolute inset-0 rounded-full"
                transition={{ duration: 0.3, ease: 'easeOut' }}
              />
            )}
            {item.icon && <span className="z-10 flex">{item.icon}</span>}
            <span className="z-10">{item.label}</span>
          </button>
        );
      })}
    </div>
  );
}

export default SegmentedToggle;
