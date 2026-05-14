import { DialogTitle } from '@headlessui/react';
import { useTranslation } from 'react-i18next';
import { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { ButtonIcon, ButtonNew, Dialog, MsIcon } from '../ui';
import { useAppStore } from '../store';

export function SystemAuthentication() {
  const daemonStatus = useAppStore((s) => s.daemonStatus);

  const { t } = useTranslation('system-authentication');

  const [isOpen, setIsOpen] = useState(false);
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    setIsOpen(daemonStatus === 'auth-denied');
  }, [daemonStatus]);

  const handleAuthenticate = async () => {
    setLoading(true);
    try {
      await invoke('retry_authentication');
    } catch (e: unknown) {
      console.error('retry_authentication failed', e);
    } finally {
      setLoading(false);
    }
  };

  return (
    <Dialog open={isOpen} onClose={() => setIsOpen(false)}>
      <div className="mx-auto flex flex-col items-center gap-6">
        <ButtonIcon
          className="self-end"
          color="chalk"
          icon="close"
          onClick={() => setIsOpen(false)}
        />
        <div className="bg-malachite-moss/10 border-malachite-moss flex items-center justify-center rounded-xl border p-3">
          <MsIcon icon="lock" className="text-primary leading-none" />
        </div>
        <DialogTitle
          as="h3"
          className="text-text-primary w-full truncate text-center text-xl"
        >
          {t('modal.title')}
        </DialogTitle>
        <div className="border-cheddar dark:border-king-nacho text-cheddar dark:text-king-nacho bg-cheddar/10 dark:bg-king-nacho/10 flex flex-row items-center gap-3 rounded-lg border p-3">
          <MsIcon icon="report" />
          <p>{t('modal.description')}</p>
        </div>
        <ButtonNew onClick={handleAuthenticate} loading={loading}>
          {t('modal.authenticate-button')}
        </ButtonNew>
      </div>
    </Dialog>
  );
}
