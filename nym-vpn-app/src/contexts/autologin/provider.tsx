import { useCallback, useMemo, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { openUrl } from '@tauri-apps/plugin-opener';
import { invoke } from '@tauri-apps/api/core';
import { TAutologinResponse } from '../../types/tauri';
import { useInAppNotify } from '../in-app-notification';
import { AutologinContext, AutologinKind } from './context';
import { PincodeDialog } from './PincodeDialog';

export function AutologinProvider({ children }: { children: React.ReactNode }) {
  const { i18n, t } = useTranslation('account');

  const [pinCode, setPinCode] = useState('');
  const [open, setOpen] = useState(false);

  const { push } = useInAppNotify();

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
        setOpen(true);

        openUrl(response.url);
      } catch (error) {
        console.error('Failed to get autologin deeplink', error);
        push({
          message: t('autologin.initialization-error'),
          type: 'error',
          duration: 3000,
        });
      }
    },
    [i18n.language, t, push],
  );

  const ctx = useMemo(() => ({ autologin }), [autologin]);

  return (
    <AutologinContext.Provider value={ctx}>
      {children}
      <PincodeDialog code={pinCode} open={open} setOpen={setOpen} />
    </AutologinContext.Provider>
  );
}
