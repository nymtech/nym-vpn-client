import clsx from 'clsx';
import MsIcon from '../../../ui/MsIcon';

export type AppEntry = {
  id: string;
  name: string;
  iconColor: string;
  state: 'excluded' | 'included';
};

type Props = {
  app: AppEntry;
  onStateChange: (id: string, state: 'excluded' | 'included') => void;
};

function AppItem({ app, onStateChange }: Props) {
  return (
    <div className="flex items-center gap-3 px-4 py-3 bg-white dark:bg-charcoal">
      <div
        className="w-7 h-7 rounded-md flex items-center justify-center text-white text-sm font-bold select-none"
        style={{ backgroundColor: app.iconColor }}
      >
        {app.name[0].toUpperCase()}
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
          )}
          onClick={() => onStateChange(app.id, 'excluded')}
          aria-label={`Exclude ${app.name} from VPN`}
        >
          <MsIcon icon="block" className="text-base" />
        </button>
        <button
          className={clsx(
            'px-2  h-full w-full  flex items-center justify-center cursor-default transition-noborder rounded-r-lg',
            app.state === 'included'
              ? 'bg-malachite-moss/15 dark:bg-malachite/15 text-malachite-moss dark:text-malachite'
              : 'text-iron dark:text-bombay',
          )}
          onClick={() => onStateChange(app.id, 'included')}
          aria-label={`Include ${app.name} in VPN`}
        >
          <MsIcon icon="shield" className="text-base" />
        </button>
      </div>
    </div>
  );
}

export default AppItem;
