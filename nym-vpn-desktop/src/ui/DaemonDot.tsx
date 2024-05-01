import clsx from 'clsx';
import { DaemonStatus } from '../types';

type ButtonProps = {
  status: DaemonStatus;
};

function DaemonDot({ status }: ButtonProps) {
  return (
    <div className="absolute z-50 left-1 top-1 pointer-events-none select-none">
      <div className="relative flex h-3 w-3">
        <div
          className={clsx([
            'absolute inline-flex h-full w-full rounded-full',
            status === 'Ok'
              ? 'bg-vert-menthe opacity-0'
              : 'animate-ping bg-teaberry opacity-75',
          ])}
        />
        <div
          className={clsx([
            'relative inline-flex dot h-3 w-3',
            status === 'Ok' ? 'animate-pulse bg-vert-menthe' : 'bg-teaberry',
          ])}
        />
      </div>
    </div>
  );
}

export default DaemonDot;
