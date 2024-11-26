import clsx from 'clsx';
import { DaemonStatus } from '../types';

type ButtonProps = {
  status: DaemonStatus;
};

function DaemonDot({ status }: ButtonProps) {
  const bgColor = () => {
    switch (status) {
      case 'Ok':
        return 'bg-vert-menthe';
      case 'NonCompat':
        return 'bg-liquid-lava';
      default:
        return 'bg-teaberry';
    }
  };

  return (
    <div
      className={clsx([
        'absolute z-30 left-1 top-1 pointer-events-none select-none',
        status === 'Ok' ? 'animate-pulse' : 'animate-pulse-fast',
      ])}
    >
      <div className={clsx(['relative w-2.5 h-2.5 rounded-full', bgColor()])} />
    </div>
  );
}

export default DaemonDot;
