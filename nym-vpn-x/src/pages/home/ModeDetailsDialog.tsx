import {
  Dialog,
  DialogBackdrop,
  DialogPanel,
  DialogTitle,
} from '@headlessui/react';
import clsx from 'clsx';
import { useTranslation } from 'react-i18next';
import { Button, MsIcon } from '../../ui';
import { capitalizeFirst } from '../../helpers';

export type Props = {
  isOpen: boolean;
  onClose: () => void;
};

function ModeDetailsDialog({ isOpen, onClose }: Props) {
  const { t } = useTranslation('home');

  return (
    <Dialog
      as="div"
      className="relative z-50 focus:outline-none select-none cursor-default"
      open={isOpen}
      onClose={onClose}
    >
      <DialogBackdrop
        className={clsx([
          'fixed inset-0 bg-black/30',
          'duration-150 ease-in-out data-[closed]:opacity-0',
        ])}
      />
      <div className="fixed inset-0 z-50 w-screen overflow-y-auto">
        <div className="flex min-h-full items-center justify-center p-4 mx-4">
          <DialogPanel
            transition
            className={clsx([
              'text-base min-w-80',
              'w-full max-w-md rounded-xl bg-oil p-6',
              'flex flex-col items-center gap-6',
              'duration-150 ease-in-out data-[closed]:ease-out data-[closed]:scale-95 data-[closed]:opacity-0',
            ])}
          >
            <div className="flex flex-col items-center gap-4">
              <MsIcon icon="info" className="text-3xl text-mercury-pinkish" />
              <DialogTitle
                as="h3"
                className="text-lg text-mercury-pinkish font-bold"
              >
                {t('modes-dialog.title')}
              </DialogTitle>
            </div>
            <div className="flex flex-col gap-2">
              <div className="flex flex-row items-center text-white gap-2">
                <MsIcon icon="visibility_off" className="" />
                <h4 className="font-bold">
                  {t('vpn-modes.privacy', { ns: 'common' })}
                </h4>
              </div>
              <p className="text-laughing-jack md:text-nowrap">
                {t('modes-dialog.privacy-description')}
              </p>
            </div>
            <div className="flex flex-col gap-2">
              <div className="flex flex-row items-center text-white gap-2">
                <MsIcon icon="speed" className="" />
                <h4 className="font-bold">
                  {t('vpn-modes.fast', { ns: 'common' })}
                </h4>
              </div>
              <p className="text-laughing-jack">
                {t('modes-dialog.fast-description')}
              </p>
            </div>
            <Button onClick={onClose} className="mt-2">
              <span className="text-base text-baltic-sea">
                {capitalizeFirst(t('ok', { ns: 'glossary' }))}
              </span>
            </Button>
          </DialogPanel>
        </div>
      </div>
    </Dialog>
  );
}

export default ModeDetailsDialog;
