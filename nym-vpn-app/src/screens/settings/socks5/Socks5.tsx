import { useEffect, useState } from 'react';
import { useTranslation } from 'react-i18next';
import clsx from 'clsx';
import { CardSwitch, PageAnim, SettingsMenuCardBig } from '../../../ui';
import { useInAppNotify, useMainState, useSocks5 } from '../../../contexts';
import { useClipboard } from '../../../hooks';
import {
  ProxyFieldSection,
  ProxyInfoCard,
  ProxyInfoMessage,
} from './components';

const DefaultSocks5Address = '127.0.0.1:1080';
const DefaultHttpRpcAddress = '127.0.0.1:8545';

function Socks5() {
  const { status, isLoading, enable, disable } = useSocks5();
  const { exitNode } = useMainState();
  const { push } = useInAppNotify();
  const { t } = useTranslation('settings');
  const { copy } = useClipboard();

  // default listen addresses
  const [socks5Address, setSocks5Address] = useState(DefaultSocks5Address);
  const [httpRpcAddress, setHttpRpcAddress] = useState(DefaultHttpRpcAddress);

  // sync input fields with actual values when status changes
  useEffect(() => {
    if (status?.socks5Settings?.listenAddress) {
      setSocks5Address(status.socks5Settings.listenAddress);
    }
    if (status?.httpRpcSettings?.listenAddress) {
      setHttpRpcAddress(status.httpRpcSettings.listenAddress);
    }
  }, [
    status?.socks5Settings?.listenAddress,
    status?.httpRpcSettings?.listenAddress,
  ]);

  const isEnabled = !!status?.state && status?.state !== 'disabled';
  const isConnected = status?.state === 'idle' || status?.state === 'connected';
  const hasError = status?.state === 'error';
  const socks5Url = status?.socks5Settings?.listenAddress
    ? `socks5h://${status.socks5Settings.listenAddress}`
    : null;
  const httpRpcUrl = status?.httpRpcSettings?.listenAddress
    ? `http://${status.httpRpcSettings.listenAddress}?p=<your-provider-url>`
    : null;

  useEffect(() => {
    if (hasError) {
      push({
        id: 'socks5-error',
        message: status?.errorMessage ?? t('app-proxy.error-unknown'),
        close: true,
        type: 'error',
      });
    }
  }, [hasError, status?.errorMessage, push, t]);

  const getStatusString = () => {
    if (isLoading) {
      return t('app-proxy.status.enabling');
    }
    switch (status?.state) {
      case 'idle':
      case 'connected':
        return t('app-proxy.status.connected');
      case 'error':
      case 'disabled':
        return t('app-proxy.status.disabled');
      default:
        return t('app-proxy.status.unknown');
    }
  };

  const getStatusColor = () => {
    if (isLoading) {
      return 'text-baltic-sea dark:text-white';
    }
    switch (status?.state) {
      case 'idle':
      case 'connected':
        return 'text-malachite-moss dark:text-malachite';
      case 'error':
        return 'text-aphrodisiac';
      case 'disabled':
        return 'text-baltic-sea dark:text-white';
      default:
        return 'text-baltic-sea dark:text-white';
    }
  };

  // enable/disable socks5
  const handleToggle = async () => {
    // Prevent duplicate calls while loading
    if (isLoading) {
      return;
    }

    try {
      if (isEnabled) {
        await disable();
        push({
          id: 'socks5-disabled',
          message: t('app-proxy.snackbar-disabled'),
          duration: 3000,
          close: true,
        });
      } else {
        await enable(
          { listenAddress: socks5Address || DefaultSocks5Address },
          { listenAddress: httpRpcAddress || DefaultHttpRpcAddress },
          exitNode,
        );
        push({
          id: 'socks5-enabled',
          message: t('app-proxy.snackbar-enabled'),
          duration: 3000,
          close: true,
        });
      }
    } catch (error) {
      const explicitErrorMessage = String((error as Error)?.message) || '';

      // Explicit error we want to show, show specific error
      if (explicitErrorMessage.includes('Gateway does not support')) {
        push({
          id: 'socks5-error',
          message: t('app-proxy.error-gateway-not-supported'),
          duration: 5000,
          close: true,
          type: 'error',
        });
      }
      // Unknown error, show generic error
      else {
        push({
          id: 'socks5-error',
          message: t('app-proxy.error-unknown'),
          duration: 5000,
          close: true,
          type: 'error',
        });
      }
    }
  };

  return (
    <PageAnim className="relative h-full flex flex-col mt-2 gap-6 select-none">
      <div className="text-iron dark:text-bombay">{t('app-proxy.intro')}</div>

      <SettingsMenuCardBig
        header={
          <CardSwitch
            header={t('app-proxy.switch-title')}
            checked={isEnabled}
            onClick={handleToggle}
            disabled={isLoading || (!isEnabled && isLoading)}
          />
        }
      >
        <div>
          <ul
            className={clsx([
              'flex flex-col justify-center items-center gap-0',
              'bg-white dark:bg-charcoal',
              'cursor-default',
            ])}
          >
            <li
              className={clsx(
                'w-full flex border-b last:border-b-0',
                'py-2 last:pb-0 first:pt-0 border-bombay dark:border-iron',
              )}
            >
              <div className="w-full flex items-center gap-2 justify-between">
                <span className="text-iron dark:text-bombay truncate select-none">
                  {t('app-proxy.proxy-status')}
                </span>
                <span className={clsx(getStatusColor())}>
                  {getStatusString()}
                </span>
              </div>
            </li>
            <li
              className={clsx(
                'w-full flex border-b last:border-b-0',
                'py-2 last:pb-0 first:pt-0 border-bombay dark:border-iron',
              )}
            >
              <div className="w-full flex items-center gap-2 justify-between">
                <span className="text-iron dark:text-bombay truncate select-none">
                  {t('app-proxy.active-connections')}
                </span>
                <span
                  className={clsx(
                    status?.state === 'connected'
                      ? 'text-malachite-moss dark:text-malachite'
                      : 'text-baltic-sea dark:text-white',
                  )}
                >
                  {status?.activeConnections ?? 0}
                </span>
              </div>
            </li>
          </ul>
        </div>
      </SettingsMenuCardBig>

      {/* SOCKS5 Proxy Info Card */}
      <ProxyInfoCard>
        <ProxyFieldSection
          title={t('app-proxy.socks5.proxy-title')}
          value={socks5Address}
          onValueChange={setSocks5Address}
          onCopy={() => copy(socks5Address, false)}
          disabled={isEnabled || isLoading}
          showInput={true}
        />

        {/* SOCKS5 URL (for apps) */}
        {isConnected && socks5Url && (
          <>
            <ProxyFieldSection
              title={t('app-proxy.socks5.url-title')}
              value={socks5Url}
              onCopy={() => copy(socks5Url, false)}
              showInput={false}
            />
            <ProxyInfoMessage message={t('app-proxy.socks5.info')} />
          </>
        )}
      </ProxyInfoCard>

      {/* HTTP RPC Proxy Info Card */}
      <ProxyInfoCard>
        <ProxyFieldSection
          title={t('app-proxy.http-rpc.proxy-title')}
          value={httpRpcAddress}
          onValueChange={setHttpRpcAddress}
          onCopy={() => copy(httpRpcAddress, false)}
          disabled={isEnabled || isLoading}
          showInput={true}
        />

        {/* HTTP RPC URL (for wallets) */}
        {isConnected && httpRpcUrl && (
          <>
            <ProxyFieldSection
              title={t('app-proxy.http-rpc.url-title')}
              value={httpRpcUrl}
              onCopy={() => copy(httpRpcUrl, false)}
              showInput={false}
            />
            <ProxyInfoMessage message={t('app-proxy.http-rpc.info')} />
          </>
        )}
      </ProxyInfoCard>
    </PageAnim>
  );
}

export default Socks5;
