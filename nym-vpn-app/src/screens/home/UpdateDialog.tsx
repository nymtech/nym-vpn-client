import { DialogTitle } from '@headlessui/react';
import { useTranslation } from 'react-i18next';
import { type } from '@tauri-apps/plugin-os';
import { openUrl } from '@tauri-apps/plugin-opener';
import { Button, Dialog, MsIcon } from '../../ui';
import { DownloadAppUrl } from '../../constants';

export type Props = {
  isOpen: boolean;
  onClose: () => void;
  // either app update is required
  appUpdate: boolean;
  // either daemon update is required
  daemonUpdate: boolean;
};

function UpdateDialog({ isOpen, onClose, appUpdate, daemonUpdate }: Props) {
  const { t } = useTranslation('home');
  const os = type();

  const handleClose = () => {
    if (os === 'linux') {
      openUrl(`${DownloadAppUrl}/linux`);
    }
    if (os === 'windows') {
      openUrl(`${DownloadAppUrl}/windows`);
    }
    onClose();
  };

  const description = () => {
    if (os === 'linux') {
      if (appUpdate && daemonUpdate) {
        return t('update-dialog.description-1-other');
      }
      if (appUpdate) {
        return t('update-dialog.description-1-app');
      }
      if (daemonUpdate) {
        return t('update-dialog.description-1-daemon');
      }
    }

    if (os === 'windows') {
      return t('update-dialog.description-1-app');
    }
  };

  return (
    <Dialog
      open={isOpen}
      onClose={onClose}
      className="flex flex-col items-center gap-6"
    >
      <div className="flex flex-col items-center gap-4">
        <MsIcon
          icon="info"
          className="text-3xl text-baltic-sea dark:text-white"
        />
        <DialogTitle
          as="h3"
          className="text-xl text-baltic-sea dark:text-white"
        >
          {t('update-dialog.title')}
        </DialogTitle>
      </div>
      <p className="text-iron dark:text-bombay md:text-nowrap">
        {description()} {t('update-dialog.description-2')}
      </p>
      <Button onClick={handleClose} className="mt-2">
        <span className="text-lg text-black dark:text-baltic-sea">
          {t('update-dialog.button-update')}
        </span>
      </Button>
    </Dialog>
  );
}

export default UpdateDialog;
