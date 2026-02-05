import { useEffect, useState } from 'react';
import clsx from 'clsx';
import { useTranslation } from 'react-i18next';
import { DialogTitle } from '@headlessui/react';
import { capFirst } from '../../util';
import { useMainState } from '../../contexts';
import { Button, Dialog, MsIcon, SettingsMenuCard } from '../../ui';
import { useLogout } from '../../hooks';

function Logout() {
  const [isOpen, setIsOpen] = useState(false);

  const { account } = useMainState();
  const { t } = useTranslation('settings');
  const { logout, loading } = useLogout();

  const logoutCopy = capFirst(t('logout', { ns: 'glossary' }));

  useEffect(() => {
    if (!loading && isOpen) {
      setIsOpen(false);
    }
  }, [loading, isOpen]);

  const handleLogout = async () => {
    await logout();
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
            <Button
              onClick={handleLogout}
              className="min-w-32"
              color="red"
              outline
            >
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
