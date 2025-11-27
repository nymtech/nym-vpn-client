import { useEffect, useState } from 'react';
import { useTranslation } from 'react-i18next';
import clsx from 'clsx';
import {
  Button,
  CardSwitch,
  MsIcon,
  PageAnim,
  PulseDot,
  SettingsMenuCardBig,
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
  }, [status]);

  const isEnabled = !!status?.state && status?.state !== 'disabled';
  const isConnected = status?.state === 'idle' || status?.state === 'connected';
  const hasError = status?.state === 'error';
  const socks5Url = status?.socks5Settings?.listenAddress
    ? `socks5h://${status.socks5Settings.listenAddress}`
    : null;
  const httpRpcUrl = status?.httpRpcSettings?.listenAddress
    ? `http://${status.httpRpcSettings.listenAddress}?p=<your-provider-url>`
    : null;

  // enable/disable socks5
  const handleToggle = async () => {
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
    <PageAnim className="xs:max-w-lg h-full flex flex-col mt-2 gap-6 select-none">
      <div className="text-iron dark:text-bombay">{t('app-proxy.intro')}</div>

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

      <SettingsMenuCardBig
        header={
          <CardSwitch
            header={t('app-proxy.label')}
            subheader={t('app-proxy.description')}
            checked={isEnabled}
            onClick={handleToggle}
            disabled={isLoading}
          />
        }
      >
        {isLoading ? (
          <div className="flex flex-col items-center justify-center gap-3 py-8">
            <PulseDot color="cornflower" />
            <span className="text-sm text-iron dark:text-bombay">
              {isEnabled ? t('app-proxy.disabling') : t('app-proxy.enabling')}
            </span>
          </div>
        ) : (
          <div className="flex flex-col gap-4">
            <div className="flex items-center justify-between">
              <span className="text-sm text-iron dark:text-bombay">
                {t('app-proxy.status')}:
              </span>
              <span
                className={clsx(
                  'text-xs font-medium px-2 py-1 rounded',
                  'text-iron dark:text-bombay',
                  hasError && 'text-aphrodisiac dark:text-aphrodisiac',
                )}
              >
                {status ? status.state : 'unknown'}
              </span>
            </div>

            <div className="flex items-center justify-between">
              <span className="text-sm text-iron dark:text-bombay">
                {t('app-proxy.active-connections')}:
              </span>
              <span className="text-sm font-medium text-mine-shaft dark:text-mercury">
                {status?.activeConnections ?? 0}
              </span>
            </div>

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
        )}
      </SettingsMenuCardBig>
    </PageAnim>
  );
}

export default Socks5;
