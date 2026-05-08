import clsx from 'clsx';
import { DaemonStatus } from '../types';

const devMode = window._APP.devMode;

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

  if ((!devMode && status === 'ok') || status === 'auth-denied') {
    return null;
  }

  const testId = rest['data-testid'] || 'daemon-dot';

  return (
    <div
      className={clsx([
        'pointer-events-none absolute top-1 left-1 z-30 select-none',
        status === 'ok' ? 'animate-pulse' : 'animate-pulse-fast',
      ])}
      data-testid={testId}
      data-test-status={status}
    >
      <div
        className={clsx(['relative h-2.5 w-2.5 rounded-full', bgColor()])}
        data-testid={`${testId}-indicator`}
      />
    </div>
  );
}

export default DaemonDot;
