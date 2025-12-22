import { useEffect, useState } from 'react';
import { useNavigate } from 'react-router';
import { invoke } from '@tauri-apps/api/core';
import { useTranslation } from 'react-i18next';
import { usePrivy, useWallets } from '@privy-io/react-auth';
import { routes } from '../../../router';
import { StateDispatch } from '../../../types';
import { Button } from '../../../ui';
import {
  useInAppNotify,
  useMainDispatch,
  useMainState,
} from '../../../contexts';

function PrivyLoginButton() {
  const { ready: privyReady, authenticated, login } = usePrivy();
  const { wallets, ready: walletsReady } = useWallets();
  const navigate = useNavigate();

  const { state } = useMainState();
  const dispatch = useMainDispatch() as StateDispatch;
  const { push } = useInAppNotify();
  const { t } = useTranslation('login');

  const [loading, setLoading] = useState(false);

  const handleLogin = () => {
    if (state !== 'disconnected') {
      console.warn(`cannot login while tunnel state is ${state}`);
      return;
    }

    try {
      login();
    } catch (e) {
      console.error('[handleLogin] Privy login error:', e);
      push({
        message: t('privy.error.login'),
        close: true,
        type: 'error',
      });
    }
  };

  const handlePrivyLogin = async () => {
    if (loading) return;
    setLoading(true);

    try {
      const embeddedWallet = wallets.find(
        (w) =>
          w.walletClientType === 'privy' ||
          w.connectorType === 'privy' ||
          w.connectorType === 'embedded',
      );

      if (!embeddedWallet) throw new Error('No embedded wallet found');

      const message = await invoke<string>('get_privy_derivation_message');
      const hashToSign = `0x${message}`;
      const provider = await embeddedWallet.getEthereumProvider();

      const signatureHex = (await provider.request({
        method: 'secp256k1_sign',
        params: [hashToSign],
      })) as string;
      await invoke<number | null>('add_account', {
        signature: signatureHex.slice(2),
      });

      navigate(routes.root);
      dispatch({ type: 'set-account', stored: true });
      push({
        message: t('notification.added'),
        close: true,
      });
    } catch (e) {
      console.error('[handlePrivyLogin Privy login error:', e);
      push({
        message: t('privy.error.login'),
        close: true,
        type: 'error',
      });
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    if (privyReady && authenticated && walletsReady && wallets.length > 0) {
      handlePrivyLogin();
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [privyReady, authenticated, walletsReady, wallets]);

  return (
    <Button
      outline
      color="gray"
      onClick={handleLogin}
      spinner={loading}
      disabled={loading}
      className={
        'group border border-iron dark:border-bombay hover:ring-0! dark:hover:ring-0!'
      }
    >
      <span className="text-black dark:text-white group-hover:text-black/50 dark:group-hover:text-white/80">
        {t('social-account-button')}
      </span>
    </Button>
  );
}

export default PrivyLoginButton;
