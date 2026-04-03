import { DialogTitle } from '@headlessui/react';
import { useTranslation } from 'react-i18next';
import { Button, Dialog, MsIcon } from '../../../ui';

export type Props = {
  isOpen: boolean;
  onClose: () => void;
};

function InfoDialog({ isOpen, onClose }: Props) {
  const { t } = useTranslation('settings');

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
          className="text-baltic-sea dark:text-white"
          data-testid="split-tunneling-info-icon"
        />
        <DialogTitle
          as="h3"
          className="text-xl font-medium text-baltic-sea dark:text-white text-center"
          data-testid="split-tunneling-info-title"
        >
          {t('split-tunneling.info-dialog.title')}
        </DialogTitle>
      </div>

      {/* Body */}
      <div className="flex flex-col gap-4 text-sm text-iron dark:text-bombay">
        <p className="whitespace-pre-line">
          {t('split-tunneling.info-dialog.description')}
        </p>

        {/* Direct */}
        <div className="flex flex-col gap-2 text-sm">
          <div className="flex items-center gap-1">
            <MsIcon icon="block" className=" text-baltic-sea dark:text-white" />
            <span className="font-bold text-baltic-sea dark:text-white">
              {t('split-tunneling.info-dialog.direct.label')}
            </span>
          </div>
          <p>{t('split-tunneling.info-dialog.direct.description')}</p>
        </div>

        {/* Via NymVPN */}
        <div className="flex flex-col gap-2 text-sm">
          <div className="flex items-center gap-1">
            <MsIcon
              icon="shield"
              className=" text-baltic-sea dark:text-white"
            />
            <span className="font-bold text-baltic-sea dark:text-white">
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
