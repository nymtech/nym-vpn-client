import { motion } from 'motion/react';
import { invoke } from '@tauri-apps/api/core';
import { useState } from 'react';
import { useTranslation } from 'react-i18next';
import {
  Button,
  ButtonVariant,
  ConfirmationDialog,
  type countryCode,
} from '../../ui';
import { dispatch, useMainState } from '../../store';
import { useToast } from '../../hooks';
import { useAnimatedNavigate } from '../../hooks/useAnimatedNavigate';
import { routes } from '../../router';
import { Score } from '../../types';
import type { TActiveVpn } from '../../types/tauri';
import { InteractiveCard } from './InteractiveCard';
import { ModeToggle } from './ModeToggle';
import { NodeRow } from './NodeRow';

export type FoldState = 0 | 1 | 2;

const DURATION = 0.3;
const easeOutQuart = [0.22, 1, 0.36, 1] as const;

export type SelectedNodeDisplayProps = {
  countryCode?: countryCode;
  name: string;
  location?: string;
  ip?: string;
  showQuic?: boolean;
  disabled?: boolean;
  showStreamOptimized?: boolean;
  showFastest?: boolean;
  score?: Score;
};

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

  // Pre-connect: if the daemon refuses because another VPN is already active
  // on the host, surface a confirmation dialog rather than just failing.
  const [conflictVpns, setConflictVpns] = useState<TActiveVpn[] | null>(null);
  const [forcing, setForcing] = useState(false);

  const tryConnect = async (force: boolean) => {
    try {
      await invoke('connect', force ? { force: true } : undefined);
      // Only flip the local UI to "connecting" once the daemon accepted the
      // request. If we did this optimistically and the daemon rejected with
      // AnotherVpnActive, the UI would be stuck on "Connecting" forever (no
      // daemon event ever arrives to take it back to disconnected).
      dispatch({ type: 'connect' });
      setConflictVpns(null);
    } catch (error: unknown) {
      // Tauri rejects with the serialized BackendError shape.
      const err = error as { key?: string; data?: { vpns?: string } } | string;
      if (typeof err === 'object' && err?.key === 'another-vpn-active') {
        let vpns: TActiveVpn[] = [];
        try {
          vpns = JSON.parse(err.data?.vpns ?? '[]') as TActiveVpn[];
        } catch {
          vpns = [];
        }
        setConflictVpns(vpns);
        return;
      }
      console.error('failed to connect', error);
      add({
        id: 'connect-error',
        title: 'Failed to connect',
        type: 'error',
      });
    }
  };

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
      dispatch({ type: 'reset-error' });
      await tryConnect(false);
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

      <ConfirmationDialog
        icon="warning"
        title={t('another-vpn.title')}
        description={
          conflictVpns?.length
            ? t('another-vpn.description', {
                vpns: conflictVpns
                  .map((v) =>
                    v.isDefaultRoute ? `${v.interface} (default)` : v.interface,
                  )
                  .join(', '),
              })
            : ''
        }
        confirmButtonText={t('another-vpn.connect-anyway')}
        cancelButtonText={t('another-vpn.close')}
        isOpen={conflictVpns !== null}
        isLoading={forcing}
        onConfirm={async () => {
          setForcing(true);
          try {
            await tryConnect(true);
          } finally {
            setForcing(false);
          }
        }}
        onCancel={() => setConflictVpns(null)}
      />
    </div>
  );
}
