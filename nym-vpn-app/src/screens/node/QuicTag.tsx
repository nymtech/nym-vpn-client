import clsx from 'clsx';
import { MsIcon } from '../../ui';

type QuicTagProps = {
  className?: string;
  label?: string;
};

function QuicTag({ className, label = 'QUIC' }: QuicTagProps) {
  return (
    <div
      className={clsx(
        'text-status-info dark:text-illustration-accent rounded border select-none',
        'border-status-info/50 dark:border-illustration-accent/50',
        'flex flex-nowrap items-center gap-1 px-2 py-0 text-xs font-medium',
        className,
      )}
    >
      <MsIcon
        icon="package_2"
        className="text-status-info dark:text-illustration-accent text-base"
      />
      <p>{label}</p>
    </div>
  );
}

export default QuicTag;
