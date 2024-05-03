import clsx from 'clsx';
import { useTranslation } from 'react-i18next';
import { ConnectionState } from '../../types';
import { AnimateIn } from '../../ui';

function ConnectionBadge({ state }: { state: ConnectionState }) {
  const { t } = useTranslation('home');

  const statusBadgeDynStyles = {
    Connected: ['text-vert-menthe', 'bg-vert-prasin bg-opacity-10'],
    Disconnected: [
      'bg-cement-feet bg-opacity-10',
      'text-coal-mine-light',
      'dark:bg-oil dark:bg-opacity-15',
      'dark:text-coal-mine-dark',
    ],
    Connecting: [
      'bg-cement-feet bg-opacity-10',
      'text-baltic-sea',
      'dark:bg-oil dark:bg-opacity-15',
      'dark:text-white',
    ],
    Disconnecting: [
      'bg-cement-feet bg-opacity-10',
      'text-baltic-sea',
      'dark:bg-oil dark:bg-opacity-15',
      'dark:text-white',
    ],
    Unknown: [
      'bg-cement-feet bg-opacity-10',
      'text-coal-mine-light',
      'dark:bg-oil dark:bg-opacity-15',
      'dark:text-coal-mine-dark',
    ],
  };

  const getStatusText = (state: ConnectionState) => {
    switch (state) {
      case 'Connected':
        return t('status.connected');
      case 'Disconnected':
        return t('status.disconnected');
      case 'Connecting':
        return t('status.connecting');
      case 'Disconnecting':
        return t('status.disconnecting');
      case 'Unknown':
        return t('status.unknown');
    }
  };

  return (
    <AnimateIn
      from="opacity-0 scale-x-90 translate-y-4"
      to="opacity-100 scale-100 translate-y-0 translate-x-0"
      duration={150}
      className={clsx([
        'flex justify-center items-center tracking-normal gap-4',
        ...statusBadgeDynStyles[state],
        'text-lg font-bold py-3 px-6 rounded-full tracking-normal',
      ])}
    >
      {getStatusText(state)}
      {(state === 'Connecting' || state === 'Disconnecting') && (
        <div className="relative flex h-3 w-3">
          <div className="animate-ping absolute inline-flex h-full w-full rounded-full bg-cornflower opacity-75" />
          <div className="relative inline-flex dot h-3 w-3 bg-cornflower" />
        </div>
      )}
    </AnimateIn>
  );
}

export default ConnectionBadge;
