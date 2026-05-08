import { useCallback, useMemo, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { invoke } from '@tauri-apps/api/core';
import { TAutologinResponse } from '../../types/tauri';
import { useToast } from '../../hooks';
import { AutologinContext, AutologinKind } from './context';
import { PincodeDialog } from './PincodeDialog';

export function AutologinProvider({ children }: { children: React.ReactNode }) {
  const { i18n, t } = useTranslation('account');

  const [pinCode, setPinCode] = useState('');
  const [url, setUrl] = useState('');
  const [open, setOpen] = useState(false);

  const { add } = useToast();

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
      } catch (error) {
        console.error('Failed to get autologin deeplink', error);
        add({
          title: t('autologin.initialization-error', { ns: 'errors' }),
          description: t('autologin.initialization-error', { ns: 'errors' }),
          type: 'error',
        });
      }
    },
    [i18n.language, add, t],
  );

  const closeDialog = useCallback(() => {
    setOpen(false);
  }, []);

  const ctx = useMemo(
    () => ({ autologin, closeDialog }),
    [autologin, closeDialog],
  );

  return (
    <AutologinContext.Provider value={ctx}>
      {children}
      <PincodeDialog code={pinCode} url={url} open={open} setOpen={setOpen} />
    </AutologinContext.Provider>
  );
}
