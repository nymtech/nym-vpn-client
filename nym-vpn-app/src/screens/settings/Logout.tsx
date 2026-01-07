import { useEffect, useState } from 'react';
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
  const [loggingOut, setLoggingOut] = useState(false);

  const { account, state } = useMainState();
  const dispatch = useMainDispatch() as StateDispatch;
  const { t } = useTranslation('settings');
  const { tE } = useI18nError();
  const { push } = useInAppNotify();

  const logoutCopy = capFirst(t('logout', { ns: 'glossary' }));

  useEffect(() => {
    if (!loggingOut) return;

    if (state === 'disconnected') {
      setLoggingOut(false);
      (async () => {
        try {
          console.info('logging out');
          await invoke('forget_account');
          dispatch({ type: 'set-account', stored: false });
          await CCache.del('cache-account-id');
          await CCache.del('cache-device-id');
          dispatch({ type: 'reset-error' });

          push({
            message: t('logout.success', { ns: 'notifications' }),
          });
        } catch (e) {
          console.error('[logout] error', e);
          push({
            message: `${t('logout.error', { ns: 'notifications' })}: ${tE((e as BackendError).key || 'unknown')}`,
          });
        } finally {
          setIsOpen(false);
          setLoading(false);
        }
      })();
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [state, loggingOut]);

  const logout = async () => {
    setLoading(true);
    setLoggingOut(true);

    if (
      state === 'connected' ||
      state === 'connecting' ||
      state === 'offline-auto-reconnect' ||
      state === 'error'
    ) {
      try {
        dispatch({ type: 'disconnect' });
        await invoke('disconnect');
      } catch (e: unknown) {
        console.error('[logout] disconnect error', e);
        setIsOpen(false);
        setLoading(false);
        setLoggingOut(false);
        push({
          message: `${t('logout.error', { ns: 'notifications' })}: ${tE((e as BackendError).key || 'unknown')}`,
        });
      }
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
        color="red"
        title={logoutCopy}
        onClick={() => setIsOpen(true)}
      />
      <Dialog
        open={isOpen}
        onClose={onClose}
        className="flex flex-col items-center gap-6"
      >
        <div className="flex flex-col items-center gap-4 w-11/12">
          <MsIcon
            icon="logout"
            className="text-3xl text-baltic-sea dark:text-white"
          />
          <DialogTitle
            as="h3"
            className="text-xl text-baltic-sea dark:text-white text-center w-full truncate"
          >
            {t('logout-confirmation.title')}
          </DialogTitle>
        </div>

        <p className="text-center text-iron dark:text-bombay max-w-80">
          {t('logout-confirmation.description')}
        </p>

        {loading ? (
          <div className="flex flex-row items-center justify-center gap-4">
            <p className="text-center text-baltic-sea dark:text-white max-w-80">
              {t('logout-confirmation.logging-out')}
            </p>
            <MsIcon
              icon="progress_activity"
              className="text-cheddar dark:text-king-nacho animate-spin leading-none"
            />
          </div>
        ) : (
          <div
            className={clsx(
              'flex flex-col flex-nowrap justify-center mt-2 w-full gap-3',
            )}
          >
            <Button onClick={logout} className="min-w-32" color="red" outline>
              {logoutCopy}
            </Button>
            <Button onClick={onClose} className="min-w-32" outline color="gray">
              {capFirst(t('cancel', { ns: 'glossary' }))}
            </Button>
          </div>
        )}
      </Dialog>
    </>
  );
}

export default Logout;
