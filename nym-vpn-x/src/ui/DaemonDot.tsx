import clsx from 'clsx';
import { DaemonStatus } from '../types';

type ButtonProps = {
  status: DaemonStatus;
};

function DaemonDot({ status }: ButtonProps) {
  return (
    <div
      className={clsx([
        'absolute z-30 left-1 top-1 pointer-events-none select-none',
        status === 'Ok' ? 'animate-pulse' : 'animate-pulse-fast',
      ])}
    >
      <div
        className={clsx([
          'relative dot w-3 h-3',
          status === 'Ok' ? 'bg-vert-menthe' : 'bg-teaberry',
        ])}
      />
    </div>
  );
}

export default DaemonDot;
