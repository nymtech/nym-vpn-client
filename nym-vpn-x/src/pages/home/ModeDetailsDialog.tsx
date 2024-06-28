import { Dialog, DialogPanel, DialogTitle } from '@headlessui/react';
import { Button } from '../../ui';
import { useTranslation } from 'react-i18next';
import { capitalizeFirst } from '../../helpers.ts';
import MsIcon from '../../ui/MsIcon.tsx';
import clsx from 'clsx';

export type Props = {
  isOpen: boolean;
  onClose: () => void;
};

function ModeDetailsDialog({ isOpen, onClose }: Props) {
  const { t } = useTranslation('home');

  return (
    <Dialog
      as="div"
      className="relative z-10 focus:outline-none"
      open={isOpen}
      onClose={onClose}
    >
      <div className="fixed inset-0 z-10 w-screen overflow-y-auto">
        <div className="flex min-h-full items-center justify-center p-4">
          <DialogPanel
            transition
            className="w-full max-w-md rounded-xl bg-white/5 p-6 backdrop-blur-2xl duration-300 ease-out data-[closed]:transform-[scale(95%)] data-[closed]:opacity-0"
          >
            <MsIcon icon="info" className={clsx([''])} />
            <DialogTitle as="h3" className="text-base/7 font-medium text-white">
              {t('modes-dialog.title')}
            </DialogTitle>
            <div>
              <div className="flex flex-row items-center">
                <MsIcon
                  icon="visibility_off"
                  className="text-baltic-sea dark:text-mercury-pinkish"
                />
                <h4>{t('vpn-modes.privacy', { ns: 'common' })}</h4>
              </div>
              <p>{t('modes-dialog.privacy-description')}</p>
            </div>
            <div>
              <div className="flex flex-row items-center">
                <MsIcon
                  icon="speed"
                  className="text-baltic-sea dark:text-mercury-pinkish"
                />
                <h4>{t('vpn-modes.fast', { ns: 'common' })}</h4>
              </div>
              <p>{t('modes-dialog.fast-description')}</p>
            </div>
            <Button onClick={onClose}>
              {capitalizeFirst(t('ok', { ns: 'glossary' }))}
            </Button>
          </DialogPanel>
        </div>
      </div>
    </Dialog>
  );
}

export default ModeDetailsDialog;
