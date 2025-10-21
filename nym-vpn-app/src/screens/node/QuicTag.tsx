import clsx from 'clsx';
import { MsIcon } from '../../ui';

function QuicTag() {
  return (
    <div
      className={clsx(
        'border rounded border-azur/50 text-azur select-none',
        'flex items-center flex-nowrap gap-1 font-medium text-xs py-0.2 px-2',
      )}
    >
      <MsIcon icon="package_2" className="text-base text-azur" />
      <p>QUIC</p>
    </div>
  );
}

export default QuicTag;
