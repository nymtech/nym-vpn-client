import { DialogTitle } from '@headlessui/react';
import { useTranslation } from 'react-i18next';
import { type } from '@tauri-apps/plugin-os';
import { openUrl } from '@tauri-apps/plugin-opener';
import { Button, Dialog, MsIcon } from '../../ui';
import { DownloadAppUrl } from '../../constants';
import {
  networkUpdateBodyKey,
  networkUpdateDownloadUrl,
  networkUpdateTitleKey,
} from './networkUpdate';

export type Props = {
  isOpen: boolean;
  onClose: () => void;
  required?: boolean;
  // either app update is required
  appUpdate: boolean;
  // either daemon update is required
  daemonUpdate: boolean;
};

function NetworkUpdateDialog({
  isOpen,
  onClose,
  required = false,
  appUpdate,
  daemonUpdate,
}: Props) {
  const { t } = useTranslation('home');
  const os = type();

  const handleClose = () => {
    const downloadUrl = networkUpdateDownloadUrl(os, DownloadAppUrl);
    if (downloadUrl) {
      openUrl(downloadUrl);
    }
    if (!required) {
      onClose();
    }
  };

  const description = () => {
    if (os === 'linux' || os === 'macos' || os === 'windows') {
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
  };

  return (
    <Dialog
      open={isOpen}
      onClose={required ? () => undefined : onClose}
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
          {t(networkUpdateTitleKey(required))}
        </DialogTitle>
      </div>
      <p
        className="text-text-secondary"
        data-testid="update-dialog-description"
      >
        {required
          ? `${description() ?? ''} ${t(networkUpdateBodyKey(true))}`.trim()
          : t(networkUpdateBodyKey(false))}
      </p>
      <Button
        variant="primary"
        onClick={handleClose}
        className="mt-2"
        data-testid="update-dialog-button"
      >
        {t('update-dialog.button-update')}
      </Button>
    </Dialog>
  );
}

export default NetworkUpdateDialog;
