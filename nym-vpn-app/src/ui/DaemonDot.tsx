import clsx from 'clsx';
import { DaemonStatus } from '../types';
import { S_STATE } from '../static';

type DaemonDotProps = {
  status: DaemonStatus;
  'data-testid'?: string;
};

function DaemonDot({ status, ...rest }: DaemonDotProps) {
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

  const testId = rest['data-testid'] || 'daemon-dot';

  return (
    <div
      className={clsx([
        'absolute z-30 left-1 top-1 pointer-events-none select-none',
        status === 'ok' ? 'animate-pulse' : 'animate-pulse-fast',
      ])}
      data-testid={testId}
      data-status={status}
    >
      <div
        className={clsx(['relative w-2.5 h-2.5 rounded-full', bgColor()])}
        data-testid={`${testId}-indicator`}
      />
    </div>
  );
}

export default DaemonDot;
