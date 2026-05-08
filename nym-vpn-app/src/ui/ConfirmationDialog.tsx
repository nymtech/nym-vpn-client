import { DialogTitle } from '@headlessui/react';
import { useTranslation } from 'react-i18next';
import { ButtonNew, Dialog, MsIcon } from '.';

export type ConfirmationDialogProps = {
  icon: string;
  title: string;
  description: string;
  confirmButtonText: string;
  cancelButtonText?: string;
  isOpen: boolean;
  isLoading: boolean;
  onConfirm: () => Promise<void>;
  onCancel: () => void;
};

function ConfirmationDialog({
  icon,
  title,
  description,
  confirmButtonText,
  cancelButtonText,
  isOpen,
  isLoading,
  onConfirm,
  onCancel,
}: ConfirmationDialogProps) {
  const { t } = useTranslation('common');

  return (
    <Dialog open={isOpen} onClose={onCancel}>
      <div className="mx-auto flex w-11/12 flex-col items-center gap-4">
        <MsIcon icon={icon} className="text-text-primary" />

        <DialogTitle
          as="h3"
          className="text-text-primary w-full truncate text-center text-xl"
        >
          {title}
        </DialogTitle>
      </div>
      <p className="text-text-secondary mt-4 max-w-80 text-center whitespace-pre-line">
        {description}
      </p>
      <div className="mt-6 flex w-full flex-col flex-nowrap justify-center gap-2">
        <ButtonNew
          onClick={onConfirm}
          className="min-w-32"
          variant="primary"
          loading={isLoading}
          disabled={isLoading}
        >
          {confirmButtonText}
        </ButtonNew>
        <ButtonNew onClick={onCancel} className="min-w-32" variant="outlined">
          {cancelButtonText || t('cancel')}
        </ButtonNew>
      </div>
    </Dialog>
  );
}

export default ConfirmationDialog;
