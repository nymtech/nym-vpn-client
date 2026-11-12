import clsx from 'clsx';
import { useTranslation } from 'react-i18next';

type BetaPillProps = {
  className?: string;
  label?: string;
};

function BetaPill({ className, label }: BetaPillProps) {
  const { t } = useTranslation('settings');

  return (
    <span
      className={clsx(
        'inline-flex items-center rounded-full px-2 py-0.5',
        'text-brand-primary border-brand-primary border',
        'text-xs leading-none font-medium tracking-wide select-none',
        className,
      )}
    >
      {label ?? t('geo-exclusion.beta')}
    </span>
  );
}

export default BetaPill;
