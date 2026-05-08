import { DialogTitle } from '@headlessui/react';
import { useTranslation } from 'react-i18next';
import { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Button, ButtonIcon, Dialog, MsIcon } from '../ui';
import { useAppStore } from '../store';

export function SystemAuthentication() {
  const daemonStatus = useAppStore((s) => s.daemonStatus);
  console.log('[SystemAuthentication] daemonStatus', daemonStatus);

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
        <div className="flex items-center justify-center p-3 bg-malachite-moss/10 rounded-xl border border-malachite-moss">
          <MsIcon
            icon="lock"
            className="text-primary leading-none"
          />
        </div>
        <DialogTitle
          as="h3"
          className="text-xl text-text-primary text-center w-full truncate"
        >
          {t('modal.title')}
        </DialogTitle>
        <div className="p-3 flex flex-row items-center gap-3 border border-cheddar dark:border-king-nacho rounded-lg text-cheddar dark:text-king-nacho bg-cheddar/10 dark:bg-king-nacho/10">
          <MsIcon icon="report" />
          <p>{t('modal.description')}</p>
        </div>
        <Button
          onClick={handleAuthenticate}
          spinner={loading}
          disabled={loading}
        >
          {t('modal.authenticate-button')}
        </Button>
      </div>
    </Dialog>
  );
}
