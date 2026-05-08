import { DialogTitle } from '@headlessui/react';
import { useTranslation } from 'react-i18next';
import { type } from '@tauri-apps/plugin-os';
import { Button, Dialog, MsIcon } from '../../../ui';

export type Props = {
  isOpen: boolean;
  onClose: () => void;
};

function InfoDialog({ isOpen, onClose }: Props) {
  const { t } = useTranslation('settings');
  const os = type();

  return (
    <Dialog
      open={isOpen}
      onClose={onClose}
      className="flex flex-col gap-6"
      data-testid="split-tunneling-info-dialog"
    >
      {/* Icon + Title */}
      <div className="flex flex-col items-center gap-4">
        <MsIcon
          icon="info"
          className="text-text-primary"
          data-testid="split-tunneling-info-icon"
        />
        <DialogTitle
          as="h3"
          className="text-text-primary text-center text-xl font-medium"
          data-testid="split-tunneling-info-title"
        >
          {t('split-tunneling.info-dialog.title')}
        </DialogTitle>
      </div>

      {/* Body */}
      <div className="text-text-secondary flex flex-col gap-4 text-sm">
        <p className="whitespace-pre-line">
          {os === 'linux'
            ? t('split-tunneling.info-dialog.description-linux')
            : t('split-tunneling.info-dialog.description-windows')}
        </p>

        {/* Direct */}
        <div className="flex flex-col gap-2 text-sm">
          <div className="flex items-center gap-2">
            {os === 'linux' && (
              <div className="bg-primary h-2 w-2 rounded-full"></div>
            )}
            {os === 'windows' && (
              <MsIcon icon="block" className="text-text-primary" />
            )}
            <span className="text-text-primary font-bold">
              {t('split-tunneling.info-dialog.direct.label')}
            </span>
          </div>
          <p>{t('split-tunneling.info-dialog.direct.description')}</p>
        </div>

        {/* Via NymVPN */}
        <div className="flex flex-col gap-2 text-sm">
          <div className="flex items-center gap-2">
            {os === 'linux' && (
              <div className="bg-ash dark:bg-mercury h-2 w-2 rounded-full"></div>
            )}
            {os === 'windows' && (
              <MsIcon icon="shield" className="text-text-primary" />
            )}
            <span className="text-text-primary font-bold">
              {t('split-tunneling.info-dialog.via-nym-vpn.label')}
            </span>
          </div>
          <p>{t('split-tunneling.info-dialog.via-nym-vpn.description')}</p>
        </div>
      </div>

      {/* Button */}
      <Button onClick={onClose}>
        {t('split-tunneling.info-dialog.got-it')}
      </Button>
    </Dialog>
  );
}

export default InfoDialog;
