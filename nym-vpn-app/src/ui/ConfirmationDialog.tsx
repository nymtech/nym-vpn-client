import { DialogTitle } from '@headlessui/react';
import { useTranslation } from 'react-i18next';
import { Button, ButtonProps, Dialog, MsIcon } from '.';

export type ConfirmationDialogProps = {
  icon: string;
  title: string;
  description: string;
  confirmButtonText: string;
  confirmButtonColor: ButtonProps['color'];
  confirmButtonOutline: ButtonProps['outline'];
  cancelButtonText?: string;
  cancelButtonColor: ButtonProps['color'];
  cancelButtonOutline: ButtonProps['outline'];

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
  confirmButtonColor = 'malachite',
  confirmButtonOutline = false,
  cancelButtonText,
  cancelButtonColor = 'gray',
  cancelButtonOutline = true,
  isOpen,
  isLoading,
  onConfirm,
  onCancel,
}: ConfirmationDialogProps) {
  const { t } = useTranslation('common');

  return (
    <Dialog open={isOpen} onClose={onCancel}>
      <div className="flex flex-col items-center gap-4 w-11/12">
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
        <Button
          onClick={onConfirm}
          className="min-w-32"
          color={confirmButtonColor}
          outline={confirmButtonOutline}
          spinner={isLoading}
          disabled={isLoading}
        >
          {confirmButtonText}
        </Button>
        <Button
          onClick={onCancel}
          className="min-w-32"
          color={cancelButtonColor}
          outline={cancelButtonOutline}
        >
          {cancelButtonText || t('cancel')}
        </Button>
      </div>
    </Dialog>
  );
}

export default ConfirmationDialog;
