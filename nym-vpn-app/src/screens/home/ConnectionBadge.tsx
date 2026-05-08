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
        return ['text-primary bg-malachite/10!'];
      case 'disconnected':
        return ['text-text-secondary'];
      case 'connecting':
      case 'disconnecting':
        return ['text-text-primary'];
      case 'error':
      case 'offline':
      case 'offline-auto-reconnect':
      case 'unknown':
        return ['text-baltic-sea bg-aphrodisiac!'];
    }
  };

  const getStatusText = (state: TunnelState) => {
    switch (state) {
      case 'connected':
        return t('status.connected');
      case 'disconnected':
      case 'unknown':
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
        'flex min-w-36 items-center justify-center gap-4 tracking-normal',
        'bg-mercury dark:bg-mine-shaft',
        ...getBadgeStyle(state),
        'rounded-full px-6 py-3 text-lg font-medium tracking-normal',
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
