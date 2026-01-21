import { useCallback, useEffect, useState } from 'react';
import { useNavigate } from 'react-router';
import { DialogTitle } from '@headlessui/react';
import { useTranslation } from 'react-i18next';
import {
  Button,
  Dialog,
  MsIcon,
  ConfirmationDialog as ConfirmationDialogUI,
} from '../../ui';
import { useTopBar } from '../../contexts';

type ConfirmationDialogProps = {
  hasUnsavedChanges: boolean;
  onConfirm: () => Promise<void>;
  onCancel: () => void;
};

export function ConfirmationDialog({
  hasUnsavedChanges,
  onConfirm,
  onCancel,
}: ConfirmationDialogProps) {
  const { t } = useTranslation('settings');
  const { setCustomLeftNavHandler } = useTopBar();
  const navigate = useNavigate();

  const [isConfirmationDialogOpen, setIsConfirmationDialogOpen] =
    useState(false);
  const [isApplying, setIsApplying] = useState(false);

  const handleBackNavigation = useCallback(() => {
    if (hasUnsavedChanges) {
      setIsConfirmationDialogOpen(true);
    } else {
      navigate(-1);
    }
  }, [hasUnsavedChanges, navigate, setIsConfirmationDialogOpen]);

  useEffect(() => {
    setCustomLeftNavHandler(handleBackNavigation);
    return () => {
      setCustomLeftNavHandler(null);
    };
  }, [handleBackNavigation, setCustomLeftNavHandler]);

  const handleConfirm = async () => {
    setIsApplying(true);
    try {
      await onConfirm();
    } finally {
      setIsApplying(false);
    }
  };

  const handleCancel = () => {
    setIsConfirmationDialogOpen(false);
    onCancel();
  };

  return (
    <ConfirmationDialogUI
      isOpen={isConfirmationDialogOpen}
      isLoading={isApplying}
      onConfirm={handleConfirm}
      onCancel={handleCancel}
      icon="settings"
      title={t('confirmation-dialog.title')}
      description={t('confirmation-dialog.description')}
      confirmButtonText={t('confirmation-dialog.save')}
      confirmButtonColor="malachite"
      confirmButtonOutline={false}
      cancelButtonColor={undefined}
      cancelButtonOutline={undefined}
    />
  );

  // return (
  //   <Dialog
  //     open={isConfirmationDialogOpen}
  //     onClose={() => setIsConfirmationDialogOpen(false)}
  //   >
  //     <div className="flex flex-col items-center gap-4 w-11/12">
  //       <MsIcon icon="settings" className="text-baltic-sea dark:text-white" />

  //       <DialogTitle
  //         as="h3"
  //         className="text-xl text-baltic-sea dark:text-white text-center w-full truncate"
  //       >
  //         {t('confirmation-dialog.title')}
  //       </DialogTitle>
  //     </div>
  //     <p className="mt-4 text-center text-iron dark:text-bombay max-w-80 whitespace-pre-line">
  //       {t('confirmation-dialog.description')}
  //     </p>
  //     <div className="mt-6 flex flex-col flex-nowrap justify-center w-full gap-2">
  //       <Button
  //         onClick={handleConfirm}
  //         className="min-w-32"
  //         color="malachite"
  //         spinner={isApplying}
  //         disabled={isApplying}
  //       >
  //         {t('confirmation-dialog.save')}
  //       </Button>
  //       <Button
  //         onClick={handleCancel}
  //         className="min-w-32 text-aphrodisiac!"
  //         color="gray"
  //         outline
  //       >
  //         {t('confirmation-dialog.cancel')}
  //       </Button>
  //     </div>
  //   </Dialog>
  // );
}
