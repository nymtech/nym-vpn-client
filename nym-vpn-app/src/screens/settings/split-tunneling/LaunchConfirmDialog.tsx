import { DialogTitle } from '@headlessui/react';
import { useTranslation } from 'react-i18next';
import { Button, Dialog, MsIcon } from '../../../ui';

export type Props = {
  isOpen: boolean;
  appName: string;
  onConfirm: () => void;
  onCancel: () => void;
};

function LaunchConfirmDialog({ isOpen, appName, onConfirm, onCancel }: Props) {
  const { t } = useTranslation('settings');

  return (
    <Dialog open={isOpen} onClose={onCancel} className="flex flex-col gap-6">
      <div className="flex flex-col items-center gap-4">
        <MsIcon icon="info" className="text-text-primary" />
        <DialogTitle
          as="h3"
          className="text-xl font-medium text-text-primary text-center"
        >
          {t('split-tunneling.launch-confirm-dialog.title', { appName })}
        </DialogTitle>
      </div>

      <div className="text-sm text-text-secondary whitespace-pre-line">
        <p>{t('split-tunneling.launch-confirm-dialog.description')}</p>
      </div>

      <div className="flex flex-col gap-3">
        <Button onClick={onConfirm}>
          {t('split-tunneling.launch-confirm-dialog.launch')}
        </Button>
        <Button onClick={onCancel} color="gray" outline>
          {t('split-tunneling.launch-confirm-dialog.cancel')}
        </Button>
      </div>
    </Dialog>
  );
}

export default LaunchConfirmDialog;
