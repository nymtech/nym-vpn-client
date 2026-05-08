import clsx from 'clsx';
import { MsIcon } from '../../ui';

type QuicTagProps = {
  className?: string;
};

function QuicTag({ className }: QuicTagProps) {
  return (
    <div
      className={clsx(
        'text-ozone dark:text-azur rounded border select-none',
        'border-ozone/50 dark:border-azur/50',
        'flex flex-nowrap items-center gap-1 px-2 py-0 text-xs font-medium',
        className,
      )}
    >
      <MsIcon
        icon="package_2"
        className="text-ozone dark:text-azur text-base"
      />
      <p>QUIC</p>
    </div>
  );
}

export default QuicTag;
