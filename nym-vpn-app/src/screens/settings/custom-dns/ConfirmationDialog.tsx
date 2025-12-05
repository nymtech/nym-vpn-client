import { useState } from 'react';
import { DialogTitle } from '@headlessui/react';
import { useTranslation } from 'react-i18next';
import { Button, Dialog, MsIcon } from '../../../ui';
import { useMainState } from '../../../contexts';

export function ConfirmationDialog({
  isOpen,
  onClose,
  onConfirm,
}: {
  isOpen: boolean;
  onClose: () => void;
  onConfirm: () => Promise<void>;
}) {
  const { t } = useTranslation('settings');
  const [isApplyingDns, setIsApplyingDns] = useState(false);
  const { state } = useMainState();

  const description =
    state === 'connected'
      ? t('dns.dialog.connected.description')
      : t('dns.dialog.disconnected.description');
  const apply =
    state === 'connected'
      ? t('dns.dialog.connected.apply')
      : t('dns.dialog.disconnected.apply');

  const handleConfirm = async () => {
    setIsApplyingDns(true);
    try {
      await onConfirm();
    } finally {
      setIsApplyingDns(false);
    }
  };

  return (
    <Dialog
      open={isOpen}
      onClose={onClose}
      className="flex flex-col items-center gap-6"
    >
      <div className="flex flex-col items-center gap-4 w-11/12">
        <MsIcon icon="dns" />

        <DialogTitle
          as="h3"
          className="text-xl text-baltic-sea dark:text-white text-center w-full truncate"
        >
          {t('dns.dialog.title')}
        </DialogTitle>
      </div>
      <p className="text-center text-iron dark:text-bombay max-w-80 whitespace-pre-line">
        {description}
      </p>
      <div className="flex flex-col flex-nowrap justify-center mt-2 w-full gap-3">
        <Button
          onClick={handleConfirm}
          className="min-w-32"
          color="malachite"
          spinner={isApplyingDns}
          disabled={isApplyingDns}
        >
          {apply}
        </Button>
        <Button onClick={onClose} className="min-w-32" color="gray" outline>
          {t('dns.dialog.cancel')}
        </Button>
      </div>
    </Dialog>
  );
}
