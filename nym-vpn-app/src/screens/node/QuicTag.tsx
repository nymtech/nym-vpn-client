import clsx from 'clsx';
import { MsIcon } from '../../ui';

type QuicTagProps = {
  className?: string;
};

function QuicTag({ className }: QuicTagProps) {
  return (
    <div
      className={clsx(
        'border rounded text-ozone  dark:text-azur select-none',
        'border-ozone/50 dark:border-azur/50',
        'flex items-center flex-nowrap gap-1 font-medium text-xs py-0 px-2',
        className,
      )}
    >
      <MsIcon
        icon="package_2"
        className="text-base text-ozone dark:text-azur"
      />
      <p>QUIC</p>
    </div>
  );
}

export default QuicTag;
