import { useEffect, useState } from 'react';
import { useTranslation } from 'react-i18next';
import clsx from 'clsx';
import { CardSwitch, PageAnim, SettingsMenuCardBig } from '../../../ui';
import { useMainState, useSocks5 } from '../../../store';
import { useToast } from '../../../hooks/index';
import ProxyInfoCard from './ProxyInfoCard';
import { ProxyInfo, ProxyPortInput, ProxyUrl } from './components';

const DefaultSocks5Port = '1080';
const DefaultSocks5Address = '127.0.0.1';
const DefaultHttpRpcPort = '8545';
const DefaultHttpRpcAddress = '127.0.0.1';

function Socks5() {
  const { status, isLoading, enable, disable } = useSocks5();
  const { exitNode } = useMainState();
  const { add } = useToast();
  const { t } = useTranslation('settings');

  const [socks5Address, setSocks5Address] = useState(DefaultSocks5Address);
  const [socks5Port, setSocks5Port] = useState(DefaultSocks5Port);
  const [httpRpcAddress, setHttpRpcAddress] = useState(DefaultHttpRpcAddress);
  const [httpRpcPort, setHttpRpcPort] = useState(DefaultHttpRpcPort);

  const [socks5PortValid, setSocks5PortValid] = useState(true);
  const [httpRpcPortValid, setHttpRpcPortValid] = useState(true);
  const portValid = socks5PortValid && httpRpcPortValid;

  useEffect(() => {
    const [socks5Address, socks5Port] =
      status?.socks5Settings?.listenAddress?.split(':') || [];
    setSocks5Address(socks5Address || DefaultSocks5Address);
    setSocks5Port(socks5Port || DefaultSocks5Port);

    const [httpRpcAddress, httpRpcPort] =
      status?.httpRpcSettings?.listenAddress?.split(':') || [];
    setHttpRpcAddress(httpRpcAddress || DefaultHttpRpcAddress);
    setHttpRpcPort(httpRpcPort || DefaultHttpRpcPort);
  }, [
    status?.socks5Settings?.listenAddress,
    status?.httpRpcSettings?.listenAddress,
  ]);

  const isEnabled = !!status?.state && status?.state !== 'disabled';
  const isConnected = status?.state === 'idle' || status?.state === 'connected';
  const hasError = status?.state === 'error';
  const socks5Url = `socks5h://${socks5Address}:${socks5Port}`;
  const httpRpcUrl = `http://${httpRpcAddress}:${httpRpcPort}?p=<your-provider-url>`;

  useEffect(() => {
    if (hasError) {
      add({
        id: 'socks5-error',
        title: status?.errorMessage ?? t('app-proxy.error-unknown'),
        type: 'error',
      });
    }
  }, [hasError, status?.errorMessage, add, t]);

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
      return 'text-text-primary';
    }
    switch (status?.state) {
      case 'idle':
      case 'connected':
        return 'text-primary';
      case 'error':
      case 'disabled':
        return 'text-aphrodisiac';
      default:
        return 'text-text-primary';
    }
  };

  const handleToggle = async () => {
    if (isLoading) return;

    try {
      if (isEnabled) {
        await disable();
        add({
          id: 'socks5-disabled',
          title: t('app-proxy.snackbar-disabled'),
          type: 'info',
        });
      } else {
        await enable(
          { listenAddress: `${socks5Address}:${socks5Port}` },
          { listenAddress: `${httpRpcAddress}:${httpRpcPort}` },
          exitNode,
        );
        add({
          id: 'socks5-enabled',
          title: t('app-proxy.snackbar-enabled'),
          type: 'info',
        });
      }
    } catch (error) {
      const explicitErrorMessage = String((error as Error)?.message) || '';

      if (explicitErrorMessage.includes('Gateway does not support')) {
        add({
          id: 'socks5-error',
          title: t('app-proxy.error-gateway-not-supported'),
          type: 'error',
        });
      } else {
        add({
          id: 'socks5-error',
          title: t('app-proxy.error-unknown'),
          type: 'error',
        });
      }
    }
  };

  return (
    <PageAnim className="mt-2 flex h-full flex-col gap-6 select-none">
      <div className="text-text-secondary">{t('app-proxy.intro')}</div>

      <SettingsMenuCardBig
        header={
          <CardSwitch
            checked={isEnabled}
            onClick={handleToggle}
            header={t('app-proxy.switch-title')}
            disabled={isLoading || !portValid}
          />
        }
      >
        <div>
          <ul
            className={clsx([
              'flex flex-col items-center justify-center gap-0',
              'dark:bg-charcoal bg-white',
              'cursor-default',
            ])}
          >
            <li
              className={clsx(
                'flex w-full border-b last:border-b-0',
                'border-bombay dark:border-iron py-2 first:pt-0 last:pb-0',
              )}
            >
              <div className="flex w-full items-center justify-between gap-2">
                <span className="text-text-secondary truncate select-none">
                  {t('app-proxy.proxy-status')}
                </span>
                <span className={clsx(getStatusColor())}>
                  {getStatusString()}
                </span>
              </div>
            </li>
            <li
              className={clsx(
                'flex w-full border-b last:border-b-0',
                'border-bombay dark:border-iron py-2 first:pt-0 last:pb-0',
              )}
            >
              <div className="flex w-full items-center justify-between gap-2">
                <span className="text-text-secondary truncate select-none">
                  {t('app-proxy.active-connections')}
                </span>
                <span
                  className={clsx(
                    status?.state === 'connected'
                      ? 'text-primary'
                      : 'text-text-primary',
                  )}
                >
                  {status?.activeConnections ?? 0}
                </span>
              </div>
            </li>
          </ul>
        </div>
      </SettingsMenuCardBig>
      <ProxyInfoCard title={t('app-proxy.socks5.proxy-title')}>
        <div className="flex flex-col gap-4">
          <ProxyPortInput
            value={socks5Port}
            defaultValue={DefaultSocks5Port}
            disabled={isEnabled || isLoading}
            onChange={(value, valid) => {
              setSocks5Port(value);
              setSocks5PortValid(valid);
            }}
          />
          <ProxyUrl
            value={`${socks5Address}:${socks5Port}`}
            title={t('app-proxy.socks5.listen-address')}
            borderBottom={isConnected}
          />
          {isConnected && (
            <>
              <ProxyUrl
                value={socks5Url}
                title={t('app-proxy.socks5.url-title')}
                borderBottom={isConnected}
              />
              <ProxyInfo text={t('app-proxy.socks5.info')} />
            </>
          )}
        </div>
      </ProxyInfoCard>

      <ProxyInfoCard title={t('app-proxy.http-rpc.proxy-title')}>
        <div className="flex flex-col gap-4">
          <ProxyPortInput
            value={httpRpcPort}
            defaultValue={DefaultHttpRpcPort}
            disabled={isEnabled || isLoading}
            onChange={(value, valid) => {
              setHttpRpcPort(value);
              setHttpRpcPortValid(valid);
            }}
          />
          <ProxyUrl
            value={`${httpRpcAddress}:${httpRpcPort}`}
            title={t('app-proxy.http-rpc.listen-address')}
            borderBottom={isConnected}
          />
          {isConnected && (
            <>
              <ProxyUrl
                value={httpRpcUrl}
                title={t('app-proxy.http-rpc.url-title')}
                borderBottom={isConnected}
              />
              <ProxyInfo text={t('app-proxy.http-rpc.info')} />
            </>
          )}
        </div>
      </ProxyInfoCard>
    </PageAnim>
  );
}

export default Socks5;
