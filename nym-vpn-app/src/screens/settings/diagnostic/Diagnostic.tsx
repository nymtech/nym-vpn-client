import { useCallback, useEffect, useRef, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { useTranslation } from 'react-i18next';
import { Button, ButtonIconNew, PageAnim, Spinner } from '../../../ui';
import { useClipboard } from '../../../hooks';

function Diagnostic() {
  const { t } = useTranslation('settings');
  const { copy } = useClipboard();

  // Starts true: the run begins as soon as the page opens
  const [diagnosticRunning, setDiagnosticRunning] = useState(true);
  const [diagnosticResult, setDiagnosticResult] = useState<string | null>(null);
  // Every finished run leaves either a report or nothing, so failure is derived, not stored
  const diagnosticFailed = !diagnosticRunning && !diagnosticResult;
  const [shareLoading, setShareLoading] = useState(false);
  const startedRef = useRef(false);

  const handleRunDiagnostic = useCallback(async () => {
    setDiagnosticRunning(true);
    setDiagnosticResult(null);
    try {
      const report = await invoke('run_diagnostic', {
        params: {
          gateway: null,
          skipDns: false,
          skipHttp: false,
          skipHybridTransport: false,
        },
      });
      setDiagnosticResult(JSON.stringify(report, null, 2));
    } catch (error) {
      console.error('failed to run diagnostic', error);
    } finally {
      setDiagnosticRunning(false);
    }
  }, []);

  // StrictMode runs effects twice in dev; the ref keeps it to one run per visit
  useEffect(() => {
    if (startedRef.current) return;
    startedRef.current = true;
    handleRunDiagnostic();
  }, [handleRunDiagnostic]);

  const handleShareReport = async () => {
    if (!diagnosticResult) return;

    setShareLoading(true);
    try {
      await invoke('share_diagnostic', {
        report: JSON.parse(diagnosticResult),
      });
    } catch (error) {
      console.error('failed to share diagnostic report', error);
    } finally {
      setShareLoading(false);
    }
  };

  return (
    <PageAnim className="flex h-full flex-col">
      <div className="flex min-h-0 flex-1 flex-col gap-6">
        {diagnosticRunning && (
          <div
            className="flex justify-center pt-6"
            role="status"
            aria-label={t('diagnostic.running')}
          >
            <Spinner className="border-brand-primary! border-b-transparent!" />
          </div>
        )}
        {!diagnosticRunning && (
          <Button onClick={handleRunDiagnostic}>{t('diagnostic.rerun')}</Button>
        )}
        {diagnosticFailed && (
          <p className="text-text-secondary text-sm" role="alert">
            {t('diagnostic.failed')}
          </p>
        )}
        {!diagnosticRunning && diagnosticResult && (
          <>
            <Button
              variant="outlined"
              onClick={handleShareReport}
              disabled={shareLoading}
              loading={shareLoading}
            >
              {t('diagnostic.share')}
            </Button>
            <div className="dark:bg-surface-elev flex min-h-0 flex-1 flex-col space-y-4 rounded-lg bg-white p-6">
              <div className="flex items-center justify-between">
                <span className="text-text-primary text-base">
                  {t('diagnostic.report-title')}
                </span>
                <ButtonIconNew
                  className="self-start"
                  icon="content_copy"
                  onClick={() => copy(diagnosticResult, false)}
                  clickFeedback
                  size="small"
                  // noDefaultSize
                />
              </div>
              <div className="min-h-0 flex-1 overflow-y-auto">
                <pre className="text-text-secondary font-mono text-xs wrap-break-word whitespace-pre-wrap">
                  {diagnosticResult}
                </pre>
              </div>
            </div>
          </>
        )}
      </div>
    </PageAnim>
  );
}

export default Diagnostic;
