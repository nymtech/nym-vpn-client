import { useState } from 'react';
import clsx from 'clsx';
import { useTranslation } from 'react-i18next';
import { useNavigate } from 'react-router';
import { invoke } from '@tauri-apps/api/core';
import { DialogTitle } from '@headlessui/react';
import { capFirst } from '../../util';
import { useInAppNotify, useMainDispatch, useMainState } from '../../contexts';
import { Button, Dialog, MsIcon, SettingsMenuCard } from '../../ui';
import { routes } from '../../router';
import { BackendError, StateDispatch } from '../../types';
import { useI18nError } from '../../hooks';
import { MCache } from '../../cache';

function Logout() {
  const [isOpen, setIsOpen] = useState(false);

  const { account, daemonStatus, state } = useMainState();
  const dispatch = useMainDispatch() as StateDispatch;
  const { t } = useTranslation('settings');
  const { tE } = useI18nError();
  const navigate = useNavigate();
  const { push } = useInAppNotify();
  const logoutCopy = capFirst(t('logout', { ns: 'glossary' }));

  const logout = async () => {
    setIsOpen(false);
    navigate(routes.root);
    if (state !== 'Disconnected') {
      console.warn(`cannot logout while tunnel state is ${state}`);
      push({
        text: t('logout.from-state', { ns: 'notifications', state }),
        position: 'top',
        autoHideDuration: 5000,
      });
      return;
    }

    try {
      await invoke('forget_account');
      dispatch({ type: 'set-account', stored: false });
      push({
        text: t('logout.success', { ns: 'notifications' }),
        position: 'top',
      });
      MCache.del('account-id');
      MCache.del('device-id');
      dispatch({ type: 'reset-error' });
    } catch (e) {
      console.warn('failed to logout', e);
      push({
        text: `${t('logout.error', { ns: 'notifications' })}: ${tE((e as BackendError).key || 'unknown')}`,
        position: 'top',
        autoHideDuration: 5000,
      });
    }
  };

  const onClose = () => {
    setIsOpen(false);
  };

  if (!account) {
    return null;
  }

  return (
    <>
      <SettingsMenuCard
        title={logoutCopy}
        onClick={() => setIsOpen(true)}
        disabled={daemonStatus === 'NotOk' || state !== 'Disconnected'}
      />
      <Dialog open={isOpen} onClose={onClose}>
        <div className="flex flex-col items-center gap-4 w-11/12">
          <MsIcon
            icon="info"
            className="text-3xl text-baltic-sea dark:text-mercury-pinkish"
          />
          <DialogTitle
            as="h3"
            className="text-lg text-baltic-sea dark:text-mercury-pinkish font-bold text-center w-full truncate"
          >
            {t('logout-confirmation.title')}
          </DialogTitle>
        </div>

        <p className="text-center text-cement-feet dark:text-laughing-jack md:text-nowrap max-w-80">
          {t('logout-confirmation.description')}
        </p>

        <div
          className={clsx(
            'flex flex-row flex-nowrap justify-center mt-2 w-full gap-3',
          )}
        >
          <Button onClick={onClose} className="min-w-32">
            {capFirst(t('cancel', { ns: 'glossary' }))}
          </Button>
          <Button onClick={logout} className="min-w-32" outline>
            {logoutCopy}
          </Button>
        </div>
      </Dialog>
    </>
  );
}

export default Logout;
