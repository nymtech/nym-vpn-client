import clsx from 'clsx';
import { useTranslation } from 'react-i18next';
import { motion } from 'motion/react';
import { TunnelState } from '../../types';
import { PulseDot } from '../../ui';

function ConnectionBadge({ state }: { state: TunnelState }) {
  const { t } = useTranslation('home');

  const getBadgeStyle = (state: TunnelState) => {
    switch (state) {
      case 'connected':
        return ['text-malachite-moss dark:text-malachite bg-malachite/10!'];
      case 'disconnected':
        return ['text-iron dark:text-bombay'];
      case 'connecting':
      case 'disconnecting':
        return ['text-baltic-sea dark:text-white'];
      case 'error':
      case 'offline':
      case 'offline-auto-reconnect':
        return ['text-baltic-sea bg-aphrodisiac!'];
    }
  };

  const getStatusText = (state: TunnelState) => {
    switch (state) {
      case 'connected':
        return t('status.connected');
      case 'disconnected':
        return t('status.disconnected');
      case 'connecting':
        return t('status.connecting');
      case 'disconnecting':
        return t('status.disconnecting');
      case 'error':
        return t('status.error');
      case 'offline':
      case 'offline-auto-reconnect':
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
      data-testid="connection-badge"
      data-status={state}
    >
      <span data-testid="connection-status-text">{getStatusText(state)}</span>
      {(state === 'connecting' || state === 'disconnecting') && (
        <PulseDot color="cornflower" data-testid="connection-pulse-dot" />
      )}
    </motion.div>
  );
}

export default ConnectionBadge;
