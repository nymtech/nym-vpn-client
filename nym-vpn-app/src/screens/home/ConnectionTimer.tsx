import { useEffect, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { AnimatePresence, motion } from 'motion/react';
import dayjs from 'dayjs';
import { useAppStore } from '../../store';
import { ScrambleIn } from '../../ui/ScrambleIn';

function ConnectionTimer() {
  const state = useAppStore((s) => s.state);
  const tunnelConnectedAt = useAppStore((s) => s.tunnelConnectedAt);
  const [connectionTime, setConnectionTime] = useState('00:00:00');
  const { t } = useTranslation('home');

  useEffect(() => {
    if (!tunnelConnectedAt) {
      return;
    }

    const elapsed = dayjs.duration(dayjs().diff(tunnelConnectedAt));
    setConnectionTime(elapsed.format('HH:mm:ss'));

    const interval = setInterval(() => {
      const elapsed = dayjs.duration(dayjs().diff(tunnelConnectedAt));
      setConnectionTime(elapsed.format('HH:mm:ss'));
    }, 500);

    return () => {
      clearInterval(interval);
    };
  }, [tunnelConnectedAt]);

  if (state !== 'connected') {
    return null;
  }

  return (
    <AnimatePresence mode="wait">
      <motion.div
        initial={{ opacity: 0, scale: 0.9 }}
        animate={{ opacity: 1, scale: 1 }}
        transition={{ duration: 0.1, ease: 'easeOut' }}
        className="flex cursor-default flex-col items-center gap-2 select-none"
        data-testid="connection-timer"
      >
        <ScrambleIn
          text={t('connection-time')}
          className="text-text-secondary text-base"
          scrambledClassName="text-[9px] text-[#8b8b90]/50"
          scrambleSpeed={20}
        />
        <motion.p
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          exit={{ opacity: 0 }}
          transition={{ duration: 0.15 }}
          className="text-text-primary text-base"
          data-testid="connection-time-value"
        >
          {connectionTime}
        </motion.p>
      </motion.div>
    </AnimatePresence>
  );
}

export default ConnectionTimer;
