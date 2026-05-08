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
      <div className="mx-auto flex flex-col items-center gap-4 w-11/12">
        <MsIcon icon={icon} className="text-baltic-sea dark:text-white" />

        <DialogTitle
          as="h3"
          className="text-xl text-baltic-sea dark:text-white text-center w-full truncate"
        >
          {title}
        </DialogTitle>
      </div>
      <p className="mt-4 text-center text-iron dark:text-bombay max-w-80 whitespace-pre-line">
        {description}
      </p>
      <div className="mt-6 flex flex-col flex-nowrap justify-center w-full gap-2">
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
