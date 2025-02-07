import clsx from 'clsx';
import { useTranslation } from 'react-i18next';
import { motion } from 'motion/react';
import { TunnelState } from '../../types';
import { PulseDot } from '../../ui';

function ConnectionBadge({ state }: { state: TunnelState }) {
  const { t } = useTranslation('home');

  const statusBadgeDynStyles = {
    Connected: [
      'text-malachite-moss dark:text-malachite',
      'bg-vert-prasin bg-opacity-10',
    ],
    Disconnected: [
      'bg-cement-feet bg-opacity-10',
      'text-coal-mine-light',
      'dark:bg-oil dark:bg-opacity-15',
      'dark:text-coal-mine-dark',
    ],
    Connecting: [
      'bg-cement-feet/10',
      'text-baltic-sea',
      'dark:bg-oil/15',
      'dark:text-white',
    ],
    Disconnecting: [
      'bg-cement-feet bg-opacity-10',
      'text-baltic-sea',
      'dark:bg-oil dark:bg-opacity-15',
      'dark:text-white',
    ],
    Error: [
      'bg-cement-feet bg-opacity-10',
      'text-teaberry',
      'dark:bg-oil dark:bg-opacity-15',
    ],
    Offline: [
      'bg-rose-bruni/95 dark:bg-rouge-basque/85',
      'text-baltic-sea dark:text-mercury-pinkish',
    ],
    OfflineAutoReconnect: [
      'bg-cement-feet/10 dark:bg-oil/15',
      'text-baltic-sea dark:text-white',
    ],
  };

  const getStatusText = (state: TunnelState) => {
    switch (state) {
      case 'Connected':
        return t('status.connected');
      case 'Disconnected':
        return t('status.disconnected');
      case 'Connecting':
        return t('status.connecting');
      case 'Disconnecting':
        return t('status.disconnecting');
      case 'Error':
        return t('status.error');
      case 'Offline':
      case 'OfflineAutoReconnect':
        return t('status.offline');
    }
  };

  return (
    <motion.div
      initial={{ opacity: 0, scaleX: 0.8, translateY: 4 }}
      animate={{ opacity: 1, scaleX: 1, translateY: 0 }}
      transition={{ duration: 0.1, ease: 'easeOut' }}
      className={clsx([
        'flex justify-center items-center tracking-normal gap-4',
        ...statusBadgeDynStyles[state],
        'text-lg font-bold py-3 px-6 rounded-full tracking-normal',
      ])}
    >
      {getStatusText(state)}
      {(state === 'Connecting' || state === 'Disconnecting') && (
        <PulseDot color="cornflower" />
      )}
      {state === 'OfflineAutoReconnect' && <PulseDot color="red" />}
    </motion.div>
  );
}

export default ConnectionBadge;
