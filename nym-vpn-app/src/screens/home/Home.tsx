import { useCallback, useEffect, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { invoke } from '@tauri-apps/api/core';
import { type } from '@tauri-apps/plugin-os';
import { useNavigate } from 'react-router';
import clsx from 'clsx';
import { motion } from 'motion/react';
import {
  Focused,
  useGateways,
  useInAppNotify,
  useMainDispatch,
  useMainState,
  useNodeListState,
} from '../../contexts';
import {
  BackendError,
  StateDispatch,
  isCountry,
  isGateway,
  isRegion,
} from '../../types';
import { routes } from '../../router';
import { Button, Switch } from '../../ui';
import { capFirst } from '../../util';
import NetworkModeSelect from './NetworkModeSelect';
import TunnelState from './TunnelState';
import HopSelect from './HopSelect';
import NetworkUpdateDialog from './NetworkUpdateDialog';
import UpdateDialog from './UpdateDialog';
import { regionToCountryCode } from './util';

const updaterEnabled = window._APP.updaterEnabled;
const devMode = window._APP.devMode;
const os = type();
let compatChecked = false;

function Home() {
  const {
    state,
    tunnel,
    connectingState,
    accountState,
    entryNode,
    exitNode,
    daemonStatus,
    account,
    networkCompat,
    vpnMode,
    gatewaySelectionAlgorithm,
  } = useMainState();

  const dispatch = useMainDispatch() as StateDispatch;
  const { setFocused, setSearch, setExpanded } = useNodeListState();
  const { lookupGw } = useGateways();
  const { push } = useInAppNotify();
  const navigate = useNavigate();
  const { t } = useTranslation('home');
  const loading = state === 'disconnecting';
  const daemonUnavailable =
    daemonStatus === 'auth-denied' || daemonStatus === 'down';
  const needAPlan =
    !daemonUnavailable &&
    state === 'disconnected' &&
    account &&
    (accountState === 'no-subscription' ||
      accountState === 'bandwidth-exceeded');

  const entryGwId = tunnel?.entryGwId || connectingState?.entryGwId || null;
  const exitGwId = tunnel?.exitGwId || connectingState?.exitGwId || null;

  const [isDialogUpdateOpen, setIsDialogUpdateOpen] = useState(false);
  const quickConnect = gatewaySelectionAlgorithm === 'auto';
  const canShowQuickConnect = vpnMode === 'wg';
  const quickConnectDisabled = daemonUnavailable || state !== 'disconnected';

  const setQuickConnect = useCallback(
    async (enabled: boolean) => {
      const algorithm = enabled ? 'auto' : 'explicit';
      dispatch({
        type: 'set-gateway-selection-algorithm',
        algorithm,
      });
      try {
        await invoke('set_gateway_selection_algorithm', { algorithm });
      } catch (e) {
        dispatch({
          type: 'set-gateway-selection-algorithm',
          algorithm: enabled ? 'explicit' : 'auto',
        });
        throw e;
      }
    },
    [dispatch],
  );

  const handleClick = () => {
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
    dispatch({ type: 'disconnect' });
    if (
      state === 'connected' ||
      state === 'connecting' ||
      state === 'offline-auto-reconnect' ||
      state === 'error'
    ) {
      console.info('disconnect');
      if (state === 'connecting') {
        dispatch({ type: 'new-progress-message', message: 'canceling' });
      }
      invoke('disconnect')
        .then((result) => {
          console.log(result);
        })
        .catch((e: unknown) => {
          dispatch({ type: 'set-error', error: e as BackendError });
        });
    } else if (state === 'disconnected') {
      console.info('connect');
      dispatch({ type: 'reset-error' });
      dispatch({ type: 'connect' });
      invoke('connect')
        .then((result) => {
          console.log(result);
        })
        .catch((e: unknown) => {
          dispatch({ type: 'set-error', error: e as BackendError });
        });
    }
  };

  useEffect(() => {
    if (devMode || compatChecked) {
      return;
    }
    if (
      networkCompat &&
      (networkCompat.core === false || networkCompat.tauri === false)
    ) {
      // if either core or tauri is not compatible, show the update dialog
      compatChecked = true;
      setIsDialogUpdateOpen(true);
    }
  }, [networkCompat]);

  useEffect(() => {
    if (vpnMode !== 'wg' && quickConnect) {
      setQuickConnect(false).catch((e: unknown) => {
        console.error('set_gateway_selection_algorithm failed', e);
      });
      push({
        id: 'quick-connect-disabled-mixnet',
        message: t('quick-connect.disabled-in-anonymous'),
        throttle: 2,
        clickAway: true,
      });
    }
  }, [push, quickConnect, setQuickConnect, t, vpnMode]);

  const getButtonText = useCallback(() => {
    const stop = capFirst(t('stop', { ns: 'glossary' }));
    const cancel = capFirst(t('cancel', { ns: 'glossary' }));

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
        return t('disconnect');
      case 'disconnected':
      case 'unknown':
        return t('connect');
      case 'connecting':
        return stop;
      case 'disconnecting':
        return null;
      case 'offline':
        return t('connect');
      case 'offline-auto-reconnect':
        return stop;
      case 'error':
        return cancel;
    }
  }, [state, t, needAPlan, account, daemonStatus]);

  const getButtonColor = () => {
    if (daemonStatus === 'auth-denied') {
      return 'cornflower';
    }

    switch (state) {
      case 'disconnected':
      case 'offline':
        return 'malachite';
      case 'connected':
      case 'connecting':
      case 'offline-auto-reconnect':
      case 'disconnecting':
      case 'error':
        return 'red';
      case 'unknown':
        return 'gray';
    }
  };

  const getButtonDisabled = () => {
    if (daemonStatus === 'auth-denied') {
      return false;
    }

    return (
      loading ||
      daemonUnavailable ||
      state === 'offline' ||
      accountState === 'pending-subscription'
    );
  };

  const goToNodeList = (hop: 'entry' | 'exit') => {
    const expanded: string[] = [];
    let focused: Focused | null = null;
    const node = hop === 'entry' ? entryNode : exitNode;

    if (isCountry(node)) {
      focused = { type: 'country', key: node.country.code };
    } else if (isRegion(node)) {
      const code = regionToCountryCode(node.region);
      if (code) {
        expanded.push(code.toUpperCase());
        focused = { type: 'region', key: node.region };
      }
    } else if (isGateway(node)) {
      focused = { type: 'gateway', key: node.gateway.id };
      const gw = lookupGw(node.gateway.id, hop);
      if (gw) {
        expanded.push(gw.country.code.toUpperCase());
        if (gw.country.code.toLowerCase() === 'us') {
          expanded.push(gw.location.region);
        }
      }
    }

    setExpanded(hop, expanded);
    setFocused(hop, focused);
    setSearch(hop, null);

    if (hop === 'entry') {
      navigate(routes.entryNodeLocation);
    } else {
      navigate(routes.exitNodeLocation);
    }
  };

  return (
    <>
      {updaterEnabled && <UpdateDialog />}
      {os !== 'windows' && (
        <NetworkUpdateDialog
          isOpen={isDialogUpdateOpen}
          onClose={() => setIsDialogUpdateOpen(false)}
          appUpdate={!networkCompat?.tauri}
          daemonUpdate={!networkCompat?.core}
        />
      )}
      <motion.div
        initial={{ opacity: 0, x: '-1rem' }}
        animate={{ opacity: 1, x: 0 }}
        transition={{ duration: 0.2, ease: 'easeOut' }}
        className="h-full flex flex-col"
        data-testid="home-container"
      >
        <div className="grow" data-testid="home-tunnel-state-container">
          <TunnelState />
        </div>
        <div
          className="flex flex-col justify-between gap-y-8 select-none"
          data-testid="home-controls-container"
        >
          <div className="flex flex-col justify-between gap-y-4">
            <NetworkModeSelect />
            <div
              className="flex flex-col gap-6"
              data-testid="home-node-select-section"
            >
              {canShowQuickConnect && (
                <div
                  className="flex flex-row items-center justify-between gap-4 rounded-lg border border-bombay dark:border-iron px-4 py-3 text-baltic-sea dark:text-white"
                  data-testid="home-quick-connect"
                >
                  <div className="flex flex-col cursor-default">
                    <span className="text-base font-medium">
                      {t('quick-connect.title')}
                    </span>
                    <span className="text-sm text-bombay dark:text-iron">
                      {t('quick-connect.desc')}
                    </span>
                  </div>
                  <Switch
                    checked={quickConnect}
                    onChange={(enabled) => {
                      setQuickConnect(enabled).catch((e: unknown) => {
                        console.error(
                          'set_gateway_selection_algorithm failed',
                          e,
                        );
                      });
                    }}
                    disabled={quickConnectDisabled}
                    data-testid="home-quick-connect-switch"
                  />
                </div>
              )}
              <div
                className="mt-3 text-base font-medium cursor-default"
                data-testid="home-node-select-title"
              >
                {t('select-node-title')}
              </div>
              <div
                className="flex flex-col gap-5"
                data-testid="home-hop-selects-container"
              >
                <HopSelect
                  node={entryNode}
                  gatewayId={entryGwId}
                  onClick={() => goToNodeList('entry')}
                  nodeHop="entry"
                  disabled={daemonUnavailable || quickConnect}
                  forceAuto={quickConnect}
                />
                <HopSelect
                  node={exitNode}
                  gatewayId={exitGwId}
                  onClick={() => goToNodeList('exit')}
                  nodeHop="exit"
                  disabled={daemonUnavailable || quickConnect}
                  forceAuto={quickConnect}
                />
              </div>
            </div>
          </div>
          <Button
            onClick={handleClick}
            color={getButtonColor()}
            disabled={getButtonDisabled()}
            spinner={loading}
            className={clsx(['h-14', loading && 'data-disabled:opacity-80'])}
            textSize="base"
            data-testid="home-connection-button"
            data-state={state}
          >
            {getButtonText()}
          </Button>
        </div>
      </motion.div>
    </>
  );
}

export default Home;
