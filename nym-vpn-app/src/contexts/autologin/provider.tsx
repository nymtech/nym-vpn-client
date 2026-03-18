import { useCallback, useMemo, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { openUrl } from '@tauri-apps/plugin-opener';
import { invoke } from '@tauri-apps/api/core';
import { TAutologinResponse } from '../../types/tauri';
import { AutologinContext, AutologinKind } from './context';
import { PincodeDialog } from './PincodeDialog';

export function AutologinProvider({ children }: { children: React.ReactNode }) {
  const { i18n } = useTranslation();

  const [pinCode, setPinCode] = useState('');
  const [open, setOpen] = useState(false);
  const [autologinLoading, setAutologinLoading] = useState(false);

  const autologin = useCallback(
    async (kind: AutologinKind) => {
      setAutologinLoading(true);
      const response = await invoke<TAutologinResponse>(
        'get_autologin_deeplink',
        {
          locale: i18n.language,
          kind,
        },
      );
      console.log('pincode', response['pin-code']);
      console.log('url', response.url);

      setPinCode(response['pin-code']);
      setOpen(true);

      // dev
      // openUrl(
      //   response.url.replace(
      //     'https://nymcom-git-deploy-sandbox-nyx-network-staging.vercel.app',
      //     'http://localhost:3000',
      //   ),
      // );

      // // prod
      openUrl(response.url);
      setAutologinLoading(false);
    },
    [i18n.language],
  );

  const ctx = useMemo(
    () => ({ autologin, autologinLoading }),
    [autologin, autologinLoading],
  );

  return (
    <AutologinContext.Provider value={ctx}>
      <PincodeDialog code={pinCode} open={open} setOpen={setOpen} />
      {children}
    </AutologinContext.Provider>
  );
}
