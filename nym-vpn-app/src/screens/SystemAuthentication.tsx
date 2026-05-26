import { DialogTitle } from '@headlessui/react';
import { useTranslation } from 'react-i18next';
import { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Button, ButtonIcon, Dialog, MsIcon } from '../ui';
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
        <div className="bg-brand-primary/10 border-brand-primary flex items-center justify-center rounded-xl border p-3">
          <MsIcon icon="lock" className="text-brand-primary leading-none" />
        </div>
        <DialogTitle
          as="h3"
          className="text-text-primary w-full truncate text-center text-xl"
        >
          {t('modal.title')}
        </DialogTitle>
        <div className="border-status-warning text-status-warning bg-status-warning/10 flex flex-row items-center gap-3 rounded-lg border p-3">
          <MsIcon icon="report" />
          <p>{t('modal.description')}</p>
        </div>
        <Button onClick={handleAuthenticate} loading={loading}>
          {t('modal.authenticate-button')}
        </Button>
      </div>
    </Dialog>
  );
}
