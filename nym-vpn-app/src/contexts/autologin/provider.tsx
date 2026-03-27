import { useCallback, useMemo, useRef, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { invoke } from '@tauri-apps/api/core';
import { TAutologinResponse } from '../../types/tauri';
import { useInAppNotify } from '../in-app-notification';
import { useDeepLink } from '../../hooks';
import { AutologinContext, AutologinKind } from './context';
import { PincodeDialog } from './PincodeDialog';

export function AutologinProvider({ children }: { children: React.ReactNode }) {
  const { i18n, t } = useTranslation('account');

  const [pinCode, setPinCode] = useState('');
  const [url, setUrl] = useState('');
  const [open, setOpen] = useState(false);

  const { push } = useInAppNotify();
  const { startListening } = useDeepLink();
  const timeoutIdRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  const autologin = useCallback(
    async (kind: AutologinKind) => {
      try {
        const response = await invoke<TAutologinResponse>(
          'get_autologin_deeplink',
          {
            locale: i18n.language,
            kind,
          },
        );

        setPinCode(response['pin-code']);
        setUrl(response.url);
        setOpen(true);

        const timeoutPromise = new Promise<never>((_, reject) => {
          timeoutIdRef.current = setTimeout(
            () => reject(new Error('Login timeout')),
            300000,
          );
        });

        await Promise.race([startListening(), timeoutPromise]);

        await invoke<void>('handle_subscription_payment');

        // close the dialog
        setOpen(false);
      } catch (error) {
        console.error('Failed to get autologin deeplink', error);
        push({
          message: t('autologin.initialization-error'),
          type: 'error',
          duration: 3000,
        });
      }
    },
    [i18n.language, startListening, push, t],
  );

  const ctx = useMemo(() => ({ autologin }), [autologin]);

  return (
    <AutologinContext.Provider value={ctx}>
      {children}
      <PincodeDialog code={pinCode} url={url} open={open} setOpen={setOpen} />
    </AutologinContext.Provider>
  );
}
