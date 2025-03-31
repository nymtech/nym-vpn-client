import clsx from 'clsx';
import { useTranslation } from 'react-i18next';
import { motion } from 'motion/react';
import { TunnelState } from '../../types';
import { PulseDot } from '../../ui';

function ConnectionBadge({ state }: { state: TunnelState }) {
  const { t } = useTranslation('home');

  const getBadgeStyle = (state: TunnelState) => {
    switch (state) {
      case 'Connected':
        return ['text-malachite-moss dark:text-malachite bg-malachite/10!'];
      case 'Disconnected':
        return ['text-iron dark:text-bombay'];
      case 'Connecting':
      case 'Disconnecting':
        return ['text-baltic-sea dark:text-white'];
      case 'Error':
      case 'Offline':
      case 'OfflineAutoReconnect':
        return ['text-baltic-sea bg-aphrodisiac!'];
    }
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
        'flex justify-center items-center tracking-normal gap-4 min-w-36',
        'bg-mercury dark:bg-mine-shaft',
        ...getBadgeStyle(state),
        'text-lg font-medium py-3 px-6 rounded-full tracking-normal',
      ])}
    >
      {getStatusText(state)}
      {(state === 'Connecting' || state === 'Disconnecting') && (
        <PulseDot color="cornflower" />
      )}
    </motion.div>
  );
}

export default ConnectionBadge;
