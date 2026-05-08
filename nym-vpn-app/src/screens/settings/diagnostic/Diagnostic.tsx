import { useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { useTranslation } from 'react-i18next';
import { Separator } from '@base-ui-components/react';
import { ButtonIconNew, ButtonNew, PageAnim } from '../../../ui';
import { useClipboard } from '../../../hooks';

function Diagnostic() {
  const { t } = useTranslation('settings');
  const { copy } = useClipboard();

  const [diagnosticRunning, setDiagnosticRunning] = useState(false);
  const [diagnosticResult, setDiagnosticResult] = useState<string | null>(null);
  const [shareLoading, setShareLoading] = useState(false);

  const handleRunDiagnostic = async () => {
    setDiagnosticRunning(true);
    try {
      const report = await invoke('run_diagnostic', {
        params: { gateway: null, skipDns: false, skipHttp: false },
      });
      setDiagnosticResult(JSON.stringify(report, null, 2));
    } finally {
      setDiagnosticRunning(false);
    }
  };

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
        <ButtonNew
          onClick={handleRunDiagnostic}
          disabled={diagnosticRunning}
          loading={diagnosticRunning}
        >
          {t('diagnostic.run')}
        </ButtonNew>
        {diagnosticResult && (
          <>
            <ButtonNew
              variant="outlined"
              onClick={handleShareReport}
              disabled={shareLoading}
              loading={shareLoading}
            >
              {t('diagnostic.share')}
            </ButtonNew>
            <div className="dark:bg-charcoal flex min-h-0 flex-1 flex-col space-y-4 rounded-lg bg-white p-6 font-mono text-xs">
              <div className="flex items-center justify-between">
                <span className="text-sm font-semibold">
                  {t('diagnostic.report-title')}
                </span>
                <ButtonIconNew
                  className="self-start"
                  icon="content_copy"
                  onClick={() => copy(diagnosticResult, false)}
                  clickFeedback
                  noDefaultSize
                />
              </div>
              <Separator
                orientation="horizontal"
                className="bg-bombay dark:bg-iron h-px w-full"
              />
              <div className="min-h-0 flex-1 overflow-y-auto">
                <pre className="text-text-secondary wrap-break-word whitespace-pre-wrap">
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
