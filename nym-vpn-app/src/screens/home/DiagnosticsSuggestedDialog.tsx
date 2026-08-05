import { useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { useTranslation } from 'react-i18next';
import { ConfirmationDialog } from '../../ui';
import { useToast } from '../../hooks';
import { dispatch, useAppStore } from '../../store';

function DiagnosticsSuggestedDialog() {
  const { t } = useTranslation('home');
  const { add } = useToast();
  const reason = useAppStore((s) => s.diagnosticsSuggestedReason);
  const [isLoading, setIsLoading] = useState(false);

  const dismiss = () => {
    dispatch({ type: 'set-diagnostics-suggested-reason', reason: null });
  };

  const handleRun = async () => {
    setIsLoading(true);
    try {
      const saved = await invoke<boolean>('share_diagnostics_and_logs');
      if (saved) {
        add({
          title: t('diagnostics-suggested.success', { ns: 'notifications' }),
          type: 'info',
        });
      }
    } catch (error) {
      console.error('failed to run diagnostics', error);
      add({
        title: t('diagnostics-suggested.error', { ns: 'notifications' }),
        type: 'error',
      });
    } finally {
      setIsLoading(false);
      dismiss();
    }
  };

  return (
    <ConfirmationDialog
      icon="troubleshoot"
      title={t('diagnostics-suggested.title')}
      description={t('diagnostics-suggested.description')}
      confirmButtonText={t('diagnostics-suggested.button-run')}
      cancelButtonText={t('diagnostics-suggested.button-dismiss')}
      isOpen={!!reason}
      isLoading={isLoading}
      onConfirm={handleRun}
      onCancel={dismiss}
    />
  );
}

export default DiagnosticsSuggestedDialog;
