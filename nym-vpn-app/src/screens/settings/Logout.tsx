import { useState } from 'react';
import clsx from 'clsx';
import { useTranslation } from 'react-i18next';
import { invoke } from '@tauri-apps/api/core';
import { DialogTitle } from '@headlessui/react';
import { capFirst } from '../../util';
import { useInAppNotify, useMainDispatch, useMainState } from '../../contexts';
import { Button, Dialog, MsIcon, SettingsMenuCard } from '../../ui';
import { BackendError, StateDispatch } from '../../types';
import { useI18nError } from '../../hooks';
import { CCache } from '../../cache';

function Logout() {
  const [isOpen, setIsOpen] = useState(false);
  const [loading, setLoading] = useState(false);

  const { account, daemonStatus, state } = useMainState();
  const dispatch = useMainDispatch() as StateDispatch;
  const { t } = useTranslation('settings');
  const { tE } = useI18nError();
  const { push } = useInAppNotify();
  const logoutCopy = capFirst(t('logout', { ns: 'glossary' }));

  const logout = async () => {
    if (state !== 'Disconnected') {
      console.warn(`cannot logout while tunnel state is ${state}`);
      push({
        message: t('logout.from-state', { ns: 'notifications', state }),
      });
      return;
    }

    setLoading(true);
    let hasFailed = false;
    try {
      console.info('logging out');
      await invoke('forget_account');
      dispatch({ type: 'set-account', stored: false });
      await CCache.del('cache-account-id');
      await CCache.del('cache-device-id');
      dispatch({ type: 'reset-error' });
    } catch (e) {
      hasFailed = true;
      console.warn('failed to logout', e);
      push({
        message: `${t('logout.error', { ns: 'notifications' })}: ${tE((e as BackendError).key || 'unknown')}`,
      });
    } finally {
      setIsOpen(false);
      setLoading(false);
    }
    if (!hasFailed) {
      push({
        message: t('logout.success', { ns: 'notifications' }),
      });
    }
  };

  const onClose = () => {
    if (loading) {
      return;
    }
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
        disabled={daemonStatus === 'down' || state !== 'Disconnected'}
      />
      <Dialog
        open={isOpen}
        onClose={onClose}
        className="flex flex-col items-center gap-6"
      >
        {loading ? (
          <>
            <div className="flex justify-center w-11/12">
              <MsIcon
                icon="pending"
                className="text-3xl text-baltic-sea dark:text-white"
              />
            </div>

            <p className="text-center text-iron dark:text-bombay md:text-nowrap max-w-80">
              {t('logout-confirmation.logging-out')}
            </p>
          </>
        ) : (
          <>
            <div className="flex flex-col items-center gap-4 w-11/12">
              <MsIcon
                icon="info"
                className="text-3xl text-baltic-sea dark:text-white"
              />
              <DialogTitle
                as="h3"
                className="text-xl text-baltic-sea dark:text-white text-center w-full truncate"
              >
                {t('logout-confirmation.title')}
              </DialogTitle>
            </div>

            <p className="text-center text-iron dark:text-bombay md:text-nowrap max-w-80">
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
          </>
        )}
      </Dialog>
    </>
  );
}

export default Logout;
