import { useState, useEffect } from 'react';
import { useTranslation } from 'react-i18next';
import { useSocks5, useMainState } from '../../../contexts';
import { getSocks5StateLabel, Socks5State } from '../../../types';
import {
  Button,
  CardSwitch,
  MsIcon,
  PageAnim,
  SettingsMenuCardBig,
} from '../../../ui';
import { useInAppNotify } from '../../../contexts';
import { PulseDot } from '../../../ui';

function Socks5() {
  const { status, isLoading, enable, disable } = useSocks5();
  const { state: vpnState, exitNode } = useMainState();
  const { push } = useInAppNotify();
  const { t } = useTranslation('settings');
  const [isCopying, setIsCopying] = useState(false);

  // default listen addresses
  const [socks5Address, setSocks5Address] = useState('127.0.0.1:1080');
  const [httpRpcAddress, setHttpRpcAddress] = useState('127.0.0.1:8545');

  // sync input fields with actual values when status changes
  useEffect(() => {
    if (status?.socks5Settings?.listenAddress) {
      setSocks5Address(status.socks5Settings.listenAddress);
    }
    if (status?.httpRpcSettings?.listenAddress) {
      setHttpRpcAddress(status.httpRpcSettings.listenAddress);
    }
  }, [status]);

  const isEnabled =
    !!status?.state &&
    status?.state !== Socks5State.Disabled &&
    status?.state !== Socks5State.Unknown;
  const isConnected =
    status?.state === Socks5State.Idle ||
    status?.state === Socks5State.Connected;
  const hasError = status?.state === Socks5State.Error;
  const socks5Url = status?.socks5Settings?.listenAddress
    ? `socks5://${status.socks5Settings.listenAddress}`
    : null;
  const httpRpcUrl = status?.httpRpcSettings?.listenAddress
    ? `http://${status.httpRpcSettings.listenAddress}?p=<your-provider-url>`
    : null;

  // Check if VPN is connected in 5-hop mode (mixnet)
  const showDualModeWarning =
    isEnabled && (vpnState === 'connected' || vpnState === 'connecting');

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
          { listenAddress: socks5Address || '127.0.0.1:1080' },
          { listenAddress: httpRpcAddress || '127.0.0.1:8545' },
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
      push({
        id: 'socks5-error',
        message:
          error instanceof Error ? error.message : t('app-proxy.error-unknown'),
        duration: 5000,
        close: true,
      });
    }
  };

  // copy to clipboard
  const handleCopy = async (url: string, id: string) => {
    if (!url) return;

    try {
      await navigator.clipboard.writeText(url);
      setIsCopying(true);
      setTimeout(() => setIsCopying(false), 2000);
      push({
        id: `${id}-copied`,
        message: t('app-proxy.copied-to-clipboard'),
        duration: 2000,
        close: true,
      });
    } catch (error) {
      console.error('Failed to copy:', error);
    }
  };

  // get status color
  const getStatusColor = () => {
    if (hasError) return 'text-melon';
    if (isConnected) return 'text-green-500';
    return 'text-iron dark:text-bombay';
  };

  // get status badge
  const getStatusBadge = () => {
    const label = status ? getSocks5StateLabel(status.state) : 'Unknown';
    return (
      <span
        className={`text-xs font-medium px-2 py-1 rounded ${getStatusColor()}`}
      >
        {label}
      </span>
    );
  };

  return (
    <PageAnim className="xs:max-w-lg h-full flex flex-col mt-2 gap-6 select-none">
      <div className="text-iron dark:text-bombay">{t('app-proxy.intro')}</div>

      {showDualModeWarning && (
        <div className="bg-king-nacho/10 border border-king-nacho rounded-lg p-4">
          <div className="flex items-start gap-2">
            <MsIcon icon="warning" className="text-king-nacho mt-0.5" />
            <div className="text-sm text-iron dark:text-bombay">
              {t('app-proxy.dual-mode-warning')}
            </div>
          </div>
        </div>
      )}

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
              className="px-3 py-2 bg-baltic dark:bg-shark rounded-lg text-sm font-mono text-mine-shaft dark:text-mercury border border-transparent focus:border-cornflower focus:outline-none disabled:opacity-50 disabled:cursor-not-allowed"
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
              className="px-3 py-2 bg-baltic dark:bg-shark rounded-lg text-sm font-mono text-mine-shaft dark:text-mercury border border-transparent focus:border-cornflower focus:outline-none disabled:opacity-50 disabled:cursor-not-allowed"
            />
          </div>

          {hasError && status?.errorMessage && (
            <div className="bg-melon/10 border border-melon rounded-lg p-3">
              <p className="text-sm text-melon">{status.errorMessage}</p>
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
              {getStatusBadge()}
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
                      onClick={() => handleCopy(socks5Url, 'socks5')}
                      className="!w-8 !h-8 !p-0 !min-w-8 flex-shrink-0 flex items-center justify-center"
                      disabled={isCopying}
                    >
                      <MsIcon icon="content_copy" className="text-white" />
                    </Button>
                  </div>
                  <div className="flex items-start gap-2 mt-1">
                    <MsIcon
                      icon="info"
                      className="text-cornflower text-sm mt-0.5 flex-shrink-0"
                    />
                    <span className="text-xs text-iron dark:text-bombay">
                      Add this URL to your browser's proxy settings to route
                      traffic through the Nym mixnet
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
                        onClick={() => handleCopy(httpRpcUrl, 'http-rpc')}
                        className="!w-8 !h-8 !p-0 !min-w-8 flex-shrink-0 flex items-center justify-center"
                        disabled={isCopying}
                      >
                        <MsIcon icon="content_copy" className="text-white" />
                      </Button>
                    </div>
                    <div className="flex items-start gap-2 mt-1">
                      <MsIcon
                        icon="info"
                        className="text-cornflower text-sm mt-0.5 flex-shrink-0"
                      />
                      <span className="text-xs text-iron dark:text-bombay">
                        Use this URL in MetaMask or other Web3 wallets to make
                        RPC calls through the Nym mixnet. Replace{' '}
                        <code className="text-cornflower">
                          &lt;your-provider-url&gt;
                        </code>{' '}
                        with your actual provider URL.
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
