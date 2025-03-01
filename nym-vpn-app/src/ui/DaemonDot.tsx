import clsx from 'clsx';
import { DaemonStatus } from '../types';
import { S_STATE } from '../static';

type DaemonDotProps = {
  status: DaemonStatus;
};

function DaemonDot({ status }: DaemonDotProps) {
  const bgColor = () => {
    switch (status) {
      case 'ok':
        return 'bg-vert-menthe';
      case 'non-compat':
        return 'bg-liquid-lava';
      default:
        return 'bg-rouge-ecarlate';
    }
  };

  if (!S_STATE.devMode && status === 'ok') {
    return null;
  }

  return (
    <div
      className={clsx([
        'absolute z-30 left-1 top-1 pointer-events-none select-none',
        status === 'ok' ? 'animate-pulse' : 'animate-pulse-fast',
      ])}
    >
      <div className={clsx(['relative w-2.5 h-2.5 rounded-full', bgColor()])} />
    </div>
  );
}

export default DaemonDot;
