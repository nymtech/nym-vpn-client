import { useCallback, useEffect, useState } from 'react';
import clsx from 'clsx';
import { useNavigate } from 'react-router';
import { invoke } from '@tauri-apps/api/core';
import { useTranslation } from 'react-i18next';
import { usePrivy, useWallets } from '@privy-io/react-auth';
import { routes } from '../../../router';
import { StateDispatch } from '../../../types';
import { Button, ButtonProps } from '../../../ui';
import {
  useInAppNotify,
  useMainDispatch,
  useMainState,
} from '../../../contexts';

type PrivyLoginButtonProps = {
  outline: ButtonProps['outline'];
  color: ButtonProps['color'];
};

function PrivyLoginButton({ outline, color }: PrivyLoginButtonProps) {
  const { ready, authenticated, login } = usePrivy();
  const { wallets } = useWallets();
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

    if (!ready) {
      console.warn('Privy not ready');
      return;
    }

    try {
      login();
    } catch (e) {
      console.error('Privy login error:', e);
    }
  };

  const handlePrivyLogin = useCallback(async () => {
    const embeddedWallet = wallets.find(
      (w) =>
        w.walletClientType === 'privy' ||
        w.connectorType === 'privy' ||
        w.connectorType === 'embedded',
    );

    if (!embeddedWallet) throw new Error('No embedded wallet found');

    try {
      setLoading(true);

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
      console.error('Privy login error:', e);
      push({
        message: t('privy.error.login'),
        close: true,
        type: 'error',
      });
    } finally {
      setLoading(false);
    }
  }, [wallets, navigate, dispatch, push, t]);

  useEffect(() => {
    if (authenticated) {
      handlePrivyLogin().catch((e: unknown) =>
        console.error('Privy login handler error:', e),
      );
    }
  }, [authenticated, handlePrivyLogin]);

  return (
    <Button
      outline={outline}
      color={color}
      onClick={handleLogin}
      spinner={loading}
      disabled={loading}
      className={clsx(
        outline &&
          'group border border-iron dark:border-bombay hover:ring-0! dark:hover:ring-0!',
      )}
    >
      {outline ? (
        <span className="text-black dark:text-white group-hover:text-black/50 dark:group-hover:text-white/80">
          {t('social-account-button')}
        </span>
      ) : (
        <>{t('privy.login-button')}</>
      )}
    </Button>
  );
}

export default PrivyLoginButton;
