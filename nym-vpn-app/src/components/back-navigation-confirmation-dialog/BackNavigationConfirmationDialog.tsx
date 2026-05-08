import { useCallback, useEffect, useState } from 'react';
import { useNavigate } from 'react-router';
import { useTranslation } from 'react-i18next';
import { ConfirmationDialog as ConfirmationDialogUI } from '../../ui';
import { useTopBar } from '../../contexts';

type ConfirmationDialogProps = {
  hasUnsavedChanges: boolean;
  onConfirm: () => Promise<void>;
  onCancel: () => void;
};

function BackNavigationConfirmationDialog({
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
    />
  );
}

export default BackNavigationConfirmationDialog;
