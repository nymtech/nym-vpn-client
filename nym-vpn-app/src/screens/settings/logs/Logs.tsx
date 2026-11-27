import { useCallback, useState } from 'react';
import { openPath } from '@tauri-apps/plugin-opener';
import { invoke } from '@tauri-apps/api/core';
import { useTranslation } from 'react-i18next';
import { Button, ConfirmationDialog, PageAnim } from '../../../ui';
import { useToast } from '../../../hooks';
import SettingsGroup from '../SettingsGroup';

function Logs() {
  const { t } = useTranslation('settings');
  const { add } = useToast();
  const [sharing, setSharing] = useState(false);
  const [deleteOpen, setDeleteOpen] = useState(false);
  const [deleting, setDeleting] = useState(false);

  // Desktop has no share sheet, so sharing saves a zip of the app and daemon
  // logs through a save dialog, as Share report does on the Diagnostic page
  const handleShareLogs = useCallback(async () => {
    setSharing(true);
    try {
      await invoke('zip_logs');
    } catch (error) {
      console.error('failed to zip logs', error);
      add({ title: t('logs.actions.export.error'), type: 'error' });
    } finally {
      setSharing(false);
    }
  }, [add, t]);

  const handleDeleteLogs = useCallback(async () => {
    setDeleting(true);
    try {
      await invoke('delete_logs');
      await invoke('delete_app_logs');
      add({ title: t('logs.actions.delete.success'), type: 'info' });
    } catch (error) {
      console.error('failed to delete logs', error);
      add({ title: t('logs.actions.delete.error'), type: 'error' });
    } finally {
      setDeleteOpen(false);
      setDeleting(false);
    }
  }, [add, t]);

  const handleAppLogs = async () => {
    try {
      const dir = await invoke<string | undefined>('log_dir');
      if (dir) {
        await openPath(dir);
      }
    } catch (e) {
      console.error(e);
    }
  };

  const handleDaemonLogs = async () => {
    try {
      const dir = await invoke<string | undefined>('vpnd_log_dir');
      if (dir) {
        await openPath(dir);
      }
    } catch (e) {
      console.error(e);
    }
  };

  return (
    <PageAnim
      className="mt-2 flex h-full flex-col gap-6"
      data-testid="logs-page"
    >
      <div className="flex flex-col gap-2">
        <Button onClick={handleShareLogs} disabled={sharing} loading={sharing}>
          {t('logs.actions.export.action-button')}
        </Button>
        <p className="text-text-secondary text-sm">
          {t('logs.actions.export.description')}
        </p>
      </div>
      <Button
        variant="destructive-outlined"
        onClick={() => setDeleteOpen(true)}
      >
        {t('logs.actions.delete.confirmButtonText')}
      </Button>
      <SettingsGroup
        settings={[
          {
            title: t('logs.app'),
            leadingIcon: 'sort',
            onClick: handleAppLogs,
            trailingIcon: 'open_in_new',
          },
          {
            title: t('logs.daemon'),
            leadingIcon: 'sort',
            onClick: handleDaemonLogs,
            trailingIcon: 'open_in_new',
          },
        ]}
      />
      {/* Deleting can't be undone, so it keeps its confirm step */}
      <ConfirmationDialog
        icon="delete"
        title={t('logs.actions.delete.title')}
        description={t('logs.actions.delete.description')}
        confirmButtonText={t('logs.actions.delete.confirmButtonText')}
        cancelButtonText={t('logs.actions.delete.cancelButtonText')}
        isOpen={deleteOpen}
        isLoading={deleting}
        onConfirm={handleDeleteLogs}
        onCancel={() => setDeleteOpen(false)}
      />
    </PageAnim>
  );
}

export default Logs;
