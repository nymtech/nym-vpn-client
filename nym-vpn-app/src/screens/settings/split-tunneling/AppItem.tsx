import clsx from 'clsx';
import { convertFileSrc } from '@tauri-apps/api/core';
import { type } from '@tauri-apps/plugin-os';
import { Command } from '@tauri-apps/plugin-shell';
import MsIcon from '../../../ui/MsIcon';
import { App } from '../../../types';
import { useInAppNotify } from '../../../contexts';

export type AppEntry = App & {
  state: 'excluded' | 'included';
};

type AppItemProps = {
  app: AppEntry;
  onStateChange: (
    app: AppEntry,
    state: 'excluded' | 'included',
  ) => Promise<void>;
};

function AppItem({ app, onStateChange }: AppItemProps) {
  const os = type();
  const { push } = useInAppNotify();

  const handleClick = async () => {
    if (os === 'linux') {
      try {
        const result = await Command.create(
          'nym-exclude',
          app.executable_path.split(' '),
        ).execute();
        console.info('[nym-exclude] stdout', result.stdout);
        console.info('[nym-exclude] stderr', result.stderr);
      } catch (error) {
        console.error('[nym-exclude] Failed to execute command', error);
        push({
          message: 'Failed to open app',
          close: true,
          type: 'error',
        });
      }
    }
  };

  return (
    <div
      className={clsx(
        'flex items-center gap-3 px-4 py-3 bg-white dark:bg-charcoal',
        os === 'linux' && 'hover:bg-black/10 dark:hover:bg-charcoal/75',
      )}
      onClick={handleClick}
    >
      <div className={'w-7 h-7 flex items-center justify-center'}>
        {app.icon && (
          <img
            src={convertFileSrc(app.icon)}
            alt={app.name}
            className="w-full h-full"
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
      {/* Only Windows can include/exclude apps from inside the app */}
      {/* Linux uses custom app launcher to launch the app and immediately exclude it from the tunnel */}
      {os === 'windows' && (
        <div className="flex items-center rounded-lg border border-iron dark:border-bombay">
          <button
            className={clsx(
              'px-2  h-full w-full flex items-center justify-center cursor-default transition-noborder border-r border-r-iron dark:border-r-bombay rounded-l-lg',
              app.state === 'included'
                ? 'bg-aphrodisiac/20 text-aphrodisiac'
                : 'text-iron dark:text-bombay',
            )}
            onClick={() => onStateChange(app, 'included')}
            aria-label={`Exclude ${app.name} from VPN`}
          >
            <MsIcon icon="block" className="text-base" />
          </button>
          <button
            className={clsx(
              'px-2  h-full w-full  flex items-center justify-center cursor-default transition-noborder rounded-r-lg',
              app.state === 'excluded'
                ? 'bg-malachite-moss/15 dark:bg-malachite/15 text-malachite-moss dark:text-malachite'
                : 'text-iron dark:text-bombay',
            )}
            onClick={() => onStateChange(app, 'excluded')}
            aria-label={`Include ${app.name} in VPN`}
          >
            <MsIcon icon="shield" className="text-base" />
          </button>
        </div>
      )}
    </div>
  );
}

export default AppItem;
