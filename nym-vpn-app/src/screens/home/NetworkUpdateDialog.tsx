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

function NetworkUpdateDialog({
  isOpen,
  onClose,
  appUpdate,
  daemonUpdate,
}: Props) {
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
      data-testid="update-dialog"
    >
      <div className="flex flex-col items-center gap-4">
        <MsIcon
          icon="info"
          className="text-text-primary text-3xl"
          data-testid="update-dialog-info-icon"
        />
        <DialogTitle
          as="h3"
          className="text-text-primary text-xl"
          data-testid="update-dialog-title"
        >
          {t('update-dialog.title')}
        </DialogTitle>
      </div>
      <p
        className="text-text-secondary"
        data-testid="update-dialog-description"
      >
        {description()} {t('update-dialog.description-2')}
      </p>
      <Button
        onClick={handleClose}
        className="mt-2"
        data-testid="update-dialog-button"
      >
        <span className="dark:text-baltic-sea text-lg text-black">
          {t('update-dialog.button-update')}
        </span>
      </Button>
    </Dialog>
  );
}

export default NetworkUpdateDialog;
