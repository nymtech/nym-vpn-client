import clsx from 'clsx';
import MsIcon from '../../../ui/MsIcon';
import { useMainState } from '../../../contexts/index';
import { convertFileSrc } from '@tauri-apps/api/core';

export type AppEntry = {
  id: string;
  name: string;
  exec: string;
  icon: string | null;
  desktop_file: string;
  state: 'excluded' | 'included';
};

type Props = {
  app: AppEntry;
  onStateChange: (id: string, state: 'excluded' | 'included') => void;
};

function AppItem({ app, onStateChange }: Props) {
  const {
    splitTunnel: { enabled },
  } = useMainState();
  return (
    <div className="flex items-center gap-3 px-4 py-3 bg-white dark:bg-charcoal">
      <div className="w-7 h-7 flex items-center justify-center">
        {app.icon && (
          <img
            src={convertFileSrc(app.icon)}
            alt={app.name}
            className="w-4 h-4"
          />
        )}
        {!app.icon && (
          <div className="h-full w-full rounded-md  bg-faded-lavender dark:bg-ash text-baltic-sea dark:text-white flex items-center justify-center text-sm leading-none">
            {app.name[0].toUpperCase()}
          </div>
        )}
      </div>
      <span className="flex-1 text-sm text-baltic-sea dark:text-white truncate select-none">
        {app.name}
      </span>
      <div className="flex items-center rounded-lg border border-iron dark:border-bombay">
        <button
          className={clsx(
            'px-2  h-full w-full flex items-center justify-center cursor-default transition-noborder border-r border-r-iron dark:border-r-bombay rounded-l-lg',
            app.state === 'excluded'
              ? 'bg-aphrodisiac/20 text-aphrodisiac'
              : 'text-iron dark:text-bombay',
            !enabled && 'opacity-50 cursor-not-allowed',
          )}
          onClick={() => onStateChange(app.id, 'excluded')}
          aria-label={`Exclude ${app.name} from VPN`}
          disabled={!enabled}
        >
          <MsIcon icon="block" className="text-base" />
        </button>
        <button
          className={clsx(
            'px-2  h-full w-full  flex items-center justify-center cursor-default transition-noborder rounded-r-lg',
            app.state === 'included'
              ? 'bg-malachite-moss/15 dark:bg-malachite/15 text-malachite-moss dark:text-malachite'
              : 'text-iron dark:text-bombay',
            !enabled && 'opacity-50 cursor-not-allowed',
          )}
          onClick={() => onStateChange(app.id, 'included')}
          aria-label={`Include ${app.name} in VPN`}
          disabled={!enabled}
        >
          <MsIcon icon="shield" className="text-base" />
        </button>
      </div>
    </div>
  );
}

export default AppItem;
