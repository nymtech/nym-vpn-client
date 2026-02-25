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
import { Button } from '../../ui';
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
  } = useMainState();
  console.log('mainstate', useMainState());
  const dispatch = useMainDispatch() as StateDispatch;
  const { setFocused, setSearch, setExpanded } = useNodeListState();
  const { lookupGw } = useGateways();
  const navigate = useNavigate();
  const { t } = useTranslation('home');
  const loading = state === 'disconnecting';
  const needAPlan =
    daemonStatus !== 'down' &&
    state === 'disconnected' &&
    account &&
    (accountState === 'no-subscription' ||
      accountState === 'bandwidth-exceeded');

  const entryGwId = tunnel?.entryGwId || connectingState?.entryGwId || null;
  const exitGwId = tunnel?.exitGwId || connectingState?.exitGwId || null;

  const [isDialogUpdateOpen, setIsDialogUpdateOpen] = useState(false);
  const [diagnosticRunning, setDiagnosticRunning] = useState(false);
  const [diagnosticResult, setDiagnosticResult] = useState<{
    ok: boolean;
    data: string;
  } | null>(null);

  const handleDiagnostic = async () => {
    setDiagnosticRunning(true);
    setDiagnosticResult(null);
    try {
      const report = await invoke('run_diagnostic', {
        params: { gateway: null, skipDns: false, skipHttp: false },
      });
      console.log('Diagnostic report:', report);
      setDiagnosticResult({
        ok: true,
        data: JSON.stringify(report, null, 2),
      });
    } catch (e: unknown) {
      console.error('Diagnostic failed:', e);
      const err = e as BackendError;
      setDiagnosticResult({
        ok: false,
        data: err?.message || 'Unknown error',
      });
    } finally {
      setDiagnosticRunning(false);
    }
  };

  const handleClick = () => {
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

  const getButtonText = useCallback(() => {
    const stop = capFirst(t('stop', { ns: 'glossary' }));
    const cancel = capFirst(t('cancel', { ns: 'glossary' }));
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
  }, [state, t, needAPlan, account]);

  const getButtonColor = () => {
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
                  disabled={daemonStatus === 'down'}
                />
                <HopSelect
                  node={exitNode}
                  gatewayId={exitGwId}
                  onClick={() => goToNodeList('exit')}
                  nodeHop="exit"
                  disabled={daemonStatus === 'down'}
                />
              </div>
            </div>
          </div>
          <Button
            onClick={handleClick}
            color={getButtonColor()}
            disabled={loading || daemonStatus === 'down' || state === 'offline'}
            spinner={loading}
            className={clsx(['h-14', loading && 'data-disabled:opacity-80'])}
            textSize="base"
            data-testid="home-connection-button"
            data-state={state}
          >
            {getButtonText()}
          </Button>
          <Button
            onClick={handleDiagnostic}
            color="gray"
            disabled={diagnosticRunning || daemonStatus === 'down'}
            spinner={diagnosticRunning}
            className="h-10"
            textSize="base"
            data-testid="home-diagnostic-button"
          >
            {diagnosticRunning ? 'Running diagnostic...' : 'Run diagnostic'}
          </Button>
          {diagnosticResult && (
            <div
              className={clsx(
                'mt-2 p-3 rounded-lg text-xs font-mono max-h-48 overflow-auto',
                diagnosticResult.ok
                  ? 'bg-green-900/20 text-green-300 border border-green-800'
                  : 'bg-red-900/20 text-red-300 border border-red-800',
              )}
              data-testid="home-diagnostic-result"
            >
              <div className="flex justify-between items-center mb-1">
                <span className="font-semibold text-sm">
                  {diagnosticResult.ok
                    ? 'Diagnostic report'
                    : 'Diagnostic error'}
                </span>
                <button
                  onClick={() => setDiagnosticResult(null)}
                  className="text-sm opacity-60 hover:opacity-100 cursor-pointer"
                >
                  ✕
                </button>
              </div>
              <pre className="whitespace-pre-wrap wrap-break-word">
                {diagnosticResult.data}
              </pre>
            </div>
          )}
        </div>
      </motion.div>
    </>
  );
}

export default Home;
