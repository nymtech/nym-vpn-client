import { useCallback, useState } from 'react';
import { useTranslation } from 'react-i18next';
import clsx from 'clsx';
import { invoke } from '@tauri-apps/api/core';
import { Button, MsIcon } from '../ui';
import { describeError } from '../errors';

type ExportStatus = 'pending' | 'success' | 'cancelled' | 'error';

export type AppErrorProps = {
  error: unknown;
  onReload: () => void;
};

// Fatal error screen. Deliberately free of app context: it is rendered when
// the provider tree, the router or the startup sequence has already failed, so
// it may only rely on module-level singletons and direct Tauri commands.
function AppError({ error, onReload }: AppErrorProps) {
  const { t } = useTranslation('errors');
  const [exportStatus, setExportStatus] = useState<ExportStatus | null>(null);

  const details = describeError(error);

  const handleExportLogs = useCallback(async () => {
    setExportStatus('pending');
    try {
      // `zip_logs` opens a native save dialog and resolves false when the user
      // dismisses it, so a cancel must not be reported as a success
      const saved = await invoke<boolean>('zip_logs');
      setExportStatus(saved ? 'success' : 'cancelled');
    } catch (e) {
      console.error('failed to zip logs', e);
      setExportStatus('error');
    }
  }, []);

  return (
    <div
      className={clsx([
        'text-text-primary flex h-full min-w-64 flex-col',
        'cursor-default items-center justify-center gap-6 p-6 select-none',
      ])}
      data-testid="app-error-container"
    >
      <div className="flex flex-col items-center justify-center gap-2">
        <MsIcon
          className="text-status-error text-2xl font-medium"
          icon="error"
          data-testid="app-error-icon"
        />
        <h1
          className="text-xl leading-loose font-medium tracking-wider"
          data-testid="app-error-title"
        >
          {t('fatal.title')}
        </h1>
      </div>

      <p
        className="text-text-secondary max-w-md text-center text-sm"
        data-testid="app-error-description"
      >
        {t('fatal.description')}
      </p>

      {details && (
        <details className="w-full max-w-md" data-testid="app-error-details">
          <summary className="text-text-secondary cursor-pointer text-center text-xs">
            {t('fatal.details')}
          </summary>
          <pre
            className={clsx([
              'bg-surface-sunken mt-2 max-h-44 overflow-auto rounded-md p-2',
              'text-status-error cursor-auto text-xs break-words',
              'whitespace-pre-wrap select-text',
            ])}
          >
            {details}
          </pre>
        </details>
      )}

      <div className="flex w-full max-w-xs flex-col items-center gap-3">
        <div className="w-full" data-testid="app-error-reload-button">
          <Button variant="primary" onClick={onReload}>
            {t('fatal.reload')}
          </Button>
        </div>
        <div className="w-full" data-testid="app-error-export-button">
          <Button
            variant="outlined"
            onClick={() => void handleExportLogs()}
            loading={exportStatus === 'pending'}
          >
            {t('fatal.export.action')}
          </Button>
        </div>
        {exportStatus && exportStatus !== 'pending' && (
          <p
            className={clsx([
              'text-xs',
              exportStatus === 'error'
                ? 'text-status-error'
                : 'text-text-secondary',
            ])}
            role="status"
            data-testid="app-error-export-status"
          >
            {t(`fatal.export.${exportStatus}`)}
          </p>
        )}
      </div>
    </div>
  );
}

export default AppError;
