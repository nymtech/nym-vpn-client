import { useEffect, useState } from 'react';
import { useTranslation } from 'react-i18next';
import clsx from 'clsx';
import {
  Button,
  ButtonIcon,
  CardSwitch,
  MsIcon,
  PageAnim,
  SettingsMenuCardBig,
  TextInput,
} from '../../../ui';
import { useInAppNotify, useMainState, useSocks5 } from '../../../contexts';
import { useClipboard } from '../../../hooks';

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

  const getStatusString = () => {
    if (isLoading) {
      return 'Enabling...';
    }
    switch (status?.state) {
      case 'idle':
      case 'connected':
        return 'Connected';
      case 'error':
      case 'disabled':
        return 'Disabled';
      default:
        return 'Unknown';
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

  // copy to clipboard
  const handleCopy = async (url: string) => {
    if (!url) return;

    try {
      await copy(url, true);
    } catch (error) {
      console.error('Failed to copy:', error);
    }
  };

  return (
    <PageAnim className="h-full flex flex-col mt-2 gap-6 select-none">
      <div className="text-iron dark:text-bombay">{t('app-proxy.intro')}</div>

      <SettingsMenuCardBig
        header={
          <CardSwitch
            header="Enable proxy"
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
                  Proxy status:
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
                  Active connections:
                </span>
                <span
                  className={clsx(
                    status?.state === 'connected'
                      ? 'text-malachite'
                      : 'text-white',
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
      <div className="bg-charcoal rounded-xl flex flex-col gap-4">
        {/* SOCKS5 proxy (for apps) */}
        <div className="flex flex-col gap-2 border-b border-black p-4">
          <div className="flex items-center gap-2">
            <MsIcon icon="tag" className="text-bombay text-2xl" />
            <p className="text-white text-base font-medium">
              SOCKS5 proxy (for apps)
            </p>
          </div>
          <div className="flex items-center justify-between gap-4">
            {/* <p className="text-bombay font-mono text-sm">127.0.0.1:1080</p> */}
            <TextInput
              onChange={(e) => {
                setSocks5Address(e);
              }}
              disabled={isEnabled}
              value={socks5Address}
              color="default"
            />
            <ButtonIcon
              icon="content_copy"
              color="chalk"
              onClick={() => copy(socks5Address, false)}
              clickFeedback
              noDefaultSize
            />
          </div>
        </div>

        {/* SOCKS5 URL (for apps) */}
        {isConnected && socks5Url && (
          <div className="flex flex-col gap-2 border-b border-black p-4">
            <div className="flex items-center gap-2">
              <MsIcon icon="tag" className="text-bombay text-2xl" />
              <p className="text-white text-base font-medium">
                SOCKS5 URL (for apps)
              </p>
            </div>
            <div className="flex items-center justify-between">
              <p className="text-bombay font-mono text-sm">
                {/* socks5h://127.0.0.1:1080 */}
                {socks5Url}
              </p>
              <ButtonIcon
                icon="content_copy"
                color="chalk"
                // onClick={() => handleCopy('127.0.0.1:1080')}
                onClick={() => copy(socks5Url, false)}
                clickFeedback
                noDefaultSize
              />
            </div>
          </div>
        )}

        {/* Info message */}
        <div className="flex items-start gap-2 p-4">
          <span className="text-bombay text-sm">
            ℹ️ Add this to your browser's proxy settings to route traffic
            through the Nym mixnet
          </span>
        </div>
      </div>

      {/* SOCKS5 Proxy Info Card */}
      <div className="bg-charcoal rounded-xl flex flex-col gap-4">
        {/* HTTP RPC proxy (for wallets) */}
        <div className="flex flex-col gap-2 border-b border-black p-4">
          <div className="flex items-center gap-2">
            <MsIcon icon="tag" className="text-bombay text-2xl" />
            <p className="text-white text-base font-medium">
              HTTP RPC proxy (for wallets)
            </p>
          </div>
          <div className="flex items-center justify-between">
            <p className="text-bombay font-mono text-sm">127.0.0.1:8545</p>
            <ButtonIcon
              icon="content_copy"
              color="chalk"
              onClick={() => copy(httpRpcAddress, false)}
              clickFeedback
              noDefaultSize
            />
          </div>
        </div>

        {/* HTTP RPC URL (for wallets) */}
        {isConnected && httpRpcUrl && (
          <div className="flex flex-col gap-2 border-b border-black p-4">
            <div className="flex items-center gap-2">
              <MsIcon icon="tag" className="text-bombay text-2xl" />
              <p className="text-white text-base font-medium">
                HTTP RPC URL (for wallets)
              </p>
            </div>
            <div className="flex items-center justify-between">
              <p className="text-bombay font-mono text-sm">
                {/* {'http://127.0.0.1:8545?p=<your-provider-url>'} */}
                {httpRpcUrl}
              </p>
              <ButtonIcon
                icon="content_copy"
                color="chalk"
                // onClick={() => handleCopy('127.0.0.1:1080')}
                onClick={() => copy(httpRpcUrl, false)}
                clickFeedback
                noDefaultSize
              />
            </div>
          </div>
        )}

        {/* Info message */}
        <div className="flex items-start gap-2 p-4">
          <span className="text-bombay text-sm">
            {
              'ℹ️  Use this in MetaMask or other Web3 wallets to make RPC calls through the Nym mixnet. Replace <your-provider-url> with your actual RPC endpoint.'
            }
          </span>
        </div>
      </div>

      <SettingsMenuCardBig
        header={
          <CardSwitch
            header="Enable proxy"
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
              'bg-white dark:bg-charcoal rounded-lg px-4',
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
                  Proxy status:
                </span>
                <span
                  className={clsx(
                    status?.state === 'connected'
                      ? 'text-malachite'
                      : 'text-white',
                  )}
                >
                  {status ? status.state : 'unknown'}
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
                  Active connections:
                </span>
                <span
                  className={clsx(
                    status?.state === 'connected'
                      ? 'text-malachite'
                      : 'text-white',
                  )}
                >
                  {status?.activeConnections ?? 0}
                </span>
              </div>
            </li>
          </ul>
        </div>

        <div className="flex flex-col gap-4">
          {isConnected && socks5Url && (
            <div className="flex flex-col gap-4">
              {/* SOCKS5 URL Section */}
              <div className="flex flex-col gap-2">
                <div className="flex items-center gap-2">
                  <span className="text-sm font-medium text-mine-shaft dark:text-mercury">
                    SOCKS5 URL
                  </span>
                </div>
                <div className="flex items-center gap-2 bg-baltic dark:bg-shark rounded-lg p-3">
                  <code className="flex-1 text-sm font-mono text-mine-shaft dark:text-mercury break-words">
                    {socks5Url}
                  </code>
                  <Button
                    onClick={() => handleCopy(socks5Url)}
                    className="!w-8 !h-8 !p-0 !min-w-8 flex-shrink-0 flex items-center justify-center"
                  >
                    <MsIcon
                      icon="content_copy"
                      className="text-white dark:text-charcoal"
                    />
                  </Button>
                </div>
                <div className="flex items-start gap-2 mt-1">
                  <MsIcon
                    icon="info"
                    className="text-cornflower text-sm mt-0.5 flex-shrink-0"
                  />
                  <span className="text-xs text-iron dark:text-bombay">
                    {t('app-proxy.add-to-browser-proxy-settings')}
                  </span>
                </div>
              </div>

              {/* HTTP RPC URL Section */}
              {httpRpcUrl && (
                <div className="flex flex-col gap-2">
                  <div className="flex items-center gap-2">
                    <span className="text-sm font-medium text-mine-shaft dark:text-mercury">
                      HTTP RPC URL
                    </span>
                  </div>
                  <div className="flex items-center gap-2 bg-baltic dark:bg-shark rounded-lg p-3">
                    <code className="flex-1 text-sm font-mono text-mine-shaft dark:text-mercury break-words">
                      {httpRpcUrl}
                    </code>
                    <Button
                      onClick={() => handleCopy(httpRpcUrl)}
                      className="!w-8 !h-8 !p-0 !min-w-8 flex-shrink-0 flex items-center justify-center"
                    >
                      <MsIcon
                        icon="content_copy"
                        className="text-white dark:text-charcoal"
                      />
                    </Button>
                  </div>
                  <div className="flex items-start gap-2 mt-1">
                    <MsIcon
                      icon="info"
                      className="text-cornflower text-sm mt-0.5 flex-shrink-0"
                    />
                    <span className="text-xs text-iron dark:text-bombay">
                      {t('app-proxy.use-in-wallet')}
                    </span>
                  </div>
                </div>
              )}
            </div>
          )}
        </div>
      </SettingsMenuCardBig>

      <SettingsMenuCardBig
        header={t('app-proxy.configuration')}
        className="pt-4"
      >
        <div className="flex flex-col gap-4">
          <div className="flex flex-col gap-2">
            <label className="text-sm font-medium text-mine-shaft dark:text-mercury">
              {t('app-proxy.socks5-address-label')}
            </label>
            <input
              type="text"
              value={socks5Address}
              onChange={(e) => setSocks5Address(e.target.value)}
              disabled={isEnabled}
              placeholder={t('app-proxy.socks5-address-placeholder')}
              className={clsx(
                'px-3 py-2 bg-baltic rounded-lg text-sm font-mono',
                'dark:bg-shark text-mine-shaft dark:text-mercury border border-transparent',
                'focus:border-cornflower focus:outline-none disabled:opacity-50 disabled:cursor-not-allowed',
              )}
            />
          </div>

          <div className="flex flex-col gap-2">
            <label className="text-sm font-medium text-mine-shaft dark:text-mercury">
              {t('app-proxy.http-rpc-address-label')}
            </label>
            <input
              type="text"
              value={httpRpcAddress}
              onChange={(e) => setHttpRpcAddress(e.target.value)}
              disabled={isEnabled}
              placeholder={t('app-proxy.http-rpc-address-placeholder')}
              className={clsx(
                'px-3 py-2 bg-baltic rounded-lg text-sm font-mono',
                'dark:bg-shark text-mine-shaft dark:text-mercury border border-transparent',
                'focus:border-cornflower focus:outline-none disabled:opacity-50 disabled:cursor-not-allowed',
              )}
            />
          </div>

          {hasError && status?.errorMessage && (
            <div className="bg-malachite/10 border border-malabg-malachite rounded-lg p-3">
              <p className="text-sm text-aphrodisiac">{status.errorMessage}</p>
            </div>
          )}
        </div>
      </SettingsMenuCardBig>
    </PageAnim>
  );
}

export default Socks5;
