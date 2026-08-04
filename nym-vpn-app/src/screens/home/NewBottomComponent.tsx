import { motion } from 'motion/react';
import { invoke } from '@tauri-apps/api/core';
import { useTranslation } from 'react-i18next';
import { Button, ButtonVariant } from '../../ui';
import { dispatch, useMainState } from '../../store';
import { useConnect, useToast } from '../../hooks';
import { useAnimatedNavigate } from '../../hooks/useAnimatedNavigate';
import { routes } from '../../router';
import { InteractiveCard } from './InteractiveCard';
import { ModeToggle } from './ModeToggle';
import { NodeRow } from './NodeRow';

export type FoldState = 0 | 1 | 2;

const DURATION = 0.3;
const easeOutQuart = [0.22, 1, 0.36, 1] as const;

export function NewBottomComponent() {
  const navigate = useAnimatedNavigate();
  const { t } = useTranslation('home');
  const { state, daemonStatus, accountState, account } = useMainState();

  const daemonUnavailable =
    daemonStatus === 'auth-denied' || daemonStatus === 'down';
  const needAPlan =
    !daemonUnavailable &&
    state === 'disconnected' &&
    account &&
    (accountState === 'no-subscription' ||
      accountState === 'bandwidth-exceeded');

  const { add } = useToast();
  const connect = useConnect();

  const handleConnect = async () => {
    if (daemonStatus === 'auth-denied') {
      invoke('retry_authentication').catch((e: unknown) => {
        console.error('retry_authentication failed', e);
      });
      return;
    }

    if (state === 'disconnected' && !account) {
      navigate(routes.onboarding);
      return;
    }

    if (needAPlan) {
      navigate(routes.selectPlan);
      return;
    }

    if (
      state === 'connected' ||
      state === 'connecting' ||
      state === 'offline-auto-reconnect' ||
      state === 'error'
    ) {
      console.info('disconnect attempt');
      dispatch({ type: 'disconnect' });
      try {
        await invoke('disconnect');
      } catch (error: unknown) {
        console.error('failed to disconnect', error);
        add({
          id: 'disconnect-error',
          title: t('failed-to-disconnect'),
          type: 'error',
        });
      }
    }
    if (state === 'disconnected') {
      console.info('connect attempt');
      try {
        await connect();
      } catch (error: unknown) {
        console.error('failed to connect', error);
        dispatch({ type: 'set-tunnel-disconnected' });
        add({
          id: 'connect-error',
          title: 'Failed to connect',
          type: 'error',
        });
      }
    }
  };

  const getButtonText = () => {
    if (daemonStatus === 'auth-denied') {
      return t('authenticate');
    }

    if (!account) {
      return t('get-started');
    }

    if (needAPlan) {
      return t('choose-plan');
    }

    switch (state) {
      case 'connected':
        return t('status.connected');
      case 'disconnected':
      case 'unknown':
        return t('status.disconnected');
      case 'connecting':
      case 'offline-auto-reconnect':
        return t('status.connecting');
      case 'disconnecting':
        return t('status.disconnecting');
      case 'offline':
        return t('status.offline');
      case 'error':
        return t('stop', { ns: 'glossary' });
    }
  };

  const getButtonVariant = (): ButtonVariant => {
    if (!account || needAPlan) {
      return 'primary';
    }

    switch (state) {
      case 'disconnected':
      case 'offline':
        return 'primary';
      case 'connected':
      case 'connecting':
        return 'outlined';
      case 'offline-auto-reconnect':
      case 'disconnecting':
      case 'error':
        return 'destructive';
      case 'unknown':
        return 'outlined';
    }
  };

  const getButtonDisabled = () => {
    if (daemonStatus === 'auth-denied') return false;

    return (
      daemonStatus === 'down' ||
      state === 'offline' ||
      state === 'disconnecting' ||
      accountState === 'pending-subscription'
    );
  };

  return (
    <div className="flex flex-col">
      {/* ── Main card ─────────────────────────────────────────────────────── */}

      <motion.div
        className="mb-4"
        initial={{ opacity: 0, y: 100 }}
        animate={{ opacity: 1, y: 0 }}
        exit={{ opacity: 0, y: 100 }}
        transition={{ duration: DURATION, ease: easeOutQuart }}
      >
        <ModeToggle />
      </motion.div>

      <InteractiveCard>
        <div className="relative z-20 mb-4 flex flex-col">
          <div className="flex flex-row items-center gap-2">
            <motion.div className="flex w-full min-w-0 flex-col overflow-hidden">
              <div className="space-y-4">
                <NodeRow type="exit" />
                <NodeRow type="entry" />
              </div>
            </motion.div>
          </div>
        </div>
        {/* ── Main card ─────────────────────────────────────────────────────── */}

        {/* Button ───────────────────────────────────────────────────────── */}
        <div className="z-10">
          <Button
            disabled={getButtonDisabled()}
            variant={getButtonVariant()}
            onClick={handleConnect}
          >
            {getButtonText()}
          </Button>
        </div>
        {/* Button ───────────────────────────────────────────────────────── */}
      </InteractiveCard>
    </div>
  );
}
